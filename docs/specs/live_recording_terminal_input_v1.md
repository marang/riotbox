# Live-recording terminal input V1

Status: frozen before implementation. RIOTBOX-1602 / RBX-463.
Date: 2026-10-17. Owner: the existing master-recording CLI/controller.

## Intent and compatibility

Make the existing lowercase `f` Fill action reachable during a bounded master
take. This integrates an existing performer gesture; it adds no musical
mechanism, recorder, action vocabulary, DSP or source intelligence.
`--live-recording-terminal-input` is explicit opt-in and valid only with
`--live-master-recording-execute`. Existing2/8/16-bar durations, recording
receipts/proofs, callback tap and writer remain unchanged. Omission retains
disabled input and existing serialized observer/summary behavior; disabled
mode must not inspect/configure a terminal or poll/read input.

The launch owns a small typed `LiveRecordingInputMode` (`Disabled`,
`TerminalFill`). The input adapter owns terminal resources only, never Session,
recording progress or action truth. The existing App queue owns all actions.

## Terminal and input lifecycle

Opt-in requires an actual input TTY and a single event reader. Acquire guarded
raw mode before audio-device start, preserve prior state, and restore on every
exit before final JSON. No alternate screen, drawing loop, stdin worker,
unbounded drain or new audio thread. Surface restoration errors separately
from an already valid committed recording; do not erase receipts or falsely
report that published artifacts are absent. Terminal setup/transport startup
remain synchronous, not instantly cancellable.

Only after `CaptureWait::poll` returns Continue may the wrapper attempt one
nonblocking event poll/read, then retain the existing10ms pending wait.
No input read or UI/observer I/O may intervene after recognized completion and
before the existing verified stop. Completion/fault checks precede input:
an unread cancel may lose to completion and normal finalization. Neither
instantaneous abort nor a backlog-independent input latency is promised.
Events are received terminal input, not proof of fresh physical keypresses;
do not infer freshness from paste/typeahead.

Accepted key events are Press only:

- unmodified lowercase `f`: request the existing `Tr909FillNext`/NextBar action;
- unmodified `q` or Escape, or ordinary `c` with Control only: cancel the take
  if read while Pending;
- Release, Repeat, uppercase `F`, other modifiers, keys and events: ignore.

No normalization of uppercase `F`: it already names a different preset in the
interactive shell. At most one event is consumed per pending iteration.
Deduplicate against the real queue's pending Fill command. Ignore a Fill
request while the current Takeover policy masks Fill; do not invent an audible
success or alter the existing shell/renderer policy.

## Action ownership and terminal cleanup

Use `JamAppState::queue_tr909_fill`, returning its existing typed ActionId for
correlation/cleanup rather than creating another enqueue path. Transient owned
IDs are correlation only: derive status from the actual queue/Core records,
never a second pending or committed history. On every terminal path explicitly
reject only still-pending input-owned actions. Never erase or relabel an
already committed action or an unrelated pending action. Input read/ActionId
failure and read cancellation consume the existing stop → abort → rejection
path with no recording publication. Preserve historical cleanup and observed
stop failure, including safe device stop when transport action allocation fails.

Post-stop input reporting separates queued requests, Core commits, ignored
events and input-owned requests retired without commit. A final sync may
commit outside the captured window; never relabel such a commit as ignored or
claim its effect is in the recording. All authoritative committed history stays
in Core/Session. Pending input ownership/counters and terminal state need no
restore/replay and must not become new persisted App state.

## Observation and evidence

Default mode retains `non_interactive_real_audio_cli` and unchanged
`riotbox.live_master_recording_window_stopped.v1` serialization. Opt-in mode
uses context `terminal_input_real_audio_cli`, typed input mode `terminal_fill`
and observation `riotbox.live_master_recording_window_stopped.v2`, with the
same plan-bound stop clocks/identity plus explicit input mode. Derive context
from typed launch mode for failure/final events and stopped milestones.
Do not change old frozen observer/host contracts or historical evidence.

Recording V3/V4 proofs still establish the exact master window and its existing
health/artifact/Session identity, not a particular keypress/action/frame link.
Core commit follows observed callback time; publication of a lane state is not
an ActionId/first-rendered-frame acknowledgment. Late or superseded delivery,
Takeover and state availability can shorten or suppress a Fill. Do not promise
sample-exact bar onset or audible execution of every queued/committed request.

The agreed engineering seams are the existing CLI parser, actual recorder
wrapper with generated RuntimeControl/input adapters and real App/Core queue,
and the Audio-internal prepared production callback plus existing CaptureTap.
Execute genuine one-test RED→GREEN input verticals before integration. Prove
no early commit, pending-only bounded input work, default compatibility,
dedup/ignored/late input, completion priority, cancellation/IO/admission/capacity
cleanup, ownership isolation and observation identity. Separately compare Fill
capture PCM against a same-state no-Fill callback control, including release
at the next bar. This is composed execution proof, not App→device E2E.

Only generated metadata/in-memory PCM and owned temporary fixtures are in scope.
No real Session migration, source/capture/Holdout/commercial/ignored-attempt
audio access/discovery, host/device/service observation/runtime, playback,
DAW or Windows-product work. No new recipe/threshold or musical, human,
source-general, hardness, physical-latency, endurance or release qualification.
1589 stays passed with its one-time approval consumed.1036/1041 and the original
ALSA cause remain open. Native/local source-free CI and normal workflow closeout
remain gates; real-host input ergonomics is a separate unqualified follow-up.
