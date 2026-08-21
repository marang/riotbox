# RIOTBOX-1451 MC-202 Sub Pressure Shove Development Exploration

Date: 2026-08-21
Partition: Development only
Result: `development_keep_then_product_qualification_rejected`
Decisions: RBX-309, RBX-310, RBX-311
Holdout access: none
Commercial-reference access: none

## Narrow Question

Can the existing source-derived MC-202 Pressure role become a phrase-carrying
low-end pedal that adds clearly useful bass pressure without replacing the
source groove?

This was bounded musical discovery, not product qualification, percussive
hardness evidence, universal source-quality evidence, or release evidence.

## Bounded Source Access

A fresh access log existed before the only selected-file read. It admitted the
registered Development case `freesound_cyclez_493560`, path
`data/test_audio/external/RIOTBOX-1430/freesound/sparse_freesound_cyclez_493560.wav`,
declared tempo `150.0` BPM, and expected SHA-256
`4c17679b77bc78376282c5db118ef2fa9f87a5c669cf0cc284b91a38ad3fb485`.
The actual hash matched. The completed access-log SHA-256 is
`79b973eefb65ca05f58cfba90792a1915c68e3153ef67956a54a80ee0d2b84d6`.

No source directory was discovered. The access owner compared the exact
identity against active Holdout metadata before opening it. No Holdout or
commercial-reference audio was opened, hashed, decoded, rendered, classified,
or played.

## Bounded Variants

Variant 1 extended Pressure anchors into a continuous phrase but produced
limiter activity in the exact RuntimeMix preflight. It was rejected before
human playback.

Variant 2 changed the causal topology rather than applying a small level
adjustment. Each source-derived anchor carries until the next active anchor in
the sixteen-step cycle. The lane gain is multiplied by the square root of the
active-anchor fraction, preventing sparse plans from becoming disproportionately
loud. Destructive anchors keep their existing short articulation; ordinary
anchors use the source-derived inter-anchor duration. Existing pitch, pressure
sound design, touch, rhythm cells, two-bar turnaround, other lanes, and limiter
remain unchanged.

## Exact Kept Artifact

| Artifact | SHA-256 | Duration | LUFS | True peak |
| --- | --- | ---: | ---: | ---: |
| A: W-30 + TR-909 control | `c52af84397f3685e5f06565f197245d06db118579ea52c629405e41bc3e3b4dd` | `6.4` s | `-20.6` | `-4.9` dBFS |
| B: A + MC-202 pressure pedal | `7177ad49b69058fce3854460b47d5377fbfa2af6066f4db74a0a02a5c7c60f78` | `6.4` s | `-15.0` | `-0.7` dBFS |
| source / A / B review | `90c0903ef134e4d9f207b07323ca53d6ef21f3f9120c744934280f251c9569fc` | `21.7` s | `-17.2` | `-0.7` dBFS |

All files are stereo PCM16 at 48 kHz and have zero clipped samples. A and B
have waveform correlation `0.39581` and delta RMS `0.17784`; the delta is active
for `30.18%` of frames. It matches the isolated MC-202 stem within one PCM16
least-significant bit, so the audible A/B change is assigned to MC-202 rather
than an unnamed composite contributor. The MC-202 stem has RMS `0.17783834`
and peak `0.68861514`.

The exact controlled diagnostic reports one expected non-applicable failure:
its older source-role assertion says this source must not assign bass ownership.
The RIOTBOX-1451 experiment explicitly assigns MC-202 bass ownership, so that
legacy assertion cannot qualify or reject this mechanism. Every applicable
safety gate passed: no pre-limiter clips, limiter intervention, or post-limiter
clips occurred in the kept held and destructive renders.

## Human Result

After exact-artifact preflight and fresh readiness, the project musician heard
source, A, and B. Both transformations were judged musically successful. The
additional gentle bass underneath B was correctly identified as MC-202 and was
judged clearly audible and valuable. The source-derived timing and recognizable
transformation remained intact. This is a provisional Development keep for the
exact second variant, not a formal product verdict.

## Attempted Promotion

The exact heard mechanism is frozen as `mc202_sub_pressure_shove_v1` in
`docs/benchmarks/mc202_sub_pressure_shove_development_v1.json` under RBX-309.
The subsequent source-blind rebuild added the proposed explicit Session profile,
queue/commit projection, replay, observer, callback, and RuntimeMix proof. The
frozen qualification contract then required a fresh fixed Development source
set and prohibited retuning after source access.

## Frozen v1 Product Qualification Result

The first exact case of the frozen v1 product matrix stopped fail-closed before
human playback. The product timing probe committed `149.79622` BPM and derived
four active anchors, two of them destructive. The unchanged v1 anchor-count
multiplier did not account for the resulting effective held duration: the
isolated MC-202 was active for `48.92%` of frames at RMS `0.250163`. The held
mix produced `42` pre-limiter clips and required `552` limited samples. The
failed exclusive access-log SHA-256 is
`33e84061805658eccf8c2687114e53087992018577528b3a0cdf1ab479ab87fd`;
the failed exact manifest SHA-256 is
`ccb258617c1145824b8f971792a7e03a15c9ab37a497f92106069fc93c15e366`.

No later source in the matrix was opened. v1 was not retuned, and its positive
single-source exploration remains valid only for the exact heard artifact. The
transfer claim is rejected. Per the ticket's no-tuning-after-qualification rule,
no v2 successor is admitted here. The unqualified product behavior and its
qualification-only renderer/script were removed; existing historical MC-202
Pressure behavior remains unchanged. A successor requires a new Linear-first
audible slice with a materially different mix/ownership mechanism, not another
scalar or duration-normalization retry on this evidence.
