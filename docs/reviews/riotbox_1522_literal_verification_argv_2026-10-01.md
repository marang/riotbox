# RIOTBOX-1522 — literal Feral-grid verification arguments

Date: 2026-10-01. Classification: maintenance/regression. Implementation base
`9b7da7fca0d2482abb52904ac86e561c6bab131f` (RIOTBOX-1521); archive-only
main advance `082037cbcfdda407ab4f2987d2b3275e9d6aa132` changes no code.

## Correction and contract

The existing manifest verification command inserted the source inside unescaped
double quotes and the date unquoted. A successfully rendered fresh control named
`control"quote.wav` produced a command that fails parse-only `bash -n` with
status 2. The Just recipe independently reinserted every value inside unescaped
double quotes. Its complete script actually passes syntax checking: repeated
quotes can balance across branches while corrupting argv. The preliminary
inner-recipe parse-failure claim was corrected in Linear. A Cargo argv-only
function proves the real mismatch; no filesystem/network payload is executed.

The production change is one private POSIX single-quote helper, its two callers
in verification-command formatting, and `quote()` at all twelve Just recipe
interpolations. Auto/explicit BPM, numeric order and three-decimal format stay
unchanged. RBX-396 and the owning source-timing spec define the command's shell
grammar; manifest schema and renderer remain unchanged. Relative paths still
need the original working directory. This is not exact product Session replay,
nor Windows cmd.exe/PowerShell syntax.

## Regression evidence

- Both new Rust regressions fail on the original implementation in Debug and
  actual Release, then pass. Final suites execute all **46 unit tests plus one
  actual CLI integration test** in both profiles. The **44 previous regressions**
  remain present and pass; no ignores, suppressions or test removal.
- **22 outer shell argv cases** preserve source/date literally in both BPM
  modes: whitespace/tab/newline, apostrophe/double quote, backslash, dollar,
  controlled harmless substitution/backtick strings, semicolon, option-like
  labels, Unicode and empty strings. Probes replace Just with an argv printer.
- The CLI integration writes its own bounded four-second synthetic PCM WAV via
  the existing Rust writer, directly invokes the actual binary in both BPM
  modes, and checks manifest commands through the same no-real-Just argv seam.
  README keeps the exact source label and offline-QA disclaimer. No Python/Just
  dependency is introduced into Cargo tests. A draft Python-generator dependency
  was removed before final verification.
- The corrected Python contract tests fail on the original recipe and pass
  finally. Static coverage asserts every interpolation is quoted; **28 actual
  local Just dry-run argv probes** cover both branches and each numeric position.
  The native runner intentionally does not install Just: it runs the static
  check and Rust shell/CLI tests; the actual-Just test explicitly skips there.
  This is not a claim of native Just execution.
- Reuse only the exact named temporary control/pack outputs from the verified
  RIOTBOX-1521 technical baseline `/tmp/riotbox-1521-byteproof.UXi0Jt`.
  Pre-render hashes first match that pinned baseline. **31 CLI results** retain
  identical stdout/stderr/status. **37 of 39 file hashes** remain identical,
  including all **16 output WAVs**, the input WAV, README and numerical reports.
  Only two manifests change; complete JSON equality after removing only
  `verification_command` proves the precise intended metadata delta. Both final
  manifests independently validate their envelope and artifact existence.
- Retained red logs: `/tmp/riotbox-1522-red-debug.log`, red-release and
  recipe-red-corrected. Final Debug/Release and recipe logs are explicitly
  warning/error-free. An initial recipe-test expected-count typo was corrected
  to twelve before the meaningful original-recipe red run.
- Full source-free `just ci`: **passes** with a verified successful exit in
  `/tmp/riotbox-1522-ci.log`, explicitly warning/error-free, including workspace
  Rust/Python/contracts, synthetic audio gates, formatting/tracked JSON and
  strict all-target/all-feature Clippy. Native exact-head PR CI, merge and
  archive/cleanup remain separate gates.

## Review and scope limits

Solo sequential Maintainer, Product, Spec/Evidence, Adversarial, Risk and Rust
lenses, followed by short self-review; not an independent reviewer panel.
Two concrete P2 argument-boundary defects are fixed and covered at both seams:

- Adversarial Implementation Reviewer: `render_stems.rs::verification_command`
  corrupts accepted source/date values — **fixed** by literal POSIX escaping;
  original Debug/Release red tests and final outer/actual CLI argv probes prove it.
- Adversarial Implementation Reviewer: `Justfile::feral-grid-pack` expands or
  merges accepted values again — **fixed** by `quote()` at all twelve places;
  original-recipe red and final 28 actual dry-run argv probes prove it.

Evidence-review corrections: syntax-only recipe checking was insufficient;
native Just execution must not be claimed; all-manifest byte parity would be
false. All three are reflected in the issue, tests and this record.
**Zero additional changed-diff findings**, including short self-review, after
corrections. Ordinary adjacent tests avoid adding a large test block to the
legacy render-stems owner. Include inventory remains **26/two owners**.

No real Development/Holdout/commercial audio, source-directory discovery,
device, DAW or playback. Technical reruns under the listening-review skill do
not require duplicate human listening and cannot create or transfer a verdict.
No audio algorithm/threshold/schema, source-timing trust, frozen Stage-A,
Core/Session/action/replay, runtime/DSP or public API change. No product,
source-general, musical, hardness, release or live-device qualification.
RIOTBOX-1509 stays open; architecture cadence remains four since RIOTBOX-1516.
