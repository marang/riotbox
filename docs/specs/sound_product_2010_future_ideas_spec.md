# 20/10 Sound-Product Future Ideas Spec

Version: 0.1
Status: Draft
Audience: audio, product, QA, agents

---

## 1. Purpose

The 20/10 future-idea track keeps ambitious Riotbox directions visible after
the 10/10 sound-product release path is clear. It is not a release gate and it
must not dilute the current P021/P022/P023 evidence requirements.

The machine-checkable idea list lives in
`scripts/fixtures/sound_product_2010_future_ideas/ideas_v1.json` and is
validated by:

```bash
just sound-product-2010-future-ideas-fixtures
```

---

## 2. Boundary

Every 20/10 idea must remain `release_blocking: false` unless it is explicitly
promoted into a normal roadmap project with its own definition of done.

20/10 ideas may inspire experiments, but they cannot be used to claim:

- 10/10 sound-product readiness
- demo-ready output
- human musical pass
- technical audio-QA completeness
- a hidden taste oracle

The 10/10 path remains governed by the sound-product readiness rubric, human
listening labels, release-grade demo bank, weak-output routing, and technical
audio-QA gates.

---

## 3. Idea Contract

Each idea records:

- stable `idea_id`
- musician-facing title
- musical payoff
- product spine improved
- replay / realtime risk
- evidence required before promotion
- promotion condition
- 1.0 boundary

The product spine field must name existing Riotbox boundaries, such as Source
Graph, Session model, Action Lexicon, queue / commit, audio engine, audio QA,
demo bank, or external surfaces. Future ideas must improve these boundaries
instead of creating shadow architectures.

---

## 4. Initial Ideas

The initial list must include:

- producer-loop take selection
- source-to-performance memory
- live resampling / self-abuse
- stage-impact macros
- taste-aware demo generation
- set-level performance memory
- ecosystem surfaces that preserve sound and QA gates

---

## 5. Promotion Rule

To promote a 20/10 idea into implementation work, create or update a normal
Linear issue or project and restate:

- the bounded user workflow
- the exact product-spine contract changed
- replay / restore consequence
- realtime audio risk
- required evidence and listening proof
- why the idea no longer distracts from the 10/10 release path
