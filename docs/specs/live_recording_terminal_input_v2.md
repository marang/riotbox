# Live-recording terminal input V2

Status: frozen before the replacement input implementation.
Date: 2026-10-17. RIOTBOX-1602 / RBX-464.

## Scope and reason

This supersedes V1's terminal ingress, not its queue, recorder, observation,
ownership, cleanup or evidence contracts. V1 remains historical. The pinned
Crossterm 0.29.0 Unix event decoder can block on an incomplete sequence inside
`poll(Duration::ZERO)`. An executed anonymous-pipe witness reaches that actual
decoder; fake event-call counts cannot qualify its work bound.

Use the existing `libc` dependency for a private Linux input reader and a small
incremental decoder. Do not fork Crossterm, change its version, use its event
reader, create a worker or drain input. Crossterm retains raw-mode resource
management only. Other operating systems reject this opt-in with Unsupported
before terminal/device setup; disabled/default execution stays unchanged.

## Resource ownership

Require actual stdin to be a TTY, with one reader and no concurrent descriptor
flag mutator. Save its original `F_GETFL` flags and add only `O_NONBLOCK` before
audio start. The descriptor is not reopened through `/dev/tty` or procfs.
`dup` does not isolate open-file-description flags. This temporarily affects
aliases sharing that description, including inherited aliases; exclusive use
during this command is a prerequisite, not an automatically verified fact.

Preserve V1's prior raw-mode ownership. Roll back acquired resources on setup
failure. At explicit cleanup attempt both original flags and owned raw-mode
restoration even if either fails, retaining both errors. Retry unfinished
restoration best-effort on Drop. Never claim success without confirmed cleanup;
already committed Session/receipt/artifacts remain valid if cleanup fails.
No terminal attributes, stdin, host or device are touched by engineering tests.

## Bounded byte ingress

Only after Continue, consume at most one byte using at most one nonblocking
`read` call. No read retry, readiness loop, event queue or growing input buffer.
WouldBlock and Interrupted return control to capture supervision; EOF and other
I/O errors use the existing stop → abort → rejection path. Completion/faults
retain precedence. Kernel scheduling/stalls and synchronous setup/cleanup are
not bounded realtime operations. The existing pending wait remains 10ms.

Decoder storage is fixed and processing per call is constant. Plain ASCII `f`
requests Fill; `q` and byte 0x03 cancel. Other plain ASCII is ignored. These are
received byte controls, not proof of unmodified physical Press, Release or
Repeat. Unmarked paste, legacy autorepeat and terminal mappings can produce
the same bytes. Internal KeyEvents are normalized tokens only; they do not
restore the full Crossterm key-protocol semantics. No enhanced keyboard protocol
is enabled, negotiated or claimed. Uppercase `F` is not normalized to `f`.

An Escape prefix has a fixed 50ms monotonic ambiguity interval, measured from
its receipt. At or after expiry, the next pending input attempt emits cancel
without reading another byte. Before expiry, a continuation is interpreted as
part of that sequence. This is a terminal-only clock, not a transport clock.
A delayed/fragmented sequence can therefore cancel the take; no universal host
ergonomics or backlog-independent latency claim is made. Escape cancellation
is still subordinate to the next capture completion/fault check.

Recognize and suppress ordinary CSI (`ESC [`), SS3 (`ESC O`) and Alt character
sequences. CSI/SS3 parameter bytes 0x30–0x3f precede optional intermediate bytes
0x20–0x2f and one final byte 0x40–0x7e. At most 64 bytes including Escape are
allowed; malformed or overlong sequences fail closed. Never release an
embedded `f` or `q` as a plain control. Unsupported control-string introductions
(`ESC ]`, `ESC P`, `ESC X`, `ESC ^`, `ESC _`) fail closed instead of guessing
where their payload ends. Escape intermediate sequences (0x20–0x2f followed by
one final byte 0x30–0x7e) are suppressed as a whole, not mistaken for an Alt
punctuation character followed by a fresh control. No resize/signal processing
or shell command runs. Any read/decoder error latches the input unavailable;
subsequent calls cannot reinterpret the remaining bytes after that error.

Recognize exact bracketed-paste opener `ESC [ 2 0 0 ~`. Suppress all bytes until
the exact closer `ESC [ 2 0 1 ~`, retaining only a fixed closer-prefix state.
Permit at most 65,536 bytes after the opener, including the closer; an overrun
fails closed. Escape ambiguity expiry does not apply inside a recognized paste
or CSI/SS3. A marked paste is one ignored token after closing, never a set of
performer actions. Do not enable/disable bracketed-paste mode or claim that
unmarked paste can be distinguished.

Ignore complete valid UTF-8 scalars, including Alt-prefixed scalars. Invalid
UTF-8 fails closed before an offending ASCII control can be reinterpreted.
Partial decoder state survives only pending iterations. No bytes from an
incomplete or invalid token become a new control by resetting the decoder.
Ignored-event counts describe completed normalized tokens, not physical keys.

## Verification and unchanged boundaries

Before the repair, retain the actual pinned-decoder fragment RED and its empty/
complete-key controls. Then add a genuine executed RED→GREEN at the replacement
byte-reader seam with an owned anonymous pipe, fragmented `ESC [` and no writer
continuation: the supervisor must regain control. Test the exact one-byte/read
bound, continuous incomplete CSI, Escape expiry, ASCII/Alt/CSI/SS3/UTF-8,
marked/unmarked paste, malformed/overlong tokens, EOF/Interrupted/WouldBlock,
exact original flag preservation, setup rollback, both cleanup attempts and
failure reporting. Do not count a build failure or fake-op count as that RED.

Retain V1's real App/Core queue/controller tests and separate production
callback/CaptureTap Fill-versus-control proof. Add a byte-adapter→actual-recorder
composition control. Default summaries and WindowStopped V1 remain unchanged;
opt-in WindowStopped V2/input mode/context and recording V3/V4 meaning remain
as frozen in V1. No new action, source/DSP recipe, receipt or persistent state.

Only generated bytes/metadata/in-memory PCM and owned pipe/temp fixtures are
authorized. No real terminal/Session/source/capture/Holdout/commercial access,
host/device/service observation/runtime, playback, DAW or Windows-product work.
No physical latency, human/musical, source-general, hardness, endurance or
release qualification. Passed/consumed1589 and open1036/1041/original ALSA cause
remain unchanged. Independent review, full source-free local/native CI and
normal workflow closeout remain required.
