# `RIOTBOX-1338` P023: Add coherent realtime render-state snapshot handoff

- Ticket: `RIOTBOX-1338`
- Title: `P023: Add coherent realtime render-state snapshot handoff`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1338/p023-add-coherent-realtime-render-state-snapshot-handoff`
- Project: `None recorded in Linear`
- Milestone: `P023, as named by the ticket; no separate milestone recorded`
- Status: `Duplicate`
- Created: `2026-06-29`
- Started: `Not started`
- Finished: `2026-06-29`
- Branch: `None; no implementation branch found`
- Linear branch: `feature/riotbox-1338-p023-add-coherent-realtime-render-state-snapshot-handoff`
- Assignee: `None recorded in Linear`
- Labels: `Audio`
- PR: `None for this duplicate`
- Merge commit: `None for this duplicate`
- Verification: `Linear terminal status/history and empty comments/relations checked; no matching local/remote implementation branch, active worktree or open PR`
- Docs touched: `docs/reviews/realtime_audio_callback_hot_path_audit_2026-06-29.md` (existing duplicate disposition)
- Follow-ups: `RIOTBOX-1327, canonical historical owner; no new implementation requested`

## Why This Ticket Existed

The RIOTBOX-1326 realtime callback audit identified independent relaxed-atomic
lane reads as a coherence risk despite bounded scratch storage and reduced
source Arc cloning. This follow-up proposed a coherent old-or-new render-state
handoff without callback allocation, blocking work or musical-policy changes.

## What Shipped

- No implementation shipped under this duplicate. The existing audit explicitly
  records that RIOTBOX-1338 was closed into RIOTBOX-1327.
- Historical work belongs to [RIOTBOX-1327](./RIOTBOX-1327.md), including its
  [PR #1301](https://github.com/marang/riotbox/pull/1301) and merge
  `032307c62a398182d9d075623c775cc3b7703226`, not to this ticket.
- This archive preserves the duplicate's original scope and terminal reason;
  it does not reopen or requalify that historical implementation.

## Original Scope And Acceptance

- Design the smallest coherent render-state handoff preserving the product
  spine and realtime constraints; prefer a revisioned double buffer or a
  documented seqlock-style snapshot.
- Cover TR-909, MC-202, W-30 preview/resample, Source Monitor and transport
  timing only as far as one coherent slice needs.
- Test partial updates and revision mismatches, proving that each updated
  render group presents either a complete old or complete new state.
- Document the handoff strategy; keep decoding, file I/O, JSON, UI, Sidecar,
  analysis and logging outside the realtime callback.
- Make no musical behavior change without a separately scoped proof.

## Notes

- Original Linear UUID: `4699535a-734a-462d-a3e5-becaf06988ba`.
- State history: Backlog from `2026-06-29T21:57:20.300Z`, Duplicate from
  `2026-06-29T22:03:00.406Z`; cancellation timestamp
  `2026-06-29T22:03:01.838Z`. No implementation-start timestamp exists.
- Linear already archived this ticket on `2026-07-30T00:41:21.186Z`.
  Its last issue update is `2026-06-29T22:03:05.049Z`.
- Source metadata has no project, assignee, attachment, document, parent or
  comment. Relations lists are empty and `duplicateOf` is null. The canonical
  target above is established by the existing repository audit, not invented
  from a missing Linear relation.
- Historical issue priority: High (2). The Linear-generated branch name is
  metadata, not evidence that an implementation branch was created.
- On `2026-10-16`, the user explicitly authorized archiving then deleting
  Canceled/Duplicate tickets. Both exact terminal-state queries included
  Linear's archive and exhausted pagination: zero Canceled, one Duplicate.
  Current Done and active/backlog tickets are outside this cleanup batch.
- Before external deletion, this archive must be merged to main and the live
  terminal state and lack of active ownership checked again. The actual
  deletion outcome belongs in the closeout PR and project journal; no deletion
  date is asserted before the operation succeeds.
