#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

python3 -m py_compile scripts/archive_linear_issue.py
python3 scripts/archive_linear_issue.py --help >/dev/null

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if LINEAR_API_TOKEN=dummy LINEAR_GRAPHQL_ENDPOINT=http://127.0.0.1:9 \
  python3 scripts/archive_linear_issue.py --ticket RIOTBOX-999999 --allow-placeholders \
  >"$tmp/network.out" 2>"$tmp/network.err"; then
  echo "expected unreachable Linear endpoint to fail" >&2
  exit 1
fi
grep -q "archive_linear_issue: Linear request failed:" "$tmp/network.err"
if grep -q "Traceback" "$tmp/network.err"; then
  echo "expected clean network failure without Python traceback" >&2
  exit 1
fi

echo "archive linear issue smoke ok"
