# Riotbox - Masterplan
**A feral terminal audio instrument**

Version: 0.1  
Status: strategic and technical implementation plan  
Language: English

---

## Table of Contents

- [1. Vision](#1-vision)
- [2. Product Definition](#2-product-definition)
- [3. Guiding Principles](#3-guiding-principles)
- [4. Sound Identity: the three device personalities](#4-sound-identity-the-three-device-personalities)
- [5. Operating Modes](#5-operating-modes)
- [6. Target Users and Core Use Cases](#6-target-users-and-core-use-cases)
- [7. Product-Level Feature Scope](#7-product-level-feature-scope)
- [8. System Architecture](#8-system-architecture)
- [9. Technology Stack](#9-technology-stack)
- [10. Audio Data Flow](#10-audio-data-flow)
- [11. Analysis Pipeline](#11-analysis-pipeline)
- [12. Internal Musical Representations](#12-internal-musical-representations)
- [13. Device Engine Design](#13-device-engine-design)
- [14. Arrangement and Composition Logic](#14-arrangement-and-composition-logic)
- [15. Scoring and Selection](#15-scoring-and-selection)
- [16. Live Mutation and Performance Control](#16-live-mutation-and-performance-control)
- [17. AI Agent / Ghost System](#17-ai-agent--ghost-system)
- [18. UX Strategy](#18-ux-strategy)
- [19. TUI Concept](#19-tui-concept)
- [20. Controller and Hardware Integration](#20-controller-and-hardware-integration)
- [21. Session Model](#21-session-model)
- [22. Data Model](#22-data-model-simplified-sketch)
- [23. Capture and Looping as the Core](#23-capture-and-looping-as-the-core)
- [24. Resample Lab](#24-resample-lab)
- [25. FX and Mixer Strategy](#25-fx-and-mixer-strategy)
- [26. Export and Interoperability](#26-export-and-interoperability)
- [27. Quality Assurance and Pro Hardening](#27-quality-assurance-and-pro-hardening)
- [28. Provider Architecture](#28-provider-architecture)
- [29. Legal, Licensing, Originality](#29-legal-licensing-originality)
- [30. Repository Structure](#30-repository-structure)
- [31. MVP Definition](#31-mvp-definition)
- [32. Implementation Phases](#32-implementation-phases)
- [33. Detailed Module Backlog](#33-detailed-module-backlog)
- [34. Non-Functional Requirements](#34-non-functional-requirements)
- [35. Preset and Style System](#35-preset-and-style-system)
- [36. Preset Macros](#36-preset-macros)
- [37. Explainability and Trust](#37-explainability-and-trust)
- [38. Risk Analysis](#38-risk-analysis)
- [39. Team and Role Recommendations](#39-team-and-role-recommendations)
- [40. Early Milestones in Weekly Logic](#40-early-milestones-in-weekly-logic)
- [41. Development Decision Rules](#41-development-decision-rules)
- [42. Final Product Image](#42-final-product-image)
- [43. Next Concrete Documents](#43-next-concrete-documents)
- [44. One-Sentence Version](#44-one-sentence-version)

---

## 1. Vision

**Riotbox** is a terminal-native audio instrument for live performance, sound mutation, and controlled recomposition.  
The user loads an audio file, Riotbox analyzes its musical structure, and translates it into a playable live object. That object is then rebuilt through three clearly defined device personalities:

- **MC-202** as the monophonic sequencing and synth nerve center
- **W-30** as the sampler, slicing, and resampling machine
- **TR-909** as the drum and groove motor

The result is **not** a black-box audio-to-audio trick. It is a controllable instrument that:

- analyzes input material
- creates musical representations
- generates live-playable scenes, loops, and phrases
- can mutate on demand
- can be assisted or performed by a local AI agent
- remains reproducible, saveable, and exportable

---

## 2. Product Definition

### 2.1 What Riotbox is

Riotbox is a professional sound tool with three faces:

1. **Live remix instrument**  
   Audio in, analysis, rebuild, performance.

2. **Sound design machine**  
   Slices, resampling, phrase capture, layering, mutation.

3. **Autonomous collaborator**  
   A local agent can propose or perform musically bounded actions.

### 2.2 What Riotbox is not

Riotbox is not:

- a prompt-only "make me a song" toy
- an opaque end-to-end style transfer system
- a full DAW replacement
- a tool that simply imitates source material 1:1
- a system that requires expert-level knowledge before it becomes fun

### 2.3 Core product line

> Riotbox turns audio into a living performance object and lets the player shape that object in real time through device character, scene logic, capture, and AI co-performance.

---

## 3. Guiding Principles

### 3.1 Instrument, not black box

Important musical decisions must be reachable through visible states, actions, and macros.

### 3.2 Realtime first

The audio engine must remain stable even when analysis, AI, or file processing fails.

### 3.3 Progressive depth

The interaction model must work in layers:

- **Jam**: play immediately
- **Sculpt**: shape deliberately
- **Lab**: inspect and debug deeply

### 3.4 Repeatability

Every session must be deterministically reproducible through:

- global seeds
- local seeds
- action log
- analysis cache
- scene and capture history

### 3.5 Musical usefulness over academic completeness

The system must sound good, react quickly, and feel enjoyable. Perfect analysis matters less than musically useful decisions.

### 3.6 Originality with structural reference

Riotbox should generate material inspired by structure, not surface-level copying. Operating modes and a later similarity firewall support this.

---

## 4. Sound Identity: the three device personalities

### 4.1 MC-202 role

The MC-202 layer is the system's **mono nerve**. It handles:

- basslines
- short leads
- answer phrases
- accent behavior
- slides and glides
- nervous repetition
- supportive or opposing harmonic figures

Guiding image: few notes, high consequence.

### Sound character

- monophonic
- clear sequencing logic
- portamento / glide
- accent-driven energy
- filter-led tension
- slightly anxious forward motion

### Operating philosophy

The 202 engine must never become too busy. It should provide spine and bite, not swallow the mix.

### 4.2 W-30 role

The W-30 layer is the **sampler soul**. It handles:

- slice creation
- loop detection
- pad banks
- resampling
- repitching
- reverse
- bit and rate character
- phrase capture
- self-sampling of internal sound sources

Guiding image: everything musically interesting must be freezable, replayable, and reusable.

### 4.3 TR-909 role

The TR-909 layer is the **drum motor**. It handles:

- drum reinforcement
- kick and snare layering
- pattern rebuild
- accent mapping
- hi-hat density shaping
- fills
- drop preparation
- drum bus energy

Guiding image: punch, precision, groove tension, and controlled aggression.

---

## 5. Operating Modes

### 5.1 Derivative mode

The source remains audibly present. Riotbox works with real source material.

Uses:

- real loops
- real slices
- real textures
- real vocal fragments
- real timing material

Use case: live edit, mutation, recut, DJ-adjacent performance.

### 5.2 De Novo mode

The input contributes structure, not necessarily direct audio material.

Uses:

- BPM / grid
- key / chords
- melodic contours
- energy curves
- section logic
- embeddings / style vectors

Generates:

- new drum patterns
- new 202 phrases
- new hook samples
- new internal loops and scenes

Use case: structure-inspired, materially distinct rebuilds.

### 5.3 Hybrid mode

Combines both worlds:

- replace original drums but keep vocals
- slice the original hook but add a new bassline
- resample from extracted phrases into new playable material

This will likely be the most important live mode.

---

## 6. Target Users and Core Use Cases

### 6.1 Live performer

- loads a track
- starts Riotbox
- generates a rebuild
- controls energy, loops, scenes, and mutation live
- captures good moments on the fly

### 6.2 Producer

- extracts strong loops and phrases
- builds new hook banks
- exports stems and MIDI
- uses Ghost / AI as a co-sound-designer

### 6.3 Explorer / listener

- loads material
- switches `ghost=perform`
- watches the instrument behave
- hears how the local agent makes musical decisions

### 6.4 Sound designer

- tears down original material
- uses W-30 capture and the resample lab
- builds characterful sample banks

---

## 7. Product-Level Feature Scope

In final form, Riotbox should support at least:

- loading audio files
- creating an analysis cache
- determining beat grid and bars
- estimating key and chords
- isolating stems or partial components
- finding slice and loop candidates
- feeding internal device engines
- generating live scenes
- extracting and freezing loops
- immediate live mutation
- undo / revert / snapshots
- AI suggestions and AI performance
- MIDI / HID control
- session persistence
- stereo and stem export
- deterministic reconstruction
- performance logging and diagnostics

---

## 8. System Architecture

```text
                  +-----------------------------+
                  |        Terminal UI          |
                  |  Jam / Sculpt / Lab / Log   |
                  +-------------+---------------+
                                |
                                v
                    +-----------+-----------+
                    |      Session Core      |
                    | state / actions / undo |
                    +----+---------------+---+
                         |               |
            control bus  |               | scheduled events
                         v               v
                +--------+----+   +------+--------+
                | Composer /   |   | Realtime     |
                | Scene Brain  |   | Audio Engine |
                +--------+----+   +------+--------+
                         |               |
                         |               v
                         |         audio output
                         |
                         v
                +--------+---------------------------+
                |  Device Engines                    |
                |  MC-202 / W-30 / TR-909 / FX       |
                +--------+---------------------------+
                         ^
                         |
                +--------+---------------------------+
                | Analysis + AI Sidecar              |
                | stems / beats / chords / embeddings|
                | loop mining / local agent          |
                +------------------------------------+
```

### 8.1 Process boundaries

Riotbox should be split into at least two major processes.

### Realtime core

Responsible for:

- audio thread
- scheduler
- mixer
- device logic
- terminal UI
- controller input
- session state
- undo / snapshots

### Analysis + AI sidecar

Responsible for:

- decode and preprocessing
- stem separation
- beat / bar detection
- chord / key estimation
- loop mining
- embeddings
- candidate scoring
- local AI agent
- background and offline analysis

Rule: the sidecar may crash, the audio engine may not.

### 8.2 Architecture rules

- strictly separate realtime and non-realtime work
- only commit mutations on safe quantized boundaries
- no heap allocation in the audio callback
- keep heavy models, decoders, and analysis out of process

---

## 9. Technology Stack

### 9.1 Realtime core

Recommended: **Rust**

Reasons:

- strong control over memory and realtime behavior
- native performance
- safer concurrency
- good TUI and audio tooling

Core components:

- audio I/O
- TUI
- MIDI / HID
- scheduler
- DSP
- session serialization
- export subsystem

### 9.2 Analysis + AI

Recommended: **Python sidecar**

Reasons:

- MIR ecosystem
- easy model execution
- faster iteration
- easier research and prototyping

Core components:

- audio preprocessing
- model providers
- feature extraction
- RPC server
- local agent
- candidate and scoring logic

### 9.3 Communication

Between Rust core and Python sidecar:

- local Unix sockets or localhost TCP
- MessagePack / Protobuf / JSON-RPC
- asynchronous action protocol
- version numbers in all messages

---

## 10. Audio Data Flow

```text
Input Audio
 -> decode
 -> normalize
 -> optional loudness alignment
 -> stem separation
 -> beat/downbeat/bar grid
 -> key/chord estimation
 -> note/bass contour extraction
 -> section segmentation
 -> onset/transient slicing
 -> loop candidate mining
 -> embeddings / similarity vectors
 -> Source Graph
 -> Scene Graph + device feeds
 -> live playback / mutation / capture
 -> export
```

---

## 11. Analysis Pipeline

### 11.1 Decode and normalize

Tasks:

- read MP3 / WAV / AIFF / FLAC
- convert to internal sample rate
- normalize channels
- stabilize loudness for analysis

Goals:

- consistent input data
- fewer surprises in later feature extraction

### 11.2 Stem separation

Target output:

- drums
- bass
- vocals
- harmonic / rest
- optional FX / noise

Benefits:

- cleaner drum detection
- better pitch analysis
- clearer device assignment

### 11.3 Beat, downbeat, bars

Output:

- BPM candidate(s)
- beat frames
- downbeats
- bar boundaries
- confidence values
- alternative grid hypotheses

The bar grid is central truth for:

- scene changes
- loop capture
- quantized mutations
- Ghost actions

### 11.4 Harmony analysis

Output:

- key
- mode
- chord windows per bar / half-bar
- bass centers
- functional tension

Used by:

- MC-202 follower
- hook resynthesis
- de novo rebuild

### 11.5 Melody and contour extraction

Output:

- lead / hook contours
- bass contours
- phrase anchors
- interval patterns

Used by:

- 202 phrases
- motif answers
- similarity-aware mutation

### 11.6 Structural segmentation

Output:

- intro / build / drop / breakdown / outro candidates
- energy curve
- repeating blocks
- transition points

Used by:

- Scene Graph
- Ghost suggestions
- auto-arrangement

### 11.7 Loop and slice mining

Output:

- transient slices
- loop windows
- onset clusters
- repeatability hints
- confidence per candidate

Used by:

- W-30 pad forge
- capture-first workflows
- loop freezing and recuts

---

## 12. Internal Musical Representations

Riotbox should operate on explicit musical representations rather than a single opaque model state.

Core representations:

- **Source Graph** for analyzed source structure
- **Scene Graph** for performance structure
- **Action Log** for deterministic replay
- **Bank State** for W-30 pads and captures
- **Phrase Objects** for MC-202 and hook behavior
- **Pattern Objects** for 909 and hybrid drum logic

The system should prefer inspectable intermediate states over hidden transformations.

---

## 13. Device Engine Design

### 13.1 MC-202 engine

Responsibilities:

- mono lane
- follower / answer / instigator phrases
- accent and slide behavior
- controlled repetition
- pressure without overplaying

Core modules:

- mono synth voice
- phrase generator
- accent model
- slide model
- phrase scoring
- macro and page controls

### 13.2 W-30 engine

Responsibilities:

- slice pool
- loop miner interface
- pad forge
- resample lab
- bank management
- pitch / rate treatment
- grit / color handling
- loop freezing

Core principle:

**Capture is a primary action.**  
Every strong moment should be:

- stored as a pad
- marked as a loop
- promoted to a scene
- or resampled for immediate reuse

### 13.3 TR-909 engine

Responsibilities:

- reinforce or replace drums
- generate pattern variants from analyzed groove
- place hats, claps, accents, and fills
- prepare drops

Core modules:

- pattern generator
- drum reinforcement
- accent engine
- fill brain
- groove quantizer
- slam bus

Modes:

- `reinforce`
- `replace`
- `hybrid`
- `skeleton_only`

---

## 14. Arrangement and Composition Logic

Riotbox should not only build patterns. It should compose across three layers:

1. **Section grammar**
2. **Phrase generation**
3. **Micro variation**

### 14.1 Section grammar

Base forms:

```text
intro -> reveal -> build -> strip -> slam -> breakdown -> switchup -> final -> exit
```

Not every session uses every state. The arrangement system manages:

- energy
- contrast
- repetition
- surprise
- mutation budget
- source-material ratio

### 14.2 Phrase generators

Every lane writes phrases in the context of the current scene:

- MC-202
- W-30 pads / loops
- TR-909 drums
- FX / transitions
- optional vocal fragments

### 14.3 Micro variation

Every few bars, add small changes:

- slice swaps
- accent shifts
- filter opening
- half-bar silence
- ghost notes
- short reverse calls
- snare fills
- pattern compression

Rule: recognition plus movement.

---

## 15. Scoring and Selection

Riotbox should not choose blindly. It should use:

**Generate -> Score -> Select -> Mutate**

### 15.1 Groove Score

Measures:

- backbeat clarity
- syncopation fitness
- microtiming compatibility
- danceability in context

### 15.2 Identity Score

Measures:

- memorability
- contour sharpness
- useful friction
- rhythmic signature

### 15.3 Impact Score

Measures:

- drop usefulness
- build tension
- strip effectiveness
- section compatibility

### 15.4 Novelty Score

Measures:

- distance from recent bars
- distance from competing candidates
- internal similarity distance

### 15.5 Restraint Score

Measures:

- mix overload
- frequency conflicts
- too much ornament
- overactivity in the moment

---

## 16. Live Mutation and Performance Control

Riotbox becomes strong on stage when mutations are:

- audible
- quantized
- understandable
- reversible

Mutation types include:

- regenerate current scene
- mutate selected lane
- capture current bar group
- strip drums
- slam drums
- swap loop
- instantiate 202 answer
- promote resample
- reverse transition
- fake drop
- restore source

Quantization boundaries:

- next beat
- next half-bar
- next bar
- next phrase
- next scene

Safe defaults:

- creative changes on next bar
- hard rebuilds on next phrase

Undo / redo must support:

- revert last commit
- load snapshot
- restore scene
- revert Ghost action
- recover previous bank state

---

## 17. AI Agent / Ghost System

### 17.1 Role of the AI

The AI is **not** a direct audio generator. It is a tool-using musical agent.

It sees:

- tempo
- grid
- key / chords
- current scene
- active loops
- available pads
- analysis confidence
- mutation budget
- locks
- recent action history

It may:

- plan actions
- trigger tools
- justify suggestions
- perform as Ghost

### 17.2 Modes

**Watch**

- comments only
- points out opportunities

**Assist**

- proposes actions
- waits for confirmation

**Perform**

- executes quantized actions on its own
- respects budgets and guard rails

### 17.3 Ghost log

Example:

```text
[bar 17] ghost: detected strong 2-bar loop candidate from harmonic stem
[bar 21] ghost: generated MC-202 follower phrase in E minor
[bar 25] ghost: stripped hats for 1 bar before drop
[bar 29] ghost: promoted resampled phrase to W-30 bank B, pad 4
```

### 17.4 Safety limits

Ghost must never:

- block the audio thread
- trigger hard unquantized changes
- destroy locked elements
- act without undo support
- create infinite action loops

---

## 18. UX Strategy

UX should feel like an instrument, not a research dashboard.

Principles:

- **Jam-first** entry point
- visible confidence and state
- layered depth instead of parameter explosion
- fast route from hearing something good to capturing it
- strong defaults and presets
- logs that build trust instead of noise

Primary UX promise:

within minutes, a user should be able to load a track, hear a rebuild, shape it, and capture something worth keeping.

---

## 19. TUI Concept

### 19.1 Main pages

- Jam
- Arrange
- MC202
- W30
- TR909
- Mixer
- Assets
- Export
- Diagnostics

### 19.2 Example Jam screen

```text
Riotbox -- file breaksource.mp3 -- mode hybrid -- seed 90317

[142 BPM | E minor | scene BUILD | energy 0.71 | ghost ASSIST]

SOURCE    retain 43    sections 12   loops 27   confidence 0.84
MC202     touch 58     mode FOLLOW   bite 61    drift 04
W30       grit 49      bank B        pads 16    freeze ON
TR909     slam 63      hats 44       fills 21   hybrid ON
MUTATE    density 52   chaos 18      quant 1 bar
GHOST     ready        next: "promote loop L7"

LOG
[bar 25] captured harmonic loop -> W30 B4
[bar 26] generated 202 answer phrase
[bar 28] stripped kick for half-bar pre-drop
```

### 19.3 Keyboard shortcuts

- `space` play / pause
- `tab` cycle pages
- `1..8` scene launch / pad bank quick select
- `m` mutate selected lane
- `c` capture
- `l` lock selected object
- `u` undo
- `r` redo / reseed depending on context
- `g` ghost mode toggle
- `f` fill next bar
- `d` drop next phrase
- `x` destruct selected object
- `s` save snapshot
- `e` export
- `?` help overlay

---

## 20. Controller and Hardware Integration

Minimum support:

- MIDI CC learn
- note / pad trigger
- transport
- bank selection
- scene launch
- crossfader-like macros
- feedback where feasible

Primary live mappings:

- source <-> rebuild
- `202_touch`
- `w30_grit`
- `909_slam`
- `mutation`
- `density`
- `energy`
- `ghost aggression`

Performance rule:

a generic 8-knob controller setup should already be musically useful.

---

## 21. Session Model

### 21.1 Session contains

- project metadata
- input references
- analysis cache IDs
- global seeds
- lane seeds
- provider configuration
- device parameters
- scene history
- capture banks
- action log
- Ghost history
- snapshots
- exports

### 21.2 Determinism

For a session with identical:

- input
- analysis cache
- seed
- provider setup
- action log

Riotbox should be able to produce a reproducible rebuild.

### 21.3 Snapshot types

- quick snapshot
- scene snapshot
- full session snapshot
- export snapshot

---

## 22. Data Model (simplified sketch)

```text
Session
  id
  metadata
  source_manifest
  analysis_ref
  engine_state
  scene_graph
  bank_state
  controller_map
  action_log
  snapshots

AnalysisBundle
  stems
  bar_grid
  chord_timeline
  contour_data
  sections
  loops
  slices
  embeddings
  confidence

Scene
  id
  type
  energy
  active_lanes
  loop_refs
  pad_refs
  mutation_budget
  locks

Action
  timestamp
  bar_position
  actor(user|ghost|system)
  command
  params
  result
  undo_payload
```

---

## 23. Capture and Looping as the Core

Capture is not a side utility. It is a central musical action.

Key requirements:

- capture must be fast
- capture must be quantized
- captured material must immediately become reusable
- capture must preserve provenance
- promoted captures must fit naturally into later scenes

Strong moments should be easy to:

- freeze
- store as pads
- promote to scenes
- resample
- compare with earlier captures

---

## 24. Resample Lab

The Resample Lab is the place where Riotbox stops behaving like a source editor and starts behaving like an instrument.

Core operations:

- resample internal buses
- trim
- reverse
- repitch
- alter rate
- destruct
- freeze
- reassign to banks

Later possibilities:

- careful time-stretch
- granular treatment
- spectral abuse

Rule:

internal material created by the system should eventually compete with or outrank the original source.

---

## 25. FX and Mixer Strategy

FX should support identity, not only polish.

Key layers:

- source bus
- device buses
- send FX
- drum bus
- master limiter / safety

Desired behavior:

- controlled aggression
- fast scene-dependent changes
- musically useful dirt
- room and space as structural tools
- obvious recoverability after extreme states

### 25.3 Design rule

Effects serve performance and device personality, not maximum studio universality.

---

## 26. Export and Interoperability

### 26.1 Export types

Riotbox should support:

- stereo export
- stem export
- MIDI export where useful
- session export
- session snapshots
- provenance manifest
- optional sync / clock reference
- replay-ready logs

### 26.2 Minimum stem breakdown

At minimum:

- drums
- bass / 202
- sampler / W-30
- vocals / fragments
- FX
- full mix

### 26.3 Export requirements

- reproducible
- referencable by seed and session
- clearly named
- usable in batch contexts

Interoperability matters, but must not dominate the MVP at the cost of capture-first live usefulness.

---

## 27. Quality Assurance and Pro Hardening

Professional hardening should cover:

- robustness
- diagnostics
- crash recovery
- deterministic replay
- export consistency
- rehearsable stage behavior

### 27.1 Realtime stability

Required metrics:

- xruns
- callback timing
- buffer underruns
- CPU peak
- memory growth
- sidecar latency
- action queue lag

### 27.2 Test categories

#### Audio / signal

- voice correctness
- no-click guarantees
- envelope behavior
- scheduler timing

#### Logic

- scene transitions
- undo / redo
- capture integrity
- deterministic replay

#### Integration

- audio core <-> sidecar
- provider swapping
- model timeout handling
- crash recovery

### Golden renders

Reference renders for:

- same seeds
- identical actions
- same input files

### 27.3 Crash strategy

- sidecar restart without stopping audio
- analysis jobs cancelable
- Ghost disableable
- panic function for live use

Additional QA needs:

- fixture sessions
- golden renders
- replay tests
- confidence and error visibility
- musical review sessions, not only technical tests

---

## 28. Provider Architecture

### 28.1 Why providers

Many analysis and AI components will change over time.  
Riotbox therefore needs swappable interfaces.

### 28.2 Provider types

Provider interfaces should keep Riotbox flexible around:

- `StemProvider`
- `BeatProvider`
- `HarmonyProvider`
- `ContourProvider`
- `EmbeddingProvider`
- `AgentProvider`
- `StretchProvider`

### 28.3 Benefits

- faster prototyping
- later licensing swaps
- comparison of multiple methods
- more robust product strategy

Rules:

- provider boundaries must be explicit
- licensing and replacement must stay manageable
- no provider must own the realtime core

---

## 29. Legal, Licensing, Originality

Riotbox should support originality by design.

### 29.1 Core stance

Riotbox should be usable in professional contexts.  
That means the legal and aesthetic distinction between:

- structural inspiration
- material copying
- recombined source material

needs to be considered early.

### 29.2 Measures

- clearly separate operating modes
- store provenance at object level
- mark source contribution where needed
- keep provider licensing isolated
- design for a later similarity firewall

Principles:

- work from structure, not blind cloning
- distinguish derivative and de novo behavior
- keep provider licensing swappable
- make later similarity controls possible
- avoid building recognizability maximization into the product

### 29.3 Similarity firewall

As a later export-stage safeguard, Riotbox may check:

- rhythmic similarity
- melodic contour
- hook redundancy
- internal repetition problems

If thresholds are exceeded, possible responses include:

- mutate the phrase
- choose another loop
- substitute an alternative candidate

---

## 30. Repository Structure

Recommended top-level structure:

```text
riotbox/
  README.md
  docs/
    vision.md
    architecture.md
    tui.md
    audio-engine.md
    ai-agent.md
    providers.md
  crates/
    cli/
    core/
    audio/
    dsp/
    devices_mc202/
    devices_w30/
    devices_tr909/
    arranger/
    session/
    ui/
    export/
    ipc/
  python/
    sidecar/
      api/
      providers/
      pipelines/
      scoring/
      agent/
      cache/
  assets/
    presets/
    controller_maps/
    demo_projects/
  tests/
    golden/
    integration/
    property/
  scripts/
  data/
    caches/
    sessions/
    exports/
```

The exact final shape may vary, but the separation between core, devices, sidecar, docs, and tests should remain obvious.

---

## 31. MVP Definition

### 31.1 MVP goal

Within a few minutes, a user should be able to:

- analyze track structure
- start a playable rebuild
- capture loops
- control 202 and 909 live
- receive AI assistance
- save the result as a session

### 31.2 Required MVP contents

- load audio
- basic analysis
- Source Graph
- simple hybrid rebuild
- MC-202 follower
- W-30 pad capture
- TR-909 reinforcement
- Jam screen
- quantized mutation
- undo
- local agent suggestions
- session save / load

### 31.3 Explicitly out of MVP

- full vocal manipulation
- complete DAW export workflow
- polished installer for all platforms
- advanced granular / spectral processes
- multi-user / network features
- cloud AI
- plugin format

---

## 32. Implementation Phases

### Phase 0 - Sound bible and specification

Goals:

- write down the sound vision
- define reference vocabulary
- unify terminology
- sketch first TUI ideas
- define the action lexicon

Exit criteria:

- accepted vocabulary
- defined macros
- defined device personalities
- defined MVP

### Phase 1 - Core skeleton

Contents:

- project structure
- Rust audio core
- TUI skeleton
- transport
- scheduler
- session state
- action log
- snapshot basics

Exit criteria:

- stable playback
- responsive UI
- sessions can be stored
- scheduling is testable

### Phase 2 - Analysis vertical slice

Contents:

- load file
- decode / normalize
- bar grid
- sections
- first loop candidates
- rudimentary stem separation
- sidecar RPC

Exit criteria:

- a track becomes a Source Graph
- the Jam screen shows useful analysis values
- first quantized loop captures are possible

### Phase 3 - TR-909 MVP

Contents:

- drum detection
- reinforcement / layering
- basic pattern adoption
- accents / fills
- drum bus

Exit criteria:

- source drums can be strengthened audibly
- 909 can take over in a controlled way
- fills are triggerable live

### Phase 4 - MC-202 MVP

Contents:

- mono synth voice
- follower bassline
- accent / slide
- phrase generator
- `202_touch`

Exit criteria:

- good follower basslines
- live-controllable sound parameters
- quantized phrase mutation

### Phase 5 - W-30 MVP

Contents:

- slice pool
- pad forge
- loop freezer
- bank manager
- Resample Lab v1

Exit criteria:

- good loops can be captured
- pads are playable
- internal bus resampling works

### Phase 6 - Scene Brain

Contents:

- Scene Graph
- energy management
- arrangement rules
- strip / build / slam logic
- launch and restore

Exit criteria:

- a track yields multiple usable scenes
- scene changes sound musical

### Phase 7 - Ghost / AI Assist

Contents:

- tool API
- local agent
- Watch / Assist / Perform
- Ghost log
- limits and mutation budgets

Exit criteria:

- the agent makes useful suggestions
- the agent can safely execute quantized actions
- everything remains undoable

### Phase 8 - Pro hardening

Contents:

- robustness
- diagnostics
- crash recovery
- deterministic replay
- export
- provider replaceability

Exit criteria:

- stage-worthy run
- reliable session replay
- reproducible exports

---

## 33. Detailed Module Backlog

### 33.1 Audio core

- [ ] device selection
- [ ] sample-rate management
- [ ] buffer management
- [ ] scheduler
- [ ] voice pool
- [ ] mixer
- [ ] FX sends
- [ ] master limiter
- [ ] panic state

### 33.2 Session / state

- [ ] session ID
- [ ] serialization
- [ ] snapshot system
- [ ] action log
- [ ] undo payloads
- [ ] replay engine
- [ ] version migration

### 33.3 Analysis sidecar

- [ ] RPC server
- [ ] job queue
- [ ] cache store
- [ ] decode pipeline
- [ ] beat / bar analysis
- [ ] harmony analysis
- [ ] contour extraction
- [ ] loop miner
- [ ] embedding interface

### 33.4 MC-202

- [ ] mono oscillator core
- [ ] filter
- [ ] envelopes
- [ ] accent model
- [ ] slide model
- [ ] follower generator
- [ ] instigator generator
- [ ] phrase scoring
- [ ] UI page

### 33.5 W-30

- [ ] slice editor
- [ ] pad mapping
- [ ] bank switching
- [ ] resample routing
- [ ] reverse / pitch / rate
- [ ] capture shortcuts
- [ ] provenance tracking

### 33.6 TR-909

- [ ] kick layer
- [ ] snare layer
- [ ] hat generator
- [ ] accent lane
- [ ] fill generator
- [ ] pattern modes
- [ ] slam bus

### 33.7 Arrangement

- [ ] section classifier
- [ ] scene templates
- [ ] energy model
- [ ] mutation rules
- [ ] drop preparation
- [ ] restore rules

### 33.8 TUI

- [ ] Jam screen
- [ ] MC202 page
- [ ] W30 page
- [ ] TR909 page
- [ ] Arrange page
- [ ] mixer page
- [ ] log panel
- [ ] help overlay
- [ ] performance meters

### 33.9 AI agent

- [ ] state summarizer
- [ ] tool schema
- [ ] budget manager
- [ ] watch mode
- [ ] assist mode
- [ ] perform mode
- [ ] explanation formatter
- [ ] guard rails

---

## 34. Non-Functional Requirements

### 34.1 Latency

- playable macro reaction
- quantized commit logic
- clearly visible pending-action states

### 34.2 Stability

- audio must not crackle
- Ghost must be disableable
- sessions must continue even if analysis fails

### 34.3 Memory and CPU

- conservative realtime paths
- large samples streamed or preloaded appropriately
- bounded sidecar load
- graceful degradation

### 34.4 Usability

- sensible defaults
- strong presets
- useful logging
- no parameter flood on the Jam screen

---

## 35. Preset and Style System

### 35.1 Preset layers

- global project preset
- device preset
- scene preset
- controller mapping
- Ghost policy

### 35.2 Early style families

- `feral_break`
- `acid_wire`
- `sampler_bruise`
- `night_slam`
- `ghost_cut`

Each family defines:

- energy behavior
- device weighting
- mutation appetite
- source loyalty
- grit / drive
- Ghost autonomy

---

## 36. Preset Macros

### Global macros

- `source_retain`
- `energy`
- `mutation`
- `density`
- `ghost`

### Device macros

- `202_touch`
- `w30_grit`
- `loop_freeze`
- `909_slam`
- `hat_density`
- `bank_morph`

### Performance macros

- `fake_drop`
- `destroy_rebuild`
- `promote_now`
- `restore_source`
- `capture_now`

---

## 37. Explainability and Trust

As soon as AI and complex analysis matter, the user needs trust.

Measures:

- every Ghost action is logged
- important analysis values are visible
- confidence values are shown
- "why this action?" can be explained
- locks and budgets are visible
- undo stays close at hand

---

## 38. Risk Analysis

### 38.1 Technical risks

**Realtime vs. AI**  
Risk: model calls indirectly block flow.  
Countermeasure: strict process boundaries.

**Analysis quality**  
Risk: poor beat or chord detection.  
Countermeasure: alternative hypotheses, confidence values, manual correction paths.

**Overcomplexity**  
Risk: too many features too early.  
Countermeasure: strict MVP and hard prioritization.

**Unmusical mutation**  
Risk: the system sounds clever but not good.  
Countermeasure: scoring, capture-first, strong presets, real music test sessions.

### 38.2 Product risks

**UX drift**  
Risk: tool becomes a nerd lab instead of an instrument.  
Countermeasure: Jam-first.

**Style cliche**  
Risk: overly literal retro imitation.  
Countermeasure: devices as roles, not cosplay.

**Licensing / provider problems**  
Risk: unsuitable analysis or model licenses.  
Countermeasure: provider boundaries and replaceability.

---

## 39. Team and Role Recommendations

Useful roles:

- **Product / Sound Director**  
  owns vision, sound bible, and UX sharpness

- **Realtime Audio Engineer**  
  owns audio core, DSP, scheduling, and device logic

- **MIR / ML Engineer**  
  owns analysis pipeline, features, scoring, and agent integration

- **TUI / Interaction Engineer**  
  owns Jam / Sculpt / Lab, controller mappings, and diagnostics

- **QA / Music Tester**  
  owns real session tests, golden renders, and sound critique

Early on, roles can be combined.

---

## 40. Early Milestones in Weekly Logic

### Milestone A

**"Load a track, see it, loop it."**

- load a file
- start analysis
- show grid / sections
- capture top loops

### Milestone B

**"Mutate a track, but musically."**

- 909 reinforcement
- 202 follower
- simple scenes

### Milestone C

**"The track becomes an instrument."**

- W-30 pads
- capture-first workflow
- resampling

### Milestone D

**"The instrument gains agency."**

- Ghost Watch / Assist
- actions, budgets, logs

### Milestone E

**"Stage-worthy."**

- pro hardening
- replay
- export
- crash safety

---

## 41. Development Decision Rules

When choosing between options:

1. **Stability before cleverness**
2. **Musical impact before algorithmic elegance**
3. **Instrument character before feature mass**
4. **Capture and performance before offline perfection**
5. **Traceability before magic**
6. **MVP sharpness before concept inflation**

---

## 42. Final Product Image

Riotbox should feel like:

- a wild hardware hybrid that was never built
- a feral blend of MC-202, W-30, and TR-909
- a terminal-based live instrument, not a disguised research project
- a system that turns audio into moments, moments into phrases, and phrases into new scenes
- a tool you play, watch, harvest, and reignite

The product core is not "AI makes music", but:

> **Riotbox translates sound into musical spaces of action.**  
> The human, the Ghost, or both together perform those spaces live.

---

## 43. Next Concrete Documents

After this masterplan, the next artifacts should be:

1. **PRD v1**  
   exact product requirements for the MVP

2. **Audio Core Spec**  
   scheduler, mixer, voices, timing, FX, buffers

3. **Source Graph Spec**  
   data model and analysis outputs

4. **TUI Screen Spec**  
   pages, states, keyboard mapping, macros

5. **Ghost API Spec**  
   tool schema, budgets, actions, logs

6. **Session File Spec**  
   serialization, migration, replay

7. **Preset and Style Spec**  
   macro ranges, style families, device weighting

8. **Feral Reconstruction Addendum**  
   decision logic for harvest, break rebuild, hook resample, and abuse mix

---

## 44. One-Sentence Version

> Riotbox is a terminal-native live instrument that analyzes audio, rebuilds it through device personalities, and turns it into a controllable, capture-first performance system.
