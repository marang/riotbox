# Python WAV container admission — RIOTBOX-1562

Date: 2026-10-03. Base: `5877d116650418f416b6d45ffb16ab3aa9caf91c`
(RIOTBOX-1561, PR #1664). P017 maintenance/regression of P023 source-ingest
integrity and process failure isolation; not an audible mechanism.

## Diagnosis and contract

Generated minimal inputs expose three distinct baseline failures: truncated
RIFF/format fields escape as `EOFError`, overlong skipped metadata can escape
as `RuntimeError`, and missing/partial PCM can produce an analyzed graph.
The existing standard reader allows short frame reads and floors declared
frame counts; local conversion previously discarded incomplete frames.

Those initial truncated cases are rejected by Rust's later physical chunk/frame
checks before App publication. However, a RIFF extent ending at the data header
while PCM remains physically present is interpreted differently: Python reads
no frames while Rust accepts the PCM. Byte-hash equality alone does not resolve
that container interpretation. Initial publication consequences were inferred
from control flow; the new App regression below verifies rejection at the
provider and preservation of both old and new persistence destinations.

These deterministic, in-memory observations and the real-peer RED made a broad
history bisection or speculative hypothesis search unnecessary. No real source,
large allocation or host/device experiment was needed.

RBX-432 / Source Graph §6 were written before implementation. A single pure
`source_wave` module validates complete RIFF/chunk extents and unique ordered
format/data ownership, performs public standard-library WAV decoding, verifies
positive supported geometry and whole/exact frames, and owns the unchanged
integer scaling/channel averaging. It returns typed transient metadata and mono
samples, not another persisted source model. `source_bytes` still owns file
admission; the Sidecar still owns graph analysis and error envelopes.

Opaque metadata and padding remain supported. The existing unpadded odd
terminal-data convention emitted by Python's writer is deliberately preserved,
as is genuine empty data. Ambiguous duplicates, mismatched container sizes and
trailing/incomplete chunks now fail rather than selecting a different payload.
The parser-only guard maps malformed input to the existing request-bound,
non-retryable `source_unsupported` envelope; it does not swallow unrelated
analysis exceptions or generate substitute analysis/audio.

Structure was checked against [Microsoft's RIFF description](https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff-)
and the public [Python wave interface](https://docs.python.org/3/library/wave.html),
plus the installed implementation's concrete short-read/parser behavior. The
terminal unpadded-data exception is explicit compatibility, not a claim that the
general RIFF word-padding rule does not exist.

## Verification

- First RED: real Python peer exits 1 with `EOFError` on a 22-byte truncated
  format input, before it can answer subsequent ping/valid-analysis requests.
  Parser error normalization makes that exact roundtrip green.
- Second RED: the layout/geometry matrix records 11 failures reaching features
  and one zero-rate arithmetic error despite the initial parser exception fix.
  Complete-container and frame admission resolves them.
- Final Python suite: 18 tests pass. New cases include 22 malformed layout/
  geometry variants, all 52 proper prefixes of a generated valid file, and
  injected declared-count/short-read faults. Thirty-two positive width/channel/
  layout controls preserve exact samples, including actual stdlib-writer output;
  eight empty-data controls remain supported.
- A complete container with a declared-short format payload exercises parser
  `EOFError` normalization after the layout check. Downstream analysis
  `RuntimeError`, `EOFError` and `AssertionError` remain unmasked.
- Public Rust client: four malformed inputs each produce the exact request-bound
  error, followed by successful ping and valid analysis on the same Python peer.
  Full Sidecar suite passes: 32 unit tests and three integration tests.
- App: four source-ingest identity tests pass. The new false-RIFF-extent test
  proves unchanged Rust decoder acceptance, provider rejection before both
  overwrite/new publication, byte-identical old Session/external Graph or
  embedded state, absent new directories, and restored Session/Graph/cache for
  both external and embedded modes. No claim that Rust itself gained RIFF
  validation follows.
- Same-buffer atomic source-replacement regression remains green. The moved
  sample conversion and graph feature algorithms retain their existing values.
- Full source-free local `just ci`: passed, including workspace tests,
  Python/contracts, generated-audio smokes and strict Clippy. Native exact-head
  CI remains the merge gate.

Logs: `/tmp/riotbox-1562-red.log`, `/tmp/riotbox-1562-parser-green.log`,
`/tmp/riotbox-1562-layout-red.log`, `/tmp/riotbox-1562-layout-green.log`,
`/tmp/riotbox-1562-python-final.log`, `/tmp/riotbox-1562-sidecar.log`,
`/tmp/riotbox-1562-app.log`, `/tmp/riotbox-1562-ci.log`.

## Review

Independent Adversarial Implementation Reviewer retained no P0–P3 findings.
Executed the then-final 17 Python tests, 23,365 bounded header/geometry mutations
(no escaping exceptions), 192 generated valid-format/metadata controls,
analysis-exception probes and diff check. Rust client/App regressions were
inspected, not executed by that reviewer. These bounded probes are not a
complete fuzz/security proof.

Independent Spec and Evidence Auditor retained no P0–P3 findings. Executed the
17-test suite and 32 complete-graph compatibility comparisons against the base,
plus five complete-layout short-format parser probes and an unrelated analysis
exception probe. The reviewer noted the initial tests did not directly reach
the short-format EOF normalization branch after layout admission; a dedicated
case is now retained in the suite. The analysis-exception probe and actual
stdlib-writer controls also became durable tests. Rust/App tests were inspected,
not executed by that reviewer; the coordinator executed them as recorded above.

Final evidence/report recheck retained no findings. The Spec and Evidence
Auditor independently reran all 18 final Python tests and inspected the focused
Rust/App logs; the adversarial reviewer inspected those logs and the final test
delta without another run. Execution/inspection attribution and RED counts were
confirmed. Final spec log: `/tmp/riotbox-1562-spec-evidence-final.log`.
Coordinator self-review found no
remaining correctness, same-buffer identity, error masking, ownership, module
growth or claim-scope issue. No production Rust owner changed; the Python
extraction removes decode knowledge from the graph assembler instead of making
mechanical numbered shards. This is the fourth substantive slice after
RIOTBOX-1558's architecture checkpoint.

## Limits

Malformed/ambiguous files previously tolerated by a parser now fail explicitly.
No arbitrary metadata-content or every-extension validation is claimed. Existing
valid supported PCM, integer scaling, channel averaging, timing/features and
duration floor remain unchanged. Historical graphs are not migrated or newly
attested. Encoded admission and later Rust/App checks still apply separately;
decoded arrays, aggregate memory/RSS and execution deadlines remain outside this
contract.

No new Action, Session/replay/protocol schema, persistence owner, dependency,
DSP/runtime behavior, TUI or Windows work; no real source/Holdout/commercial
access, source-directory discovery, device, DAW or human playback. Generated
tests provide no new musical/source-general/hardness/demo/release qualification.
