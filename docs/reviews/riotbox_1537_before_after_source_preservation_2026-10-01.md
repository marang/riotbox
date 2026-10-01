# RIOTBOX-1537 — Preserve Before/After input through every output destination

Date: 2026-10-01
Baseline after normal predecessor merge: `5f3baab877afcccb791f302a8842a32de2309324`
Integrated predecessor archive: `e393456372d75ef295395bfb6169d46c96d4bdb6`
Decision: RBX-407
Classification: pre-existing offline QA input-loss correction

## Cause and regression loop

Two actual Before/After CLI runs report success while their own generated input
changes from 705644 to 17684 bytes and changes SHA-256. Separate output
preserves the same regenerated input. The renderer loads the source, then
writes its excerpt before rendering other lanes; final-only validation would
be too late to protect it.

The first public CLI test fails on changed input bytes, then passes with the
guard. The original feedback script now rejects with an explicit collision and
unchanged 705644-byte input when given the exact freshly built candidate:
`bash /tmp/riotbox-before-after-source-collision.vdbpE7/repro.sh collision /tmp/riotbox-1507-review.4spOix/target/debug/feral_before_after_pack`.
Only its exact generated `01_source_excerpt.wav` is used, never real audio.

Linear In Progress precedes the independent branch and red test on clean main.
The common helper is integrated only after #1616's normal exact-head native
merge; candidate tree equality and disjoint fast-forwards preserve the red test
and subsequent implementation. No stacked feature branch or backfilled issue.

## One actual layout and unchanged safety owner

Renderer-local `PackOutputPaths` owns six WAVs, five metrics files and
comparison/README/manifest. The composite before/after WAV intentionally has no
metrics file. All actual writes and manifest artifact projection consume that
plan and the unchanged existing metrics naming helper. Serialized fields,
artifact order and values are unchanged; normal byte parity includes JSON,
reports and README rather than only audio.

After existing source format and insufficient-window checks, the typed plan
delegates every destination to the unchanged ordinary binary-only
`qa_source_safety` preflight before source copying/first excerpt publication.
Shared implementation and all twelve existing Feral/W-30 CLI safety bodies
remain byte-identical. No another identity loop, generalized pack framework,
new target/dependency, public library API or product/replay/source authority.

The new layout owner is 57 lines and regression owner 169. Existing source/
mix/render-plan/metrics/configuration bodies remain untouched. Directory
creation retains its original position and may precede rejection.

## Technical proof and normal compatibility

Six new actual CLI cases cover:

- Every one of fourteen direct destinations, including valid WAV input stored
  under metrics/metadata suffixes; original input and all prior files intact.
- Hardlinked excerpt WAV, metrics and late manifest.
- Unix output/input/directory symlinks.
- Late manifest collision with all other artifacts absent, no earlier WAV
  publication.
- Late nonregular and, on Unix, dangling destinations; no earlier replacement
  or missing-target creation.
- Equal-content distinct output and unrelated input inside output remain valid.

All 94 focused names/statuses match and pass in Debug and Release: 70 existing
binary units, six capacity/duration/verification integration cases, twelve
preserved Feral/W-30 safety cases and six new Before/After cases.
No tests are removed or ignored; independent literal fixtures do not call the
production path planner or a private implementation seam.

Fresh standalone baseline/final builds precede normal-path comparison.
All 31 CLI status/stdout/stderr records and all 29 artifact hashes match:
28 outputs across short/long packs, including twelve WAVs, ten metrics and six
metadata files, plus input. Both listening manifests validate existing paths.
The sole parity input is exactly the fresh four-second generated control:
`/tmp/riotbox-1537-byteproof.1oScH4/synthetic-control.wav`.
No thresholds or algorithms are tuned against these controls.

## Solo review and honest limits

Code-review/Rust lenses inspect complete changed functions, every write/manifest
consumer, source/error precedence, helper identity/lifetimes, independent
regression signal, schema and dependency/visibility boundaries, workflow/docs
and soft module size, followed by short self-review. No remaining actionable
finding in this diff; no independent panel or subagents.

The shared stable-namespace/readability/unsupported-backend fail-closed contract
remains explicit. No adversarial concurrent namespace lock, atomic whole-pack
save, power-loss or Windows Audio/filesystem/device guarantee.
No Core/Session/replay/App/runtime/DSP/threshold/algorithm/schema or frozen
Stage-A change; no musical/product/source-diversity/hardness/human/DAW/release
claim or automatic missing-source music. Only bounded generated controls and
retained in-memory fixtures, no real Development/Holdout/commercial audio,
source-directory search or playback.

## Validation and remaining gates

- Red / first green: `/tmp/riotbox-1537-source-safety-{red,green}.log`
- Fresh build: `/tmp/riotbox-1537-build-reviewed.log`
- Final tests: `/tmp/riotbox-1537-focused-{debug,release}.log`, warning/error-free
- Both manifests, Rustfmt, diff/include gates and targeted RBX-407 readback pass.
- Full source-free CI: `/tmp/riotbox-1537-ci-final.log`, actual exit zero
  and explicit final warning/error scan clear.
- Exact-head native PR review/merge, archive and cleanup remain required.
