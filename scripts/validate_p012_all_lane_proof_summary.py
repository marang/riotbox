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
    "| Path | Cue | Action | Grid source | Decision | Observer grid use | Manifest grid use | Grid compat | Downbeat compat | Downbeat ambiguity | Alignment | Output issues |",
    "| cautious/manual | `needs confirm` | `confirm grid first` | `source_timing` | `source_timing_needs_review_manual_confirm` | `manual_confirm_only` | `short_loop_manual_confirm` | `compatible` | `partial` | `partial` | `aligned` | 0 |",
    "| user override | `needs confirm` | `confirm grid first` | `user_override` | `user_override` | `manual_confirm_only` | `short_loop_manual_confirm` | `compatible` | `partial` | `partial` | `aligned` | 0 |",
    "| risky override | `needs confirm` | `confirm grid first` | `user_override` | `user_override` | `manual_confirm_only` | `short_loop_manual_confirm` | `compatible` | `partial` | `partial` | `aligned` | 0 |",
    "| fallback | `not available` | `timing unavailable` | `static_default` | `source_timing_missing_bpm` | `unavailable` | `unavailable` | `aligned` | `partial` | `partial` | `aligned` | 0 |",
    "| locked grid | `grid locked` | `grid can steer moves` | `source_timing` | `source_timing_ready` | `locked_grid` | `locked_grid` | `aligned` | `aligned` | `aligned` | `aligned` | 0 |",
    "| Source | Cue | Action | Readiness | Manual confirm | Grid source | Decision | Grid use | Phrase count | Phrase bars | BPM | Downbeat | Downbeat score | Downbeat margin | Alt phases | TR-909 | MC-202 | W-30 | Mix |",
    "| Beat03 | `needs confirm` | `confirm grid first` | `needs_review` | `yes` | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | 0 | 2 |",
    "| Beat08 | `needs confirm` | `confirm grid first` | `needs_review` | `yes` | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | 0 | 2 |",
    "| DH_BeatC | `needs confirm` | `confirm grid first` | `needs_review` | `yes` | `source_timing` | `source_timing_needs_review_manual_confirm` | `short_loop_manual_confirm` | 0 | 3 |",
    "| Beat20 | `needs confirm` | `confirm grid first` | `needs_review` | `yes` | `static_default` | `source_timing_requires_manual_confirm` | `manual_confirm_only` | 0 | 2 |",
    "Generated `Cue` and `Action` columns expose the musician-facing consequence",
    "Generated `Downbeat ambiguity` shows whether the observer and manifest agree",
    "`Cue` and `Action` are the compact musician-facing consequence",
    "`Phrase count` and `Phrase bars` expose the bounded phrase-grid evidence",
    "`Downbeat score`, `Downbeat margin`, and `Alt phases` expose the bounded bar-phase confidence",
    "Generated Feral-grid observer/audio rows show whether control-path and output-path timing evidence agreed",
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
