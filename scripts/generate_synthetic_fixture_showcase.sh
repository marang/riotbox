#!/usr/bin/env bash
set -euo pipefail

echo "Synthetic fixture showcase: deterministic developer QA, not a musician-facing listening demo." >&2
exec "$(dirname "$0")/generate_representative_source_showcase.sh" "$@"
