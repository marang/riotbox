# RIOTBOX-1470 Dense W-30 Prior Evidence Reconstruction

Date: 2026-08-26  
Status: source-blind historical reconstruction; no current qualification result  
Owner: RIOTBOX-1470

## Purpose

This record reconstructs the ordinary promoted W-30 control that preceded the
RIOTBOX-1444 Pitch Dive action. It makes the previously reviewed audio identity
and its product-path identity explicit before RIOTBOX-1470 opens Development
audio. It does not create a new verdict, upgrade the prior verdict, or claim that
the current product still renders identical bytes.

## Prior Human Evidence

- Prior ticket: `RIOTBOX-1444`
- Durable review document SHA-256:
  `42a66e70c4688f78e91311cbbf68240a000498e430875d57751220f0f49e0c85`
- Structured review SHA-256:
  `99b757860d90c2bceb055c586a8005b64ee867afaea8cccfc7799469fab71a8e`
- Exact reviewed ordinary-control audio SHA-256:
  `7140c8f24e383dc6a7cb75bc6183e03727ef8b5f068b28e9d08ead8371a5ebab`
- Reconstructed product manifest:
  `docs/benchmarks/dense_w30_foundation_prior_product_manifest_v1.json`
- Reconstructed product-manifest SHA-256:
  `6b593663a24ae130e2352ea1dcbe09489ba86ac3f32d8efb68b6ac7c4709c69a`
- `human_verdict: keep`
- Strongest element: `chop`
- Source recognition: `source_transformed_but_present`
- Hook after two bars: `clear`

The structured RIOTBOX-1444 verdict judged a Pitch Dive comparison, but its
control was the exact ordinary promoted W-30 path and was explicitly retained
as useful material. The review access log binds the control audio hash above.
The manifest reconstruction uses only pinned historical JSON and Markdown
metadata: the six committed control actions, one capture lineage, confirmed
timing, preset and macro/mixer state, ordinary W-30 playback state, isolated
contributor list, render metrics, and exact audio hash. No historical WAV was
opened or played during this reconstruction.

## Current Reuse Boundary

RIOTBOX-1470 may reuse this existing verdict only if one bounded Development
qualification reproduces both the exact audio SHA-256 and the canonical product
manifest SHA-256 above. Any mismatch fails closed under the v1 contract; it may
not be explained away by listening, tolerance, retuning, or a second render.
Hash-identical reuse creates no new human verdict and requires no additional
playback.

The maximum possible result is one Dense demo-family foundation journey owned
by the ordinary W-30 source transformation while TR-909, MC-202, and Source
Monitor stay out. It is not source-general, Holdout, hardness, release-readiness,
universal-quality, automatic-arrangement, or P023-completion evidence.
