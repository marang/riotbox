# RIOTBOX-1598 — W-30 live transport phase

Date: 2026-10-16. Review base: merged main
`9cc99445a53647417fb2aad5e7a0980e2c162c9a` (RIOTBOX-1599).
P017 maintenance protecting P023 live transport/recall under RIOTBOX-1041.
The follow-up branch adopted the reviewed1599 head, then joined its tree-identical
squash main through ordinary merge `966aaeae2001b46ff44a351c38a7fa936d84def2`.
No reset/rebase or unrelated working-tree changes. The review diff is1598 only.

## Failure and repair

The shared callback clock correctly acknowledged a live position request, but
already-active W-30 preview/resample scheduling retained its prior private beat
position. The earlier1596 witness reached only legacy numeric offline ingress;
its source trace established live reachability, not executed callback proof.

Three successive exact-callback RED→GREEN verticals establish the actual repair:

1. Running Seek12.25, unchanged pad/trigger, PitchDive starting at0. Public
   acknowledgment reaches12.35, but pre-fix output stays nonzero instead of the
   frozen terminal silence from relative beat12.
2. Independent Rust/Adversarial review identified stopped audition advancing its
   local phase while the shared clock stays stationary. Stopped Seek11.99,
   ten stopped samples at1000Hz, then Play/Preserve wrongly produces complete
   terminal silence instead of a nonzero preterminal prefix and later silence.
   Ten samples complete the existing5ms Stop fade; no subsequent stopped callback
   resets activation before Play. This does not assume or alter a20ms Stop fade.
3. Ordinary Preserve with no new request, scratch overflow, then a normal block:
   transport correctly advances1.5→13.6→13.7, but pre-fix W-30 still renders its
   old preterminal phase. The overflow itself is finite zero and reported as a
   scratch fault; there is no claim that lost frames were rendered.

RBX-459/460/461 and Audio Core§7.1.2 freeze each rule before its corresponding
production change. One private helper beside the existing renderer's step
calculations aligns preview/tap beat positions and destination step cursors.
It does not cold-reset samples, age, envelope, trigger, material, character or
gesture history. A seek alone does not manufacture an immediate step attack;
ordinary subsequent steps, activation, explicit triggers and material changes
retain their existing behavior. Existing gesture active/inactive/terminal resets
still run normally at the new committed-relative position.

The live callback applies alignment for a new identity, the first rendered
running block after stopped rendering, or recovery after any scratch overflow.
Its last-applied marker and last-rendered running flag are local bookkeeping,
not a clock, public acknowledgment or persisted/App state. Scratch rejection
does not consume alignment; recovery uses the actual current callback position,
not the old requested target. Ordinary uninterrupted running Preserve/tempo and
stopped audition do not rebase every callback. No shared mixer or public numeric
offline interface changes; the new helper has only the live callback as caller.

The existing cohesive renderer stays visibly counted legacy; adding the narrow
helper next to its existing profile/step rules avoids copied musical logic or
a mechanical file-size split. Test setup is a private generated callback fixture,
not a second instrument, renderer, production test constructor or host launcher.
The already657-line callback test parent gains only nine child-module
declarations; the new semantic test owners remain at most322 lines. This does
not mechanically redistribute pre-existing tests to hide the parent's size.

## Retained evidence

Each original RED compiled and ran exactly one test, then the coordinator
independently repeated it before the corresponding production correction:

- `/tmp/riotbox-1598-w30-callback-red.log`, SHA-256
  `18952491c1a8799eafef867448e3327150a1a525b555c4d086a607000af637a2`.
  Same-age Preserve/PitchDive and dry Seek controls each have49 nonzero samples;
  Seek/PitchDive wrongly also has49. Repeat: `w30-callback-root-red.log`.
- `/tmp/riotbox-1598-w30-resume-red.log`, SHA-256
  `32d84549be6ef59e63c4fdfb0d50e3a7ca64f8d3c338a8c89be534976c4e6589`.
  Dry resume49 positive; PitchDive prefix/total0 despite correct11.99→12.09
  shared timing. Repeat: `w30-resume-root-red.log`.
- `/tmp/riotbox-1598-w30-overflow-red.log`, SHA-256
  `c525a30a2bb6f7d35e8233b3d96622bf070f6b7a358d8d0c435612cc56db08eb`.
  Prepared scratch50, oversized6050 samples allocated before measurement;
  dry/PitchDive recovery each49 nonzero after acknowledged13.6.
  Repeat: `w30-overflow-root-red.log`.

All three pass afterward in the matching `/tmp/riotbox-1598-*-green.log` files.
The exact processor is the closure installed by CPAL, invoked here directly with
generated buffers, not a device stream. Every contributor besides the named W-30
lane is silent; Source Monitor is Riotbox-only. Assertions cover finite PCM,
old/new request acknowledgment where applicable, unchanged published material/
trigger, overflow silence/health, and zero measured callback allocations,
reallocations and deallocations. Fixture/control preparation occurs outside the
measurement; no WAV/source/capture is opened or generated as a product source.

Additional controls use varying generated marker PCM rather than constant
samples, so cursor, attack, envelope and filter-delay resets are observable:

- `w30_timing_controls`: repeated, sub-step, forward/backward and numerically
  identical new requests preserve equally aged pad PCM until an actual natural
  destination step. FilterSlam reaches its frozen hold, preserves filter delay
  against a same-age Preserve control, and returns sample-exactly to dry.
  Ordinary Preserve/tempo refresh and stopped-audition/Play keep sample
  continuity; a long backward PitchDive request returns from terminal silence
  to the pre-dive dry phase. Five separately selected controls pass, then all
  eight current callback tests pass in
  `/tmp/riotbox-1598-w30-active-controls-all.log`.
- Source-window-only preview and resample-tap-only controls each preserve
  bit-exact same-age prefixes after Seek5.95, then distinguish the natural
  destination trigger near beat6. Published material remains unchanged;
  acknowledged positions, finite PCM and zero callback heap are asserted.
  Each selected one test and passed in
  `/tmp/riotbox-1598-w30-source-window-controls.log` and
  `/tmp/riotbox-1598-w30-resample-controls.log`.
- The public numeric Preview regression preserves the original1596 witness's
  forward/backward byte-identical no-seek comparisons, with finite/nonzero and
  distinct cold-target positive controls. Exactly one test passed in
  `/tmp/riotbox-1598-public-numeric-compat-executed.log`. The earlier short-name
  `--exact` command selected zero tests and is not execution or RED evidence.
- Pending, superseding and repeated-request overflow exercises a positive
  PitchDive prefix at11.95 followed by terminal silence. Repeating11.95 and
  skipping two callbacks advances the clock to12.35; recovery must be silent,
  not replay the old11.95 prefix. Dry recovery stays positive, all skipped
  outputs are finite zero, health counts three overflows, and the latest
  acknowledgment reaches12.45. Exactly one selected test passes in
  `/tmp/riotbox-1598-w30-overflow-request-controls-strong.log`. The earlier
  all-pre-dive2.25 variant checked delivery but could not distinguish stale
  from advanced render phase; coordinator self-review strengthened that oracle
  rather than treating its green result as phase proof.
- Active PitchDive history remains bit-exact against same-age Preserve after
  a fresh request at the current8.35 cursor, and differs from marked dry PCM.
  A separate callback lifecycle preserves the existing5ms fade/latch, real
  manual-trigger attack, PCM-only publication with unchanged trigger and
  noncold Play. Each selected one test and passed in
  `/tmp/riotbox-1598-w30-active-pitch-history-small-setup.log` and
  `/tmp/riotbox-1598-w30-stopped-controls-small-setup.log`.

Initial multi-control FilterSlam/PitchDive/stopped test setups exhausted an
ordinary test thread's stack before assertions. Boxing the exact prepared
callback before heap measurement, plus small private fixture-construction and
phase helpers, removes large temporary pressure. Every final control passes on
ordinary default test-thread stacks; production preparation and callback heap
policy are unchanged. These setup failures are not additional semantic REDs.

Independent Rust/Adversarial/Maintainer and Spec/Evidence/Product/Risk follow-up
reviews retain no P0–P3 findings in the stable production/tests/contracts. The
coordinator's final self-review checks sample-bit compatibility, phase-causal
oracles, preserved DSP, live-only helper reachability, scratch admission,
ownership, typed intent and access/claim limits.

## Verification and gate separation

All13 actual-callback W-30 controls pass together, zero failures/ignores:
`/tmp/riotbox-1598-w30-all-callback-controls-final.log`. Both public numeric
Preview/Tap controls subsequently pass together with explicit per-sample
`to_bits()` equality: `/tmp/riotbox-1598-public-numeric-bits-final.log`.

Full `cargo test -p riotbox-audio --all-features` passes546 tests across25
unit/integration/doc-test suites in `/tmp/riotbox-1598-full-audio.log` (unit
suite368 passes). One pre-existing manual release-mode snapshot benchmark
remains ignored; no new ignored test. The later bit-equality assertion
strengthening is covered by the separate final two-test run above and normal
full CI. A first format check caught only new module ordering; corrected
format passes in `/tmp/riotbox-1598-format-corrected.log`. Strict
`cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
in `/tmp/riotbox-1598-clippy-final.log`. These are separate gates, not semantic
RED evidence.

Same-runtime restore is composed seam evidence: the existing App
`transport_sync::tests::same_runtime_restore_delivers_the_fractional_session_cursor_before_old_timing`
proves restore→explicit fractional Reposition before stale observations through
a fake control; these new tests prove actual unchanged-material callback
delivery separately. This is not a new through-going App→CPAL/device E2E run.
Existing lower-renderer Stop/Tap/fade, gesture-curve/partition, snapshot-cache
and transport intent tests supplement, not replace, exact-callback controls.

Full source-free local `just ci`, exact-head native PR CI/review, merge, main
sync and issue/branch closeout remain mandatory independent workflow gates.
Their executed revision/run/status evidence is recorded in this slice's PR and
RIOTBOX-1598 closeout; it is not inferred from focused tests or this report.

## Scope and claim limits

This preserves a performer's committed transport/articulation timing without
turning Seek into a new pad attack. It does not add a sound mechanism, phrase,
source intelligence, style or human taste verdict. Generated PCM is output-path
regression evidence, not listening or real-host qualification.

No new ActionCommand, Session/replay/recording/observer schema, App state, clock,
queue, shared atomic protocol, public offline plan, dependency or source/DSP
recipe/threshold. No actual user Session migration, source/capture/Holdout/
commercial/ignored-attempt access or discovery, host/device/service observation
or runtime launch, consumed-phase repetition, playback, DAW or Windows/TUI
expansion. No human, musical, source-general, hardness, physical latency,
endurance or release claim.1041/1036 and the original ALSA cause remain open;
passed/consumed1589 is unchanged.1599 is separately completed through PR1704.
