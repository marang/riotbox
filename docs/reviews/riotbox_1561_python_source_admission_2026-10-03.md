# Python encoded-source admission — RIOTBOX-1561

Date: 2026-10-03. Base: `b5df129d8385b83104297b26a3bb9fec319502d5`
(RIOTBOX-1560, PR #1663). Classification: P017 maintenance/regression of the
existing P023 ingest boundary, not an audible mechanism or a new musical or
source-selection threshold.

## Change and contract

Python previously read the entire source before Rust's bounded admission ran.
RBX-431 and Source Graph §6 extend the existing fixed 256 MiB encoded-file
budget to that earlier read. One small Python owner opens nonblocking on Unix,
admits the opened descriptor as a regular file, rejects declared oversize before
payload access and bounds actual unbuffered reads despite file growth.
Regular-file symlinks and canonical-path reporting remain supported.

Exact EOF at the limit succeeds; a one-byte overrun never reaches hash/decode
or returns truncated audio. Short/interrupted reads preserve admission and
ordinary errors close the descriptor. A single bytearray accumulates admitted
bytes and produces the immutable buffer used for hashing and WAV decode. The
existing source replacement identity regression remains unchanged.

`source_resource_limit` uses the existing request-bound, non-retryable error
envelope. The Rust client returns that error and can subsequently ping the same
peer. App's existing `?` propagation precedes Rust enrichment and Session/Graph
publication; there is no alternate persistence or fallback path.

## Verification

- RED: generated sparse oversize reached `handle.read()` before metadata
  rejection. Read/hash/decode sentinels prevent the test from allocating its
  nominal 256 MiB-plus-one payload.
- A second RED caught buffered I/O reading 9 actual descriptor bytes under a
  four-byte budget plus one probe; unbuffered I/O restores the expected 5.
- Final Python suite: 12 tests passed. New admission cases cover fixed-budget identity,
  empty/exact/excess, small fragmented reads, understated metadata, interruption
  before payload and at the probe, I/O error/closure, directory rejection,
  generated regular-source symlink/hash/decode and producer-free FIFO/symlink
  rejection in timeout-bounded subprocesses followed by ping.
- Self-review exposed retained-object amplification for short reads. A third
  RED measured 16,216,705 traced bytes for a 128 KiB input read one byte at a
  time; one bytearray accumulator removes the per-read retained objects. The
  synthetic regression requires peak traced allocation below 8x payload with
  ample margin. Its isolated subprocess owns the tracing session; the test also
  passes with `PYTHONTRACEMALLOC=1` without stopping the parent process's tracing.
  This is not a general heap/RSS limit.
- Rust public-client integration rejects a generated sparse oversize with exact
  request ID/code and preserves peer synchronization. Full focused Sidecar
  suite passes: 32 unit tests and 2 integration tests.
- Full source-free local `just ci`: passed, including Rust, Python/contract,
  generated-audio smokes and strict Clippy. That run began before the final
  two-line would-block guard; its final focused Python and independent checks
  above cover that delta. Native CI must test the exact published commit.

Logs: `/tmp/riotbox-1561-red.log`, `/tmp/riotbox-1561-prefetch-red.log`,
`/tmp/riotbox-1561-green.log`, `/tmp/riotbox-1561-python-tests.log`,
`/tmp/riotbox-1561-sidecar-tests.log`, `/tmp/riotbox-1561-short-read-red.log`,
`/tmp/riotbox-1561-python-final.log`, `/tmp/riotbox-1561-tracing-final.log`,
`/tmp/riotbox-1561-ci.log`, `/tmp/riotbox-1561-ci-final.log`.

## Independent review

Initial Spec and Evidence Auditor and Adversarial Implementation Reviewer passes
retained no P0–P3 findings. Both independently ran all 10 Python tests and the compiled Rust
admission integration. The adversarial reviewer also exercised 37 additional
small-budget/descriptor-offset cases, mid-read path replacement, and
oversize → ping → valid analysis from a different CWD, plus the existing CWD
integration. Both passed diff checks and traced App failure before publication.
They inspected, rather than rebuilt, the focused Rust suite and RED evidence.

Coordinator Performance/Operations self-review raised that short-read storage
concern after the first reviews. A delegated 128 KiB microprobe reproduced
123.74x peak traced allocation for one-byte reads; the bytearray concept used
2.06x and retained exact byte identity. This finding is attributed to the
coordinator lens, not the original independent reviewer. The follow-up is
covered by the new RED/GREEN regression and re-review.

Adversarial re-review found P3 test isolation: the added in-process allocation
test inherited an existing tracing peak and stopped caller-owned tracing.
`PYTHONTRACEMALLOC=1` reproduced a false failure (11,863,140 traced bytes against
the 1,048,576 guard); `/tmp/riotbox-1561-adversarial-preexisting-tracing.log`
records it. Fixed by measuring in a fresh `python -E` subprocess, returning
byte-identity/peak evidence and checking the parent's tracing state is unchanged.
Normal and tracing-enabled suites pass. Independent recheck closed P3 without
new findings: all 11 tests pass with tracing enabled; reinstating the original
chunk-list reader only inside the child makes the guard fail at 16,216,465 bytes.
Isolation therefore removes the false alarm without weakening regression
detection. Spec re-review also passes the final 11-test suite.

The coordinator's short self-review found no remaining issue in descriptor
lifetime, raw read bounds, same-buffer identity, source-path compatibility,
module ownership or claim limits, but an additional raw-I/O check identified
`None` (would-block) being mistaken for EOF. This is a synthetic descriptor
contract failure, not a reproduced local regular-filesystem stall. Python's
[raw read contract](https://docs.python.org/3/library/io.html#io.RawIOBase.read)
distinguishes it from empty bytes. A fourth RED reached hashing with `None`
before any bytes, after a short prefix and after a complete valid WAV prefix.
Fixed with `BlockingIOError` / existing `source_unreadable`, without waiting or
spinning; `/tmp/riotbox-1561-would-block-red.log` records the RED. The final
12-test suite and tracing-enabled admission suite pass. Independent recheck
confirms all three prefix cases prevent hash/decode, close the descriptor and
return `source_unreadable`, with no further findings (log:
`/tmp/riotbox-1561-adversarial-would-block-final.log`). Full/native CI remain
separate gates. App
no-publication ordering was inspected; this slice does not add a new App-specific
oversized-source persistence test.

Final coordinator self-review retains no findings after these corrections.

## Scope and limits

This bounds encoded input, not total RSS: the accumulator (including allocator
growth/overhead) and final immutable buffer can coexist. Python WAV frame
copies, sample/analysis arrays, concurrency, kernel I/O latency and aggregate
memory remain outside this limit. The Rust
reader/decoder, protocol schema, timing/features/DSP, Session/replay ownership,
source-selection rules and historical source evidence are unchanged.

No Action/queue/commit/runtime change, new dependency, TUI or Windows work,
real-source/Holdout/commercial access, source-directory discovery, host device,
DAW, playback or musical qualification occurred. The source-read helper is a
semantic owner, not a mechanical split of the Sidecar. This is the third
substantive slice after RIOTBOX-1558's current-state architecture checkpoint.
