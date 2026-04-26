# W-30 MVP Gap Review 2026-04-26

Scope:

- ticket: `RIOTBOX-316`
- review type: current-state W-30 MVP exit review via `review-codebase`
- phase: `P007 | W-30 MVP`
- primary definition of done: `docs/phase_definition_of_done.md`

Reviewed paths:

- `crates/riotbox-app/src/jam_app.rs`
- `crates/riotbox-app/src/jam_app/capture_helpers.rs`
- `crates/riotbox-app/src/jam_app/projection.rs`
- `crates/riotbox-app/src/jam_app/side_effects.rs`
- `crates/riotbox-app/src/jam_app/w30_targets.rs`
- `crates/riotbox-audio/src/runtime.rs`
- `crates/riotbox-audio/src/w30.rs`
- `docs/benchmarks/w30_preview_smoke_listening_pack_2026-04-26.md`
- `docs/reviews/routine_audio_output_audit_2026-04-26.md`

## Summary

W-30 is the strongest musician-facing path in Riotbox today, but Phase 5 is not exit-clean yet.

Current W-30 work already proves the right shape:

- captures are represented in the session with source-window and provenance metadata
- promoted captures can target W-30 pads
- W-30 focus, trigger, audition, browse, freeze, and resample actions use the existing queue / commit seam
- source-backed W-30 preview has output proof against fallback-like collapse
- local W-30 smoke rendering exists for manual listening and metric checks

The remaining MVP gap is not "no W-30 exists"; it is that the current implementation is still a preview-and-lineage seam, not yet a small playable sampler workflow.

## Phase 5 Done Check

| Done item | Current state | Status |
| --- | --- | --- |
| useful loops can be captured | `CaptureRef` records source origin/window and session lineage, but the app does not write real `captures/*.wav` files for committed captures | partial |
| pads are playable | user can focus/trigger/audition a W-30 pad, but audio runtime renders one focused preview state, not an independent pad-bank sampler | partial |
| internal bus resampling works | `promote.resample` creates lineage-safe captures and a resample tap render state, but does not record the internal music bus into reusable audio | partial |
| captured material can be reused without leaving flow | promoted/source-window captures can be recalled and previewed in flow | mostly true for current preview seam |
| provenance for captured material is not lost | source origins, source windows, lineage refs, generation depth, pin, and target metadata are persisted | true |

## Findings

### 1. W-30 pads are still a single preview seam, not a pad-bank playback engine

Severity: blocker for Phase 5 exit

The current runtime has one `W30PreviewRenderState` with one `active_bank_id`, one `focused_pad_id`, one `capture_id`, one `trigger_revision`, and one optional fixed `2048`-sample source-window preview. The app can queue pad-like actions, but the callback renders whichever focused preview state is current.

Impact:

- pressing the W-30 hit key proves a retriggered preview, not an independent playable pad bank
- multiple pads can exist as metadata, but they are not independently loaded sample voices
- bank / pad UX risks feeling cosmetic until playback ownership is per pad

Next implementation should introduce a bounded sampler-facing pad playback state rather than widening the existing diagnostic preview forever. Keep it small: start with focused-pad playback from source-window material, explicit trigger revision, duration/loop policy, and output metrics proving the result differs from fallback.

### 2. Capture storage is metadata-only for the live app path

Severity: blocker for "useful loops can be captured"

Committed capture actions create `CaptureRef.storage_path` values such as `captures/cap-01.wav`, but the app path does not write a real capture WAV at that location. Existing WAV writers are test helpers or QA render helpers, not the normal committed capture side effect.

Impact:

- the session can say a capture exists while no reusable audio file exists beside the session
- capture provenance is strong, but capture ownership is still indirect through `source_window`
- future export, reload, and user trust will be weaker if the visible capture path is not backed by an artifact

The next bounded implementation can write a source-window-backed capture WAV for committed capture actions. That does not need realtime bus recording yet; it can render from the decoded source window outside the audio callback and persist the artifact promised by `storage_path`.

### 3. Internal resampling is lineage-safe, but not yet a real bus print

Severity: blocker for "internal bus resampling works"

`promote.resample` is a real action and its commit materializes a new `CaptureRef` with lineage and incremented generation depth. The W-30 resample tap also produces an audible diagnostic layer. However, no current path records the internal mixed output into a new reusable audio artifact.

Impact:

- resample is musically suggestive but still representational
- generated resample captures inherit source-window metadata instead of containing the processed audio result
- a musician cannot yet "print the chaos" and then reuse that exact audio as a pad

The right next step is not a large DAW-style recorder. Keep it bounded: define an offline internal-bus print seam for one W-30 source capture, write the resulting audio artifact, and prove the printed output differs from both the raw source window and the synthetic fallback.

### 4. Output proof exists, but only for the current bounded preview

Severity: important, not a blocker by itself

The W-30 preview smoke pack and source-vs-fallback comparisons are valuable and should remain. They prove the current preview seam is not silent and does not collapse to the synthetic fallback. They do not prove full pad playback, capture-file persistence, or internal bus printing.

Impact:

- future W-30 work should reuse the existing proof shape
- each new audible seam still needs its own control-path and output-path proof
- docs and PRs should keep saying "current W-30 preview seam" until pad playback and bus prints exist

## Recommended Next Slice

Create the next implementation ticket as:

`Write source-backed W-30 capture WAV artifacts for committed captures`

Why this first:

- it turns the existing `storage_path` contract into something real
- it is smaller than a full sampler engine or internal-bus recorder
- it gives musicians a concrete artifact for captured loops
- it provides the storage foundation needed by later pad playback and resampling

Suggested acceptance checks:

- committed `CaptureBarGroup` / `W30CaptureToPad` with a source window writes a PCM WAV at `CaptureRef.storage_path`
- the written WAV duration matches the captured source window within a small tolerance
- output metrics prove the file is not silent and not the synthetic fallback
- session reload can still point at the same capture artifact
- no file I/O happens in the realtime audio callback

## Exit Criteria For Phase 5

Before closing W-30 MVP, require these additional reviews or implementation proofs:

- capture WAV artifact path exists for normal committed captures
- focused W-30 pad playback is source-backed and duration-aware beyond the fixed `2048`-sample preview
- at least one W-30 hit / audition / recall recipe has an automated control-path plus output-path replay test
- internal resample can create a reusable audio artifact that differs from raw source and fallback
- docs state clearly what the user should press, what they should hear, and where the captured audio lives

## Conclusion

W-30 should continue as the next product spine because it is already the most audible and best-audited path. The immediate mistake to avoid is declaring the MVP done because preview, target state, and lineage are working. The next slices need to convert those seams into real capture files, then playable pad ownership, then internal bus prints.
