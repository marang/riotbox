# RIOTBOX-1438 Cut-Hit-Return Development Qualification

Date: 2026-08-15
Partition: Development only
Result: `terminal_fail_closed`
Holdout access: none
Commercial-reference access: none

## Frozen Boundary

RIOTBOX-1438 froze `tr909_cut_hit_return_v1` in
`docs/benchmarks/tr909_cut_hit_return_development_v1.json` under RBX-293 before
opening Development audio. The gesture composed the existing next-bar Fill and
Slam actions. It did not introduce a new renderer: the Fill supplied the
existing whole-bar source-bed cut, negative space, and late hit; the following
bar returned to the existing break reinforcement with Slam held.

The stopping rule was terminal: technical failure or perceptual near-identity
would close v1 without retuning its mappings, constants, or gates.

## Bounded Development Result

A fresh bounded access session opened only the three exact registered
Development files. Their hashes matched the contract. No source directory was
searched, and no Holdout or commercial-reference audio was accessed.

- `dense_beat03_130` qualified for the exact RuntimeMix candidate.
- `tonal_rusharp_120` returned the frozen typed source-character refusal.
- `sparse_kicksnr_120` returned the frozen typed source-character refusal.

The dense candidate passed every mechanical gate. The negative-space window
was digitally silent (`RMS 0.0`) while the Slam-only control measured
`0.079062` RMS in the same window. The late hit measured `0.268759` RMS, the
candidate/control full-mix delta measured `0.155373` RMS, the source identity
was preserved on return, callback sizes 128 and 257 produced identical output,
and no render used the limiter.

The bounded access record has SHA-256
`7086d73be3f6a21aaaf6922f10c93c47c539ba0e747c50fb6e4ba0ad27e5515f`.
The exact technical report has SHA-256
`67d264aaa2e45f86c4e6c625240049ac5ff0ad21422f2e715e057d65083100f3`.

## Human Result

The exact A-then-B artifact has SHA-256
`f162bb408afa36393e61fb7cdf5fe689ffed65d1c6096b7e58e2920ac3f395e8`.
A was the existing Slam-only control across two bars. B was the cut-hit
candidate followed by its changed return. The source remained clearly
recognizable, and both transformations were judged musically useful and well
formed. B's opening was perceptibly different.

The full arcs were nevertheless perceived as substantially similar. There was
no preference and no clearly stronger arrangement impact from B. The structured
verdict is therefore `technically_ok_but_musically_weak`, with strongest new
element `none`. The review record has SHA-256
`b6041e2a7984f56d031ed8ebb6271a571ced8be8be904bf9e59aa24a26d9eb47`.

This distinction is important: the existing source transformations are useful;
the new one-key composition did not establish the additional full-arc contrast
claimed by this slice.

## Product Consequence

The frozen near-identity rule applies. The new `S` gesture and its product/QA
implementation are removed before merge. Existing Fill, Slam, cut, hit, return,
and source-transformation behavior remains unchanged and available through its
existing actions.

The v1 contract, RBX-293/RBX-294 decisions, and this negative record remain as
evidence. No Holdout access or product hardness claim is authorized. A future
mechanism must create an unmistakably different full musical arc; it must not
retune this version after observing its result.
