# `RIOTBOX-1278` MC-202 real-source corpus listening pack for dense and non-dense proof

- Ticket: `RIOTBOX-1278`
- Title: `MC-202 real-source corpus listening pack for dense and non-dense proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1278/mc-202-real-source-corpus-listening-pack-for-dense-and-non-dense-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1278-mc202-real-source-listening-pack`
- Linear branch: `feature/riotbox-1278-mc-202-real-source-corpus-listening-pack-for-dense-and-non`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1253 (https://github.com/marang/riotbox/pull/1253)`
- Merge commit: `f8e35d1e5abfdb7e3c1649fda9e54846611f6cbe`
- Deleted from Linear: `2026-06-18`
- Verification: `python3 -m py_compile scripts/generate_mc202_real_source_listening_pack.py`; `git diff --check`; `just mc202-real-source-listening-pack-smoke artifacts/audio_qa/local-riotbox-1278-mc202-real-source-listening-pack`; `just audio-qa-ci`; `just ci`
- Docs touched: `docs/benchmarks/mc202_real_source_listening_pack_v1_2026-06-18.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

MC-202 source-derived behavior needed a real dense/non-dense listening-review pack that gives reviewers actual source windows, MC-202 stems, generated-support mixes, and expression metadata without promoting scripted diagnostics or primitive controls to product quality proof.

## What Shipped

- Added a generator and validator for riotbox.mc202_real_source_listening_pack.v1 covering one dense break and two non-dense real-source cases.
- Added a Just smoke target and wired it into audio-qa-ci so the pack contract, artifact hashes, non-silent MC-202 stems, and non-product primitive control stay enforced.
- Documented the benchmark in docs, the roadmap, and the audio QA workflow with human_verdict: unverified and quality_proof: false.

## Notes

- Primitive A/B control remains non-product evidence only; no fallback music is allowed on product output paths.
