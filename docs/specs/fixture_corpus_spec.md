# Riotbox Fixture Corpus Spec

Version: 0.1  
Status: Draft  
Audience: MIR/ML, QA, realtime, product

---

## 1. Purpose

This document defines the reference fixture corpus used for:

- analysis validation
- replay validation
- golden renders
- benchmark comparisons
- musical regression testing

It does **not** require the actual audio assets to exist yet. It defines the categories and metadata the corpus must contain.

---

## 2. Corpus Design Goals

The corpus should represent:

- normal cases
- difficult cases
- edge cases
- musically useful cases
- failure-inducing cases

The goal is not dataset scale. The goal is stable, reusable reference coverage.

---

## 3. Fixture Categories

### 3.1 Clean rhythmic track

Characteristics:

- strong kick / snare anchors
- clear tempo
- obvious phrase structure
- moderate density

Used for:

- baseline beat / bar detection
- early rebuild quality
- capture and scene tests

### 3.2 Dense break-heavy track

Characteristics:

- many transients
- more ghost-note complexity
- break fragmentation pressure
- harder drum reconstruction

Used for:

- slice mining
- break rebuild
- feral scoring

### 3.3 Hook-forward track

Characteristics:

- strong short vocal or synth phrase material
- clear fragment candidates
- recognizability risk

Used for:

- hook-fragment logic
- quote-risk tests
- W-30 capture relevance

### 3.4 Low-confidence analysis track

Characteristics:

- weak beat clarity or ambiguous phrasing
- noisy structure
- softer attacks

Used for:

- confidence handling
- degraded operation
- fallback behavior

### 3.5 Long-form track

Characteristics:

- longer duration
- several sections
- evolving dynamics

Used for:

- memory growth
- long analysis timing
- scene changes over time

### 3.6 Feral stress track

Characteristics:

- enough fragment richness for rebuild
- enough recognizability risk to exercise `quote_guard`
- good candidate density for rebake and capture

Used for:

- feral profile validation
- scorecard checks
- regression renders

---

## 4. Required Fixture Metadata

Every fixture should include:

- fixture ID
- title or alias
- duration
- expected difficulty class
- expected tempo confidence class
- expected section clarity class
- notes about hook potential
- notes about break-rebuild suitability
- notes about quote-risk sensitivity

Optional:

- expected usable slice range
- expected capture-yield range
- expected feral usefulness notes

---

## 5. File Organization

Recommended layout:

```text
tests/
  fixtures/
    corpus_manifest.json
    audio/
      fx_001_clean_rhythm.wav
      fx_002_break_dense.wav
      fx_003_hook_forward.wav
      ...
    expected/
      fx_001/
        notes.md
        expected_ranges.json
      fx_002/
        notes.md
        expected_ranges.json
```

---

## 6. Expected Ranges, Not Fake Precision

For early phases, fixtures should define expected ranges rather than brittle exact values.

Examples:

- BPM within tolerance
- section count within plausible band
- slice count above minimum floor
- candidate ranking contains at least one known-useful class
- quote-risk report exists and is non-empty for hook-forward material

---

## 7. Benchmark Usage

The corpus should support:

- latency benchmarks
- analysis duration benchmarks
- replay validation
- golden renders
- regression detection after provider or scoring changes

Not every fixture needs to run in every benchmark suite. The corpus should define which fixture belongs to which suite.

---

## 8. Minimum Initial Corpus

The first practical version should include at least:

- 2 clean rhythmic fixtures
- 2 dense break fixtures
- 2 hook-forward fixtures
- 1 low-confidence fixture
- 1 long-form fixture
- 1 feral stress fixture

This is enough to begin disciplined regression work without pretending to have a full production library.

---

## 9. Corpus Governance

Adding a new fixture should require:

- a stated reason
- fixture metadata
- expected usage category
- indication of which test or benchmark gap it fills

Removing or replacing a fixture should require:

- note in the research / decision log
- migration of affected benchmark references

---

## 10. Next Step

After this draft:

1. define `corpus_manifest.json`
2. select or create initial fixture candidates
3. assign benchmark suites
4. define expected ranges for each initial fixture
