# Riotbox Validation and Benchmark Spec

Version: 0.1  
Status: Draft  
Audience: realtime, MIR/ML, QA, product

---

## 1. Purpose

This document defines how Riotbox is validated and benchmarked.

It exists so the project does not drift into:

- subjective "sounds okay" evaluation only
- unmeasured realtime regressions
- analysis changes without fixture comparison
- Ghost behavior without safety checks

---

## 2. Validation Layers

Riotbox must be validated at five layers:

1. **Signal**
2. **State / logic**
3. **Integration**
4. **Workflow**
5. **Performance / benchmark**

---

## 3. Signal Validation

Required checks:

- no-click / no-pop behavior for common transitions
- envelope correctness
- voice allocation correctness
- stable playback under nominal load
- expected bus routing

Relevant subsystems:

- audio core
- device engines
- mixer / FX

---

## 4. State and Logic Validation

Required checks:

- action queue correctness
- quantized commit behavior
- undo / redo behavior
- scene restore behavior
- snapshot integrity
- deterministic replay of action sequences

Relevant subsystems:

- session
- scheduler
- action log
- Scene Brain

---

## 5. Integration Validation

Required checks:

- core <-> sidecar messaging
- provider swapping
- timeouts and degraded analysis behavior
- sidecar crash survival
- Ghost respect for locks and budgets

---

## 6. Workflow Validation

Critical workflow tests:

### Workflow A - Track to Jam

- load track
- finish baseline analysis
- show usable Jam state
- start playback
- mutate safely

### Workflow B - Capture

- detect useful moment
- capture on quantized boundary
- promote into W-30 path
- replay promoted result

### Workflow C - Save and restore

- save session
- reload
- recover same state and replay behavior

### Workflow D - Assisted mutation

- Ghost suggests action
- user accepts
- action commits safely
- action remains undoable and replayable

---

## 7. Benchmark Families

### 7.1 Realtime benchmarks

- callback timing
- xruns
- buffer underruns
- CPU peak
- memory growth
- action queue lag

### 7.2 Analysis benchmarks

- analysis duration per track
- sidecar job latency
- memory use per analysis run
- candidate count stability
- confidence-distribution drift
- source timing fixture agreement for BPM, beat, downbeat, phrase, ambiguity,
  and degraded-policy expectations

### 7.3 Workflow benchmarks

- time to first playable Jam state
- time to first successful capture
- session save time
- session load time
- replay completion time

### 7.4 Product-quality benchmarks

- capture yield
- variation density
- quote-risk ceiling
- number of usable break variants
- golden render consistency

---

## 8. Minimum Benchmark Matrix

Benchmarks should be run at least against:

- short track fixture
- medium track fixture
- dense / difficult fixture
- weak-analysis fixture

And on at least:

- default laptop development configuration
- one constrained configuration if available

---

## 9. Required Outputs

Every benchmark run should produce:

- timestamp
- commit SHA or working version
- fixture ID
- benchmark name
- measured values
- pass / fail judgment
- previous baseline reference when available

---

## 10. Golden Renders

Golden renders are required for:

- same input
- same seed
- same action list
- same render config

Golden renders are used to catch:

- structural drift
- accidental timing changes
- major arrangement regressions
- capture / promotion regressions

Golden renders do not replace listening review, but they make regressions visible.

---

## 11. Failure Threshold Examples

Exact numbers are still open, but these threshold types must exist:

- max acceptable xrun count
- max callback latency spike
- max analysis duration for fixture categories
- max quote-risk value for auto-promotion
- max identical-bar run in default arrange
- minimum capture-yield floor

---

## 12. Feral-Specific Validation

For feral behavior, validate:

- break variation over time
- hook-fragment non-triviality
- resample reuse ratio
- quote-risk enforcement
- capture-worthy moment frequency

These checks should tie directly into `FeralScorecard`.

---

## 13. Review Discipline

No major subsystem change is complete without:

- passing relevant automated checks
- running at least the affected benchmark family
- recording results in the research / decision log if behavior changed materially

---

## 14. Next Step

After this draft:

1. define exact benchmark commands and file locations
2. define thresholds per fixture class
3. add first golden render harness
4. connect benchmark reporting to CI or local scripts
