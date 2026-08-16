# Riotbox Audio QA Workflow Spec

Version: 0.2 · Status: Active

Normative router: [automated QA](./audio_qa/automated_qa.md), [human
review](./audio_qa/listening_review.md), and [artifact/export
proof](./audio_qa/manifests_and_artifacts.md). [Status and future
work](./audio_qa/status_history_and_future.md) are descriptive only.

## 1. Purpose

Make audible changes strict, honest, reproducible, and reviewable. Numeric
roles: [`audio_numeric_values.md`](../engineering/audio_numeric_values.md).

## 2. Core Rule

Audio QA requires automation and repeatable human review. Every audible slice
proves its control path and exact applicable output path. Offline proof is only
partial for a live-path claim; fixtures are not human coverage. Missing, stale,
contradictory, or misassigned required evidence fails closed.

Core-owned boundaries:

- Product output never substitutes fallback music for unavailable source.
- Development and holdout sources stay disjoint and use only authorized,
  bounded access; commercial references remain local comparison material and
  never become sources, fixtures, generated assets, or redistributed content.
- Source-derived claims prove source evidence changed a musical decision,
  stored it in the product spine, and changed the exact audible output.
- Analysis, file I/O, hashing, observer work, and model calls stay outside the
  realtime callback.
- Automation may reject but cannot award a musical pass.
- Before an artifact's first playback, analyze and assign it, explain its
  purpose, obtain fresh readiness, and stop at the announced bound. An explicit
  listener request for an immediate replay of unchanged bytes may run directly
  without another analysis, brief, or readiness gate; still verify the stop and
  silence. Do not replay an unchanged recipe merely because technical evidence
  changed.

## 3. Validation Stack

Hard gates, musical gates, deterministic render review, then human review.
Details: [automated QA](./audio_qa/automated_qa.md), [artifact
identity](./audio_qa/manifests_and_artifacts.md), and [human
review](./audio_qa/listening_review.md).

### 3.0 Audible delivery stages

Apply the [P023 audible-delivery plan](../plans/p023_audible_delivery_course_correction.md)
before this full promotion stack for a new mechanism. Bounded
`development_exploration` uses registered Development material, exact-artifact
safety preflight, and an early usefulness check for at most three variants. It
cannot grant a quality, product, source-general, demo, release, hardness, or
Holdout claim. Only a provisional keep permits a frozen rebuild and the full
`product_qualification` stack below. Existing regressions and already-shared
behavior skip exploration and use their applicable normal gates directly.

### 3.1 Hard technical gates

### 3.2 Musical contract gates

### 3.3 Fixture-backed golden render review

### 3.4 Human listening review

For `percussive_hard`, also apply [Percussive Force and Beat
Impact](../engineering/percussive_force_and_beat_impact.md).

## 4. Two Execution Modes

CI is deterministic and cannot issue a human verdict; local review owns taste.

## 5. Required Harnesses

Harness contracts: [automated](./audio_qa/automated_qa.md#5-required-harnesses)
and [listening](./audio_qa/listening_review.md#53-listening-pack-harness).

## 6. Output Layout

See [artifact layout](./audio_qa/manifests_and_artifacts.md#6-output-layout).

## 7. First Metrics To Enforce

See [metric rules](./audio_qa/automated_qa.md#7-first-metrics-to-enforce).

## 8. First Listening Rubric

See [rubric](./audio_qa/listening_review.md#8-first-listening-rubric).

## 9. First Fixture Packs

See [fixtures](./audio_qa/automated_qa.md#9-first-fixture-packs).

## 10. Release Gates For Audio-Producing Changes

Completion requires relevant tests, landed control-path and exact-output
assertions, the affected local pack, a human pass, and material benchmark notes.
Small changes may review only that pack; larger ones need a broader smoke pack.
State-only work proves its nearest render consumer; logs alone are insufficient.
Source-derived work names its audibility mode. A claimed new result requires
`rebuild-only`; `source-layer` must be explicitly optional or transitional.

## 11. Improvement Loop

See [improvement rules](./audio_qa/automated_qa.md#11-improvement-loop).

## 12. Role Of Agents And Ghost

Agents may choose bounded tested actions but cannot bypass replay-safe actions,
hide rendering, define unbounded output, or block realtime audio.

See [observer rules](./audio_qa/automated_qa.md#121-future-user-session-observer).

## 13. Current Repo Status

See [status](./audio_qa/status_history_and_future.md#13-current-repo-status).

## 14. Near-Term Build Order

See [build order](./audio_qa/status_history_and_future.md#14-near-term-build-order).

## 15. Success Condition

See [success](./audio_qa/status_history_and_future.md#15-success-condition).
