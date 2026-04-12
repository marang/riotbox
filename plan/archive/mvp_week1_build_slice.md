# Riotbox MVP Week 1 — Concrete Build Slice

Status: Proposed execution plan  
Scope: Week 1 only (7 days)  
Goal: Deliver a runnable terminal app that can load audio, show analysis metadata, and play a deterministic starter pattern through a low-latency engine.

---

## 1) Week-1 Outcome (Definition of Done)

By end of Week 1, we have a single executable (`riotbox`) that can:

1. Open an audio file (`wav`/`mp3`) from CLI argument.
2. Decode it to normalized mono analysis buffer (offline path).
3. Estimate basic musical metadata (duration, RMS envelope, rough BPM).
4. Start a realtime audio engine with stable callback (no dropouts under normal laptop load).
5. Generate and play a deterministic, seed-based 1-bar pattern:
   - TR lane: kick/snare/hat mock voices.
   - MC lane: mono bass synth mock voice.
   - W lane: one sample slice playback voice.
6. Render a terminal UI with transport state, seed, BPM, CPU callback stats, and lane mute toggles.
7. Accept keyboard controls for start/stop, mute lanes, reseed.
8. Persist and reload a minimal session file (`.riotbox/session.json`) with seed + bpm + mutes.

This is intentionally narrow: **playable skeleton first**, not full sound design.

---

## 2) Crate Selection (Exact)

### Core runtime
- `cpal` — realtime audio I/O backend.
- `anyhow` — top-level error handling.
- `thiserror` — typed internal errors.
- `tracing`, `tracing-subscriber` — structured logs + diagnostics.

### Terminal UX
- `ratatui` — TUI rendering.
- `crossterm` — terminal input/event backend.

### Audio file ingest & analysis
- `symphonia` — decode mp3/wav and convert to f32.
- `rubato` (optional if needed in week 1) — resampling helper for mismatched sample rates.

### DSP + musical primitives
- `rand`, `rand_chacha` — deterministic RNG (`ChaCha8Rng`) from seed.
- `biquad` (optional in week 1) — simple filter in MC voice.

### Persistence + config
- `serde`, `serde_json` — session state serialization.
- `directories` — platform-safe app data directory.
- `clap` — CLI parsing (`riotbox <audio-file> --seed <u64>`).

### MIDI (stub-ready this week)
- `midir` — add behind feature flag (`midi`) but do not wire full mappings in Week 1.

---

## 3) Module & Interface Skeleton (Exact)

```text
src/
  main.rs
  app/
    mod.rs
    state.rs
    commands.rs
  audio/
    mod.rs
    engine.rs
    callback.rs
    voices.rs
    scheduler.rs
  analysis/
    mod.rs
    decode.rs
    features.rs
  pattern/
    mod.rs
    generator.rs
    score.rs
  tui/
    mod.rs
    view.rs
    input.rs
  session/
    mod.rs
    store.rs
  util/
    time.rs
    ring_stats.rs
