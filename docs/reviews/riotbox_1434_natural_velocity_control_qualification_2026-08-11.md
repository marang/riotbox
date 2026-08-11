# RIOTBOX-1434 Natural-Velocity Control Qualification

Date: 2026-08-11

Work class: `contract_enabler`

Result: `terminal_fail_closed`

## Scope And Access

RIOTBOX-1434 measured only the six preregistered Philharmonia directional
controls. The v1 execution stopped before opening a WAV because it used the
wrong local-root prefix. V2 corrected only that documented path resolution and
then read each exact registered file once with matching SHA-256.

The completed access record contains six verified control reads and records no
directory discovery, Development audio, Holdout audio, or commercial-reference
audio. These controls remain local measurement references and are not product
sources, fixtures, generated assets, demos, or sampler material.

Branch review found one historical failure-path limitation in the frozen V2
runner: `control_audio_accessed` changes to `true` after successful hash,
format, and analysis completion rather than immediately after the contained
read returns. A hypothetical post-read rejection would therefore understate
that read in this Boolean field, although its per-file record would already show
the attempted open. The completed run is unaffected because all six reads
succeeded and the final flag is correctly `true`. Do not reuse this one-shot
runner as a general access logger.

## Technical Result

The frozen analysis produced these provisional mf → f → ff directions:

| Control | Attack time | Decay time | Body-resonance peak | Attack brightness |
| --- | --- | --- | --- | --- |
| Snare with snares | monotonic decrease | monotonic increase | monotonic increase | monotonic increase |
| Whip struck together | non-monotonic | non-monotonic | non-monotonic | monotonic decrease |

The snare therefore exhibited one coherent measured multi-cue progression. The
whip did not reproduce its attack, decay, or resonance directions, and its only
monotonic feature—attack brightness—moved opposite to the snare. These values
are observations from two small control series, not universal cue directions,
processor constants, perceptual thresholds, or hardness proof.

## Human Directional Sanity

The exact blinded artifact presented each mezzo-forte/fortissimo pair and then
the same pair in reversed order. The review result is normalized into
professional evidence language:

- The fortissimo snare was consistently perceived as more forcefully struck in
  both presentation orders.
- No meaningful force difference was perceived between the whip controls.

This is a directional pass for the body-bearing snare control only. It is not a
pass for the whip attack-edge control and does not identify which individual
snare feature caused the perceptual result.

## Contract Consequence

RIOTBOX-1434 required fail-closed termination if the snare and whip disagreed or
the human direction was not repeatable. Both the technical cross-control
directions and the human result fail that complete-control requirement.
Accordingly:

- no universal or source-general multi-cue transfer direction is frozen;
- the snare observations remain bounded evidence, not an implementation handoff;
- RIOTBOX-1435 cannot consume the planned directional precondition;
- no Development candidate, product integration, Stage-B work, or Holdout access
  is authorized by this result.

## Evidence Identities

- V2 control contract: `a0485af6cb30e401a6bc3bd6e900e3a0f8afdb64ef3c44f6f8445a5386c4c14f`
- Completed access log: `4c95af0e816a5ed87c4f9fc0d648a0b59d433d0992314f1ba02d4c67847f04ca`
- Technical analysis: `1db9d36445402baf07374dc9846906d42c1a258057653a524b51b5abe6e32ade`
- Human artifact: `fd61c45f3d50c1e4a4d4dae55575ce1508d69bbc454d5325312aa83074236847`
- Structured human verdict: `99fd145a1f34879c284a2abf78b0a52670ea70627cc5669d8a7fad75b22573bd`

- `human_verdict: terminal_fail_closed`
- `algorithm_selection_allowed: false`
- `hardness_proof: false`
- `quality_proof: false`
