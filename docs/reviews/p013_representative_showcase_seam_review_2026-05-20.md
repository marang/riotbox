# P013 Representative Showcase Seam Review

Date: 2026-05-20

## Scope

Review-codebase pass for the current P013 representative showcase seam on
`main`, covering:

- `crates/riotbox-audio/src/bin/feral_grid_pack.rs`
- `crates/riotbox-audio/src/bin/feral_grid_pack/`
- `scripts/generate_representative_source_showcase.sh`
- `scripts/validate_representative_showcase_musical_quality.py`
- `docs/benchmarks/representative_source_showcase_2026-05-07.md`

This review is not a diff review and does not include the open RIOTBOX-807
branch.

## Findings

### 1. MC-202 is not yet a first-class source-grid drift lane

- Location: `crates/riotbox-audio/src/bin/feral_grid_pack/pack_builder.rs:440`
- Category: scope
- Severity: major
- Title: MC-202 output can pass the showcase without lane-specific source-grid alignment proof
- Description: The pack renders a dedicated MC-202 bass-pressure stem at
  `pack_builder.rs:414` and records MC-202 render metrics at
  `pack_builder.rs:451`, but the source-grid alignment report is built only from
  TR-909, W-30, and the generated-support mix. The drift helper itself exposes
  only `tr909_source_grid_alignment`, `w30_source_grid_alignment`, and aggregate
  `source_grid_output_drift` in `source_grid_output_drift.rs:61`. That means a
  weak or drifting MC-202 stem could be hidden by stronger grid-locked drum or
  chop peaks in the full mix.
- Suggestion: Add `mc202_source_grid_alignment` to `SourceGridAlignmentReport`,
  `PackReport`, manifest metrics, report text, and manifest smoke assertions;
  then make the musical-quality validator reject candidates whose MC-202 stem
  does not meet the current source-grid hit-ratio budget.

### 2. `feral_grid_pack` is still mechanically split rather than semantically owned

- Location: `crates/riotbox-audio/src/bin/feral_grid_pack.rs:1`
- Category: scope
- Severity: minor
- Title: Include shards reduce immediate file size but keep one implicit module boundary
- Description: The binary root uses textual `include!` shards. The current
  review cost is manageable, but the largest shards are again above or near the
  soft 500-line budget: `pack_builder.rs` is 566 lines and `render_stems.rs` is
  549 lines. Because the split is textual, all shard-local helpers still share
  one namespace and visibility boundary, which makes future lane-proof additions
  easier to couple accidentally.
- Suggestion: When the next behavior change needs to touch these files, prefer a
  narrow semantic conversion rather than another include shard. A useful first
  boundary would be a real `manifest` or `pack_io` module owning manifest structs,
  artifact records, and pack writing, leaving render policy in lane-specific
  modules.

### 3. The local showcase generator has an intentional destructive output reset

- Location: `scripts/generate_representative_source_showcase.sh:9`
- Category: scope
- Severity: minor
- Title: Showcase reruns delete the requested output directory without a guardrail
- Description: The generator starts with `rm -rf "$output_dir"` after resolving
  the user-supplied path. This is acceptable for the default ignored artifact
  path, but the script is also used directly in review commands with arbitrary
  `/tmp/...` paths. A typo such as an empty wrapper variable or a too-broad path
  would delete more than the intended local showcase output.
- Suggestion: Add a small path guard before deletion, allowing only paths under
  `artifacts/audio_qa/` or `/tmp/riotbox-*` unless an explicit force flag is
  passed.

## Healthy Boundaries Observed

- The representative showcase stays outside realtime audio and writes ignored
  local artifacts only.
- The current pack records explicit source-first and generated-support mixes,
  preserving the source-masking boundary.
- The musical-quality validator is framed as a review gate, not automatic taste
  scoring.
- Observer/audio correlation is present in the generator and keeps the pack from
  being audio-only evidence.

## Recommended Follow-ups

1. Add MC-202 lane-specific source-grid alignment proof before deeper MC-202
   musical variation.
2. Add a deletion guard to the representative showcase generator before broader
   use outside `/tmp` and ignored artifact paths.
3. Convert the next touched `feral_grid_pack` hotspot into a real semantic
   module boundary instead of extending the textual include pattern.
