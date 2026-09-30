# RIOTBOX-1505 — Original-source file admission

Classification: maintenance/regression. Branch:
`feature/riotbox-1505-source-file-admission`; review base:
`e51887e7c012c674ea12879b81ff28fbabece847`. The P023 risk removed is a
Session/source load stranded on an invalid local file type before it can report
unavailable. This is not an audible quality improvement or a DAW prerequisite.

## Behavior and ownership

Audio's semantic `source_audio::file_io` module exposes one
`read_source_wav_bytes` interface. Direct cache loading and App original-source
restore use it rather than independent `fs::read` calls. It opens once, uses
`O_NONBLOCK` on Unix and checks the opened descriptor's metadata for a regular
file before reading. The returned buffer remains the common input to decoding
and original-content hashing; SourceRef/Graph identity checks are unchanged.

Original-source symlinks to regular files remain valid. FIFO and FIFO-symlink
inputs reject without a producer; directories reject as I/O errors. Capture
and export no-follow policies remain untouched. App reports the existing
`SourceAudioStatus::Unavailable` and admits no source cache/monitor payload.

The musician still opens the same Session. An invalid source reports unavailable
instead of hanging; valid original audio is decoded as before. No ActionCommand,
queue/commit consequence, Session/replay schema, `JamAppState` state or DSP is
added. No new fallback audio, file-size limit or cache identity is invented.
Regular-file latency, same-inode concurrent modification, remote filesystem
behavior and Windows special-file semantics are not qualified by this slice.

RBX-381 and the audio-core spec freeze the opened-file/symlink distinction.
`libc` retains its existing resolved version; workspace ownership is shared with
App and Audio's dependency is Unix-only. Cargo.lock adds only that dependency
edge, not a package upgrade. New production and test modules are semantic and
small; no textual includes are added or rearranged.

## Failure proof and regression coverage

Before implementation, the new no-producer public-interface regressions failed
on both original paths after their outer four-second safety bounds:

- `/tmp/riotbox-1505-audio-red.log`: direct loader child remained blocked.
- `/tmp/riotbox-1505-app-red.log`: Session restore child remained blocked.

Each parent kills/reaps its bounded child before reporting failure. No helper
requires a producer or a sleep to release the faulty loader. Fixed tests cover
both direct FIFO and symlink-to-FIFO through the same public interfaces.

Other focused checks cover ordinary PCM WAV, a symlink to regular WAV, directory
rejection, exact returned bytes and an opened descriptor whose original path
is replaced by a FIFO. The last test reads the original descriptor without
re-resolving the new path. App's regular-symlink restore then mutates PCM bytes
and requires hash-mismatch unavailability with unchanged Session JSON.

Focused results: Audio source-related tests 22 passed, App file-admission tests
3 passed; logs `/tmp/riotbox-1505-audio-green.log` and
`/tmp/riotbox-1505-app-green.log`. Locked Cargo metadata confirms the Unix-only
dependency and `git diff --check` passes. Full `just ci` passed
(`/tmp/riotbox-1505-ci.log`): App 770, Audio 279, Core 470 and Sidecar 14
library tests, binary tests, Python/contract fixtures, source-free synthetic
audio/observer/manifests, formatting, tracked JSON and strict Clippy.

## Branch review

Sequential `code-review` / `code-review-rust` lenses cover the App caller,
shared reader, descriptor/substitution and symlink cases, error propagation,
test process bounds, Cargo ownership and spec/decision alignment. No independent
reviewer approval is claimed; subagents are not used.

No retained P0–P3 branch finding. An implementation check caught duplicate I/O
prefix formatting when switching from `std::io::Error` to `SourceAudioError`;
the App now uses the shared error's display once. Negative-status tests and
inspection constrain the diagnostic path. The short self-review confirms the
two production callers share admission, decode/hash share bytes, no runtime
code reopens the admitted path, and only the intended Cargo dependency edge
changes.

## Evidence limits

All tests use fresh synthetic files, metadata, FIFOs and local subprocesses.
No real source, active holdout, commercial reference, host audio, human playback
or DAW is accessed. Source-free CI audio metrics do not grant a human verdict.
The Unix FIFO regression proves file-type admission, not a universal read
deadline or resistance to unbounded regular-file allocation. RIOTBOX-1504
separately owns the still-unfixed Sidecar request-write timeout.
