# `RIOTBOX-1242` P023: Strengthen sparse bass movement from source contour

- Ticket: `RIOTBOX-1242`
- Title: `P023: Strengthen sparse bass movement from source contour`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1242/p023-strengthen-sparse-bass-movement-from-source-contour`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1242-p023-sparse-bass-source-contour`
- Linear branch: `feature/riotbox-1242-p023-strengthen-sparse-bass-movement-from-source-contour`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1217 (https://github.com/marang/riotbox/pull/1217)`
- Merge commit: `491c9ee6`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_professional_source_wav_pack.py scripts/validate_professional_output_suite_contract.py; just professional-source-wav-pack-smoke; just pro-pressure-source-matrix-smoke; just professional-output-suite-smoke; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md`
- Follow-ups: `Keep release-demo candidates human_verdict: unverified until structured listening review.`

## Why This Ticket Existed

Sparse bass pressure needed stronger source-derived movement and a stricter proof that fixed placeholder contours cannot pass.

## What Shipped

- Sparse bass contour now wraps late source chunks, derives stronger frequency movement from low-band energy/timing/transient direction, raises the static-contour distance gate to 0.50 Hz, propagates child source-report failures in the professional source pack, and tightens tonal hook lift/cut/restore after the stricter gate exposed a weak case.

## Notes

- A standalone audio-qa-ci run was intentionally overlapped with just ci and hit an artifact collision; the same beat20 render passed directly and the integrated sequential just ci gate passed.
