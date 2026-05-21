#!/usr/bin/env bash
set -euo pipefail

scripts/validate_auto_feral_grid_source_timing_pack.sh beat03 "${1:-local-beat03-feral-grid-auto-proof}"
