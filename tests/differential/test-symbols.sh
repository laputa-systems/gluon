#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
. "$root/tests/differential/symbols.sh"
mkdir -p "$root/.work/tmp"
scratch=$(mktemp -d "$root/.work/tmp/symbols.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
for directory in first second; do
  mkdir "$scratch/$directory"
  printf 'int marker(void) { return 0; }\nint main(void) { return marker(); }\n' > "$scratch/$directory/input.c"
  "${CC:-cc}" -g "$scratch/$directory/input.c" -o "$scratch/$directory/app"
  defined_symbols "$scratch/$directory/app" > "$scratch/$directory/symbols"
done
test -s "$scratch/first/symbols"
cmp "$scratch/first/symbols" "$scratch/second/symbols"
printf 'int other(void) { return 0; }\nint main(void) { return other(); }\n' > "$scratch/second/input.c"
"${CC:-cc}" -g "$scratch/second/input.c" -o "$scratch/second/app"
defined_symbols "$scratch/second/app" > "$scratch/second/symbols"
if cmp -s "$scratch/first/symbols" "$scratch/second/symbols"; then
  echo 'different defined symbols compared equal' >&2
  exit 1
fi
