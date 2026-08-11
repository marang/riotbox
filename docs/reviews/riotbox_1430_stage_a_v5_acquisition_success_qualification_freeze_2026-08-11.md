# RIOTBOX-1430 Stage-A-v5 Acquisition Success and Qualification Freeze

Date: 2026-08-11

The final v5 continuation completed all eight authorized original-file requests
for pool ordinals 8–15. Every response matched the frozen byte count and MD5 and
passed strict RIFF/PCM header admission. Together with ordinals 1, 2, 4, 5, and
6 from the earlier v3 access, Riotbox now has thirteen exact new Development
sources eligible for event qualification.

Pool ordinals 3 and 7 remain useful format-negative evidence: 3 is IEEE Float32
and 7 has an incoherent extended `fmt` chunk. Neither is a musical-quality
rejection.

No source was played, no PCM sample was iterated, no source feature or event was
computed, and no candidate was rendered during acquisition. Holdout audio and
commercial references remained untouched.

RBX-268 freezes the thirteen exact paths, SHA-256 identities, formats, authors,
families, and qualification order in
`docs/benchmarks/percussive_force_stage_a_bound_source_set_v1.json`. The next
and only action is one fresh Development-only qualification over this set,
followed by deterministic four-source selection and Matrix v6 only if the
unchanged mechanical gates pass.

Current evidence state: `quality_proof=false`, `hardness_proof=false`,
`human_verdict=unverified`.

The compact v5 qualification runner reuses the existing frozen decoder,
Detector, Anatomy, source-feature, and contrast implementation. Its metadata-
only contract preflight and the shared synthetic Stage-A analysis fixtures pass
before the runner's implementation-freeze commit. The runner analyzes every
bound source once, then applies the preregistered lexicographic combination
rule; it does not acquire, discover, render, or play audio.
