# Architecture And Phase Map

Status: active orientation diagram

Linear mirror:
<https://linear.app/riotbox/document/riotbox-architecture-and-phase-map-86de17a4248a>

This document connects the functional Riotbox spine to the Linear phase
boundaries. It is an orientation map, not a replacement for specs, Linear, or
the roadmap.

## Functional Spine

```mermaid
flowchart LR
    subgraph ingest["Source understanding"]
        Source["Loaded source"]
        SourceGraph["Source Graph"]
        SourceTiming["Source Timing summary"]
    end

    subgraph control["Product truth"]
        Session["Session file / runtime state"]
        Actions["Action Lexicon"]
        Queue["Action queue"]
        Commit["Commit records"]
        Replay["Replay / restore"]
    end

    subgraph surface["User and observer surfaces"]
        JamView["Jam view model"]
        Observer["Observer / audio correlation"]
    end

    subgraph output["Musical output"]
        TR909["TR-909"]
        MC202["MC-202"]
        W30["W-30"]
        Mix["Generated-support mix"]
        AudioQA["Audio QA gates"]
    end

    subgraph policy["Policy and assist"]
        Feral["Feral policy / scorecard"]
        Ghost["Ghost Watch / Assist"]
    end

    Source --> SourceGraph --> SourceTiming
    SourceGraph --> Session
    SourceTiming --> JamView
    Actions --> Queue --> Commit --> Session
    Session --> Replay --> Session
    Session --> JamView --> Observer
    Session --> TR909 --> Mix
    Session --> MC202 --> Mix
    Session --> W30 --> Mix
    Mix --> AudioQA
    Observer --> AudioQA
    Feral --> SourceGraph
    Feral --> Actions
    Ghost --> Actions
    Ghost --> JamView
```

P014 sits on the existing Source Graph, Source Timing, Session, Action Lexicon,
queue / commit, replay, Jam view, and audio QA contracts. It must not introduce
a second arrangement truth. P012 and P013 remain regression baselines while P014
adds arrangement / scene behavior.

## Phase Boundaries

```mermaid
flowchart TB
    subgraph foundation["Foundation"]
        P000["P000 Repo Ops / QA / Workflow"]
        P001["P001 Spec Freeze + Core Model"]
        P002["P002 Core Skeleton"]
        P003["P003 Analysis Vertical Slice"]
        P004["P004 Jam-First Playable Slice"]
    end

    subgraph lanes["Lane and scene MVPs"]
        P005["P005 TR-909 MVP"]
        P006["P006 MC-202 MVP"]
        P007["P007 W-30 MVP"]
        P008["P008 Scene Brain"]
        P009["P009 Feral Policy Layer"]
        P010["P010 Ghost Watch / Assist"]
    end

    subgraph baselines["Closed regression baselines"]
        P011["P011 Pro Hardening"]
        P012["P012 Source Timing Intelligence"]
        P013["P013 All-Lane Musical Depth"]
    end

    subgraph active["Active arrangement phase"]
        P014["P014 Arrangement / Scene System"]
    end

    subgraph later["Later product phases"]
        P015["P015 Productization Alpha"]
        P016["P016 Pro Workflow / Export"]
        P017["P017 Live Performance Readiness"]
        P018["P018 Ghost + Feral Autonomy Expansion"]
        P019["P019 Beta / Release Hardening"]
        P020["P020 Riotbox 1.0 Release Cut"]
    end

    foundation --> lanes --> baselines --> active --> later
```

## Phase To Contract Map

```mermaid
flowchart LR
    subgraph phases["Phase bands"]
        Foundation["P000-P004 foundation"]
        LaneMvp["P005-P010 lane / scene / policy MVPs"]
        ClosedBase["P011-P013 regression baselines"]
        P014["P014 active arrangement"]
        Product["P015-P020 productization / release"]
    end

    subgraph contracts["Primary contracts touched"]
        Workflow["Workflow / Linear / CI"]
        Core["Source Graph / Session / Actions"]
        Device["TR-909 / MC-202 / W-30"]
        Assist["Feral / Ghost"]
        Timing["Source Timing / observer correlation"]
        MusicalQA["All-lane musical QA"]
        Arrangement["Arrangement / Scene contract"]
        ExportLive["Export / live / release gates"]
    end

    Foundation --> Workflow
    Foundation --> Core
    LaneMvp --> Device
    LaneMvp --> Assist
    LaneMvp --> Core
    ClosedBase --> Timing
    ClosedBase --> MusicalQA
    ClosedBase --> ExportLive
    P014 --> Arrangement
    P014 --> Core
    P014 --> Timing
    P014 --> MusicalQA
    Product --> ExportLive
    Product --> Assist
    Product --> MusicalQA
```
