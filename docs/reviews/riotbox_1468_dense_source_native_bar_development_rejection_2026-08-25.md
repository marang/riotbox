# RIOTBOX-1468 Dense Source-Native Bar Development Rejection

Date: 2026-08-25

Work class: `audible_vertical_slice / development_exploration`

Outcome: frozen source-native full-bar v2 rejected at technical preflight;
no human playback occurred and the temporary Runtime grammar was removed

## Frozen Boundary

RIOTBOX-1468 moved away from the exhausted six-beat-anchor/two-beat-answer
selector role. RBX-333 froze one different W-30 playback grammar: the current
half-beat retrigger control versus one candidate that plays a trusted complete
source bar continuously at original order, pitch, direction, and rate. It kept
the existing W-30 grit and music-bus level and added no lane or effect.

RBX-334 superseded only the timing authority after the first Development
session correctly stopped before render. V2 derives the exact transport tempo
from the already confirmed capture-bar frame count instead of rounded metadata.
Every musical mapping, threshold, source identity, access limit, and claim
boundary remained unchanged.

## Source-Blind Falsification

Before Development access, generated Rust tests proved that the typed grammar:

- suppresses half-beat retriggers inside the source bar;
- completes one source bar at original rate and order;
- remains sample-exact across 128- and 257-frame callback partitions;
- differs materially from the unchanged half-beat control; and
- renders silence rather than fallback music when source audio is unavailable.

Contract mutation fixtures rejected playback-rate, lane-count, source-boundary,
Holdout-access, and candidate-budget changes. That implementation and its
passing source-blind tests are preserved in branch commit `68e73383` before the
unqualified Runtime surface is removed.

## Bounded Development Result

The v1 session opened only exact registered Development case
`dense_beat03_130`, performed no directory discovery, and stopped before render
when rounded `130.0` BPM metadata disagreed with the trusted capture grid by 178
source frames. It produced no audio and no playback.

The fresh v2 session opened the same exact registered source once and completed
the renderer. Preflight then stopped before analysis because the imported metric
module's NumPy binding was not initialized. RBX-335 authorized one recovery
against only the exact pinned artifacts, without source reopening or rerender.

That recovery verified every identity, initialized the existing metric module,
and ran the unchanged preflight. Callback partition equality passed and the
control/candidate delta RMS was `0.1192705482`, well above the frozen `0.001`
minimum. The aggregate audio safety/boundary preflight nevertheless returned
`fail`.

The failure handler persisted only the aggregate error, not the already
computed individual audio-gate values. The exact failed subgate is therefore
not durable evidence. RBX-335 prohibits another analysis retry. No threshold
was changed and no diagnosis was invented after the result.

## Human Listening

No listening review occurred. The exact candidate did not pass technical
preflight, so presenting it would violate the frozen gate and create no
admissible musical verdict. No prior artifact was replayed as a substitute.

## Post-Rejection CI Boundary Incident

During closeout, the normal broad `just ci` command reached existing
source-backed audio-QA generators. The run was stopped as soon as the bounded
log made that access visible. Before stop, those legacy generators had reopened
exact registered Development paths including Beat03 and several unrelated
corpus cases. No Holdout or commercial reference was accessed and the scripts
used exact paths rather than directory discovery.

All generated incident outputs are excluded. No musical or numeric incident
result changed this ticket, and the v1 access log, v2 access log, recovery log,
and runtime report retained their exact hashes. RBX-337 therefore preserves
only the already completed negative result and prohibits resuming broad local
audio QA under this ticket. Final validation uses source-free code and exact
contract gates. The workflow now requires a static source-access compatibility
check before broad audio QA.

A subsequent documentation search contained unescaped shell backticks and
accidentally invoked `just ci` again. That orphaned process was detected and
terminated while still in `cargo test`, before reaching any source-backed
audio-QA generator. The incident record includes this operator error; it added
no detected source exposure and supplies no evidence.

## Consequence

Frozen v2 is rejected and its temporary public Runtime grammar, renderer route,
tests, Just targets, and executable exploration runners are removed from the
final tree. The immutable contracts and audit record remain as negative
evidence. Future artifact runners must durably write complete technical metrics
before applying an aggregate pass assertion.

Dense remains non-demo-ready and the only missing positive demo family. A
successor requires a new Linear-first causal slice; it may not tune v2, replay
these consumed artifacts as fresh evidence, or infer a human verdict. No
product, source-general, hardness, Holdout, demo, release, universal-quality,
or P023-completion claim follows.

## Evidence Identities

- frozen v1 contract: `027c1cb6e007f5e2d8064fa1db40a5b91dbea2c890dd101310297c4375441fef`
- frozen v2 contract: `ac71e8daa9f862a8341910d63e0457cd657e6506808eda4032d132b4fb443517`
- frozen recovery contract: `1683e1aa824dba52c6c0c55d977107cfc535fa0a5fae0da9db0a3c8806ef7278`
- v1 failed-closed access log: `b096212befcbf9fdf50a2b9f9bb200c155218dacca947514c9c24941c372de05`
- v2 failed-closed access log: `94cd1552246aeee802ad157ebd807ada01783aff3f49ed87bc764eaff9e08acf`
- failed-closed recovery log: `ee1f8aac27499571c05cf3d794f54fe81576caea04acf012b163fc3ba441cfac`
- exact runtime report: `970bd8aa7ae03950aa9e9c3991bdbc5e6ade2d05f74f520c45cb4e897f551ce9`
- post-rejection CI access incident: `ed4e1301527419c22798d7d6e74b0f225368d878867e5ff0d96c3df510db8f16`
- source-blind implementation commit: `68e73383`
