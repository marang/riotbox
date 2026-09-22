# RIOTBOX-1491 — Restore-history scaling

Classification: maintenance/regression. Base: `218994aa`. The user explicitly
prioritized autonomous technical tickets over the DAW/listening-dependent path.
This preserves P023 Session recall/recovery as histories grow; it is not an
audible advance, a demonstrated xrun fix or a new latency guarantee.

## Baseline and scope

The App validator still scanned all actions for each commit and compared every
commit with all preceding commits. Its final legacy-undo check also nested
action and record scans. Core's typed-undo relation validator, reached by the
same restore path, performed equivalent repeated searches. Removing only the
first App scan would leave quadratic behavior for undo-heavy Sessions.

The old review's undo-normalization finding is stale: that helper already uses
snapshot-ID sets and record counts. Its behavior is retained unchanged.

## Implementation and compatibility

- Validation has its own semantic `persistence/history_validation.rs` module.
  It owns transient indexes, not another persisted history or runtime model.
- Actions and first record positions use BTreeMaps. Complete boundary/sequence
  keys use a standard randomized HashMap with derived full-structure equality
  and hashing. No map iteration controls validation, output or replay order.
- First-record positions preserve the nested scan's exact diagnostic precedence
  when action-ID and boundary/sequence errors coexist. Hashing includes kind,
  beat, bar, phrase and optional scene, not only the displayed diagnostic fields.
- Core reuses its action index and preserves `.find()` first-record semantics
  for typed-undo validation before its existing duplicate-record gate.
- No schema, action, DSP, runtime state, normalization or accepted-history
  contract changes. Existing JSON remains identical. Expected work for these
  gates becomes O((actions + commits) log(actions + commits)), using linear
  transient memory instead of repeated pairwise searches.

## Reproducible measurement

Run `just restore-history-benchmark` on an otherwise idle machine. The ignored
release test synthesizes plain and alternating typed-undo histories; it writes
only a temporary Session, never opens sources or audio. Each result is the
median of five restores. Fixture construction/serialization and returned-state
destruction are outside the timer; JSON read/parse, normalization, validation,
Core replay-plan construction and view projection are inside it. These are
**end-to-end metadata restore times**, not isolated-validator timings.

Local baseline, original implementation plus the same benchmark/test fixture:

| Actions = commits | Plain (ms) | Typed undo pairs (ms) |
| --- | ---: | ---: |
| 1,000 | 4.273 | 7.315 |
| 4,000 | 47.967 | 131.678 |
| 16,000 | 634.360 | 2279.728 |

Indexed measurements on the otherwise idle workstation:

| Actions = commits | Plain (ms) | Typed undo pairs (ms) |
| --- | ---: | ---: |
| 1,000 | 3.826 | 4.026 |
| 4,000 | 17.843 | 19.045 |
| 16,000 | 78.203 | 89.565 |

At 16,000 entries this is approximately 8.1x / 25.5x faster. Multiplying
history size by 16 now multiplies observed time by about 20–22, rather than
148–312. Environment: Intel i7-8750H, x86_64 Linux, rustc 1.98.1, default Cargo
release profile. The first optimized run overlapped other build/test activity
and measured 121.409 / 114.478 ms at 16,000; it is recorded here for transparency
but is not the idle comparison. No universal latency threshold is asserted.

Timing is evidence, never a machine-dependent CI assertion. The synthetic
histories do not characterize every user Session, storage device, capture
hydration cost or realtime callback. Snapshot-cursor replay planning outside
this full-restore validation path is not optimized by this ticket.

## Review and verification

Solo `code-review` / `code-review-rust` lenses, honoring the no-subagent request.
Before implementation, four public restore tests passed against the original
code: long plain/undo histories, malformed records, duplicate-error precedence
and untrusted legacy undo. An additional boundary-field test protects complete
key identity. Existing Core replay tests cover typed-marker chronology,
target uniqueness and malformed relations.

Library checks: App 754 passed plus one deliberately ignored benchmark, Audio
263 passed, Core 453 passed, Sidecar 14 passed. One additional Core regression
also passes: duplicate undo records preserve first-record relation validation
before duplicate rejection (454 Core tests total). Full source-free `just ci`
passes, including synthetic audio/observer fixtures, format and strict Clippy.

The first complete CI attempt hit the pre-existing 50 ms Sidecar control
handshake timeout in `analysis_can_outlive_control_budget_without_timing_out`
while another test build was active. The earlier full library run and isolated
recheck pass. No Sidecar code or timeout was changed. A subsequent strict Clippy
finding in the new test table was fixed with a named local type alias; the final
complete gate passes without overlapping builds.

Review disposition (sequential maintainer, evidence, adversarial and Rust
lenses): zero outstanding findings. Review explicitly checked validation order,
duplicate earliest-position precedence, first-record legacy behavior, whole
boundary identity, unchanged JSON and the transient allocation tradeoff. Short
self-review confirms no hidden persisted state, new dependency, callback work
or audible change. Remaining measurement limits are stated above.
