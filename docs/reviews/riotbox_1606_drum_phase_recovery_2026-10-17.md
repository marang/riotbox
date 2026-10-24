# RIOTBOX-1606 — Live drum phase after silent scratch overflow

Date: 2026-10-17. Classification: maintenance/regression, not a new audible
mechanism or source-qualification attempt. Origin: the executed RIOTBOX-1603
checkpoint finding. Base: `fc81975d5032c37c0ce9846073e774e6f0f9fca6`.

## Purpose and ownership

Protect the existing P023 playable/recording path: after a rejected audio block,
drums must follow the transport again instead of staying behind the musician's
other lanes. No new performer command or UI is introduced. Overflow remains
silent and faulted; recovering output does not make a faulted take publishable.

RBX-467 and Audio Core §7.1.2 freeze the repair before production edits. One
private default-false flag belongs to the existing `Tr909CallbackState`, not
App/Core/Session truth. Only production scratch rejection sets it. The existing
Takeover and Fill renderers first apply their unchanged activation/seek/mode/
large-jump resets, then consume pending alignment before rendering samples.
The inactive branch clears pending alongside its normal position update.

Keep `last_step` and existing sound state after short/deadband recovery. A
same-step recovery does not fabricate an attack; a different destination step
uses the existing comparison once. Never replay skipped steps or simulate
unheard voice aging. Existing explicit-seek/activation and large-jump resets
remain authoritative; the non-retriggering claim is not a blanket replacement
of those policies. In particular, Fill's existing full-step jump reset must
clear hidden voices before recovery into a new-bar rest.

Offline states never receive this flag. Public plans, historical ingress,
thresholds, curves, mixer signature, shared clock/acknowledgment, W-30 safety,
capture fault semantics and all persisted schemas remain unchanged. No added
ActionCommand, queue, renderer, dependency or `JamAppState` field.

## Executed control/output evidence

All new output tests use the existing prepared production callback, its public
shell controls, generated isolated drum PCM and zero-heap measurement. W-30,
MC-202 and source contributors are Idle/absent; monitoring is Riotbox-only.
Preparation/publication and output-buffer allocation occur outside measured
callback invocations. No device is created.

- Owning RED: mono1000Hz/120BPM, fixed50-frame scratch, warm to beat0.4,
  overflow51 frames to0.502, recover50 to0.602. Before the fix, recovery is
  sample-for-sample identical to the stale no-overflow0.4→0.5 interval.
  `/tmp/riotbox-1606-short-overflow-red.log`: one actual test, one intended
  assertion failure, exit101. SHA256
  `a4370a59990183d5cb35170e178f8da2dddb8de2677672be2bc27d32a51d2ce7`.
- Same owning command GREEN after the repair: one actual test passed in
  `/tmp/riotbox-1606-short-overflow-green.log`. SHA256
  `1aa296a26e4c5ac371fb86a90102e690bcfe9ec4590ce0cacd12ae33bea6c0d1`.
- Four focused recovery tests passed in
  `/tmp/riotbox-1606-overflow-tests.log`: the owning Takeover case; short Fill
  with synthetic source-bar anchor0.125; exact same-age PCM through a same-step
  short gap followed by an on-time natural half-beat attack; and two Preserve
  overflows at60BPM whose combined0.102beat gap stays below both old deadbands.
- Prior-bar Fill→rest and repeated overflow→superseding Seek controls passed
  in `/tmp/riotbox-1606-overflow-controls.log`. The first retains the existing
  large-jump reset: sounding pickup3.85→3.95, silent155-frame overflow to4.26,
  then exact silence through4.36 without old tails or replayed downbeat.
  The second checks Takeover/Fill acknowledgments, unchanged material/counters
  and exact PCM against an equally warmed latest-Seek control.
- Ordinary output compatibility passed in
  `/tmp/riotbox-1606-running-compat.log`: Takeover, GenericFill and both current
  mainline PhraseDrive/HardCut Fill families;1000/48000Hz;mono/stereo; ordinary
  Preserve and120→90BPM. Each actual callback sample matches the unchanged
  public historical numeric offline sequence bit-for-bit. This is parity, not
  an independent musical-quality oracle.

The stopped/Idle→resume/activation control checks observable PCM, timing and
heap behavior, not an internal flag side channel. A forgotten internal clear
can be observationally equivalent at immediate activation; the trivial clear
is also inspected directly in code review, without inventing a second test API.

Root's Test Evidence check found and corrected an overbroad initial comparator:
the control rendered additional active oscillator time during the gap and then
assumed the inactive branch erased oscillator phase. Existing Takeover reset
does not make that promise. The final control advances only its clock in Idle,
retaining the same rendered oscillator age as the candidate. The first broad
run (`/tmp/riotbox-1606-audio-lib.log`,385passed/1failed/1ignored) is a failed
fixture run, not a product finding or pass. The corrected exact control really
passes in `/tmp/riotbox-1606-inactive-control-corrected.log`; no production
behavior was changed for that test assumption. Final fixtures use the normal
typed Takeover/Fill bus routes.

## Verification and review gates

Full Audio library:386passed,0failed,1existing ignored manual snapshot benchmark
in `/tmp/riotbox-1606-audio-lib-final.log`. All eight new test functions execute,
including the six inactive/resume/mode combinations. Existing frozen public
offline sample controls, W-30 phase/final-stop and callback/CaptureTap overflow
fault regressions pass in that same run. SHA256
`47f787ae9545c23383fa6473b498e84d1cf5d1ead98efef5cb5672a95fff5625`.
Formatting, `git diff --check`, Decision Log fixtures and strict all-target/
all-feature Clippy (`/tmp/riotbox-1606-clippy.log`) pass. Full exact-head
source-free `just ci` plus native PR CI remain merge gates;
existing broader tests may hydrate their own generated temporary WAV/Graph/
capture fixtures. They do not access the user's source or Session data.

Independent Spec And Evidence Auditor, Adversarial Implementation Reviewer and
Risk Assessor review:0open production/spec findings; the corrected comparator
was confirmed, not independently rediscovered. Root applies Maintainer/Product
Pragmatist/Rust and Test Evidence lenses plus a short self-review:0open findings.
Existing
cohesive legacy renderer/state files remain531/1065/533lines with only10/4/3
new lines; no mechanical split or new public ownership is warranted. The new
test owners remain bounded, semantic children of the actual callback seam.
Review/CI/merge/main synchronization and issue/branch closeout status belong
in the PR and Linear; this document does not predeclare those passes.

## Boundaries

## 2026-10-24 integration refresh

The paused implementation is retained, not rebuilt. Current main
`1fbd38fe65540a07b2142268ed8abd2898281f5e` adds the independently qualified
security/dependency maintenance (including CPAL0.18.2, quick-xml0.42 and
SHA-2 0.11). The only merge conflict is the append-only Decision Log:
retain the exact RBX-466 text before the exact RBX-467 text. No production
algorithm, threshold or historical control is changed during integration.

Fresh focused execution in `/tmp/riotbox-1606-current-overflow.log` passes
all nine selected actual-callback tests (the eight new recovery/parity
functions plus existing Fill/CaptureTap proof), not an empty exact filter.
Full exact-head source-free local/native CI and refreshed independent
Rust/spec review remain required before merge. Existing listening-review
technical-correction rules do not automatically require another musical
verdict for this unchanged sound recipe; no historical verdict is transferred
to a new artifact and no real-host/release qualification is claimed.

### Access and claim boundaries

New tests use generated metadata/in-memory PCM only. No actual Session
migration, source/capture/Holdout/commercial/ignored material access/discovery,
host/device/service/TTY operation or playback. Existing full CI's generated
fixture hydration is a separate bounded proof, not real-source evidence.
No human/musical/source-general/hardness/physical-latency/endurance/release
qualification follows. No listening artifact or human verdict was requested.
RIOTBOX-1589 stays passed/consumed;1041/1036 and the original ALSA cause remain
open. This repair closes one confirmed live-output timing defect, not P023.
