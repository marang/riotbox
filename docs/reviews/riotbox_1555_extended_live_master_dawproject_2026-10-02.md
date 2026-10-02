# RIOTBOX-1555 — extended live-master DAWproject handoff

Date: 2026-10-02
Classification: P016 contract enabler under RIOTBOX-1036, supporting the accepted
P023 recorder/export journey. Contract: RBX-424.

## Purpose and boundary

The musician can hand an already committed, ready eight-/sixteen-bar recording
to the existing `just live-master-dawproject <session> <destination> [observer]`
command. One complete recorded master becomes a 32-/64-beat clip at beat zero,
using the recording's tempo and exact WAV bytes. This is not editable stems,
an arbitrary performance recorder, a new musical transformation, or a DAW-host
import/output qualification. RIOTBOX-1036 remains open; its real-host DAW
import/output follow-up still requires the user's DAW setup.

No real source/capture/Holdout/commercial audio, source-directory discovery,
audio runtime, human playback or Windows expansion belongs to this slice.
Generated PCM is implementation evidence only. Previous listening judgments
are not transferred to these archives.

## Contract and ownership

- Core adds the DAW V3 boundary/pack and optional typed DAW Action duration.
  New V3 Action, receipt, proof and recording source must agree on EightBars or
  SixteenBars. Existing DAW V1/V2 duration keys remain omitted and their source
  versions, geometry and bytes are unchanged.
- Queue selection checks the latest eligible recording boundary before
  readiness, then pins unique source receipt, version and duration. Invalid
  newest evidence cannot select an older take; later recordings cannot redirect
  the queued export. Unavailable version selection does not claim DAW V1.
- Admission opens only the pinned recording's exact regular WAV/proof files
  once each; hashing, decoding, metrics and embedding use the same bytes.
  Existing version-specific timing/frame authority and stored lineage apply.
- Core validates new V3 Action/receipt/source identity and geometry during
  metadata-only restore/replay. No source or capture hydration, graph write,
  recapture or archive regeneration is performed.
- The existing archive publisher owns the same four members, XML/readback,
  no-clobber publication and owned-artifact rollback. No ActionCommand,
  JamAppState product truth, arrangement model or second exporter was added.
- CLI projection names the receipt's actual committed action version and new
  V3 duration; old summaries omit duration. Existing inspect-label matches are
  extended mechanically, with no TUI control or workflow added.

## Verification

The first public-interface regression created a fully-ready generated V3 take,
then failed its DAW export with the previous two-bar-only admission error
(`/tmp/riotbox-1555-extended-daw-red.log`). The same 8/16-bar tracer passes through
the new handoff (`/tmp/riotbox-1555-tracer-green.log`). CLI version/summary tests
also pass (`/tmp/riotbox-1555-cli-projection.log`, five tests). The earlier
binary-only filtered invocation selected zero tests and is not evidence.

Final generated verification passes: 22 live-master DAWproject App tests,
495 Core tests total (including six new contract tests), five CLI projection
tests and 21 existing recording controls. The extended App tests have semantic
delivery and admission/restore ownership in two sibling Rust modules.
Full source-free `just ci` passed on 2026-10-03, including workspace tests,
contract/synthetic audio checks, formatting and strict all-target/all-feature
Clippy (`/tmp/riotbox-1555-ci-final.log`). DAW-host import, audible DAW playback,
physical-device endurance, musical quality and release readiness remain
unproven regardless of those engineering results.

## Branch review

Independent Rust/compatibility/adversarial review found one P2 defect: existing
writer/release, surface-gate and operator-readiness projections recognized only
DAW V1/V2 and let malformed V3 geometry fall through to generic passed gates.
Locations: `daw_export_proof_gates.rs`, `daw_session_surface_gate.rs`,
`daw_export_operator_report.rs`. Provenance: Adversarial Implementation Reviewer
and Torvalds-Inspired Maintainer. Disposition: fixed before PR by the shared
Core `is_live_master_dawproject()` predicate and existing archive-readiness
contract in all three consumers. A generated regression first demonstrated
Core not-ready but projected writer Passed; after repair it requires writer
Failed, operator Blocked, MissingArtifactIdentity and DawWriterMissing, plus
the surface writer blocker, for both missing-duration and wrong-frame-count
mutations. RED/GREEN logs:
`/tmp/riotbox-1555-extended-daw-readiness-{red,green}.log`.

The independent reviewer verified the correction. The separate Spec/Evidence
Auditor retained no P0–P3 finding. Final diff sealing, full local CI and the
short follow-up self-review are complete; the self-review found no additional
defect. Native CI must pass on the exact reviewed head before merge. No finding
is deferred or waived.

Architecture cadence: RIOTBOX-1551 was the current-state checkpoint;
RIOTBOX-1552 was the first substantive successor. This versioned contract is
the second; the intervening host-evidence and mechanical description slices
do not advance that counter. No new whole-repository audit is claimed here.

### Supplemental claim check

Following the updated user orchestration guidance, one small Jev 1.13.0 request
on 2026-10-03 checked four bounded claims against supplied engineering evidence,
including negative controls. It selected support for generated 8/16-bar
packaging, and rejected claims of host import, human listening and already-passed
full local/native CI (which was still pending at that point). The packaging
answer had low confidence (0.45) and was checked manually against the actual
public-seam test rather than automatically trusted. Usage: 997 input / 162
output tokens; helper-estimated cost $0.00004187. This is supplemental wording
review, not a correctness, listening, permission or release gate. No TypeSafe
dependency or model call was added to Riotbox.
