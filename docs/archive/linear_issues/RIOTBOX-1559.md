# `RIOTBOX-1559` — Bind Python and Rust source-ingest evidence to one verified byte identity before persistence

- Ticket: `RIOTBOX-1559`
- Title: `Bind Python and Rust source-ingest evidence to one verified byte identity before persistence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1559/bind-python-and-rust-source-ingest-evidence-to-one-verified-byte`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1559-ingest-byte-identity`
- Linear branch: `feature/riotbox-1559-bind-python-and-rust-source-ingest-evidence-to-one-verified`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`
- PR: [#1662](https://github.com/marang/riotbox/pull/1662)
- Merge commit: `e80cb816e9510086b2e2cb2c46296728d2d6a17d`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: generated replacement RED→GREEN, Python 3, App ingest 25/timing 8 overlapping filters, source-free `just ci`; native CI 37098264348 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1559_ingest_identity_2026-10-03.md`, `docs/specs/audio_core_spec.md`, `docs/specs/source_graph_spec.md`, `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: RIOTBOX-1556 Rust payload budgets remain in force; Python admission and aggregate-memory boundaries are separate, later addressed by RIOTBOX-1561/1562/1563. Historical graphs are not automatically re-attested.

## Why This Ticket Existed

The RIOTBOX-1558 current-state audit found a pre-existing mixed-byte ingest integrity gap: Python hashed a completed read then reopened for decode, Rust reopened for timing/hook enrichment without matching the graph hash, and Graph/Session could publish before restore admission. Replacing A with B between reads could persist A's identity with B-derived analysis; later restoring A could retain that mismatch. This was independently checked static-path evidence, not a reproduced real-source/hardware event.

## What Shipped

- Python hashes and WAV-decodes the same captured admitted byte buffer rather than reopening a path between stages.
- Rust makes one bounded admitted read, requires its hash to match the provider/graph identity before enrichment or Graph/Session publication, and decodes the same verified bytes. Existing source path policy, identity algorithm and RBX-425 budgets remain.
- Generated replacements between Python stages and between provider response/Rust enrichment now remain internally consistent or reject before writing either output. Existing external and embedded saved pairs stay byte-identical and loadable; fresh destinations stay absent on disagreement.
- RBX-429 and Source Graph/Audio/Timing specs freeze the admission boundary. No new source-analysis algorithm, DSP, schema, dependency, Action or app-local durable truth was added.

## Notes

- Source-replacement regressions reproduced the defect RED before the correction and passed GREEN afterward. Unchanged generated-source ingest succeeds; timing fixtures were updated to carry their actual generated WAV identities, not fabricated placeholders.
- Independent Spec/Evidence and Rust/Adversarial reviews plus final self-review retained no P0–P3 finding. Full source-free CI, including synthetic audio/observer/manifest contracts and strict Clippy, passed.
- Reviewed head `77602f820a273c00844e3be395d1a03cd0654e8f` passed native CI. Historical closeout reports synchronized clean main and deletion of exact merged feature branches with commit/PR history retained; no outstanding PR findings.
- Matching captured buffers is future-ingest consistency, not filesystem snapshot/immutability after verification, historical authenticity or retrospective validation of old graphs. Existing hydration hash checks remain; no old identity migration is implied.
- Python and aggregate memory bounds remain outside this repair. No real Development/Holdout/commercial source reads or discovery, playback, human or new source-general/musical/release qualification occurred; all input audio was generated.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
