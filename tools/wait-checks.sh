#!/usr/bin/env bash
#
# wait-checks.sh <pr-number>
#
# Waits for a PR's checks and reports the outcome once.
#
# Exists because `gh pr checks` uses its exit code to report *state*, not
# success: 0 when all pass, 1 when any fail, and 8 while any are still pending.
# Treating non-zero as an error, which is the obvious thing to write, turns
# "still running" into "the tool broke" and hides real progress.
#
# Exit 0 if everything passed, 1 otherwise.

set -uo pipefail
PR="${1:?usage: wait-checks.sh <pr-number>}"
INTERVAL="${2:-30}"
MAX="${3:-60}"

for _ in $(seq 1 "$MAX"); do
    # Deliberately no `||` here: a non-zero exit is expected and meaningful.
    out=$(gh pr checks "$PR" 2>/dev/null)
    if [ -z "$out" ]; then
        sleep "$INTERVAL"; continue
    fi
    if ! grep -q 'pending' <<<"$out"; then
        fails=$(awk -F'\t' '$2!="pass" {print "  " $2 "  " $1}' <<<"$out")
        total=$(wc -l <<<"$out" | tr -d ' ')
        if [ -n "$fails" ]; then
            echo "FAILURES on PR #$PR:"; echo "$fails"; exit 1
        fi
        echo "all $total checks pass on PR #$PR"; exit 0
    fi
    sleep "$INTERVAL"
done

echo "timed out waiting for PR #$PR"; exit 1
