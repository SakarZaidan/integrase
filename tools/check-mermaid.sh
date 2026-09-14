#!/usr/bin/env bash
#
# check-mermaid.sh: verify every ```mermaid block in the given markdown files
# actually parses. A diagram that renders as an error block in the README is a
# visible failure, and "it looked right in my editor" is not verification.
#
# Usage: tools/check-mermaid.sh [file ...]   defaults to planning/ and docs/
# Exit 0 clean, 1 if any block fails to parse.

set -uo pipefail
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT

FILES=("$@")
if [ ${#FILES[@]} -eq 0 ]; then
  mapfile -t FILES < <(find planning docs -name '*.md' 2>/dev/null)
  [ -f README.md ] && FILES+=(README.md)
fi
[ ${#FILES[@]} -eq 0 ] && { echo "no files to check"; exit 0; }

# Puppeteer needs a sandbox opt-out under WSL/CI.
cat > "$WORK/pptr.json" <<'JSON'
{ "args": ["--no-sandbox", "--disable-setuid-sandbox", "--disable-dev-shm-usage"] }
JSON

total=0; failed=0
for f in "${FILES[@]}"; do
  [ -f "$f" ] || continue
  # Split out each fenced mermaid block into its own file.
  awk -v out="$WORK" -v src="$(basename "$f")" '
    /^```mermaid[[:space:]]*$/ { inb=1; n++; next }
    /^```[[:space:]]*$/        { if (inb) { inb=0; close(fn) } ; next }
    inb { fn = sprintf("%s/%s.%03d.mmd", out, src, n); print > fn }
  ' "$f"
done

shopt -s nullglob
for mmd in "$WORK"/*.mmd; do
  total=$((total+1))
  if err=$(npx -y @mermaid-js/mermaid-cli@11 -i "$mmd" -o "$mmd.svg" -p "$WORK/pptr.json" 2>&1); then
    printf '  ok    %s\n' "$(basename "$mmd" .mmd)"
  else
    failed=$((failed+1))
    printf '  FAIL  %s\n' "$(basename "$mmd" .mmd)"
    echo "$err" | grep -iE 'error|expect|parse' | head -4 | sed 's/^/          /'
  fi
done

if [ "$total" -eq 0 ]; then echo "no mermaid blocks found"; exit 0; fi
echo "mermaid: $((total-failed))/$total blocks parse"
[ "$failed" -eq 0 ] || exit 1
