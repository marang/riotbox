# RIOTBOX-1405 Degraded Product Review

Date: 2026-07-21
Reviewer: Markus
Verdict source: real 80x24 TUI / PipeWire user-session assignments

## Evidence Boundary

This review covers honest product handling for weak and timing-untrusted
sources. It is not a sound-quality pass, a demo-ready claim, or a general TUI
quality pass.

The exact assignments kept transport stopped, source monitoring on the raw
source-only route, the action queue and Session commit log empty, and TR-909,
MC-202, and W-30 generated output unconfigured. The real audio callback reached
884 callbacks for Beat20 and 792 callbacks for Fadapad. Both assignments
reported `fallback_music_present: false`.

## Exact Runtime Evidence

### Beat20

- Source SHA-256:
  `d3d86134e99dfb5889c9efe683ccd427cdf73e499ebfbc69dbd1f3a145bdf1e1`
- Source Graph SHA-256:
  `d79b4b24a255e3ecfd807f2cfd60670b511e869bd42fe3995e194b715d3c39dc`
- Session SHA-256:
  `017fe37b1c8ce0b8998b3669f948d9188655c72f5ec33f78f53cc5f5993cffa9`
- Observer SHA-256:
  `b109de11eef6c5f40d09b476b3aa28a709644b246b6cf38c2bf73e058d708ea2`
- Product state / reason: `degraded / needs_user_confirmation`
- Grid use / warning: `short_loop_manual_confirm / ambiguous_downbeat`
- Bar-locked permission / live source policy: `false / false`

Beat20 intentionally represents both canonical `bad_timing` and `weak_source`
corpus cases, but each family has its own structured review and demo-bank
entry. The bad-timing review SHA-256 is
`a5f8b1d7b17448629c7ef63dc2e52afe79c0ddc5959f53d7b2584c3c52fa8167`;
the weak-source review SHA-256 is
`a4c956bce9c4b796cdd1738e3f5b1064c812bb433d91da89f1e1b767443b8423`.

### Fadapad

- Source SHA-256:
  `22e825c8bf59cfd71a02ce229d222d1d35f1bae6f7d1bafab6edeb4ff4829d8c`
- Source Graph SHA-256:
  `0eb107c1e942ba1ebfd3c0241b6f4df4c0bf2bbc2c773488a1baab3134e843c3`
- Session SHA-256:
  `6393f001367daa1ab7aac34c535a176220f6576c52949aebd84660fd18988221`
- Observer SHA-256:
  `3e8d41551cf5acfab059412d467ea1dcda3585a2db40fc7c0bcc578b5f4705e5`
- Product state / reason: `unavailable / unavailable`
- Grid use / warning: `unavailable / sparse_onsets`
- Bar-locked permission / live source policy: `false / false`
- Structured review SHA-256:
  `1eca081ba58c8bd852de6aaa621155e27fd9f94a87ed089a64b58ed5423efe30`

Fadapad is supporting unavailable-state UX evidence. It is not promoted as the
`pad_noise` family success in this ticket.

## Human Product Verdict

The exact compact cues presented for review were:

```text
Beat20
Trust:      degraded | bar/live?
            why ambiguous downbeat
Start Here: [Space] source preview | [C] confirm grid

Fadapad
Trust:      unavailable | bar/live?
            why sparse onsets
Start Here: [Space] source preview | timing unavailable
```

Reviewer summary: bounded pass. The current risk guidance is adequate, while
the broader TUI still requires a separate general polish pass.

This records a bounded product-handling pass: the risk state is visible, the
reason is useful enough, and the next safe action is understandable. The
wording explicitly preserves the known general TUI polish gap; this verdict
does not waive or pre-approve that later work.

## Coverage Consequence

The local live-review demo bank SHA-256 is
`b7ea0e526eb83649197092720e3701c8287b1381fdbf254da5411e4f9c8cd0a2`.
Its live-readiness coverage report SHA-256 is
`533a7cf2a117b0a7c29cca413fd72ced3d9daee2c9777580e5901c178dc9751e`.

`weak_source` and `bad_timing` now report
`reviewed_degraded_or_reject`, satisfying their negative-family contract
without a rendered candidate or synthetic replacement music. Overall release
readiness remains correctly `blocked` by the still-unreviewed positive source
families; RIOTBOX-1405 makes no broader readiness claim.

## Validation

- Full `just ci`: pass.
- Observer, degraded-product-review, release-demo-bank, and source-family
  coverage fixture chain: pass.
- Structured live reviews validate with `--require-human-pass`.
- Branch code review found and fixed TUI/observer state drift, forged stored
  proof acceptance, incomplete observer-trajectory checks, and missing CI
  wiring before this verdict was recorded.
