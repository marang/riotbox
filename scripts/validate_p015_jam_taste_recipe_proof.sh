#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

cargo test -p riotbox-app p015_recipe -- --nocapture
scripts/validate_p014_scene_movement_observer_probe.sh
