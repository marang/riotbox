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
- `Log` shows the truth more clearly than trying to infer it from one line on `Jam`

## Recipe 2: Compare First Gestures

Goal: learn that the lanes suggest different kinds of change.

Restart the app with the same source and compare these one by one:

- `y` scene jump
- `g` MC-202 follow
- `a` MC-202 answer
- `f` TR-909 fill
- `w` W-30 hit

For each run:

1. press `Space`
2. press exactly one gesture
3. inspect `Jam`
4. inspect `Log` with `2`

What to observe:

- each lane has a different feel
- the first result is easier to compare when you only change one thing per run

## Recipe 3: Capture And Reuse

Goal: learn the `capture -> promote -> hit` loop.

1. press `Space`
2. press `f` or `g`
3. wait for the result to land
4. press `c`
5. press `4` for `Capture`
6. press `p`
7. press `w`
8. use `2` and `4` to watch both `Log` and `Capture`

What to observe:

- Riotbox is not only a mutation shell
- captured material starts becoming reusable W-30 material
- `Capture` is where the shell begins to feel sampler-like

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
7. press `p`
8. press `w`
9. press `2`
10. press `u`

What this teaches:

- queue
- commit
- capture
- promote
- reuse
- undo

That is already enough to understand much more of Riotbox than the tiny first-run loop.

What this is not:

- not the best recipe for proving wide sound variety yet
- not a guarantee that `Beat03` and `Beat08` will already feel dramatically different on the `w` step
- not a substitute for `Recipe 2` and `Recipe 5` if you want to learn lane differences and source differences separately

## Recipe 8: Jump Then Restore

Goal: learn the first Scene Brain recovery loop.

Use `Beat08_128BPM(Full).wav` or `DH_RushArp_120_A.wav`.

1. press `Space`
2. press `y`
3. press `2` and confirm the scene jump landed
4. go back to `Jam` with `1`
5. press `Y`
6. press `2` and confirm the restore landed

What to observe:

- `Y` is only meaningful after Riotbox already has a committed previous scene to restore to
- `Jam` shows the current restore target before you fire it
- `Log` is the clearest place to verify both the queued restore target and the committed restore result

If you want one more short loop:

1. press `1`
2. press `y` again
3. let the new jump land
4. press `Y` again

What this teaches:

- Scene Brain already has one explicit `jump -> restore` pair
- restore is deterministic and pointer-based, not a hidden undo mode
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

## Current Limits

The current prototype is still not a finished “load a loop and instantly get a polished remix” instrument.

So if two runs feel similar:

- try a different source
- try a different lane
- use `Recipe 2` before repeating `Recipe 7`
- use `Recipe 5` if you want to understand source-specific differences
- use `Recipe 8` if you want the first bounded Scene Brain flow instead of only lane gestures
- use `Recipe 9` if you want to compare where Scene Brain is already more legible today
- use capture/reuse instead of only the first fill
- look at `Log` to understand what actually happened

The shell already supports more exploration than the minimal quickstart suggests. These recipes are the best way to get past that first narrow loop.
