#!/usr/bin/env bash
set -euo pipefail

allowlist="${1:-docs/engineering/textual_include_allowlist.txt}"

if [[ ! -f "$allowlist" ]]; then
  echo "missing textual include allowlist: $allowlist" >&2
  exit 2
fi

current="$(mktemp)"
trap 'rm -f "$current"' EXIT

if rg -n 'include!' crates --glob '*.rs' >/dev/null; then
  rg -n 'include!' crates --glob '*.rs' \
    | cut -d: -f1 \
    | sort \
    | uniq -c \
    | awk '{count=$1; $1=""; sub(/^ /, ""); print count " " $0}' \
    > "$current"
else
  : > "$current"
fi

if ! diff -u "$allowlist" "$current"; then
  cat >&2 <<'MSG'
Unexpected Rust textual include inventory.

Use real Rust modules for new code. If this diff is intentional because a
module migration removed include sites, update the allowlist and the inventory
review note in the same PR.
MSG
  exit 1
fi
