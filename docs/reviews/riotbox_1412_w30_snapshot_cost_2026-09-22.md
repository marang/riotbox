# RIOTBOX-1412 — W-30 snapshot cost

Classification: maintenance/regression. Base `218994aa`; RIOTBOX-1491 is an
independent restore-validation change. The user explicitly prioritized
autonomous technical work over DAW and listening-dependent tasks. This reduces
avoidable P023 live-callback work without changing musical policy or claiming
an observed xrun fix.

## Measurement before optimization

The existing callback reads 2,048 source-window and 16,384 pad samples through
individual atomics on every invocation, even when the publication is unchanged.
That is 18,432 sample loads and 73,728 bytes of sample payload, plus metadata and
the callback's snapshot copies. This fixed cost also exists with empty windows.

`just w30-snapshot-benchmark` runs an ignored source-free release test. The fixture
is a full synthetic pad, never an audio file or device. Each result is the median
of seven batches of 2,000 reads. Unchanged reads are batch-timed; changed reads
are individually timed with the control-thread update excluded. The latter
includes timer overhead. Black-box consumption keeps the result observable.

Original code measured before optimization: unchanged **13.858 µs**, changed
**14.254 µs**. Same-binary comparison after adding the cache:

| State | Original path | Cached path |
| --- | ---: | ---: |
| Unchanged publication | 13.671 µs | approximately 0.001 µs |
| New publication every read | 13.984 µs | 12.938 µs |

The final module layout, integrated with `main` at `a361864f`, was measured
again: original unchanged/changed **12.731 / 13.307 µs**, cached **0.002 /
11.628 µs**. This confirms the same structural benefit; the earlier comparison
and its budget percentages are retained above/below rather than cherry-picked.

Environment: Intel i7-8750H, x86_64 Linux, rustc 1.98.1, default Cargo release
profile, no concurrent local build during the measured run. The tiny hot-loop
fast-path value is descriptive, not a portable nanosecond guarantee. Structural
no-read/in-place tests, rather than a timing assertion, guard the optimization.

The named comparison budget is the complete callback period, frames / rate:

| Frames / rate | Full callback budget | Original unchanged share |
| --- | ---: | ---: |
| 64 / 48 kHz | 1,333.3 µs | 1.03% |
| 128 / 48 kHz | 2,666.7 µs | 0.51% |
| 256 / 48 kHz | 5,333.3 µs | 0.26% |
| 64 / 96 kHz | 666.7 µs | 2.05% |
| 128 / 96 kHz | 1,333.3 µs | 1.03% |
| 256 / 96 kHz | 2,666.7 µs | 0.51% |

The original snapshot alone is within these budgets on this machine, not a
demonstrated deadline violation. Avoiding a recurring 14 µs / 74 KB cost with a
small bounded cache is nevertheless justified. These measurements cannot
certify whole-callback acceptability: DSP, capture, other groups, scheduling,
writer contention and actual device deadlines are not measured here. Changed
publications still pay for a complete read, including same-content updates.

## Ownership and compatibility

`runtime/w30_preview_snapshot.rs` owns W-30 shared/realtime types, atomic
publication and its callback-local cache. MC-202 retains its own unchanged
module; resample/callback code no longer also owns W-30 publication. This is a
semantic extraction, not new textual includes or a second render-state model.

The cache remembers the publication revision (not the trigger revision) only
after the existing even-before/equal-even-after check. Equal even revisions
return a reference to the same payload. Odd or racing revisions retain the last
complete state and revision, with the same three-attempt limit. Startup before
the first stable read stays explicitly silent, never tags an unchecked payload
with a separately read revision. The actual device callback borrows the cached
state and overwrites its three callback-owned timing fields every invocation.
It does not copy the sample arrays to another local snapshot.

There is no allocation, lock, destruction of heap payloads, file access or
unbounded retry in this handoff. The cache is permanently paired with the same
shared group in the device callback. Control-thread publication, other render
groups, Session/replay, public APIs and musical behavior are unchanged. Existing
single-writer and revision-counter assumptions remain; this ticket introduces
no new persistent contract or alternate synchronization protocol.

## Verification and review

Deterministic regressions cover unchanged no-reader/in-place borrowing, an
already-active writer, a writer starting during a read, repeated mismatched
revisions exhausting exactly the bounded attempts, retry after rejection, and
silent startup followed by adoption. The renderer test changes PCM without
changing the performer trigger revision, compares cached/uncached buffers bit
for bit, proves the changed output is non-silent/different, and checks Idle
silence. Callback timing remains refreshed separately, not frozen with samples.

The initial Audio suite passes (268 tests plus an ignored benchmark). The final
Rust suite, including the extra mid-read-writer regression (269 Audio tests) and
final module ownership move, passes. Full `RUST_TEST_THREADS=1 just ci` passes:
Rust/Python, synthetic audio/observer gates, format and strict Clippy.
After integration with current `main`, the full Library suite passes again:
App 754, Audio 269, Core 454, Sidecar 14, plus two ignored manual benchmarks.
Format and strict all-target/all-feature Clippy also pass on the integrated tree.

Normal parallel CI reproduced the unchanged Sidecar test's 50 ms initial Python
handshake timeout; serial execution passes. RIOTBOX-1498 separately owns removing
that startup-scheduling dependency while preserving timeout-policy coverage.
This branch does not change Sidecar code or weaken deadlines. Serial test
execution is reported explicitly rather than concealing the failing attempt.

Solo code-review/code-review-rust lenses cover the complete diff, live callback
consumer, unchanged public/offline consumers, initialization, revision identity,
failed-read retries, absence of payload copies on the fast path and module
ownership. No retained correctness or contract finding. No independent-agent
review is claimed. One moved-type import was corrected after compile checking;
no lint or test requirement was relaxed. Text comparisons (ignoring whitespace
and the test-only legacy-reader annotation) confirm the moved publication/field
readers and MC-202 implementation are unchanged. Final self-review finds zero
outstanding issues; CI status is recorded above.

No real source, active Holdout, commercial reference, human playback or DAW was
accessed. This is synthetic technical evidence, not a human or musical verdict.
