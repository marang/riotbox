# RIOTBOX-1602 — Bounded Fill input during master recording

Date: 2026-10-17. Review base: merged main
`349b8a64952128d11b2de911a0ed7033b897228c` (RIOTBOX-1601).
P016 existing-gesture integration under RIOTBOX-1036, protecting the P023
recording path. Linear was In Progress before branch creation. Ordinary merge
`f38dbb2e9033fd4eaa2638b294a522eb26985a15` joined the tree-identical reviewed
1601 head to main; all twenty then-dirty/new file hashes stayed unchanged.
No reset, rebase, stash or unrelated changes.

## Performer path and ownership

Explicit `--live-recording-terminal-input` makes the existing lowercase `f`
Fill reachable while a bounded 2/8/16-bar master take is pending. Omission does
no terminal work and preserves default serialized summaries/observations.
The existing App queue enqueues `Tr909FillNext` at NextBar; existing Core commits,
TR-909 projection, renderer, capture tap, writer and Session persistence own the
result. No new ActionCommand, musical mechanism, DSP or source intelligence.

The private input controller retains only its own ActionIds for correlation.
Real queue/Core status is authoritative. Deduplicate pending Fill, ignore
Takeover-masked requests and retire only still-pending input-owned IDs on exit.
Never reject a foreign pending action or relabel a committed action. Input
cancel/read/admission failure retains stop → abort → reject/no publication,
including safe device stop when action allocation fails and historical second
stop behavior. No persisted App state or second action/history system.

Capture completion/fault checks precede input. Only Continue permits an input
attempt, followed by the existing 10ms wait. Recognized completion reaches
verified device stop before observer/artifact work. An unread cancel can lose
to completion. Startup, stop and publication remain synchronous.

Opt-in uses typed `terminal_fill`, `terminal_input_real_audio_cli` and
WindowStopped V2; default context/WindowStopped V1 remain unchanged. Existing
recording V3/V4 evidence still establishes the master window, not keypress-to-
frame delivery. Reports separate queue requests, Core commits, retired requests
and ignored tokens, with `render_delivery: not_qualified`. A final sync may
commit a Fill outside the captured window; that is not an ignored action or
proof of audible delivery.

## Blocking-reader finding and V2 repair

RBX-463 and historical terminal-input V1 were frozen before initial integration.
Performance And Operations / Adversarial Implementation review then retained a
P2: pinned Crossterm 0.29.0's Unix MIO `poll(Duration::ZERO)` can block inside an
inner read when `ESC [` has not produced a complete event. Fake terminal-call
counts did not exercise that decoder.

An isolated temporary SDK copy executes the unchanged real `try_read`, parser
and FileDesc read against an owned blocking anonymous pipe. Empty and complete
`f` controls pass. `ESC [` blocks beyond 250ms; synthetic `A` releases it as Up,
then the worker joins before the failing assertion. Exactly one test fails
with exit 101, not watchdog termination. The coordinator repeated the RED.
No stdin, TTY, global event reader, SIGWINCH handling or host/audio access.
Adding only O_NONBLOCK to that generated pipe makes the same probe return None
without continuation, supporting blocking reread as the cause. Flags alone do
not bound the library's other drain, parser, EOF or resize work.

Before replacing the reader, RBX-464 and terminal-input V2 froze the narrower
Linux protocol and budgets. Existing libc now supplies at most one nonblocking
one-byte read per pending iteration. There is no library event reader, worker,
retry/drain loop, growing parser buffer or dependency fork/version change.
Original descriptor flags and prior raw-mode ownership are guarded; both
cleanup operations are attempted and failures retained, with unfinished Drop
retries. Shared descriptor aliases and absence of concurrent flag mutation are
explicit prerequisites. Other systems reject opt-in before setup; no non-Linux
execution was qualified.

The fixed-state decoder accepts received ASCII `f`, `q` and Control-C bytes,
not physical Press/Release/Repeat proof. Standalone Escape has the frozen 50ms
ambiguity interval. It suppresses supported CSI/SS3/Alt/intermediate/UTF-8
tokens and marked paste, never releasing their embedded controls. Sequences
are limited to 64 bytes, marked paste to 65,536 post-opener bytes including its
closer. Unsupported/malformed/overlong input and EOF fail closed; subsequent
calls cannot reinterpret remaining bytes. Unmarked paste/autorepeat remain
indistinguishable, and delayed Escape fragments can cancel a take. The startup
brief communicates restricted controls, accidental input and delivery limits.

## Test-first evidence

Every RED below compiled and executed exactly one failing test before its
matching GREEN. New-interface/scaffold witnesses are not claims that all these
behaviors were pre-existing main defects. Compile failures and exploratory
zero-selection invocations are excluded. The workflow now explicitly requires
checking focused Cargo targets and actual executed test counts.

Original parser, actual-recorder input and observation verticals:

- `/tmp/riotbox-1602-input-parser-red.log`, SHA256
  `9aa7f5ddfcd68a74884282355791806e77c16d706d4cb2f913a9f6bfad87131e`.
- `/tmp/riotbox-1602-recorder-input-red.log`, SHA256
  `8fb30ac6682ef7d107eee5a04f37cd50b04c92936a0304029774251854b157fb`.
- `/tmp/riotbox-1602-input-observation-red.log`, SHA256
  `dfe6be5d01cd64c7e2045a2ed292f7b9be302ed73773c13bd84dbdcca0c5c698`.

Pinned-library defect and actual replacement-resource witnesses:

- `/tmp/riotbox-1602-crossterm-fragmented-actual-red.log`, SHA256
  `1ce4839a477f44ebfc2c4b9fb6fccf2db2eb2a7922054eb3bc70dac3edf4a1a0`;
  root repeat `/tmp/riotbox-1602-crossterm-fragmented-root-red.log`, SHA256
  `9e964b95e71d0a7c5c3077c5d6cdfced54319527844fae442a068e15cd308ee2`.
- `/tmp/riotbox-1602-resource-fragment-red.log`, SHA256
  `2f307bd19deb94265e7e62d09d24e92c4646b997e790c433d463d41d5761486a`.
  An owned pipe reader must return Pending after its fragmented bytes without
  waiting for another writer byte. RED cleanup supplies a continuation and
  joins the worker; GREEN needs no continuation.
- `/tmp/riotbox-1602-resource-eof-red.log`, SHA256
  `5351b6489d6d81cbd1857c7a2f597e2a40a96865de5db0a2dc9253d5b0ec6fdf`.
  Actual EOF must latch unavailable rather than re-enter the descriptor.

Six sequential decoder verticals, all under `/tmp/riotbox-1602-`:

- `byte-ascii-real-red.log` — one byte/control without draining the next:
  `6fdb4d977f891bd5f6f5463a5ab5a7fff77c7b6e6704b0733ee0a565b97a3132`.
- `byte-csi-red.log` — fragmented CSI final `f` is not a Fill:
  `1deaa1d1aec43b9e7df93d29ee08464dde3bc6349b7c767f4111b5fe3adcaa43`.
- `byte-escape-red.log` — Escape expiry precedes reading a late continuation:
  `e82366430e81bdbb3d32e60166d2b7e367b62c792a3fa2091cefef1f24c28613`.
- `byte-paste-red.log` — marked-paste controls cannot execute:
  `11c580f7d09f18b6a0ce97f56aadf33c88dc617edcd571480c221f9b9c9e682d`.
- `byte-utf8-latch-red.log` — malformed UTF-8 cannot release ASCII input:
  `8a4c6cbb4d2762b9992b08911b2bcdf098a5aaf32808bbaee19524af6bc64b30`.
- `byte-csi-limit-red.log` — continuously available incomplete CSI stays
  fixed-work and refuses a 65th sequence byte:
  `66b56f64bd18a09af5fdce62ac8efd913a96e1a2656628f5ee08085805531a07`.

## Acceptance and output proof

The final integrated recorder namespace passes 86 tests, zero failures/ignores:
`/tmp/riotbox-1602-final-integrated-recorder.log`, SHA256
`d0730faddbd05a14a02027e72977f9c660f5492ba9d8c2971914e7ffc567a712`.
This includes 16 decoder controls, 12 native-resource/composition controls, current
four terminal-result reporting controls and existing capture supervision.
Generated virtual time verifies no early commit, dedup/filter/Takeover behavior,
owned/foreign isolation, capacity, cancel/read failure, simultaneous terminal
precedence, late Core commit and unchanged recording/persistence cleanup.
Native EINTR evidence is error classification, not delivered-signal execution;
raw-mode lifecycle tests use FakeRawMode, not actual terminal attributes.

Two focused composition controls each execute one passing test:

- `/tmp/riotbox-1602-native-decoder-composition.log`: actual owned pipe →
  production NonBlockingReader → production ByteInput, partial CSI/Pending,
  coalesced final/control bytes without drain, EOF and unavailable latch.
- `/tmp/riotbox-1602-byte-controller-executed.log`: production ByteInput →
  actual recorder/App/Core. The exact requested timestamp 1560 distinguishes
  plain `f` from an earlier suppressed CSI-final `f`; pending dedup cannot hide
  a leaked control. Exactly 600 byte attempts stop before the completion-time
  cancel. The resulting single Fill commits at beat 4/time 3000 and survives
  actual Session save with its record/recording receipt.

The separate actual App projection control observes the existing real runtime
lane state before/at/after the bar boundary and retains empty audio caches.
The Audio-internal prepared production callback/CaptureTap control compares
Fill versus same-state no-Fill PCM, with exact post-limiter captured/output
bits, finite samples, positive difference only in the published Fill bar,
release to the no-Fill control, and zero callback heap operations. Final log:
`/tmp/riotbox-1602-audio-fill-capture-post-clippy.log` (one pass).
These are composed engineering proofs, not TTY→device E2E or a human verdict.

The actual binary with owned null stdin refuses before missing-Session load or
device startup and writes no artifacts: one pass in
`/tmp/riotbox-1602-non-tty-platform-refusal.log`. Parser compatibility controls
remain covered. `cargo fmt --all --check`, strict workspace all-target/all-feature
Clippy (`/tmp/riotbox-1602-clippy-green.log`) and `git diff --check` pass.
Full source-free local/native CI and exact-head PR/merge closeout remain gates;
their final results belong in the linked PR/Linear closeout.

## Review and remaining qualification

Independent Rust Safety / Operations resource-and-facade review and Spec And
Evidence review retain zero findings in their reviewed scope. A separate
independent Rust / Adversarial Implementation parser review reads the complete
fixed-state decoder and all sixteen controls against V2/RBX-464 and also retains
zero P0–P3 findings. Reviewers exclude their own authored components from their
independent claims. Review resolved
the blocking-reader P2 through the explicit V2 route and a P3 latent portability
error in the binary test's unconditional Linux error-text expectation. The test
now expects the deliberate non-Linux Unsupported refusal where applicable.
Coordinator Maintainer / Product Pragmatist / Risk self-review likewise finds
no remaining defect or required semantic refactor. The input resource, fixed
decoder and queue-correlation owners stay private and separate; no mechanical
include split or new facade state. The cohesive lifecycle's soft size signal
does not justify another fragment. An initial V1 review is not reused as a
new-parser pass. Ready for normal PR/CI gates, not yet merged or qualified.

Jev 1.13.0 supplied a bounded wording preference for the more explicit startup
brief: probability 0.94/confidence 0.91, 708 input/44 output tokens, estimated
cost $0.00002974. No audio, secrets or private Session data were sent. This is
advisory wording evidence, not code correctness, permission or qualification.

No new source/DSP recipe, threshold, recording-receipt meaning, persisted state,
TUI/DAW/Windows scope or dependency. No actual terminal/Session migration,
source/capture/Holdout/commercial/ignored-attempt access/discovery, host/device/
service observation/runtime or playback. Real-host input ergonomics, physical
latency, musical/human, source-general, hardness, endurance and release quality
remain unqualified. RIOTBOX-1589 stays passed with one-time approval consumed;
RIOTBOX-1036/1041 and the original ALSA cause remain open.
