# Source Showcase False-Positive Review 2026-05-03

Scope:

- generated Feral grid source-showcase packs from the 2026-05-03 local review
- source files including Beat03 / Beat08 / Beat20 and related drum-chop cases
- current manifest, RMS, reproducibility, and listening-pack gates

## Verdict

The fixed 2026-05-03 source-showcase pack is not acceptable as a representative
Riotbox musical example even though the previous machine gates could pass.

The failure was not silence or nondeterminism. The failure was source diversity:
different input files could produce materially similar full mixes because a
source-independent generated stem dominated the source-backed W-30 material.

## Evidence To Preserve

- TR-909 beat/fill stems grouped by BPM and grid rather than by source identity.
- 130 BPM Beat03 cases could share the same TR-909 stem.
- 128 BPM Beat08 and Beat20 cases could share the same TR-909 stem.
- 120 BPM drum-hybrid cases could share the same TR-909 stem.
- Full-mix RMS stayed close across sources while W-30 source-chop RMS varied.
- The full mix weighted generated TR-909 support strongly enough that the source
  chop could be masked.

The old hard gates were useful but incomplete:

- non-silence proved sound existed
- reproducibility proved the same run was stable
- manifest validation proved artifacts were connected to a pack
- none of those proved that different source files led to audibly different
  Riotbox results

## Decision

Do not present the 2026-05-03 fixed source-showcase output as musically
representative.

Future source-showcase packs must distinguish:

- reproducibility within the same source
- unwanted sameness across different sources
- optional common generated support
- source-independent generated support that is too dominant

## Current Guard

`scripts/validate_source_showcase_diversity.py` adds the first lightweight
manifest-level blocker. It rejects:

- identical full-mix artifact hashes across distinct source values
- identical source-backed artifact hashes across distinct source values
- identical generated stems across distinct source values when their RMS is too
  high relative to source-backed material

The command is intentionally conservative. It blocks the known false-positive
class while later tickets make TR-909 support source-aware and improve W-30
source-chop articulation.

## Boundary

The earlier MC-202 hardcoded phrase removal remains separate. That fixed one
hardcoded phrase path; this review captures the broader QA failure where a pack
can still look valid while its source-specific musical identity is weak.
