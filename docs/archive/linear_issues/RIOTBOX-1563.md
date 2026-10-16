# `RIOTBOX-1563` — Apply the existing decoded-sample limit before Python frame conversion and analysis

- Ticket: `RIOTBOX-1563`
- Title: `Apply the existing decoded-sample limit before Python frame conversion and analysis`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1563/apply-the-existing-decoded-sample-limit-before-python-frame-conversion`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 maintenance of the accepted P023 source-ingest path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1563-python-decoded-sample-admission`
- Linear branch: `feature/riotbox-1563-apply-the-existing-decoded-sample-limit-before-python-frame`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1666](https://github.com/marang/riotbox/pull/1666)
- Merge commit: `01f4f0a5717e2dd144f17010d9ee6e2244d526c7`
- Deleted from Linear: `2026-10-16`
- Verification: Historical Python 23-test suite; Rust Sidecar 32 unit + 4 integration tests; App 5 ingest identity/persistence tests; fifth-slice architecture checkpoint; full source-free local just ci; exact-head native CI 37107333091 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1563_current_state_cadence_2026-10-03.md`, `docs/reviews/riotbox_1563_python_sample_admission_2026-10-03.md`, `docs/specs/audio_core_spec.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: RIOTBOX-1564 (local DAW writer queued-receipt identity), separately completed

## Why This Ticket Existed

A valid PCM16 mono WAV with 67,108,865 input samples occupies 134,217,774 encoded bytes, below the fixed 268,435,456-byte encoded budget but above Rust's existing 67,108,864 decoded-interleaved-sample limit. Python could read frames, convert floats and construct features before Rust rejected it. A metadata-only seam reached readframes(67108865); no large-file allocation experiment was performed. The limit had to count interleaved input samples, not the shorter averaged mono result.

## What Shipped

- After RBX-432 WAV container/format/whole-frame checks and before frame materialization, PCM conversion or features, Python enforces frame_count * channel_count <= 67,108,864 using the same frozen Rust V1 limit.

- A pure source-admission leaf owns the two explicit Python resource limits and typed errors. The existing source_resource_limit response includes required/limit/unit context and retryable=false; no new persisted policy model or public override was added.

- Exact-limit/empty and valid PCM widths/scaling/channel averaging, encoded reading, same-byte identity, Rust decoder and musical algorithms remain unchanged. Failed admission preserves old/new external/embedded Session destinations and keeps the analysis peer reusable.

- RBX-433 and Source Graph section 6 document the extension. The fifth-substantive-slice sampled architecture checkpoint was completed separately; its pre-existing local-writer receipt mismatch became RIOTBOX-1564 rather than expanding this patch.

## Historical Verification And Outcomes

- Executed RED/GREEN: a tiny over-budget stereo source originally reached a frame-read sentinel; the repair rejects it before reads, conversion or feature work.

- 23 Python tests covered widths 1-4, mono/stereo, empty/below/exact/over limits, malformed-before-resource precedence, typed count/unit context and real-peer resource error -> ping -> valid analysis.

- A metadata-only default-limit witness tested the frozen count without allocating large PCM. Real request and App persistence tests injected tiny budgets deliberately; this was test-only injection, not a product configuration override.

- Rust Sidecar 32 units + 4 integrations and five App source identity/persistence tests passed. Old bytes/restored cache were unchanged, and new paths remained absent.

- Two independent branch reviews reran Python23 and retained no findings; adversarial review additionally ran 64 tiny controls and three precedence probes. The report distinguishes their executed checks from inspected Rust/App logs and separately attributes architecture audit coverage.

- Full source-free local just ci and exact-head native CI 37107333091 passed; both existing native jobs were green at 6f968ea00f489051065c1d6275786859690486d4. Final self-review retained no findings.

## Notes

- Encoded reading and hashing still precede count admission. An input-sample count limit is not a heap/RSS, aggregate-memory or execution-time guarantee and does not migrate old graphs.

- No real source/Holdout/commercial audio, source discovery, large allocation test, host/device/DAW/TUI operation, playback or Windows expansion occurred; no musical/release claim was added.

- Historical closeout synchronized main/origin/main and removed the exact implementation branch after merged/reachable checks while preserving RIOTBOX-1564 changes. The issue was then retained in Done.

- The additional checkpoint report is docs/reviews/riotbox_1563_current_state_cadence_2026-10-03.md.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1563_python_sample_admission_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
