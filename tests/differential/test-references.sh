#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
. "$root/tests/differential/references.sh"
mkdir -p "$root/.work/tmp"
scratch=$(mktemp -d "$root/.work/tmp/references.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir "$scratch/bin"
program=$scratch/bin/gluon-test-oracle
candidate=$scratch/candidate
printf '#!/bin/sh\nprintf "1.12.1\\n"\n' > "$candidate"
chmod +x "$candidate"

reject() {
  local expected=$1
  shift
  local output
  if output=$(PATH="$scratch/bin:$PATH" reference_tool "$@" 2>&1); then
    printf 'accepted a bad oracle: %s\n' "$output" >&2
    exit 1
  fi
  case "$output" in
    *"$expected"*) ;;
    *) printf 'unexpected diagnostic: %s\n' "$output" >&2; exit 1 ;;
  esac
}

reject 'missing from PATH' gluon-test-oracle "$candidate" 1.12.1
ln -s "$candidate" "$program"
reject 'resolves to the gluon candidate' gluon-test-oracle "$candidate" 1.12.1
unlink "$program"
cp "$candidate" "$program"
reject 'resolves to the gluon candidate' gluon-test-oracle "$candidate" 1.12.1
printf '#!/bin/sh\nprintf "1.12.1\\n"\nexit 7\n' > "$program"
reject 'could not read reference' gluon-test-oracle "$candidate" 1.12.1
printf '#!/bin/sh\nprintf "1.12.0\\n"\n' > "$program"
reject 'expected reference gluon-test-oracle 1.12.1, found 1.12.0' gluon-test-oracle "$candidate" 1.12.1
printf '#!/bin/sh\n# distinct oracle\nprintf "1.12.1\\n"\n' > "$program"
resolved=$(PATH="$scratch/bin:$PATH" reference_tool gluon-test-oracle "$candidate" 1.12.1)
test "$resolved" = "$program"
mkdir "$scratch/named"
printf '#!/bin/sh\nprintf "1.13.2\\n"\n' > "$scratch/named/ninja"
chmod +x "$scratch/named/ninja"
if output=$(PATH="$scratch/named:$PATH" reference_tool ninja "$candidate" 1.13.2 2>&1); then
  printf 'accepted a ninja without the upstream subtools: %s\n' "$output" >&2
  exit 1
fi
case "$output" in
  *'is not the upstream implementation'*) ;;
  *) printf 'unexpected diagnostic: %s\n' "$output" >&2; exit 1 ;;
esac
printf '#!/bin/sh\nif [ "$1" = -t ]; then printf "ninja subtools:\\n"; else printf "1.13.2\\n"; fi\n' > "$scratch/named/ninja"
resolved=$(PATH="$scratch/named:$PATH" reference_tool ninja "$candidate" 1.13.2)
test "$resolved" = "$scratch/named/ninja"
