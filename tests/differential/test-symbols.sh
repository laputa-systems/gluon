#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
. "$root/tests/differential/symbols.sh"
mkdir -p "$root/.work/tmp"
scratch=$(mktemp -d "$root/.work/tmp/symbols.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
for directory in first second; do
  mkdir "$scratch/$directory"
  printf 'int marker(void) { return 0; }\n' > "$scratch/$directory/input.c"
  "${CC:-cc}" -g -c "$scratch/$directory/input.c" -o "$scratch/$directory/input.o"
  defined_symbols "$scratch/$directory/input.o" > "$scratch/$directory/symbols"
done
test -s "$scratch/first/symbols"
cmp "$scratch/first/symbols" "$scratch/second/symbols"
printf 'int other(void) { return 0; }\n' > "$scratch/second/input.c"
"${CC:-cc}" -g -c "$scratch/second/input.c" -o "$scratch/second/input.o"
defined_symbols "$scratch/second/input.o" > "$scratch/second/symbols"
if cmp -s "$scratch/first/symbols" "$scratch/second/symbols"; then
  echo 'different defined symbols compared equal' >&2
  exit 1
fi
