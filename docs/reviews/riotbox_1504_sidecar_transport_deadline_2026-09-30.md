# RIOTBOX-1504 — Sidecar transport deadline

Classification: maintenance/regression. Branch:
`feature/riotbox-1504-sidecar-transport-deadline`; implementation base:
`f941affd02f2ea90d5106790868b7fc44be7071c`, archive-only successor
`d9e7bf426248224adc8d5f600b30d3640160b1a3`. The P023 risk removed is an
offline analysis call stranded on a peer that does not read its request.
This does not qualify a source-analysis algorithm, audible result or DAW.

## Contract and ownership

The existing synchronous client supplies one absolute operation deadline to
the private `transport` module. Request write/flush and response framing consume
the same budget. Control and analysis retain their existing separate policies,
protocol 0.1, request IDs and provider checks. Local JSON encoding starts after
the deadline is captured but is not forcibly preempted; process spawn, local
decode work, scheduling/kernel stalls and descendant-process containment are
not claimed to have a hard wall-clock guarantee.

`pipe_platform` owns the small OS boundary: Unix nonblocking descriptors and
deadline-aware poll; Windows nonblocking byte-pipe writes, a sole stdout reader
that reads only observed available bytes, and bounded one-millisecond retries.
The pinned windows-sys 0.61.2 bindings were inspected for pointer/handle types.
Microsoft documents anonymous-pipe support and the write-handle access contract
for [SetNamedPipeHandleState](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-setnamedpipehandlestate),
partial/zero-byte nonblocking byte-pipe writes in the
[pipe wait-mode contract](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-type-read-and-wait-modes),
and available-byte observation in
[PeekNamedPipe](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-peeknamedpipe).
The latter is not a blanket guarantee for arbitrary synchronous shared handles;
this module has one private reader with no concurrent read outstanding.

No detached stdout reader or unbounded response channel remains. Timeout,
transport error, invalid framing or protocol desynchronization closes the owned
pipes, terminates/reaps the direct child and returns the existing typed timeout
or specific error. Subsequent operations return `TransportUnavailable`; a fresh
peer is required. A well-formed Sidecar error leaves a synchronized peer usable.
Final valid EOF frames without a newline, coalesced frames and Unicode across
chunks retain supported behavior. Completed frame capacity is released; frame
size itself remains uncapped, explicitly outside this slice.

No Core/Session, ActionCommand, queue/commit, replay, audio callback, DSP or
musical threshold changes. Dependency versions remain unchanged; Cargo.lock adds
only Sidecar edges to already-resolved libc and windows-sys. RBX-382 and the
technology-stack spec own the changed transport/lifecycle contract.

## Regression evidence

Two public-interface tests failed before the fix:

- `/tmp/riotbox-1504-write-red.log`: a warmed peer stopped reading a 1 MiB
  metadata request; the old client waited for its two-second exit and returned
  BrokenPipe instead of the configured 50 ms transport timeout.
- `/tmp/riotbox-1504-budget-red.log`: delayed write and delayed response exceeded
  a shared 300 ms budget, but the old client restarted response waiting and
  accepted the late, well-formed Sidecar error.

The large string is synthetic API metadata, not a filesystem locator opened by
these tests or a claimed observed production incident. Mock exits are bounded;
new timeout assertions use a generous outer bound rather than promising exact
host scheduling. Tests cover cleanup and refusal after timeout, partial-frame
expiry, EOF compatibility, request-ID desynchronization, synchronized error
reuse, zero budget, coalesced frames, Unicode and release of large-frame buffer
capacity. The last transport-only test kills/reaps its peer before asserting.

Final focused run: 24 Sidecar library tests and the unrelated-CWD integration
test pass, `/tmp/riotbox-1504-all-sidecar-final.log`. Fresh final full `just ci`
passed (`/tmp/riotbox-1504-ci-final.log`): App 770, Audio 279, Core 470 and
Sidecar 24 library tests, binary/integration tests, Python/contract fixtures,
source-free synthetic audio/observer/manifests, formatting, tracked JSON and
strict Clippy. The initial run had 23 Sidecar tests before the buffer regression.
Native Windows test and strict-Clippy CI is an additional merge gate, not a
locally executed result or Windows audio-device qualification.

## Branch review

Sequential `code-review` and `code-review-rust` lenses covered the absolute
deadline, partial I/O, lifecycle, protocol compatibility, unsafe handle/descriptor
borrowing, memory ownership, synthetic peer bounds, dependency diff and CI scope.
No independent reviewer or subagent approval is claimed.

The review caught completed-frame allocation retention; ownership now transfers
the completed buffer and retains only trailing bytes, with a dedicated test.
Pinned Windows binding inspection also removed unnecessary mutable pointers.
No retained P0–P3 finding after these corrections. A short self-review confirms
one I/O owner, no hidden retry, no reader worker, no changed wire format and no
source access.

`client.rs` exceeds the soft file-size guidance (739 lines) because its existing
public policy/error/client lifecycle and legacy inline tests remain together.
The new I/O algorithm and OS boundary have separate semantic modules, and the
new deadline regressions are a real child test module. Splitting the unchanged
legacy tests solely to clear a line budget would not improve ownership; no
textual include or numbered shard is introduced.

## Evidence limits

All checks are source-free: fresh mock peers, metadata and existing synthetic CI
fixtures. No real source, active holdout, commercial reference, host audio,
human playback or DAW access. Automated audio fixtures are not a human verdict.
Windows behavior must pass the new native CI job before integration. No frame
resource cap, process-tree sandbox or universal filesystem/kernel deadline is
claimed.
