# Routine Audio Output Audit 2026-04-26

Context:

- ticket: `RIOTBOX-298`
- trigger: pause feature buildout and validate the documented musician routines with the `riotbox-development` skill
- scope: README quickstart plus `docs/jam_recipes.md` Recipes 1-11
- rule: do not claim an audio-producing routine works from UI, log, or state assertions alone

## Summary

Riotbox now has enough real coverage to prove the current shell is not only a log demo, but the proof is uneven across routines.

Strongest today:

- queue / commit timing is covered by app tests and visible in the shell
- W-30 capture / audition / promote / hit has both state coverage and source-backed output checks
- TR-909 has callback-side audio regression coverage for render-state behavior
- Scene Brain has state / log / UI readability coverage for jump and restore

Weakest today:

- Recipe-level runs are not yet replayed end-to-end by an automated TUI harness
- MC-202 is still mostly state / phrase-generation proof, not a musician-audible output proof
- Scene Brain audio is only indirectly covered through TR-909 support accent, not through a full transition listening pack
- the README quickstart can still imply more immediate audible source playback than the current engine actually provides

## Verification Run

Commands run:

```bash
cargo test -p riotbox-app queues_first_live_safe_jam_actions
cargo test -p riotbox-app renders_jam_shell_with_first_run_onramp
cargo test -p riotbox-app w30_fixture_backed_committed_state_regressions_hold
cargo test -p riotbox-app scene_fixture_backed_committed_state_regressions_hold
cargo test -p riotbox-audio fixture_backed_tr909_audio_regressions_hold
cargo test -p riotbox-audio w30_preview
just w30-smoke-source-diff 'data/test_audio/examples/Beat03_130BPM(Full).wav' audit-beat03 0.0 0.25 2.0
just w30-smoke-source-diff 'data/test_audio/examples/Beat08_128BPM(Full).wav' audit-beat08 0.0 0.25 2.0
```

Results:

- all listed tests passed
- `audit-beat03` W-30 source-vs-fallback comparison passed
- `audit-beat08` W-30 source-vs-fallback comparison passed

Important W-30 comparison values:

```text
audit-beat03:
active_samples: 173280 -> 75018 | delta 98262 | ok
peak_abs: 0.020045 -> 0.106279 | delta 0.086234 | ok
rms: 0.005921 -> 0.007061 | delta 0.001140 | ok
sum: 675.616089 -> 1.891887 | delta 673.724202 | ok
result: pass

audit-beat08:
active_samples: 173280 -> 160250 | delta 13030 | ok
peak_abs: 0.020045 -> 0.098271 | delta 0.078226 | ok
rms: 0.005921 -> 0.010245 | delta 0.004324 | ok
sum: 675.616089 -> -15.223730 | delta 690.839819 | ok
result: pass
```

Generated local artifacts:

```text
artifacts/audio_qa/audit-beat03/w30-preview-smoke/raw_capture_source_window_preview/
artifacts/audio_qa/audit-beat08/w30-preview-smoke/raw_capture_source_window_preview/
```

## Routine Matrix

| Routine | Main promise | Control-path proof | Output-path proof | Status |
| --- | --- | --- | --- | --- |
| README quickstart | load, start, queue gesture, inspect Log | `queues_first_live_safe_jam_actions`, first-run UI test | no direct source playback proof; only lane-specific render seams | partially proven |
| Recipe 1 timing | queued action lands on musical boundary | queue / commit tests and timing regressions | not primarily audio-producing | proven for control |
| Recipe 2 first gestures | different lanes suggest different moves | queue tests, lane fixture regressions | TR-909 audio covered; W-30 audio covered; MC-202/Scene not fully audible as recipe outputs | partially proven |
| Recipe 3 capture / audition / promote / hit | captured source becomes W-30 material | W-30 app fixture and source-window preview tests | W-30 preview tests plus Beat03/Beat08 source-vs-fallback checks | proven for current W-30 preview seam |
| Recipe 4 undo | committed action can be undone | undo test coverage in app | not primarily audio-producing | proven for control |
| Recipe 5 source comparison | source choice changes shell feel | source ingest and scene/source tests exist | W-30 source-vs-fallback checked for Beat03/Beat08 only | partially proven |
| Recipe 6 Jam and Log together | Jam is flow, Log is truth | UI/log snapshot tests | not primarily audio-producing | proven for control/readability |
| Recipe 7 beginner session | queue -> commit -> capture -> audition -> promote -> hit -> undo | covered by combined W-30/control tests, not one end-to-end TUI replay | W-30 output seam covered, not full recipe replay | partially proven |
| Recipe 8 jump then restore | Scene jump / restore loop is readable | Scene fixture and UI readability tests | no full Scene transition listening pack yet | proven for state/readability only |
| Recipe 9 compare scene sources | source changes Scene Brain legibility | source and Scene state tests support the claim | no automated audio comparison for Scene contrast yet | partially proven |
| Recipe 10 scene cue reading | boundary / pulse / trail / TR-909 context cues are readable | Scene UI/log fixture coverage | only TR-909 support-accent seam is audio-covered | partially proven |
| Recipe 11 source-backed W-30 reuse | W-30 path shows `src` and differs from fallback | W-30 state and UI tests | Beat03/Beat08 source-vs-fallback output checks | proven for current W-30 preview seam |

## Findings

### 1. W-30 is the best-audited musician path today

The source-backed W-30 preview now has the right proof shape: state/provenance tests plus output-path comparison against synthetic fallback. The current seam is still a bounded `2048`-sample preview, not a full sampler engine, but it is no longer only a log assertion.

### 2. TR-909 has audio proof, but not recipe-level replay proof

The audio crate proves TR-909 render-state behavior through fixture-backed audio regressions. The gap is higher-level: Recipe 2 and Recipe 10 are not yet rendered as named listening-pack cases from their documented key sequences.

### 3. MC-202 is still mostly internally proven

MC-202 follower and answer generation are covered as state/phrase behavior. A musician-facing output proof is still weak because there is no dedicated MC-202 audible render seam or listening pack tied to Recipe 2 / Recipe 5.

### 4. Scene Brain is readable, but not yet musically audited

Scene jump / restore has solid state, log, and UI cue coverage. The audible consequence is still indirect through TR-909 support context/accent, so the docs should keep calling this a bounded readability stack rather than a finished transition engine.

### 5. The README quickstart is honest but still fragile

The README already warns that the first loop can sound similar. That is good. The remaining risk is expectation: pressing `Space` does not mean the source file itself is playing as a normal audio track. The current audible paths are lane/render seams, especially W-30 preview and TR-909 support.

## Follow-Ups

Already tracked:

- `RIOTBOX-297`: opt-in user-session and audio observer for ambiguous TUI/audio QA

New follow-up needed:

- add a lane-level routine listening-pack harness for documented recipes outside the W-30 preview path

Recommended scope for that follow-up:

- render named cases for TR-909 fill/support/takeover
- add at least one MC-202 audible contract or explicitly document why the seam is not audible yet
- add Scene Brain jump/restore output comparison tied to the existing TR-909 support accent seam
- keep generated audio artifacts local under `artifacts/audio_qa/`

## Audit Conclusion

Current Riotbox is already testable as a prototype shell, but not all documented routines carry the same proof strength.

Use this status language going forward:

- W-30 source-backed reuse: proven for the current bounded preview seam
- TR-909 lane behavior: audio-regression proven at render-state level, recipe-level listening pack still missing
- MC-202 lane behavior: state-proven, musician-audio proof still missing
- Scene Brain: state/UI-readable, audio transition quality still not fully audited
- README quickstart: useful for learning queue/commit, not proof of broad musical variety
