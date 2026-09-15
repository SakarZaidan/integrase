#!/usr/bin/env bash
#
# check-voice.sh: enforce the Integrase writing rules mechanically.
#
# The project's voice rules are load-bearing, not decoration, and rules that are
# only enforced by good intentions decay. This catches the mechanical subset.
# Judgment calls (false enthusiasm, metronomic sentence rhythm, text that could
# be pasted into another project unchanged) still need a human or the
# writing-voice skill.
#
# Usage:
#   tools/check-voice.sh [path ...]     defaults to planning/ docs/ *.md
#
# Exit 0 clean, 1 on violations.

set -uo pipefail

TARGETS=("$@")
if [ ${#TARGETS[@]} -eq 0 ]; then
  TARGETS=()
  for p in planning docs README.md CONTRIBUTING.md CODE_OF_CONDUCT.md; do
    [ -e "$p" ] && TARGETS+=("$p")
  done
fi
[ ${#TARGETS[@]} -eq 0 ] && { echo "nothing to check"; exit 0; }

fail=0
red()  { printf '\033[31m%s\033[0m\n' "$1"; }
grey() { printf '\033[90m%s\033[0m\n' "$1"; }

# This script necessarily contains the patterns it searches for, so it must
# never check itself. Same for the lockfiles and anything binary.
PRUNE=( --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=target
        --exclude=check-voice.sh --exclude=*.lock --exclude=*.svg )

# Build the list of files to scan, honouring the opt-out marker.
#
# A document that *teaches* these rules has to quote the patterns it forbids.
# `writing-voice` contains every banned word by definition, and the review skills
# show bad prose next to its corrected form. Those files declare the marker
# below, which is deliberate and greppable rather than a silent exclusion.
OPTOUT='voice-check: teaches-by-example'
collect() {
  grep -rlIZ --include='*.md' "${PRUNE[@]}" '' "${TARGETS[@]}" 2>/dev/null \
    | while IFS= read -r -d '' f; do
        grep -q "$OPTOUT" "$f" || printf '%s\0' "$f"
      done
}

# Prose only: drop fenced code blocks so that Rust macros (assert!, panic!) and
# shell snippets do not trip prose rules. Keeps line numbers accurate.
prose() {
  awk 'BEGIN{inb=0}
       /^[[:space:]]*```/ {inb=!inb; print ""; next}
       {print (inb ? "" : $0)}' "$1"
}

report() { # name, pattern, explanation, [extra-filter]
  local name="$1" pat="$2" why="$3" filter="${4:-cat}"
  local hits=""
  local f
  while IFS= read -r -d '' f; do
    local found
    found=$(prose "$f" | grep -nEi "$pat" 2>/dev/null | eval "$filter" | sed "s|^|$f:|") || true
    [ -n "$found" ] && hits="${hits}${found}"$'\n'
  done < <(collect)
  hits=$(printf '%s' "$hits" | sed '/^$/d')
  if [ -n "$hits" ]; then
    red "FAIL: $name"
    grey "      $why"
    echo "$hits" | sed 's/^/      /'
    echo
    fail=1
  fi
}

# 1. Em dashes. The rule is absolute: rewrite the sentence instead.
report "em dash" '—' \
  "Use a period, comma, colon, or restructure. Never an em dash."

# 2. AI-tell vocabulary.
#    'leverage' is banned as a VERB only. The noun ('high-leverage', 'ordered by
#    leverage') is ordinary precise English and is deliberately allowed, so the
#    pattern requires verb morphology or a following article/pronoun.
report "AI-tell vocabulary" \
  '\b(delve[sd]?|delving|seamless(ly)?|furthermore|moreover|game[- ]chang(er|ing)|cutting[- ]edge|unlock(s|ing)? (the|your|new)|elevate (the|your)|in todays? fast[- ]paced)\b' \
  "Plainer word, or cut the sentence."

report "leverage as a verb" \
  '\b(leverages|leveraged|leveraging|to leverage|will leverage|can leverage|leverage (the|this|these|that|those|our|your|its|their|a|an) )' \
  "Use 'use', 'apply', 'build on', or name the actual mechanism."

# 3. Hedging filler that carries no information.
report "hedging filler" \
  "\b(it('s| is) (important|worth) (to )?(note|noting|mention|mentioning)|it should be (noted|mentioned)|needless to say)\b" \
  "If it is worth saying, say it directly."

# 4. 'robust' used as a filler adjective. Allowed in a defined technical sense
#    (robust statistics, robustness analysis), so only bare adjectival use is
#    flagged for a human to look at.
report "filler 'robust'" \
  '\brobust\b' \
  "Banned as filler. Fine as a defined technical term; confirm which this is." \
  "grep -viE 'robust(ness)? (statistic|analys|estimat|control|check)|technical term'"

# 5. Exclamation points outside code, links, and comparison operators.
report "exclamation point" '!' \
  "No manufactured enthusiasm." \
  "grep -vE '!=|!\[|\[!|#!|\!important|<!--|\!\[|\`[^\`]*!'"

# 6. Reflexive rule-of-three adjective cadence. Advisory: three parallel
#    adjectives are sometimes genuinely clearest. Flagged for a look, not failed.
trio=$(collect | xargs -0 -r grep -nEI \
  '\b(fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient), (fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient), and (fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient)\b' \
  "${TARGETS[@]}" 2>/dev/null) || true
if [ -n "$trio" ]; then
  printf '\033[33m%s\033[0m\n' "WARN: rule-of-three cadence"
  grey "      Fine if genuinely the clearest phrasing. Not a failure."
  echo "$trio" | sed 's/^/      /'
  echo
fi

if [ "$fail" -eq 0 ]; then
  echo "voice check: clean (${#TARGETS[@]} target(s))"
else
  red "voice check: violations found"
fi
exit "$fail"
