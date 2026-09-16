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
# Covers markdown prose and the comments in source files. Source is included
# because most of this project's prose now lives in Rust doc comments, and a
# rule enforced on the documentation but not on the code it documents is a rule
# with a hole in it exactly where the writing happens.
#
# Only comment text is read. Code is never checked, so an identifier named
# `delve_into` is fine and a comment saying "delve into" is not.
#
# Usage:
#   tools/check-voice.sh [path ...]     defaults to planning/ docs/ crates/ tools/ *.md
#
# Exit 0 clean, 1 on violations.

set -uo pipefail

TARGETS=("$@")
if [ ${#TARGETS[@]} -eq 0 ]; then
  TARGETS=()
  for p in planning docs crates python tools .githooks README.md CONTRIBUTING.md CODE_OF_CONDUCT.md; do
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
        --exclude-dir=.venv --exclude-dir=__pycache__
        --exclude=check-voice.sh --exclude=*.lock --exclude=*.svg )

# Build the list of files to scan, honouring the opt-out marker.
#
# A document that *teaches* these rules has to quote the patterns it forbids.
# `writing-voice` contains every banned word by definition, and the review skills
# show bad prose next to its corrected form. Those files declare the marker
# below, which is deliberate and greppable rather than a silent exclusion.
OPTOUT='voice-check: teaches-by-example'

# Extensions whose comments carry prose. Anything not listed here is skipped
# rather than guessed at, because a file type this script does not know how to
# strip to comments would have its code checked as if it were prose.
INCLUDES=( --include='*.md' --include='*.rs' --include='*.py'
           --include='*.sh' --include='*.go' --include='*.toml' )

collect() {
  grep -rlIZ "${INCLUDES[@]}" "${PRUNE[@]}" '' "${TARGETS[@]}" 2>/dev/null \
    | while IFS= read -r -d '' f; do
        grep -q "$OPTOUT" "$f" || printf '%s\0' "$f"
      done
}

# Reduce a file to the prose it contains, one output line per input line so that
# reported line numbers stay accurate.
#
# Markdown: everything except fenced code blocks, so that Rust macros
# (`assert!`, `panic!`) and shell snippets in examples do not trip prose rules.
#
# Source: comment text only. Code is not prose, and checking it would flag every
# identifier that happens to contain a banned substring. Fenced blocks inside doc
# comments are dropped too, since a Rust doctest is code that happens to live in
# a comment.
prose() {
  case "$1" in
    *.rs|*.go)        marker='//' ; quotes='"'  ;;
    *.py)             marker='#'  ; quotes="\"'" ;;
    *.sh|*.toml)      marker='#'  ; quotes='"'  ;;
    *)                marker=''   ; quotes=''   ;;
  esac

  if [ -z "$marker" ]; then
    awk 'BEGIN{inb=0}
         /^[[:space:]]*```/ {inb=!inb; print ""; next}
         {print (inb ? "" : $0)}' "$1"
    return
  fi

  # Find the comment marker outside any string literal, then emit what follows.
  #
  # Scanning for quotes rather than taking the first marker matters here: this
  # codebase is full of ontology IRIs, and `"https://identifiers.org/SO:0000167"`
  # would otherwise be read as a comment starting at `//`. Rust lifetimes are the
  # reason `.rs` tracks double quotes only, since `'a` would open a single-quote
  # state that never closes and swallow the rest of the line.
  awk -v marker="$marker" -v quotes="$quotes" '
    BEGIN { mlen = length(marker); inb = 0 }
    {
      line = $0
      n = length(line)
      q = ""
      pos = 0
      for (i = 1; i <= n; i++) {
        c = substr(line, i, 1)
        if (c == "\\" && q != "") { i++; continue }
        if (q != "") { if (c == q) q = ""; continue }
        if (index(quotes, c) > 0) { q = c; continue }
        if (substr(line, i, mlen) == marker) { pos = i; break }
      }
      if (pos == 0) { print ""; next }

      text = substr(line, pos + mlen)
      # Drop the doc-comment and shebang sigils that follow the marker.
      sub(/^[\/!]+/, "", text)
      sub(/^[[:space:]]+/, "", text)

      # A fenced block inside a doc comment is code that happens to live in a
      # comment, so it is skipped the same way a markdown fence is.
      if (text ~ /^```/) { inb = !inb; print ""; next }
      print (inb ? "" : text)
    }' "$1"
}

# Every scanned file reduced to prose once, as `path:lineno:text` lines. Rules
# then grep this bundle. Extracting per rule instead would re-read every file for
# each pattern, which is most of what the old shape spent its time on.
BUNDLE=$(
  while IFS= read -r -d '' f; do
    prose "$f" | grep -n '' | sed "s|^|$f:|"
  done < <(collect)
)
SCANNED=$(printf '%s\n' "$BUNDLE" | cut -d: -f1 | sort -u | grep -c . || true)

report() { # name, pattern, explanation, [extra-filter]
  local name="$1" pat="$2" why="$3" filter="${4:-cat}"
  local hits
  hits=$(printf '%s\n' "$BUNDLE" | grep -Ei "$pat" 2>/dev/null | eval "$filter") || true
  # Blank lines are placeholders that keep line numbers aligned, so a pattern
  # matching nothing but `path:12:` is not a hit.
  hits=$(printf '%s' "$hits" | grep -vE '^[^:]*:[0-9]+:[[:space:]]*$' || true)
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
trio=$(printf '%s\n' "$BUNDLE" | grep -E \
  '\b(fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient), (fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient), and (fast|simple|clean|modern|powerful|flexible|scalable|reliable|secure|efficient)\b' \
  2>/dev/null) || true
if [ -n "$trio" ]; then
  printf '\033[33m%s\033[0m\n' "WARN: rule-of-three cadence"
  grey "      Fine if genuinely the clearest phrasing. Not a failure."
  echo "$trio" | sed 's/^/      /'
  echo
fi

if [ "$fail" -eq 0 ]; then
  echo "voice check: clean ($SCANNED file(s))"
else
  red "voice check: violations found"
fi
exit "$fail"
