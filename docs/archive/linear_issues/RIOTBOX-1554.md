# `RIOTBOX-1554` — Make extended recording alignment-gate summaries report the actual bar count

- Ticket: `RIOTBOX-1554`
- Title: `Make extended recording alignment-gate summaries report the actual bar count`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1554/make-extended-recording-alignment-gate-summaries-report-the-actual-bar`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1554-recording-gate-duration-summary`
- Linear branch: `feature/riotbox-1554-make-extended-recording-alignment-gate-summaries-report-the`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1657](https://github.com/marang/riotbox/pull/1657)
- Merge commit: `b8af65947ef8b3dab3e1729a3a73781883479e13`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: RED/GREEN real publication regression, 21 App recording tests, Core helper/legacy serialization tests, source-free `just ci`; native CI 37064500099 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/execution_roadmap.md`, `docs/specs/session_file_spec.md`
- Follow-ups: RIOTBOX-1036 remained open at implementation closeout. Preserve RIOTBOX-1553's exact hash-bound take and bounded human verdict; no retrospective summary migration or new listening is needed.

## Why This Ticket Existed

RIOTBOX-1553's first real eight-bar V3 take had correct Action/duration, 32-beat proof, 705,600-frame/16-second WAV, receipt and readiness, but its descriptive live_recording_bar_window_alignment gate still claimed two bars. The defect was misleading exported evidence, not capture geometry or a readiness failure.

## What Shipped

- Core keeps the historical two-bar helper and adds a summary helper derived from existing typed LiveRecordingDuration::bars(). Future V3 eight/sixteen-bar and V4 two-bar receipts report their actual 8/16/2-bar duration.
- Legacy V2 with omitted duration identity retains the exact old literal. Stored historical proof/Session/receipt summaries, including RIOTBOX-1553's hash-bound take, are not normalized, rewritten or migrated.
- The owning Session spec and roadmap state this summary-only boundary; no audio, timing, thresholds, schema, callback, DSP, readiness or replay behavior changed.

## Notes

- The original real publication test failed RED on two-bar versus expected 8-bar wording. GREEN coverage includes V3 eight/sixteen, V4 two, the legacy V2 literal and historical V3 restore/readiness with unchanged stored bytes; the Core test also pins gate ID/status/artifact roles and legacy serialization.
- Independent code-review/Rust review and final self-review retained zero findings. The actual owner stays cohesive: only the receipt producer call changes, without mechanical module sharding or new durable app state.
- Reviewed head `ad2ad03f0336794418ea16f03c6c3b49614ada97` passed both existing native jobs. Historical closeout reports clean synchronized main, reviewed-tree equality and removal of only exact merged feature branches.
- Source-free maintenance only; no source/capture/Holdout/commercial audio, host playback or new human verdict. This does not change the prior recording's sound or broaden its qualification.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
