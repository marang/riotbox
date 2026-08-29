# `RIOTBOX-1485` Record the real live Riotbox master callback as a Session-owned WAV

- Ticket: `RIOTBOX-1485`
- Title: `Record the real live Riotbox master callback as a Session-owned WAV`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1485/record-the-real-live-riotbox-master-callback-as-a-session-owned-wav`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-28`
- Started: `2026-08-28`
- Finished: `2026-08-29`
- Branch: `feature/riotbox-1485-record-the-real-live-riotbox-master-callback-as-a-session`
- Linear branch: `feature/riotbox-1485-record-the-real-live-riotbox-master-callback-as-a-session`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1504 (https://github.com/marang/riotbox/pull/1504)`
- Merge commit: `1e37c22c7feb9a3c831ee750c05bf76a1901c87f`
- Deleted from Linear: `2026-08-29`
- Verification: `focused live-recording and transport-boundary tests passed; textual-include guard, fmt, diff check, live-master smoke, and full just ci passed; formal code/Rust re-review found no surviving findings; GitHub rust-ci passed; real user-session eight-beat callback capture completed without callback overruns or capture errors`
- Docs touched: `docs/engineering/textual_include_allowlist.txt, docs/engineering/textual_include_inventory_2026-06-29.md, docs/execution_roadmap.md, docs/research_decision_log.md, docs/specs/action_lexicon_spec.md, docs/specs/audio_core_spec.md, docs/specs/audio_qa/listening_review.md, docs/specs/session_file_spec.md, docs/workflow_conventions.md`
- Follow-ups: `RIOTBOX-1036 remains the P016 backlog anchor for broader full-arrangement, stem, and live-recording export workflows.`

## Why This Ticket Existed

RIOTBOX-1036 still lacked an honest live-recording boundary from the actual running Riotbox callback; the reserved action contract could not produce a musician-usable Session-owned recording.

## What Shipped

- Added bounded eight-beat capture of the exact post-limiter engine samples from the real CPAL callback through a preallocated non-blocking realtime tap.
- Published a no-clobber float32 WAV and versioned proof outside realtime audio before canonical Action, Session receipt, replay, and observer commit.
- Recorded host/device, timing, scene, Source Graph, capture-lineage, duration, callback-gap, overflow, stream-error, silence, clip, readback, and hash evidence with fail-closed gates.

## Notes

- The audio behavior itself did not change, so the real listening pass established source-to-live-master preservation rather than a new DSP or musical-quality claim.
- The initial review module-policy finding was fixed by replacing new textual includes with real Rust modules; a repeat formal review found no surviving findings.
