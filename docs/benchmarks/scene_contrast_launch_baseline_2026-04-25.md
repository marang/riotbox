# Scene Contrast Launch Baseline 2026-04-25

This is the first bounded readability baseline for Scene Brain launch target selection after `RIOTBOX-226`.

It covers one specific behavior:

- when the immediately adjacent next scene has the same known energy as the live scene
- and a later deterministic candidate has different known energy
- `y` may target the contrast candidate instead of the adjacent same-energy scene

This is not a full arranger benchmark. It is a small target-policy baseline for keeping the current `Jam` cue language understandable.

## Setup

Use a source/session fixture with at least three projected scenes:

- `scene-01-drop` with `high` energy
- `scene-02-chorus` with `high` energy
- `scene-03-intro` with `medium` energy

Start with:

- active scene: `scene-01-drop`
- current transport scene: `scene-01-drop`
- no pending Scene launch or restore

## Expected Jam Readability

The immediately adjacent scene is `scene-02-chorus`, but it has the same `high` energy as the live drop.

The preferred launch target should therefore be:

- `scene-03-intro`
- compact label: `intro/medium`
- suggested gesture direction: a drop from `high` to `medium`

Expected cue shapes:

- Suggested gesture: `[y] jump intro (drop)`
- Overview: `next scene intro/med`
- Queue result after pressing `y`: `launch -> scene-03-intro @ next bar`
- Log confirmation after commit: a landed `scene.launch` targeting `scene-03-intro`

## Pass Criteria

- A performer can tell from `Jam` which scene will launch before pressing `y`.
- The chosen target appears intentionally contrast-based, not random.
- The same target appears in the queued action and committed Log result.
- If energy is missing or `unknown`, Riotbox falls back to deterministic scene order instead of pretending to know a contrast.

## Notes

- This baseline relies on current text cues, not a finished graphical arranger.
- The policy stays deterministic and replay-safe because it only uses committed session scenes plus Source Graph section energy.
- Future arranger work can become stronger, but should not make this basic contrast target unreadable.
