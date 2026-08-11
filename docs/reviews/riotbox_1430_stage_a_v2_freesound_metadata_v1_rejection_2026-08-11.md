# RIOTBOX-1430 Stage-A-v2 Freesound Metadata-v1 Rejection — 2026-08-11

## Verdict

`rejected_fail_closed_no_audio_access`

The one metadata-only pass authorized by RBX-257 is consumed and must not be
retried or resumed. The first exact Sound Instance request returned one
bounded status-200 JSON object, but the helper rejected before accepting the
entry because the `license` value was not byte-equal to the expected
documentation label `Creative Commons 0`.

This is a license-representation validation rejection only. It is not a
license, source-family, format, event, hardness, force, musical-quality, or
human-listening verdict.

## Durable Attempt Facts

- Decision: `RBX-257`
- Repository HEAD: `ffdbb676f74d4e52ad6eccb702b2768585ac512c`
- Ordered request started: `1` of `3`
- Sound identity requested: Freesound `724939`
- HTTP result: status `200`, bounded JSON at or below `65536` bytes
- Accepted metadata entries: `0`
- Rejection stage: `request_1_license_representation`
- Expected representation: exact JSON string `Creative Commons 0`
- Actual representation: `not_durably_captured`
- Raw response byte count / SHA-256: `not_durably_captured`
- Attempt ID / access-log SHA-256: `not_durably_captured`
- Later requests for `493560` and `458897`: not issued
- Further requests after rejection: false
- Metadata snapshot publication: not started

The helper did not create a durable access log before touching the network and
did not preserve safe response evidence before semantic validation. Therefore
the exact returned license scalar, raw response bytes, response hash, and an
attempt UUID are not part of the evidence. They must not be reconstructed or
invented after the fact.

## Source-Blind Root Cause

After the rejection, official provider material was inspected without calling
another sound endpoint. Freesound's API documentation describes `license` as a
string such as `Creative Commons 0`, while the official source tree at commit
`4318dcdce8dbf5663658e0d9287401bb0ff5e140` serializes
`obj.license.deed_url`; its CC0 fixture binds that deed URL to
`http://creativecommons.org/publicdomain/zero/1.0/`. The inspected source is a
plausible provider-schema explanation, not evidence that this exact commit was
deployed for the consumed response.

Provider evidence:

- [Sound Instance documentation](https://freesound.org/docs/api/resources_apiv2.html#sound-instance)
- [Freesound serializer at the inspected commit](https://github.com/MTG/freesound/blob/4318dcdce8dbf5663658e0d9287401bb0ff5e140/apiv2/serializers.py#L220-L223), raw SHA-256 `760280657c7fbe0958d6e51328ca78bb8b545727ea6988fa5b8c54fb73d43360`
- [Freesound license fixture at the inspected commit](https://github.com/MTG/freesound/blob/4318dcdce8dbf5663658e0d9287401bb0ff5e140/sounds/fixtures/licenses.json), raw SHA-256 `f2e47f403c3a87b9222c834b4ea2e6968fa8b95e63286cbfc71b9901eec2502c`
- [Official Creative Commons CC0 1.0 deed](https://creativecommons.org/publicdomain/zero/1.0/)

This provider-wide serialization fact explains a plausible representation
mismatch; it is not evidence of the exact value returned in the consumed
request. The defect was expecting only the prose label and, independently,
failing to make request intent and bounded response evidence durable before
validation.

## Access Boundaries Preserved

No original-file or download-endpoint request, preview access, audio byte,
source-directory access, source or holdout audio, commercial reference,
decode, PCM iteration, detector/anatomy/source-contrast computation, event
selection, candidate render, or playback occurred. The account credential was
not written to argv, the environment, the repository, an ignored project file,
the exception, stdout, or stderr.

## Change Control

A further metadata pass requires a new targeted decision and a separately
versioned mechanism that, before DNS or socket activity:

1. creates and fsyncs a unique no-replace access log;
2. durably records request intent before each possible request;
3. preserves bounded non-secret response evidence before semantic validation;
4. accepts only a closed, source-independent set of official CC0
   representations;
5. rejects the entire attempt without survivors on any failure.

Protocol P2, Registry v2, Matrix v2, detector/anatomy/source-contrast catalogs,
event ordinals, algorithms, thresholds, and numeric passports remain unchanged.
No original-file download is authorized by this report.
