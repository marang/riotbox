#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
usage: scripts/guard_showcase_output_dir.sh OUTPUT_DIR LABEL [--force-output-reset]

Refuse destructive showcase output resets outside known local artifact roots
unless --force-output-reset or RIOTBOX_FORCE_SHOWCASE_OUTPUT_RESET=1 is set.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if [[ "$#" -lt 2 || "$#" -gt 3 ]]; then
    usage
    exit 2
fi

output_dir="$(realpath -m "$1")"
label="$2"
force_arg="${3:-}"

if [[ -n "$force_arg" && "$force_arg" != "--force-output-reset" ]]; then
    echo "unsupported output reset guard argument: $force_arg" >&2
    usage
    exit 2
fi

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
case "$output_dir" in
    "$repo_root"/artifacts/audio_qa/*|/tmp/riotbox-*) ;;
    *)
        if [[ "$force_arg" == "--force-output-reset" || "${RIOTBOX_FORCE_SHOWCASE_OUTPUT_RESET:-}" == "1" ]]; then
            echo "forcing $label output reset outside artifacts/audio_qa or /tmp/riotbox-*: $output_dir" >&2
            exit 0
        fi
        echo "refusing to reset $label output outside artifacts/audio_qa or /tmp/riotbox-*: $output_dir" >&2
        exit 1
        ;;
esac
