#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


REQUIRED_SNIPPETS = (
    "# P012 All-Lane Source-Grid Output Proof Summary",
    "Status: `pass`",
    "Observer/audio generated Feral-grid correlation: `pass`",
    "Recipe 2 observer/audio gate: `pass`",
    "Recipe 15 real-source auto/fallback proof: `pass`",
    "| Source | Grid source | Decision | Grid use | Action | BPM | Downbeat | TR-909 | MC-202 | W-30 | Mix |",
    "| Beat03 | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | `confirm grid first` |",
    "| Beat08 | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | `confirm grid first` |",
    "| DH_BeatC | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | `confirm grid first` |",
    "| Beat20 | `static_default` | `source_timing_requires_manual_confirm` | `manual_confirm_only` | `confirm grid first` |",
    "`Action` is the compact musician-facing consequence",
)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: validate_p012_all_lane_proof_summary.py <summary.md>")

    path = Path(sys.argv[1])
    text = path.read_text(encoding="utf-8")
    for snippet in REQUIRED_SNIPPETS:
        if snippet not in text:
            raise SystemExit(f"{path}: missing summary snippet: {snippet}")
    print(f"ok: P012 all-lane proof summary {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
