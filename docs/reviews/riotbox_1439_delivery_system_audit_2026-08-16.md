# RIOTBOX-1439 Delivery-System Audit

Date: 2026-08-16
Scope: environment, research direction, roadmap, plans, workflow contracts,
audio-QA contracts, and project-owned entry skills
Result: course correction accepted; no product, audio, source, or Holdout
behavior changed

## Summary

Riotbox has a sound architectural spine, unusually strong source/Holdout and
listening boundaries, sufficient research, and a working local toolchain. The
project drift was in execution order: several recent audible ideas paid
product-integration and multi-source qualification cost before the cheapest
musician-value question had a positive answer. The gates worked correctly and
prevented weak behavior from landing; they were being invoked too early.

The accepted correction is a two-stage audible slice: bounded Development
exploration first, then frozen product qualification only after a provisional
keep. This restores a fast sound-design loop without weakening product-spine,
replay, realtime, source, Holdout, exact-output, formal listening, or PR/CI
requirements. The direct product follow-up is RIOTBOX-1417. The existing
RIOTBOX-1399 owns measured validation-cost optimization; the audit found no
reason to create additional maintenance or research tickets.

## Findings And Resolutions

### Major — Qualification preceded musician value

- Location: `docs/specs/audio_qa/automated_qa.md:301`
- Category: execution order
- Finding: the previous matrix-before-listening wording made a full contrasting
  source matrix a prerequisite even when the musical idea had not yet earned a
  cheap provisional keep. That protected against overfitting but coupled
  discovery to promotion cost.
- Resolution: the matrix remains mandatory after a kept mechanism is frozen;
  an earlier bounded usefulness check is explicitly non-promotional. The full
  stage boundary is owned by `docs/specs/audio_qa_workflow_spec.md:47` and
  `docs/plans/p023_audible_delivery_course_correction.md:19`.

### Major — Capability inventory could displace audible priority

- Location: `docs/plans/riotbox_improvement_tracks_plan.md:152`
- Category: roadmap alignment
- Finding: the plan correctly said audible quality was primary, but its numbered
  backlog could be read as nine structural/runtime slices before instrument
  work.
- Resolution: the old list is now explicitly a capability inventory. Current
  order is audible exploration, qualification of one kept mechanism, then only
  the smallest measured blocker. This matches the roadmap at
  `docs/execution_roadmap.md:1448`.

### Major — Historical MC-202 plan looked like a live backlog

- Location: `docs/plans/mc202_source_phrase_planning_plan.md:7`
- Category: planning freshness
- Finding: the deep historical issue chain remained useful architecture but was
  easy to mistake for the current execution queue.
- Resolution: the plan now names RIOTBOX-1417 as the sole direct audible
  follow-up, requires RIOTBOX-1439's discovery order, and forbids recreating the
  historical chain without checking Linear and `main`.

### Major — Entry skills omitted discovery-before-qualification

- Location: `.codex/skills/riotbox-development/SKILL.md:53`
- Category: agent guidance
- Finding: the skills strongly enforced output proof and formal review but did
  not tell an agent how to hear and stop an unproven causal idea before building
  its full promotion apparatus.
- Resolution: development, rave-punk taste, and listening-review guidance now
  share the same bounded exploration/provisional-keep boundary. Playback safety
  remains unchanged in `docs/specs/audio_qa/listening_review.md:182`.

### Moderate — Descriptive Audio-QA build order was stale

- Location: `docs/specs/audio_qa/status_history_and_future.md:17`
- Category: documentation drift
- Finding: generic metric/fixture expansion was still presented as near-term
  order even though P023 needs musician-facing sound progress.
- Resolution: new metrics, manifests, fixtures, and CI expansion are subordinate
  to a named product claim or measured regression gap. Existing regressions
  still enter their normal gates directly.

### Moderate — Local validation and retained evidence are expensive

- Location: `docs/dev_environment.md:76`
- Category: developer environment
- Finding: the environment is capable, but 104 `just` recipes, 222 script files,
  roughly 44 GiB of reusable build state, roughly 30 GiB of artifacts, and a
  roughly 22-minute final CI run make the wrong inner loop costly.
- Resolution: exploration uses focused seam tests and one exact render;
  qualification and final PR retain their applicable broad gates. RIOTBOX-1399
  may improve measured cost and retention behavior. No automated deletion is
  authorized.

## Healthy Boundaries Preserved

- The masterplan already says musical impact comes before algorithmic elegance
  and that Riotbox is an instrument rather than a disguised research project.
  The course correction implements that existing direction; it does not replace
  the product vision.
- `AGENTS.md:130` and `docs/workflow_conventions.md:151` preserve Linear-first,
  PR/CI/review, clean closeout, and exactly-one-active-slice discipline.
- The Source Graph, Session, Action Lexicon, queue/commit, replay, callback, and
  exact RuntimeMix boundaries remain required during qualification.
- Development source registration, fail-closed behavior, commercial-reference
  restrictions, and unopened Holdout protection remain unchanged. Exploration
  grants no new access or source-general claim.
- Exact-artifact preflight, fresh first-play readiness, bounded playback,
  immediate unchanged replay, stop, and silence verification apply equally to
  exploratory listening.
- `.env.local` is ignored and no environment/credential file is tracked. Secret
  contents were not inspected or copied during this audit.

## Linear And Execution Outcome

- RIOTBOX-1439: accepted course-correction contract and this audit.
- RIOTBOX-1417: narrowed to one unmistakably useful MC-202 realized role;
  blocked until RIOTBOX-1439 closes.
- RIOTBOX-1399: retained as subordinate, measured validation-loop optimization;
  it must not become a broad cleanup project or delete reusable state
  automatically.

The next product action after RIOTBOX-1439 is therefore clear: RIOTBOX-1417
starts with one registered Development source and at most three genuinely
different MC-202 role variants. Only a provisional keep earns the cost of a
frozen contract and full product qualification.
