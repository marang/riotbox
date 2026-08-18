# RIOTBOX-1442 W-30 Pitch-Dive Development Exploration

Date: 2026-08-18
Partition: Development only
Result: `provisional_keep`
Holdout access: none
Commercial-reference access: none

## Narrow Question

Can a source-backed W-30 phrase end with an immediately obvious downward pitch
gesture that remains recognizable, creates a useful transition, and feels worth
triggering live?

This was bounded musical discovery, not product implementation, source-diverse
qualification, hardness evidence, or release evidence.

## Bounded Source Access

Before any source read, a fresh access log admitted only the exact registered
Development case `dense_beat03_130`:

- path: `data/test_audio/examples/Beat03_130BPM(Full).wav`
- expected and actual SHA-256:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- declared tempo: `130.0` BPM
- access-log SHA-256:
  `556e86984a365cef8c0800d040e01c418b8a0d80446104b123ee1970745fe21a`

The session opened one Development file, performed no source-directory
discovery, and opened no Holdout or commercial-reference audio.

## Exact Kept Artifact

The existing product-path W-30 live-hook render supplied the source-backed
control. The exploration preserved its first eight beats sample-exactly, then
replayed the final four-beat exit through a continuous exponential slowdown
from rate `1.0` toward `0.35`, using a continuous cursor, linear interpolation,
and a `0.15`-beat terminal fade. Source PCM, the preceding W-30 transform, grit,
and bus level were unchanged.

| Artifact | SHA-256 | Duration | Peak | RMS | Clips |
| --- | --- | ---: | ---: | ---: | ---: |
| source anchor | `d22c8b7b5cafacebbf14ad89eafa868658ecc6dedae8598b5e26be59be9b7149` | `3.692313` s | n/a | n/a | n/a |
| input W-30 control | `a7326f8a16fd76d4b9d7959c3a871724a763555f05b0edc4f6fb1ac69d6c31f7` | n/a | n/a | n/a | n/a |
| bounded A control | `2f88e4bcfd8097d0dba26e36d54c1ba09b03a79ff1d888ebdea17d9cc4f5ab10` | `5.538458` s | n/a | n/a | `0` |
| B continuous tape brake | `95479de1a150993e76de4dc37e7eb3fb9f92e928655cedcdafe16ef42205b0e0` | `5.538458` s | `0.399884` | `0.066991` | `0` |

The control and candidate are stereo PCM16 at 48 kHz. Their intended final
four-beat delta RMS is `0.088134`; the candidate's terminal frame is silent.
The principal source/control/candidate review artifact SHA-256 is
`3c8c441f1fec7af62ca467aaf84d1cb378acd3a672afb9b93a9c8b265aaceace`.
The requested source-then-candidate confirmation artifact SHA-256 is
`a96450a74c6d2e1b0a9a6f443b9dbca7002c5376a8ae59fa881142941dad006b`.
Both stopped at their announced endpoints with playback silence verified.

## Human Result

The project musician judged all presented material useful, the pitch dive
especially successful, and the candidate clearly transformed while retaining
recognizable source identity. This is a provisional Development keep for
`continuous_tape_brake` and supports transfer observation of the exact frozen
recipe. It does not establish source-general quality, beat hardness, Golden
Path completion, or release readiness.

Three materially different candidates were rendered within the exploration
budget, but the stopping rule ended the search after the first topology earned
the keep. `stepped_machine_fall` and `late_plunge_choke` were not played and
therefore carry no human evidence or ranking.

## Promotion Boundary

The exact kept mechanism is frozen in
`docs/benchmarks/w30_pitch_dive_development_v1.json` under RBX-299. The
temporary exploration renderer is not product code and must not be promoted as
qualification evidence. A separate Linear-first slice may first observe the
unchanged recipe on four additional registered Development sources. Product
work must still rebuild the recipe source-blind through the existing W-30
queue/commit, Session/replay, observer/UI, and exact RuntimeMix paths, then
complete fresh source-diverse Development qualification and formal human
review. Transfer observation and qualification may reject v1 but may not tune
its curve, timeline, fade, or ownership.
