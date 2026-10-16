# `RIOTBOX-1575` — Make live-master recorder lifecycle device-free testable through one production seam

- Ticket: `RIOTBOX-1575`
- Title: `Make live-master recorder lifecycle device-free testable through one production seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1575/make-live-master-recorder-lifecycle-device-free-testable-through-one`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-13`
- Started: `2026-10-13`
- Finished: `2026-10-13`
- Branch: `feature/riotbox-1575-make-live-master-recorder-lifecycle-device-free-testable`
- Linear branch: `feature/riotbox-1575-make-live-master-recorder-lifecycle-device-free-testable`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: [#1678](https://github.com/marang/riotbox/pull/1678)
- Merge commit: `fe92445e87e0f5301877da658a5d44c41599d658`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: 62 preexisting tests PASS before changes; new lifecycle 12 PASS and combined recorder/export 74 PASS (overlapping counts); stack-pressure and stop-projection RED→GREEN; full local `just ci` and exact native CI 38443530920 PASS; independent reviews/self-review: zero retained findings.
- Docs touched: `docs/engineering/module_policy.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1575_recorder_control_lifecycle_2026-10-13.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1036 and RIOTBOX-1041 remain open; RIOTBOX-1576 checkpoint subsequently completed; original RIOTBOX-1572 trigger unresolved.

## Why This Ticket Existed

The live-master CLI combined transport confirmation, capture, fault/timeout handling, stop, finalization and publication in a device-dependent path. A private production seam was needed to test actual recorder-control ordering without claiming an ALSA repair.

## What Shipped

- Extracted one private semantic lifecycle owner used by the real CLI and generated virtual-time/runtime adapters, keeping preflight, Session hydration and device startup in the facade.

- Reused real App/Core queueing, commits, capture tap, writer, receipt, Session save and rollback; preserved all polling budgets, failure precedence, versions and transport ownership.

- Borrowed the large App facade and constructed the shell afterward, avoiding extraction-induced stack pressure; corrected failed-stop summaries to report observed lifecycle rather than hardcoded stopped success.

- Added twelve deterministic lifecycle controls covering confirmation/queue/arming failures, callback faults, incomplete progress/timeouts, stop-before-finish, stop/finalize failures and transport ownership; pinned RBX-442 and owning docs.

## Notes

- Exact reviewed head: `9774908a26da3a54eecadac54d85c4103698b16b`; merge tree was identical. Both independent auditors ran twelve lifecycle controls; one also ran three observer compatibility controls.

- The mistakenly qualified zero-test filter and corrected fixture assumption were excluded from evidence. Extraction-induced extra by-value App-facade stack pressure was repaired without changing stack limits.

- Generated control-path proof is not an actual callback, host, musical, DAW, endurance or original ALSA-trigger pass. No consumed phase was retried and no host/device/service/source/old-attempt/playback access occurred.

- No new action, persistent App state, recorder, runtime, DSP, dependency, recording version, threshold or product policy. Existing Core/Session/publication truth remains authoritative.

- Historical closeout recorded clean synchronized main and exact feature-branch absence locally/remotely; subsequent RIOTBOX-1576 was a current-state checkpoint.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
