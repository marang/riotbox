#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

python3 -m py_compile scripts/archive_linear_issue.py
python3 scripts/archive_linear_issue.py --help >/dev/null

echo "archive linear issue smoke ok"
