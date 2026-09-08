# RIOTBOX-1493 — DAWproject archive ownership

Classification: maintenance/regression. Direct follow-up: RIOTBOX-1494,
unchanged committed V2 live-master recording to DAWproject.

## Scope

The existing W-30 exporter mixed musical eligibility, project/proof semantics,
archive serialization, exact readback, publication, and receipt orchestration.
Only the archive mechanism moves to an app-local module with a small interface.
W-30 policy and the existing Action/Session/receipt contracts remain in place.
There is no second export state or runtime audio path.

The input identity check is also made single-buffer: hashing, PCM inspection,
and archive embedding operate on one read. The old hash/read separation could
reject drift during archive validation, but the additional decoder reopen could
inspect a different file generation than the embedded bytes.

RBX-371 records the ownership decision. The extraction must preserve the
previous W-30 archive bytes, not merely produce another deterministic archive.

## Evidence boundary

Only synthetic fixtures are used. No registered source, Holdout, commercial
reference, or existing human-review WAV is opened. No human playback is needed:
this change neither creates a new sound nor changes the accepted audio.
RIOTBOX-1492's keep remains limited to its exact hook-plus-beat composite.

Baseline `cargo test -p riotbox-app w30_hook_dawproject` passed before extraction
(`/tmp/riotbox-1493-baseline.log`). After extraction, the three archive tests
and five W-30 tests pass, as do formatting and whitespace checks. The archive
test compares bytes against the original writer call sequence retained as a
test reference. The W-30 model/proof construction is unchanged in the diff.

Full source-free `just ci` passed (`/tmp/riotbox-1493-ci.log`), including Rust
tests, synthetic audio/observer and manifest checks, sidecar contracts, tracked
JSON, and Clippy. A final `cargo test -p riotbox-app dawproject --no-fail-fast`
passes all eight relevant tests (`/tmp/riotbox-1493-final-focus.log`).

## Review

Initial independent Rust branch review compared the diff and new files against
`4e88e1cf`: zero P0–P3 findings. It checked the narrow app-local interface,
exact four-member/model/audio/proof readback, no-clobber publication,
hash-owned cleanup, unchanged W-30 policy/Action/receipt ownership, and
single-buffer input identity. GitHub CI subsequently found the test-portability
issue below; the initial zero-findings review did not catch it.

The implementation review corrected an initial draft that retained the W-30
audio filename inside the shared module: the final module accepts one safe
relative audio member and derives its exact member set, while the W-30 caller
owns its frozen filename. This avoids a W-30-only pseudo-abstraction.

Follow-up self-review: zero outstanding findings, no new Action or Session
fields, no new textual includes, and no product-quality claim. The existing
W-30 policy file remains larger than the soft budget because its cohesive
eligibility/model/proof/receipt policy is retained; no mechanical split is
introduced. RIOTBOX-1494 remains the actual musician-facing follow-up.

## CI correction

GitHub run `34255676291` passed 718 app tests but rejected the added fixed W-30
archive hash: local `86602ee9...` versus runner `c42f04b3...`. The hash had been
measured after extraction and was not a historical compatibility baseline.
Pinning it also asserted cross-host byte identity of an unpinned render
fixture. That is not the contract under test.

The fixture uses `f32::sin()` (`p016_product_export_action.rs`) and the resulting
samples pass through RuntimeMix and PCM quantization before archive embedding.
[Rust documents platform-dependent precision](https://doc.rust-lang.org/std/primitive.f32.html#method.sin)
for this operation. Platform-dependent sample generation is the leading
hypothesis, not a proven identification of the differing archive member.

The correction compares the actual W-30 export with the previous writer
sequence using the same exact input on each host. It does not change a product
threshold, accept a list of machine-specific hashes, or alter DSP/archive bytes.
Repeat-export determinism, exact embedded audio, model/proof readback and receipt
hash checks remain required. The eight focused tests pass after correction.
Independent delta review against `b4cff4c0` found zero P0–P3 findings and
confirmed that only test code and review documentation changed. Self-review
also found no outstanding issue. The corrected full local and GitHub CI runs
remain merge gates.
