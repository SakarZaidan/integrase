#!/usr/bin/env bash
#
# mutate.sh: verify a test suite actually catches a specific bug.
#
# R-2.5 requires checkers and suites to be tested against input they should
# reject. This applies that to code: introduce a known defect, confirm the suite
# turns red, restore.
#
# Usage:
#   tools/mutate.sh <file> <crate> "<find>" "<replace>" "<description>"
#
# Exit 0 if the mutation was CAUGHT, 1 otherwise.
#
# ---------------------------------------------------------------------------
# Why this exists rather than a one-line sed and a grep
#
# The ad-hoc version of this reported MISSED twice for mutations that were
# never actually applied: once because shell quoting mangled the search pattern,
# and once because the mutated code did not compile (`Option::filter` inside a
# `const fn`) and the grep for "FAILED" saw a compile error instead.
#
# Those are the opposite conclusion from a real miss. A mutation tool that
# cannot tell "your tests are weak" from "my mutation did nothing" reports your
# suite as weaker than it is, and invites writing tests you do not need.
#
# So this script distinguishes four outcomes explicitly, and treats anything it
# cannot interpret as INVALID rather than guessing.

set -uo pipefail

FILE="${1:?usage: mutate.sh <file> <crate> <find> <replace> [description]}"
CRATE="${2:?}"
FIND="${3:?}"
REPLACE="${4?usage: mutate.sh <file> <crate> <find> <replace> [description]}"
DESC="${5:-$FILE}"

[ -f "$FILE" ] || { printf '  %-46s ERROR: no such file\n' "$DESC"; exit 1; }

BACKUP=$(mktemp)
cp "$FILE" "$BACKUP"
restore() { cp "$BACKUP" "$FILE"; rm -f "$BACKUP"; }
trap restore EXIT

# 1. Apply. Fail loudly if the pattern is not present, rather than silently
#    testing an unmutated file.
if ! FIND="$FIND" REPLACE="$REPLACE" FILE="$FILE" python3 - <<'PY'
import os, sys, pathlib
p = pathlib.Path(os.environ["FILE"])
s = p.read_text()
find = os.environ["FIND"]
if find not in s:
    sys.exit(1)
p.write_text(s.replace(find, os.environ["REPLACE"], 1))
PY
then
    printf '  %-46s INVALID: pattern not found in %s\n' "$DESC" "$FILE"
    exit 1
fi

# 2. Confirm the mutant compiles. A mutation that does not build proves nothing
#    about the tests, and is the failure that produced both earlier false
#    negatives.
if ! cargo check -p "$CRATE" --all-targets >/dev/null 2>&1; then
    printf '  %-46s INVALID: mutant does not compile\n' "$DESC"
    exit 1
fi

# 3. Run. A non-zero exit from a compiling mutant means the suite objected.
if cargo test -p "$CRATE" >/dev/null 2>&1; then
    printf '  %-46s MISSED  <- the suite does not catch this\n' "$DESC"
    exit 1
fi

printf '  %-46s CAUGHT\n' "$DESC"
exit 0
