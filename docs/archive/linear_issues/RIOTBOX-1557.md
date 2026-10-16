# `RIOTBOX-1557` — Measure and bound snapshot/undo cursor replay planning cost without changing history semantics

- Ticket: `RIOTBOX-1557`
- Title: `Measure and bound snapshot/undo cursor replay planning cost without changing history semantics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1557/measure-and-bound-snapshotundo-cursor-replay-planning-cost-without`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1557-indexed-replay-cursors`
- Linear branch: `feature/riotbox-1557-measure-and-bound-snapshotundo-cursor-replay-planning-cost`
- Assignee: `Markus`
- Labels: `benchmark`, `Core`, `Improvement`
- PR: [#1660](https://github.com/marang/riotbox/pull/1660)
- Merge commit: `170a88a6a24ce97046cbbdf130851e097422b3de`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical: source-free `just ci`, 44 focused tests pre/post, 30 median-of-five release benchmark rows with 60 values checked; native CI 37079395869 passed. Current archival check: Linear Done and merged PR metadata only; benchmark not rerun.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1557_replay_cursor_scaling_2026-10-03.md`, `docs/specs/replay_model_spec.md`
- Follow-ups: RIOTBOX-1558 owns separately approved actual production callback heap evidence. RIOTBOX-1491's completed full-restore validation remains separate and is not reopened.

## Why This Ticket Existed

RIOTBOX-1491 had already indexed full Session restore validation but explicitly excluded snapshot cursor safety, which repeatedly searched undo targets, markers and commit records per candidate. The second approved engineering slice measured that exact public replay-target/snapshot-comparison path before optimizing it.

## What Shipped

- RBX-426 replaces repeated scans with one private transient Core cursor-safety projection/index. Vector-first action identity, typed and legacy undo rules, first unresolved identity, cursor validation/error precedence, highest cursor/latest-on-tie selection, replay ordering and suffixes remain unchanged.
- The existing public validation order, historical rejection and log-end allowance are preserved. Log-end/no-eligible-nonzero-anchor replay skips safety construction; plain histories avoid unnecessary prefix/record-index allocation.
- A source-free ignored release benchmark measures the actual public planner on bounded plain/typed-undo histories and snapshots. Complete public-result scan-oracle differential regressions prevent a shadow replay contract.

## Notes

- Forty-four focused tests passed before and after. Coverage compares every cursor in generated typed/legacy/plain/mixed histories, nonmonotonic IDs, reordered records, queued/rejected actions, ties, absent anchors and ten malformed histories with exact error precedence.
- Local median-of-five measurements observed 4096 actions / 128 snapshots / typed undo planning at 898.846364 ms before versus 2.582450 ms after (about 348×). Fixture creation and returned-result teardown are excluded; working-desktop noise and plain/no-snapshot variability are documented. This is not a universal latency/no-regression/realtime guarantee, and CI has no wall-clock assertion.
- Independent Rust/performance/API and Spec/Evidence/Product/Risk reviews found zero actionable code defects. A final P3 inaccurate suffix-destruction timing description was fixed; all 60 table values were independently checked. Self-review retained no additional finding or waiver.
- Exact reviewed head `f447ead395420c7f663cce9f254ad97dc8be384f` passed native CI; historical closeout reports clean synchronized main and no outstanding PR comments.
- The index has linear transient memory, not a fallible/aggregate/process-memory guarantee. No public interface, persisted history/schema/state, Action, dependency, app shadow model, runtime/DSP, TUI or README change.
- Only generated metadata histories were used: no real sources/audio/Holdout/commercial access or directory discovery, device/runtime/DAW/playback, physical-device, human, musical or release qualification.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
