# `RIOTBOX-1564` — Bind local DAW writer commits to the queued receipt before publishing proof files

- Ticket: `RIOTBOX-1564`
- Title: `Bind local DAW writer commits to the queued receipt before publishing proof files`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1564/bind-local-daw-writer-commits-to-the-queued-receipt-before-publishing`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `P016 maintenance under RIOTBOX-1036, protecting accepted P023 export handoff; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1564-writer-receipt-admission`
- Linear branch: `feature/riotbox-1564-bind-local-daw-writer-commits-to-the-queued-receipt-before`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1667](https://github.com/marang/riotbox/pull/1667)
- Merge commit: `2fe224692a5217303c190b63b58aa4ca81e05ec6`
- Deleted from Linear: `2026-10-16`
- Verification: Historical four new receipt-admission regressions; broader 45 unit + 12 integration DAW tests; full source-free local just ci; exact-head native CI 37108191274 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1564_local_writer_receipt_2026-10-03.md`, `docs/specs/action_lexicon_spec.md`
- Follow-ups: RIOTBOX-1036 remains the wider export-workflow anchor; RIOTBOX-1565 separately repaired the probe prerequisite for RIOTBOX-1041

## Why This Ticket Existed

RIOTBOX-1563's architecture checkpoint found a pre-existing queue/commit identity mismatch. LocalProjectWriterV1 queued receipt A, but commit and proof construction independently selected the latest receipt. If B appeared before commit, the Action could retain A while files and proof/gate targeted B, violating the existing matching-receipt contract. Initial evidence was a static code/contract trace; this slice then executed the missing generated regression.

## What Shipped

- Before any writer invocation or filesystem effect, commit validates the queued receipt against the current latest DAW receipt. Missing, empty, unknown, wrong-scope, removed, scope-changed or stale identity fails visibly with no writer publication or Session mutation.

- The smallest explicit policy rejects stale selection and requires a deliberate requeue; it does not introduce a historical-receipt writer API or change the writer/plan format. The admitted index is retained for success attachment rather than selecting again.

- Successful retry aligns Action, commit, returned receipt, both written proof JSONs and observer completion to the admitted receipt; a failed stale action is not reported completed. Existing auto-queue and neighboring host-import/audible-proof historical-receipt targeting remain intact.

- RBX-434 and Action Lexicon record the local stale/requeue policy. Existing Action/Session/replay ownership is unchanged, with no schema or runtime-state additions.

## Historical Verification And Outcomes

- Executed baseline RED: queue A -> append B -> commit unexpectedly succeeded. The pre-write guard turned the exact case green.

- Four focused generated regressions covered stale -> reject -> explicit requeue, invalid/missing IDs, removed/changed scope and a later non-DAW control. Rejection compared full Session and generated filesystem directory/file-byte snapshots.

- Retry proved that A remained intact while B owned the Action, commit, returned receipt, both JSONs and observer lifecycle. No proof publication, gate mutation or committed replay evidence was permitted for rejection.

- The separate broader DAW suite passed 45 unit + 12 integration tests, preserving auto-queue, explicit host-import/audible-proof targeting and reporting/observer behavior.

- Both independent adversarial Rust and spec/evidence/product reviewers executed the four new tests from the compiled binary, inspected coordinator RED/GREEN evidence and retained no P0-P3 findings. The cohesive production owner was 482 lines with an ordinary semantic test child, not mechanical shards.

- Full source-free local just ci and native CI 37108191274 passed at reviewed head dc7e324d2bb51d13183ef423a043e6ed9a6babaf; final self-review and PR comment/review checks were clear.

## Notes

- This protects the older developer-only local JSON skeleton, not a new DAW feature, Bitwig setup, actual host import or audibility proof. No real source/Holdout/commercial audio, source discovery, host/DAW/audio/TUI access, playback or Windows expansion occurred.

- Historical closeout synchronized clean main/origin/main and removed exact local/remote feature branches. The Done issue was intentionally retained at that time.

- Subsequent read-only assessment found the legacy CPAL diagnostic's handled-failure exit gap; that was explicitly assigned to a separate Linear-first issue, RIOTBOX-1565, with no host run granted by this ticket.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1564_local_writer_receipt_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
