# Riotbox Rave-Punk Taste Rubric

Use this rubric to make musician-facing taste judgments. Measurable gates,
source/holdout rules, playback procedure, and typed hardness remain canonical
in the [audio-QA router](../../../../docs/specs/audio_qa_workflow_spec.md) and
[Percussive Force and Beat Impact](../../../../docs/engineering/percussive_force_and_beat_impact.md).

## Direction

Use The Prodigy's full-era production arc as a quality reference class, never a
copy target: early rave-break urgency, mid-era big-beat/punk attack, later dense
drum/bass pressure, vocal or stab hooks, harsh stops, and live-room impact.
Riotbox must retain an original, source-backed identity.

Prefer output that is:

- aggressive and physically present without becoming unusable
- recognizably sample-based but transformed through purposeful chopping,
  pitching, reversing, gating, filtering, resampling, or recontextualization
- riff-led and beat-forward, with one memorable central identity
- punk in arrangement: hard entrances, dropouts, stops, crashes, abrupt mutes,
  fast contrast, and changed returns
- playable live, with immediate stage meaning
- raw but controlled; grit is useful only when it increases intent and impact

Reject generic EDM, ambient wallpaper, sterile demos, polite preset browsing,
random motion without performance logic, over-quantized lifelessness, and busy
output without a memorable hook.

## Hook, Loop, And Development

- Establish one supported Golden Path with a clear hook and hardest element
  before widening families or multiplying variations.
- Prefer a few strong lanes over many weak ones.
- After the core identity works, add a destructive variation such as choke,
  reverse, retrigger, pitch dive, filter slam, bitcrush, or silence cut.
- Mutes and triggers must create musical drama, not merely toggle state.
- Stay-out, restraint, and silence are valid only when source context selects
  them and they improve impact; they cannot conceal weak generation.
- A near-identical short loop across the review window fails unless the mode
  promises a held loop and the hook already has a human pass. Micro-dropouts do
  not substitute for macro development.
- Prefer development that establishes the hook, lifts an intended pressure
  domain, creates a destructive role swap/drop, and returns with a changed
  payoff.
- A compact scripted arc may prove gesture reachability but not loopability,
  reusable material, or performer freedom. Those claims require a sustained
  isolated audition in which the musician can keep or reject the component.
- Positive demo families require a real human pass. Weak or bad-timing sources
  may succeed through reviewed degraded/unavailable/reject behavior instead of
  forced demo-ready music.

For a new mechanism, run the bounded Development exploration owned by the
audio-QA spec before full product implementation: one exact registered source,
the owning plan's bounded variant budget, and an early usefulness check. A
provisional keep then authorizes the frozen rebuild and development-source
matrix before the formal product verdict. Existing shared behavior and
regressions use their normal gates directly. The matrix prevents overfitting
and collapse; it never replaces human judgment or upgrades the exploratory
direction into evidence.

## Pressure And Gesture Judgment

Never use “pressure” without naming its domain and owner: bass/low-end,
drum/transient, midrange/hook, or arrangement/performance. A strong unrelated
domain cannot rescue failure in the intended one. When bass is assigned, judge
absolute low-band presence as well as relative share; do not fail unassigned
bass ownership for absent bass.

For “harder,” apply the exact typed contract in the percussive-force engineering
document. Lower, louder, darker, dirtier, damaged, or merely different is not
automatically harder. A human “different but not harder” verdict rejects and
freezes that recipe rather than starting another scalar retune.

Compare raw and loudness-matched candidates when useful. Evaluate each explicit
gesture in its local audible window against its immediate counterfactual; more
global gain cannot stand in for articulation.

## Review Questions

- What is the hook?
- What hits hardest: kick, snare, bass, break, stab, vocal hit, or silence?
- What can the player do that changes the room immediately?
- Where is the choke, stop, fill, drop, or changed return?
- Does transformed source character survive, or has output collapsed into a
  placeholder?
- Is continued triggering intentional and satisfying, or merely annoying?
- Is the result recognizably Riotbox, or only cleaner, louder, darker, or busier?

Say plainly whether the hook, impact, source character, and gesture work. Tie
every failure to one sample transform, drum/bass/mix policy, trigger behavior,
preset change, regression, threshold, or UI cue.
