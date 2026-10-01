# RIOTBOX-1514 — Semantic metadata-only Observer/audio correlation

Date: 2026-10-01. Behavior baseline:
`96adfb322530d59560a26da01e2303a2ef3bb387`; subsequent RIOTBOX-1513 archive
sync changes only closeout docs. Classification: maintenance/regression, not
audible progress, source qualification or live-device proof.

## Ownership and compatibility

Replace ten textual includes with Cargo-standard directory `main.rs` and
ordinary private modules. Existing typed report data and scalar JSON readers
are dependency leaves. Metadata I/O, observer commit/scene/timing collection,
manifest timing/metric collection and anchor/groove metadata feed composition
and alignment. Evidence policy consumes the report; JSON and Markdown consume
report/evidence/shared timing labels. Existing lane-recipe and observer-envelope
modules remain unchanged. There is no dependency back from evidence to rendering
or from the report to composition. Visibility stays private or binary-parent-only.

Cargo metadata retains exactly one `bin` target named `observer_audio_correlate`;
only its source path changes to the directory main. No Cargo/dependency edit,
library API, ActionCommand, JamAppState field or Core/Session/replay truth changes.
The twelve moved report types are existing ephemeral diagnostic views, not a
new product model or listening authority. Main retains the complete CLI/file-I/O
body. The tool reads named metadata files, never referenced source/artifact audio.

Ordinary regression families import their actual owners explicitly. Physical
fixture paths and test hierarchy remain. Cohesive CLI/drift tests are 509/524
lines after explicit imports, within the policy's review guidance; keeping their
metadata contracts together is clearer than a mechanical size split. Largest
production owner is the 470-line Markdown presentation module.

## Retention and technical proof

- All **272 complete definitions** retained: **145 production definitions**
  (including the existing test-only composition helper), **127 regression/helper
  definitions**. **268** normalized token matches retain bodies/literals,
  attributes and qualified paths after only visibility/trailing-comma changes.
- Four differences inspected individually: Rustfmt adds/removes expression
  blocks in groove-residual filtering, string-list mapping, locked-policy and
  MC-202 failure match arms. Expressions, unit return behavior, error strings,
  evaluation order and data are unchanged. No broad normalization masks these.
- All **61** baseline/final executed leaf/status entries match exactly and pass.
  No ignored, deleted or weakened assertion and no replacement test family.
  Logs: `/tmp/riotbox-1514-baseline-tests.log`,
  `/tmp/riotbox-1514-final-tests.log`.
- First all-target App check and final build/tests succeed; all three logs
  explicitly scanned warning/error-free. Logs: `/tmp/riotbox-1514-check.log`,
  `/tmp/riotbox-1514-final-build.log`, final test log above.
- Include guard passes at **44 sites / six owners**, down from 54 / seven,
  without an allowance increase. Formatting and whitespace checks pass.
- Full source-free `just ci` passes, including strict all-target/all-feature
  Clippy, contract/metadata fixtures and fresh synthetic smokes. Exact-head
  native PR CI remains a separate merge gate. Log: `/tmp/riotbox-1514-ci.log`.

## CLI byte contract

A bounded **21-case** catalog compares complete stdout/stderr bytes and exit
status before/after, using only committed NDJSON/JSON metadata and named synthetic
invalid metadata. Cases cover long/short help; missing paths; unknown flag;
missing observer/manifest/output values; base and locked-grid JSON/Markdown in
local/strict modes; absent observer/manifest; invalid manifest JSON/envelope;
invalid observer JSON shape; and missing committed evidence. All 21 match.
These are local Linux comparisons, not cross-platform filesystem/device proof.

Both strict base file-output modes create nested output directories successfully,
emit no stdout/stderr, and write the exact previously captured stdout bytes:

- Markdown: 3,293 bytes, SHA-256
  `c1f3218b5db98ba56f9503ce400b156aaec3c76a5da2c907ae2b66031cef920b`.
- JSON: 7,039 bytes, SHA-256
  `692edada19bb3db4251ff0ae9150ff2491be068a46db06cafe246d96c4723bc0`.

Catalog fixture SHA-256 identities under
`crates/riotbox-app/tests/fixtures/observer_audio_correlation/`:

- `events.ndjson`:
  `dba5a3a71154d75a664a2bd6af63e905e7b773c54a2667498fac9d5612628a63`.
- `manifest.json`:
  `c3f7f74453b4f1cd77d36dd6ba87679d909650560e7b6dc94fda10779d9e623f`.
- `events_locked_grid.ndjson`:
  `a163d2a6a0d62b893b33b2cbde2cb7c988576dba61edb86c036bac52b97e26c8`.
- `manifest_locked_grid.json`:
  `caaaecd5078dbeefebda0421718889b4431ed116df0b83544576efb1c070b9a6`.

## Review and claims boundary

Sequential solo code-review/Rust checks target identity, parent visibility,
dependency direction, metadata-only access, fail-closed strict-evidence order,
schema/numeric retention, test/fixture identities and actual output bytes.
No migration-introduced correctness, architecture, test or scope finding remains.
Short self-review confirms those boundaries; this is not an independent panel.

An existing unchecked `u64` addition in the retained anchor-count impl is a
separate demonstrated P2 defect, tracked as **RIOTBOX-1515**. Synthetic metadata
with declared/typed kick count `u64::MAX`, backbeat count one and transient count
zero causes Debug CLI exit 101 (`attempt to add with overflow`); stdout is empty.
The identical original impl proves this is not introduced by module ownership.
Wrapped acceptance without overflow checks is code-derived, not release-tested
yet. A validation-behavior fix belongs to the separate bug slice, not this
mechanical migration. No source/artifact audio is involved in the reproduction.

RBX-391/module policy record ownership only. Existing schemas, strict-output
floor, source-grid hit floor, BPM tolerance and all policies/algorithms remain.
No real source, holdout, commercial reference, device, DAW, playback or musical/
human verdict is accessed or claimed. Source-free CI may create fresh synthetic
test WAVs; none is product-source or listening evidence. After merge this is the
fifth substantive ownership slice since RIOTBOX-1507, requiring an architecture
checkpoint (**RIOTBOX-1516**) before further structural migration.
