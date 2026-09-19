# RIOTBOX-1497 — Capture content identity

Scope: maintenance/regression follow-up, compared with `695b5cdf` on
`feature/riotbox-1497-capture-identity`. Preserves capture/restore trust in the
P023 playable path; no new musical mechanism or quality claim.

## Contract and implementation

- Core `CaptureRef.audio_identity` owns the expected complete-WAV SHA-256 and
  typed creation/adopted-legacy provenance (RBX-375).
- Source-window capture and internal bus printing hash the encoded buffer used
  for writing and immediate cache decoding. Restore hashes/decodes one read.
- Invalid, changed, absent and legacy-unverified content cannot enter the cache.
  File-backed preview and resampling cannot substitute original source audio.
- Explicit selected-ID migration previews by default. Acceptance adopts current
  bytes, not historical authenticity; it never overwrites an existing identity.
  All selected entries validate before atomic Session publication. Repeat
  adoption is byte-preserving. Existing single-writer constraints still apply.
- Runtime warnings expose degraded content; recovery inventory explicitly
  describes path readiness with content identity unchecked. Inventory does not
  decode audio or imply content verification.

## Review

Solo review using `code-review` and `code-review-rust`; no independent agents,
as requested. Applied maintainer, product, evidence, adversarial and migration
lenses sequentially. No unresolved finding in this bounded change.

Resolved during review:

- Adversarial lens: duplicate capture IDs could retain a previously trusted
  cache entry. A regression failed before the duplicate-ID prepass and passes
  with every duplicate excluded.
- Evidence/product lens: path-only recovery inventory said artifacts were
  ready. It now explicitly leaves content identity unchecked; actual hydration
  remains authoritative.

No new ActionCommand: this is offline metadata migration, not a queued performer
gesture. Creation still uses existing capture actions and commit paths. Replay
preserves CaptureRef metadata; adopted legacy snapshot/replay tests prove actual
artifact-backed output. Expected identity is not hidden in JamAppState; only
observed load status lives in existing AppRuntimeState. No new persistence or
replay system and no additional dependency. New modules have semantic ownership;
existing large test files only receive fixture/assertion changes.

## Evidence and limits

- Red/green: legacy capture trust and duplicate-ID cache regressions.
- Synthetic tests: creation hashes match written bytes; internal resample
  identity survives reload; explicit adoption enables snapshot/replay output;
  changed PCM, malformed hashes, missing/invalid WAVs fail closed; preview,
  partial selection failure, repeat adoption and existing-hash refusal preserve
  Session bytes as specified. CLI parsing requires explicit selection/acceptance.
- Full source-free `just ci`: passed, including 749 App library tests, two
  migration CLI tests, all workspace tests, synthetic audio/observer/manifest
  gates, Python/contracts/JSON, formatting and strict Clippy. A read-only CLI
  preview also passed against the newly generated synthetic smoke Session.
- No real Session migration, source/holdout/commercial audio access, hardware
  playback, human listening or sound-quality verdict. This integrity maintenance
  does not change the trusted capture DSP. OS/filesystem matrix and concurrent
  writers are not claimed. A hash protects bytes relative to its persisted
  baseline, not authenticity against coordinated Session-and-WAV modification.

Usage is documented in the Session spec; keep a backup and close other writers
before explicit legacy adoption. Old Sessions remain readable but unverified
capture audio is unavailable until explicitly adopted.
