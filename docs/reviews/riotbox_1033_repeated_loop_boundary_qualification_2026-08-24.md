# RIOTBOX-1033 Repeated-Loop Boundary Qualification

- Date: 2026-08-24
- Result: bounded Development qualification passed
- Product role: suggested downbeat-phase correction with explicit musician confirmation
- Holdout access: none
- Audio playback: none in this ticket

## Frozen change

RBX-318 and Source Timing Intelligence spec v0.2 freeze
`source-timing-probe.repeated-loop-boundary-prior.v1` before fresh source
qualification. The cue may reorder an already-ambiguous primary toward phase
zero only when the file contains two through eight complete bars, phase zero is
already within the existing ambiguity margin, the boundary contains an onset,
and every later bar passes the frozen onset-layout and accent-strength
similarity gates. It cannot raise confidence, remove alternate hypotheses, or
confirm a grid automatically.

Source-blind Core coverage proves the positive repeated-two-bar case and
rejects incomplete bars, missing boundary onsets, non-repeating layouts,
mismatched accents, and a clear non-boundary winner. The complete
`riotbox-core` test suite and strict Core Clippy passed before the access
session opened any source.

## Access boundary

Final session `20260824T143110Z` opened only the five preregistered Development
paths below after exact path, case ID, and SHA-256 comparison against the v2
and v3 rotation metadata found no Holdout collision. No source directory,
Holdout audio, or commercial reference was opened or discovered.

The local access log is
`artifacts/development/riotbox-1033/access-log-20260824T143110Z.json`, SHA-256
`03f773b65a8a93bbcbf21b574b1181c20d035abb847e9ead2d41931c9eba6571`.
An earlier session was superseded after self-review found incomplete provenance
on alternative BPM hypotheses. The correction changed only metadata, not phase
selection or thresholds; the complete matrix was rerun from a fresh access log.

## Exact Development result

| Case | Family | Primary phase | New cue | Product handling |
|---|---|---:|---|---|
| `weak_source_beat20_128` | weak source / bad-timing reference | `0` (previously `2`) | yes | ambiguous, short-loop manual confirm |
| `oga_cinameng_can_be_so_beautiful` | dense break | `0` | no | ambiguous, short-loop manual confirm |
| `oga_marwan_cinematic_percussion` | sparse drums | `0` | no | ambiguous, short-loop manual confirm |
| `oga_fupi_plimplom` | tonal riff | `1` | no | ambiguous, short-loop manual confirm |
| `oga_frosty_ham_osdrums` | electronic drums | `2` | no | ambiguous, short-loop manual confirm |

Only Beat20 activates the new cue. Its exact live Source Graph moves bar one
from approximately `0.9346` seconds to `0.0`, retains the former phase as one
of three alternatives, adds the frozen provenance token, and replaces the
generic warning with a specific explanation that repeated full-bar structure
suggests the boundary but still requires confirmation. The graph has SHA-256
`d5310d5acbe6d350802138976832d04e31ef6614ead7fba2024454b692c9313c`.

The exact observer reports phase `0`, `ambiguous`,
`short_loop_manual_confirm`, and `degraded / needs_user_confirmation`.
Transport remains stopped, no generated lane is configured, and no fallback
music exists. Restore preserves the same Source Graph and Session hashes and
the same timing and safety state. The initial and restart observers have
SHA-256 `271e75448a829696a10b627302e7d38aa44bba4404de2a8d8b14fac085df1b21`
and `12fd1fe887ed67ffaed1a8b9e36a7c4c4c2b26e3d68e61ba743d090cf1cccaeb`.

A broad-research CLI comparison was not treated as product evidence because it
uses a different policy surface. Qualification used the exact Jam live-ingest,
Source Graph, Session, observer, and restart path named by the issue.

## Human timing alignment

No repeated listening was requested. Beat20's audio bytes and audible
contributors are unchanged from the exact source-only RIOTBOX-1459 review, and
the listening contract says technical-only timing corrections do not justify
replaying unchanged audio. The listener's existing observation is normalized
as: the file boundary is probably the musical downbeat. The newly qualified
phase-zero suggestion is consistent with that observation while its remaining
uncertainty is preserved through explicit confirmation.

The bound timing-alignment record is
`artifacts/development/riotbox-1033/timing-review-20260824T141657Z.json`,
SHA-256
`15877dd64ffb4c47b9642ca19cf9d95ec8cc42f197be20aeb1b433314716d46e`.

## Outcome

The detector explanation is now more accurate for the exact Beat20 case; the
four contrasting registered cases retain their prior primary phases and do not
activate the cue. This passes the bounded
RIOTBOX-1033 Development and product-handling scope. It does not grant an
automatic grid lock, production-grade arbitrary-audio timing, source-general,
Holdout, sound-quality, demo, release, or P023-completion claim. The rejected
RIOTBOX-1459 family records remain immutable; any promotion must be rebuilt
from the corrected current product state.
