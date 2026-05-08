# Jam Recipes

Concrete practice flows for learning Riotbox with the current TUI.

These recipes are meant for the current prototype state:

- the shell is already capable
- the first-run path is intentionally narrow
- different lanes already do different things
- `Log` is still the fastest place to confirm what really landed

Use one recipe at a time. Do not try to memorize the whole keymap first.

## Before You Start

Good starter files from the local example set:

- `data/test_audio/examples/Beat03_130BPM(Full).wav`
- `data/test_audio/examples/Beat08_128BPM(Full).wav`
- `data/test_audio/examples/DH_BeatC_120-01.wav`
- `data/test_audio/examples/DH_RushArp_120_A.wav`

Source choice depends on what you are testing:

- `Beat08_128BPM(Full).wav` is the safest first timing/queue source.
- `Beat03_130BPM(Full).wav` is useful, but current Feral grid listening packs should use explicit `130 BPM`.
- `DH_BeatC_120-01.wav` should use explicit `120 BPM` for Feral grid listening packs.
- `DH_RushArp_120_A.wav` is useful for tonal scene contrast, but not a good TR-909 drum-support Feral grid pack source yet.

Example launch:

```bash
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
```

## Read This Once

Useful screens:

- `1` `Jam`: what is happening now and next
- `2` `Log`: what really committed
- `3` `Source`: what Riotbox thinks the file contains
- `4` `Capture`: what you captured and can reuse

Useful rule:

- unsure whether something worked -> press `2`

## Recipe 1: Learn The Timing Model

Goal: understand quantized commit timing.

1. start Riotbox with `Beat08_128BPM(Full).wav`
2. press `Space`
3. press `f`
4. stay on `Jam` briefly
5. press `2`

What to observe:

- the action is queued before it lands
- it commits on a musical boundary, not instantly
- `wait [..>] next ...` is the compact timing rail for the queued gesture
- `Log` shows the truth more clearly than trying to infer it from one line on `Jam`

## Recipe 2: Compare First Gestures

Goal: learn that the lanes suggest different kinds of change.

Restart the app with the same source and compare these one by one:

- `y` scene jump
- `g` MC-202 follow
- `a` MC-202 answer
- `P` MC-202 pressure
- `I` MC-202 instigator
- `G` MC-202 phrase mutation after `g`, `a`, `P`, or `I` lands
- `<` / `>` MC-202 touch lower / higher after `g`, `a`, `P`, or `I` lands
- `f` TR-909 fill
- `w` W-30 hit

For each run:

1. press `Space`
2. press exactly one gesture
3. inspect `Jam`
4. inspect `Log` with `2`

What to observe:

- each lane has a different feel
- after `g`, `a`, `P`, or `I` commits, the MC-202 lane now has a first bounded bass audio seam in the live runtime
- after `P` commits, the MC-202 lane should hold a sparser offbeat pressure cell rather than the follower drive line
- after `I` commits, the MC-202 lane should shove with a sharper high-register instigator spike rather than the follower drive line
- MC-202 phrases now carry a small note budget; `pressure` stays sparse, `instigate` stays punchy, and follower/answer avoid filling every available step
- when source sections are known, MC-202 also gets a first contour hint (`lift`, `drop`, or `hold`) from the current source/scene section; this is a small interval nudge, not full melody extraction
- on hook-like chorus sections, follower/leader material uses `answer_space` response so MC-202 answers around the hook instead of doubling the hook downbeats
- after the MC-202 line is audible, press `G` to queue a phrase mutation for the next phrase; the Jam card should switch to `variant mutated_drive`
- after the MC-202 line is audible, tap `>` to push the touch harder or `<` to back it off; the Jam card should show the touch value changing
- if you want an offline proof before listening live, run `just lane-recipe-pack local-mc202` and compare `mc202-follower-to-answer`, `mc202-touch-low-to-high`, `mc202-follower-to-pressure`, `mc202-follower-to-instigator`, `mc202-follower-to-mutated-drive`, `mc202-neutral-to-lift-contour`, and `mc202-direct-to-hook-response`
- the first result is easier to compare when you only change one thing per run
- when Riotbox can infer the next scene, the `y` suggestion may name it as `[y] jump <scene> (rise/drop/hold)`
- if source energy is known, the named Scene may skip over an adjacent same-energy section and choose the next contrast section instead
- if there is not enough scene material yet, the same slot may say `[y] jump waits for 2 scenes` instead of pretending a jump is ready
- MC-202 is still not a finished synth engine; treat this as first follower/answer/pressure/instigator/contour/hook-response bass feedback, not full sound design

Current offline MC-202 proof:

```bash
just lane-recipe-pack local-mc202
```

Listen to these files under `artifacts/audio_qa/local-mc202/lane-recipe-listening-pack/`:

- `mc202-follower-to-answer/baseline.wav` vs `candidate.wav`
- `mc202-touch-low-to-high/baseline.wav` vs `candidate.wav`
- `mc202-follower-to-pressure/baseline.wav` vs `candidate.wav`
- `mc202-follower-to-instigator/baseline.wav` vs `candidate.wav`
- `mc202-follower-to-mutated-drive/baseline.wav` vs `candidate.wav`

For machine-readable QA, inspect the generated `manifest.json` in the same directory. It records every recipe case, its baseline/candidate artifacts, comparison report, thresholds, and output metrics, so the listening check is tied to concrete non-silent/non-collapsed audio evidence.

For the current CI-safe observer/audio correlation proof of this MC-202 recipe
path, run:

```bash
just recipe2-observer-audio-gate
```

That gate generates a headless app-level Recipe 2 observer stream (`Space`, `g`,
`a`, `P`, `I`, `G`, and touch raise) and pairs it with a generated lane recipe
listening-pack manifest. It proves the expected key-dispatch / queue / commit
control path and the current MC-202 Recipe 2 output cases, not a live
host-session recording.

The generated observer stream uses the same snapshot envelope and outcome label
helpers as the live `riotbox-app --observer` path. The gate rejects Recipe 2
observer output that omits transport, queue, runtime, or recovery snapshots, so
control-path proof stays comparable to interactive terminal recordings.

## Recipe 3: Capture, Audition, Promote, Hit

Goal: learn the first source-backed W-30 reuse loop.

1. press `Space`
2. press `f` or `g`
3. wait for the result to land
4. press `c`
5. press `4` for `Capture`
6. wait until the capture has committed; use `2` if unsure
7. press `o` to audition the raw captured moment
8. press `p` to promote the capture to the focused W-30 pad
9. wait until promotion has committed; use `2` if unsure
10. press `w` to hit the promoted pad
11. use `1`, `2`, and `4` to compare `Jam`, `Log`, and `Capture`

What to observe:

- Riotbox is not only a mutation shell
- captured material starts becoming reusable W-30 material
- `Capture` is where the shell begins to feel sampler-like
- do not rush `p` immediately after `c`; Riotbox first has to land the capture on a musical boundary
- do not rush `w` immediately after `p`; the pad is only playable after promotion lands
- `Do Next` is the fastest place to read the next capture/promote/hit step
- `1 hear it: [o] audition raw ...` is the raw preview path; it is the quickest way to check that the stored moment is audible
- `2 keep it: [p] promote ...` then `3 play it: [w] hit after promote (src/fallback)` is the reuse path
- while an audition is queued, `Do Next` should say `wait, then hear raw preview` or `wait, then hear promoted preview`
- `hear ... stored src/fallback [o] raw or [p]->[w]` means the capture exists; `[o]` auditions the raw moment, while `[p]` then `[w]` promotes it into a playable W-30 hit
- `.../src` means the W-30 preview is source-backed; `.../fallback` means Riotbox stayed on the safe synthetic preview
- after promotion, `hear ... [w]/[o] src/fallback` means you can trigger it with `w` or audition it with `o`

## Recipe 4: Undo On Purpose

Goal: make undo part of normal use.

1. press `Space`
2. queue one gesture such as `f`, `y`, or `g`
3. wait for it to commit
4. press `u`
5. press `2`

What to observe:

- `undo` is part of experimentation
- Riotbox is designed around trying moves, keeping some, and rejecting others

## Recipe 5: Try More Than One Source

Goal: learn that source material changes the shell feel.

Compare these:

- `Beat03_130BPM(Full).wav`
- `Beat08_128BPM(Full).wav`
- `DH_BeatC_120-01.wav`
- `DH_RushArp_120_A.wav`

Suggested focus:

- drum-heavy loops: better for timing and queue/commit learning
- more tonal loops: better for scene and MC-202 exploration

What to observe:

- Riotbox is currently more legible on some source material than on others
- the source itself is already part of the instrument

## Recipe 6: Watch Jam And Log Together

Goal: stop treating `Jam` as the only truth surface.

1. start transport
2. queue one gesture
3. look at `Jam`
4. switch to `Log`
5. switch back to `Jam`

What to learn:

- `Jam` is for flow
- `Log` is for confirmation
- both matter

## Recipe 7: A Good Beginner Session

If you want one slightly longer practice run:

1. load `Beat08_128BPM(Full).wav`
2. press `Space`
3. press `f`
4. press `2` and confirm the fill landed
5. press `c`
6. press `4`
7. wait until `Capture` shows the stored capture
8. press `o` to audition the raw capture
9. press `p`
10. wait until promotion lands
11. press `w`
12. press `2`
13. press `u`

What this teaches:

- queue
- commit
- capture
- audition
- promote
- reuse
- undo

That is already enough to understand much more of Riotbox than the tiny first-run loop.

What this is not:

- not the best recipe for proving wide sound variety yet
- not a guarantee that `Beat03` and `Beat08` will already feel dramatically different on the `w` step
- not a substitute for `Recipe 2` and `Recipe 5` if you want to learn lane differences and source differences separately

## Recipe 8: Jump Then Restore

Goal: learn the first Scene Brain recovery loop and the contrast between `restore not ready` and `restore ready`.

Use `Beat08_128BPM(Full).wav` or `DH_RushArp_120_A.wav`.

1. press `Space`
2. before pressing anything else, read the suggested jump cue if it is visible:
   - `[y] jump <scene> (rise/drop/hold)`
3. press `y`
4. before the jump lands, notice that `Y` is still only a wake-up cue, not a ready restore
   - `[Y] restore waits for one landed jump`
5. press `2` and confirm the scene jump landed
6. go back to `Jam` with `1`
7. look for the new restore-ready cue:
   - `Scene: restore .../<energy> ready | rise/drop/hold | Y brings back .../<energy>`
   - `[Y] restore ... now`
8. press `Y`
9. press `2` and confirm the restore landed

What to observe:

- the suggested `y` cue is the pre-queue hint; `launch -> ... @ next ... | policy ...` is the queued action once you press `y`
- before the first landed jump, `Y` explicitly waits for one landed jump
- after the jump lands, `Jam` shows that restore is actually ready and names the current restore target
- when both current and restore energies are known, the cue also names whether restore is an energy `rise`, `drop`, or `hold`
- when launch energy is known, Riotbox may choose the next contrast scene rather than the immediately adjacent same-energy scene
- if Riotbox cannot infer the next launch target, the suggested jump cue falls back to the generic `[y] jump`; if it knows there are too few scenes, it says `[y] jump waits for 2 scenes`
- `Log` is the clearest place to verify both the queued restore target and the committed restore result

If you want one more short loop:

1. press `1`
2. press `y` again
3. let the new jump land
4. press `Y` again

What this teaches:

- Scene Brain already has one explicit `jump -> restore` pair
- restore is deterministic and pointer-based, not a hidden undo mode
- the shell now distinguishes `restore waits for one landed jump` from `restore ready now`
- scene changes are becoming recoverable without opening a second arrangement model

## Recipe 9: Compare Two Scene Sources

Goal: feel how source choice changes Scene Brain legibility.

Use these two sources back to back:

- `Beat08_128BPM(Full).wav`
- `DH_RushArp_120_A.wav`

Run the same loop on both:

1. press `Space`
2. press `y`
3. press `2` and confirm the scene jump landed
4. press `1`
5. press `Y`
6. press `2` and confirm the restore landed

What to compare:

- how easy it is to notice that a scene jump happened at all
- whether the restore feels like a clear “go back” move or only a status change
- whether `Jam` plus `Log` together make the loop understandable without guesswork

What to expect right now:

- `Beat08_128BPM(Full).wav` is still the easier source for learning timing and queue/commit behavior
- `DH_RushArp_120_A.wav` is currently the clearer source for reading Scene Brain contrast as an actual musical shift
- neither source makes Scene Brain feel finished yet, but the tonal example is the better one when you want to learn `jump -> restore` as contrast instead of only as action ids

What this teaches:

- source choice already changes how readable the same Scene Brain flow feels
- Riotbox is not only action-driven; the source itself still shapes what the shell teaches well today
- the best learning source for one seam is not automatically the best learning source for every seam

## Recipe 10: Read The Scene Cues On Purpose

Goal: practice the current Scene Brain readability stack instead of only firing `jump` and `restore` blindly.

Use `DH_RushArp_120_A.wav` if you want the clearest contrast, or `Beat08_128BPM(Full).wav` if you want the steadier timing feel.

1. press `Space`
2. on `Jam`, first read the suggested jump gesture:
   - `[y] jump <scene> (rise/drop/hold)`
3. queue that jump with `y`
4. before it lands, look for all three of these cues:
   - `launch -> ... @ next ... | policy ...`
   - `pulse [..>.] ...`
   - the current `live .../<energy> <> restore .../<energy>` contrast line
5. press `2` after the jump lands
6. on `Log`, confirm the scene result and note the `trail ...` cue
7. still on `Log`, check the TR-909 render line:
   - `render source_support | accent scene` means the landed Scene target is getting the subtle TR-909 support accent
   - `... | <profile> / scene_target` means TR-909 source support is following the landed Scene target
   - `... | <profile> / transport_bar` means TR-909 source support is still following the current transport bar's source section
   - `accent off fallback` means Riotbox stayed on the safe transport-bar fallback and did not apply the Scene-target accent
8. press `1`
9. on `Jam`, confirm the restore-ready cue now appears:
   - `Scene: restore .../<energy> ready | rise/drop/hold | Y brings back .../<energy>`
   - `[Y] restore ... now`
10. queue restore with `Y`
11. before restore lands, read the same three `Jam` cues again
12. press `2` after restore lands and confirm the new `trail ...` entry
13. still on `Log`, check the TR-909 render line again:
   - `render source_support | accent scene` after restore means the restored Scene target is now getting the same subtle support accent
   - if restore falls back to `transport_bar`, Riotbox kept the safer source-section timing instead of forcing a Scene-target accent

What to observe:

- `[y] jump <scene> (rise/drop/hold)` is a suggested next move; it is not yet queued
- `launch -> ... @ next ...` tells you the boundary the queued scene action is waiting for
- `policy rise/drop/hold | 909 ... | 202 ...` tells you the bounded transition intent before it becomes a fuller arranger
- after a Scene move lands, `move rise/drop/hold 909 ... 202 ...` tells you the replay-safe movement that is now shaping the landed Scene
- `pulse [..>.] ...` is the compact countdown cue toward that boundary
- `live .../<energy> <> restore .../<energy>` tells you which scene is active, which scene `Y` would return to, and whether restore means going up, down, or sideways in energy
- the restore-ready cue is the positive mirror of the earlier `restore waits for one landed jump` state
- if Riotbox cannot infer the next launch target, the suggested jump cue falls back to the generic `[y] jump`; if it knows there are too few scenes, it says `[y] jump waits for 2 scenes`
- if Riotbox cannot infer both energy sides, the restore-ready cue falls back to the older target-only shape without `rise/drop/hold`
- `trail ...` on `Log` is the fastest way to reconstruct whether the last scene move was a jump or a restore
- `scene_target` is a render-state diagnostic, not a promise of a finished transition engine; it means the TR-909 support profile is now using the target Scene's projected source section
- `accent scene` is a small audible support lift tied to that `scene_target` context; after restore, the target is the restored Scene, not the previous live Scene
- Scene movement now also shapes TR-909 phrase variation and MC-202 contour/touch, so restore is no longer expected to collapse exactly back to the pre-jump buffer
- `transport_bar` is the safe fallback; it means the Scene id could not be mapped to a source section, so the TR-909 profile still follows the current bar

What this teaches:

- Scene Brain now has a real small readability stack, not just action ids
- `Jam` is enough to follow the next scene move, while `Log` is enough to confirm what actually landed
- TR-909 support can now be inspected as Scene-coupled or transport-bar-driven instead of guessed by ear
- Scene transition policy is now explicit before the move, and the landed movement is persisted after the move; this is still a bounded MVP movement seam, not a finished full arranger
- the current shell can already teach `jump -> restore` timing and role contrast, even though it is still not a finished visual timing instrument

Low-energy contrast note:

- the regression vocabulary now also covers `drop/high <> break/low`, not only `intro/medium <> drop/high`
- read `break/low` as a quieter recovery or decompression target, not as a finished arranger promise yet
- if you see `live break/low <> restore drop/high`, Riotbox is telling you the restore path crosses an energy boundary downward and can later return to the stronger drop
- this is currently most useful for checking whether the Scene Brain cues are legible; the musical transition quality is still part of the Scene Brain buildout

## Recipe 11: Check Source-Backed W-30 Reuse

Goal: test whether the W-30 path is using captured source material or the safe fallback preview.

Use a local WAV example, preferably:

- `Beat08_128BPM(Full).wav`
- `Beat03_130BPM(Full).wav`

Start with:

```bash
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
```

Then run this loop:

1. press `Space`
2. press `f` or `g`
3. wait for the result to land
4. press `c`
5. press `4` for `Capture`
6. wait until the capture has committed; `Do Next` should move from queued capture to raw audition / promote guidance
7. press `o` to audition the raw captured moment
8. press `p` to promote the capture
9. wait until promotion has committed; `Do Next` should offer the promoted hit/audition path
10. press `w` to hit the promoted W-30 pad
11. press `2` for `Log`
12. switch between `1`, `2`, and `4` to compare `Jam`, `Log`, and `Capture`

Optional resample reuse check:

13. after a promoted W-30 pad is working, press `e` to print a W-30 resample on the next phrase
14. wait until the resample lands as a new capture
15. press `p` again to explicitly promote that printed resample into the focused W-30 pad
16. wait until promotion commits, then press `w` to hit the resampled pad

What to observe:

- `audition raw/src` means raw audition is using captured source material
- `audition/src` means promoted audition is source-backed
- `recall/.../src` or `prev recall/src` means the promoted hit or recall path is source-backed
- on `Jam`, a source-backed raw audition can show `src: [o] raw source | 4 Capture`
- on `Jam`, a source-backed promoted audition can show `src: [o] source | 4 Capture`
- on `Jam`, a source-backed live recall can show `src: [w] source | 4 Capture`
- `Capture -> Do Next` should explain the audible handoff while actions are queued, for example `wait, then hear raw preview`
- `Log` can also show `win 1.25-3.75s src-1`; that is the source excerpt backing the current W-30 cue
- `.../fallback` means Riotbox is still using the safe synthetic preview for that path
- `fallback: [o] raw safe | 4 Capture` and similar `fallback:` Jam cues mean the action is still playable, but it is using the safe preview instead of decoded source-window material
- `fallback` is not automatically a bug; it means the current session did not have a decoded source-window preview available for that cue

What this teaches:

- `[o]` is the quickest way to test the raw captured moment
- `[p]` promotes the capture into the W-30 reuse path
- after `[e]` prints an internal resample, `[p]` is also the explicit gesture that assigns that printed resample to the focused W-30 pad
- `[w]` tests the promoted hit / recall path
- if you press the next key before the current action commits, you are mostly testing queue state, not the audible result yet
- Riotbox now exposes whether that path is source-backed, but it is still a bounded preview excerpt, not a full W-30 sampler engine

## Recipe 12: Follow The Feral-Ready Path

Goal: learn the current Feral-ready cue without expecting a full break-rebuild engine yet.

Use a local WAV example, preferably:

- `Beat08_128BPM(Full).wav`
- `Beat03_130BPM(Full).wav`

Start with:

```bash
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav"
```

Then:

1. press `Space`
2. stay on `Jam`
3. read the header and `Suggested gestures`
4. if you see `feral ready`, try one promoted gesture: `j`, `f`, `g`, `a`, or `c`
5. wait for the queued action to land
6. press `2` for `Log`
7. press `3` for `Source`
8. compare the `feral ready` / `feral needs support` line with what landed

What to observe:

- `feral ready` means the Source Graph has high break-rebuild potential plus supported hook or capture evidence
- `feral needs support` means Riotbox sees promising material, but the shared `supports_break_rebuild` evidence is missing
- on `Jam`, `feral ready` promotes existing keys such as `[j] browse`, `[f] fill`, `[g] follow`, `[a] answer`, and `[c] capture`
- `j` exercises the current W-30 slice-pool preference path
- `f` exercises the current TR-909 support/fill path
- `g` and `a` exercise the current MC-202 follow/answer path
- `c` captures the moment so you can reuse it through Recipe 11

What this teaches:

- Feral readiness is not a separate mode
- it is a policy cue that steers existing W-30, TR-909, MC-202, and capture paths
- this is still bounded prototype behavior, not full automatic feral composition
- if it sounds too similar, use `Log` and `Source` to check whether you were in `ready`, `needs support`, or fallback territory before judging the gesture

## Recipe 13: Prove W-30 Source-Backed Audio Is Not Fallback

Goal: get one quick offline proof that the W-30 source-backed preview can differ from the safe synthetic fallback.

This is not an interactive TUI recipe. Use it when you are unsure whether a W-30 result is really using source material or only the fallback preview.

Run the CI-safe generated check:

```bash
just w30-smoke-generated-source-diff
```

Expected result:

- the command renders a synthetic fallback `baseline.wav`
- it renders a source-backed `candidate.wav` from deterministic generated source material
- it compares both metrics and fails if the RMS / sum deltas are too small
- it validates the generated `manifest.json` and all referenced artifact paths

What to look for in the output:

- `result: pass`
- `rms ... delta` at or above `0.001000`
- `sum ... delta` at or above `1.000000`
- `valid riotbox listening manifest v1`

If you want a local listening file from one of the downloaded example WAVs, use the source-diff wrapper with an explicit date label:

```bash
just w30-smoke-source-diff "data/test_audio/examples/Beat03_130BPM(Full).wav" local-w30-source-diff
```

Then listen to:

```text
artifacts/audio_qa/local-w30-source-diff/w30-preview-smoke/raw_capture_source_window_preview/baseline.wav
artifacts/audio_qa/local-w30-source-diff/w30-preview-smoke/raw_capture_source_window_preview/candidate.wav
```

How to interpret it:

- `baseline.wav` is the safe fallback preview
- `candidate.wav` is the source-backed preview
- if they sound identical and the command still passes, create a follow-up with the metrics and source file
- if the command fails, Riotbox may have collapsed to fallback or the source window may not be useful enough for the current threshold

What this teaches:

- `src` / `fallback` is not only UI language; there is now an output-path check for it
- the Jam `src:` cue is the fast live check; Recipe 13 is the offline proof when the live path still sounds suspicious
- the current source-backed W-30 path is a short preview excerpt, not full sample streaming
- use this recipe before judging W-30 by a confusing first TUI run

## Recipe 14: Probe The First-Playable Jam Path

Goal: run one CI-safe control-plus-output check for the first source-backed Jam loop without relying on a live audio device.

Run:

```bash
just first-playable-jam-probe
```

Expected result:

- the probe renders W-30 fallback baseline and source-backed candidate WAVs from deterministic synthetic material
- it validates the generated `manifest.json`
- it generates and validates an app-level observer probe for `space`, `c`, `o`, `p`, and `w`
- it correlates that generated control evidence with the W-30 output evidence
- it fails if the output is silent, missing, or collapsed back to the fallback-like source-diff metrics

What this proves:

- the current first-playable path has committed control evidence
- capture / raw audition / promote / promoted hit intent is present in generated observer evidence
- the W-30 output seam is source-backed enough to produce measurable candidate audio and source-vs-fallback delta

What it does not prove yet:

- full live TUI usability
- device-level playback on the host
- finished sampler/sequencer behavior

## Recipe 15: Render A Feral Grid Pack With Honest BPM

Goal: generate one local Feral grid audio pack without pretending auto timing is stronger than it is today.

This is not an interactive TUI recipe. Use it when you want WAVs to audition what the current offline Feral grid seam can render.

Start with one explicit-BPM example:

```bash
just feral-grid-pack "data/test_audio/examples/Beat03_130BPM(Full).wav" local-beat03-feral-grid 130.0 8 1.0 0.0
```

Then listen in this order:

```text
artifacts/audio_qa/local-beat03-feral-grid/feral-grid-demo/stems/01_tr909_beat_fill.wav
artifacts/audio_qa/local-beat03-feral-grid/feral-grid-demo/stems/02_w30_feral_source_chop.wav
artifacts/audio_qa/local-beat03-feral-grid/feral-grid-demo/03_riotbox_source_first_mix.wav
artifacts/audio_qa/local-beat03-feral-grid/feral-grid-demo/04_riotbox_generated_support_mix.wav
```

Try these variants:

```bash
just feral-grid-pack "data/test_audio/examples/Beat08_128BPM(Full).wav" local-beat08-feral-grid-auto auto 8 1.0 0.0
just feral-grid-pack "data/test_audio/examples/DH_BeatC_120-01.wav" local-dh-beatc-feral-grid 120.0 8 1.0 0.0
```

How to interpret `auto`:

- `grid_bpm_source: source_timing` means the current readiness report drove the
  grid because it was ready and did not require manual confirmation.
- `grid_bpm_source: static_default` means auto mode fell back to the static default grid.
- `grid_bpm_decision_reason` explains why the source was selected or why
  `static_default` was used, for example `source_timing_requires_manual_confirm`.
- `source_timing.policy_profile: dance_loop_auto_readiness` means the pack used the stricter dance-loop auto-trust policy, not the broader diagnostic timing policy.
- `source_timing.bpm_agrees_with_grid: false` means the generated pack is timing-risky and should not be judged as a successful beat-grid example.
- `source_timing.readiness: weak` means the detector saw useful evidence but not enough downbeat/phrase confidence for automatic trust.
- `metrics.tr909_source_grid_alignment.hit_ratio` is the lane-specific TR-909 support proof; values below `0.5` mean the drum support is not landing reliably near the chosen grid.
- `metrics.source_grid_output_drift.hit_ratio` is an early generated-output smoke metric; values below `0.5` mean the support render is not landing reliably near the chosen grid.

You no longer need to inspect only raw JSON for the first timing read. The generated
`README.md` and `grid-report.md` also show compact source timing readiness,
downbeat, phrase, and warning lines.

Current local benchmark result:

- `Beat03_130BPM(Full).wav`: use explicit `130 BPM`; auto currently falls back to `128 BPM`.
- `Beat08_128BPM(Full).wav`: BPM is close in auto mode, but readiness is still weak.
- `Beat20_128BPM(Full).wav`: similar to Beat08.
- `DH_BeatC_120-01.wav`: use explicit `120 BPM`; auto fallback is musically misleading.
- `DH_RushArp_120_A.wav`: do not use this path for Feral grid drum-support examples yet; it needs a separate melodic/source-chop showcase path.

What this proves today:

- Riotbox can generate a bounded TR-909 plus W-30 Feral grid pack with manifest-backed output metrics.
- The pack is useful for listening and QA, not yet a full composition/export workflow.
- Explicit BPM is still the honest path for some real examples until source timing reaches `ready`.

## Current Limits

The current prototype is still not a finished “load a loop and instantly get a polished remix” instrument.

So if two runs feel similar:

- try a different source
- try a different lane
- use `Recipe 2` before repeating `Recipe 7`
- use `Recipe 5` if you want to understand source-specific differences
- use `Recipe 8` if you want the first bounded Scene Brain flow instead of only lane gestures
- use `Recipe 9` if you want to compare where Scene Brain is already more legible today
- use `Recipe 11` if you want to check whether W-30 capture reuse is source-backed or on fallback
- use `Recipe 12` if you want to understand the new `feral ready` gesture path
- use `Recipe 13` if you want an offline W-30 source-vs-fallback proof before judging the live TUI path
- use `Recipe 14` if you want a CI-safe first-playable control-plus-output probe
- use `Recipe 15` if you want a local Feral grid listening pack and need to choose `auto` versus explicit BPM honestly
- use capture/reuse instead of only the first fill
- look at `Log` to understand what actually happened

The shell already supports more exploration than the minimal quickstart suggests. These recipes are the best way to get past that first narrow loop.
