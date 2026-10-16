# `RIOTBOX-1573` — Diagnose recorded silent V3 ALSA/PipeWire buffer error without retesting or relaxing gates

- Ticket: `RIOTBOX-1573`
- Title: `Diagnose recorded silent V3 ALSA/PipeWire buffer error without retesting or relaxing gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1573/diagnose-recorded-silent-v3-alsapipewire-buffer-error-without`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1573-buffer-error-attribution`
- Linear branch: `feature/riotbox-1573-diagnose-recorded-silent-v3-alsapipewire-buffer-error`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1676](https://github.com/marang/riotbox/pull/1676)
- Merge commit: `6edf4ec9f31332dcb9495776907fc7b97bb4ce30`
- Deleted from Linear: `2026-10-16`
- Verification: 12 driver tests, 1 selected AudioRuntime fault projection and 4 corrected telemetry selections PASS; full local `just ci` PASS; exact final native CI 37984429181 PASS; independent reviews/self-review: zero retained findings after P3 test-owner wording correction.
- Docs touched: `docs/reviews/riotbox_1573_buffer_error_attribution_2026-10-09.md`
- Follow-ups: RIOTBOX-1574 (separately authorized instrumented diagnostic seam and observation, subsequently completed); RIOTBOX-1041 remains open; original trigger still requires causal evidence.

## Why This Ticket Existed

RIOTBOX-1572 reached the isolated virtual route but failed on a real generic backend buffer error. Source-free attribution and fail-closed stop coverage were needed before anyone could propose a causal repair.

## What Shipped

- Traced pinned CPAL 0.17.3 ALSA BufferUnderrun reporting/recovery through Audio runtime telemetry, Faulted health, explicit driver stop and retained failed terminal evidence.

- Added generated error injections at samples 1, 36 and 60 proving bounded waits, raw error retention, explicit stop before terminal flush and no retry.

- Committed conditional attribution and the missing evidence requirements without changing production driver/operator/runtime, dependencies, buffers, priorities, configuration or qualification gates.

## Notes

- Exact reviewed head: `43c6d9eaa0f0fbd17427b36c017aee6f68cda55e`. The initial zero-test telemetry selector is explicitly excluded from proof.

- Pinned CPAL/ALSA output EPIPE paths can emit the generic message and may recover after reporting. The historical event lacks raw errno, detecting call site and negotiated geometry; its direction/trigger/root cause was not reproduced or fixed.

- Replaying the then-current retained run-2 transcript through the unchanged validator failed on the same error. That was downstream rejection, not an original-backend reproduction, another host invocation or permission to reopen consumed evidence now.

- AudioRuntime, not Core, owns the selected fault-projection test. The evidence attribution correction was independently confirmed before merge.

- Historical closeout recorded clean synchronized main, identical reviewed changes and exact feature-branch absence locally/remotely. RIOTBOX-1572 stays FAILED/consumed; host/endurance/physical-device/musical qualification did not change.

- This archive preserves historical scope, verification and outcomes only. It grants no new source, host, device, consumed-attempt, playback or trial access.
