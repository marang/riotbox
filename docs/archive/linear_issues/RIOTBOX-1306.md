# `RIOTBOX-1306` P023: Strengthen destructive gesture impact for stage-meaningful cuts

- Ticket: `RIOTBOX-1306`
- Title: `P023: Strengthen destructive gesture impact for stage-meaningful cuts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1306/p023-strengthen-destructive-gesture-impact-for-stage-meaningful-cuts`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1306-p023-strengthen-destructive-gesture-impact-for-stage-meaningful-cuts`
- Linear branch: `feature/riotbox-1306-p023-strengthen-destructive-gesture-impact-for-stage`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1280 (https://github.com/marang/riotbox/pull/1280)`
- Merge commit: `633e08e7`
- Deleted from Linear: `2026-06-29`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_destructive_variation_professional.py; git diff --check; just destructive-variation-professional-smoke; just pro-pressure-source-matrix-smoke; just professional-source-wav-pack-smoke; just professional-output-listening-pack-smoke; just professional-output-suite-smoke; just ci; GitHub rust-ci passed`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 weak-output routing still flagged flat destructive gestures; dropout/stutter/restore needed a deeper cut and harder restore without fallback product audio or false quality proof.

## What Shipped

- Added source-family destructive tail impact controls, deepened dense/tonal dropout silence, increased source-derived stutter and restore impact, tightened professional destructive cut-depth thresholds, and documented RIOTBOX-1306 in the roadmap.

## Notes

- Reports remain diagnostic/listening-scaffold evidence with quality_proof false and human_verdict unverified; dense remains snare-led and sparse remains bass-led in the source matrix.
