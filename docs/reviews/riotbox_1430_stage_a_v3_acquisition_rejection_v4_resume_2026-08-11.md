# RIOTBOX-1430 Stage-A-v3 Acquisition Rejection and v4 Resume

Date: 2026-08-11

Protocol v3 stopped fail-closed at original-file request 7 of 15. Requests 8–15
were not made, no qualification started, no PCM samples were iterated, and no
audio was rendered or played.

The bounded result was:

- five strict PCM header-only admissions: ordinals 1, 2, 4, 5, and 6;
- one expected unsupported-format rejection: ordinal 3, IEEE Float32;
- one complete body with matching identity and core PCM24/48 kHz fields at
  ordinal 7, rejected because the inherited parser required a 16-byte `fmt`
  payload exactly;
- zero holdout-audio or commercial-reference access.

This is a container-admission failure, not a source-quality, event, force, or
human verdict. The v3 access log is pinned by RBX-266 and remains local and
ignored.

Protocol v4 versions only the PCM/WAVE admission component. It permits the
base 16-byte PCM structure or a bounded coherent WAVEFORMATEX payload whose
declared extension length matches the actual `fmt` payload. All detector,
anatomy, source-contrast, event-ordinal, F1, F2, and F3 behavior remains frozen.
The five unanalysed admitted files are retained, Float32 stays rejected, and
only ordinals 7–15 may be requested under the new freeze.

Evidence state remains `quality_proof=false`, `hardness_proof=false`, and
`human_verdict=unverified`.
