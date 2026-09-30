#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderModePolicy {
    Idle,
    SourceSupport,
    Fill,
    BreakReinforce,
    Takeover,
}

impl Tr909RenderModePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SourceSupport => "source_support",
            Self::Fill => "fill",
            Self::BreakReinforce => "break_reinforce",
            Self::Takeover => "takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderRoutingPolicy {
    SourceOnly,
    DrumBusSupport,
    DrumBusTakeover,
}

impl Tr909RenderRoutingPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceOnly => "source_only",
            Self::DrumBusSupport => "drum_bus_support",
            Self::DrumBusTakeover => "drum_bus_takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909SourceSupportProfilePolicy {
    SteadyPulse,
    BreakLift,
    DropDrive,
}

impl Tr909SourceSupportProfilePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SteadyPulse => "steady_pulse",
            Self::BreakLift => "break_lift",
            Self::DropDrive => "drop_drive",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909SourceSupportContextPolicy {
    SceneTarget,
    TransportBar,
}

impl Tr909SourceSupportContextPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SceneTarget => "scene_target",
            Self::TransportBar => "transport_bar",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909SourceSupportReasonPolicy {
    FeralBreakLift,
}

impl Tr909SourceSupportReasonPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FeralBreakLift => "feral_break_lift",
        }
    }

    #[must_use]
    pub const fn cue_label(self) -> &'static str {
        match self {
            Self::FeralBreakLift => "feral break lift",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909TakeoverRenderProfilePolicy {
    ControlledPhrase,
    SceneLock,
}

impl Tr909TakeoverRenderProfilePolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ControlledPhrase => "controlled_phrase",
            Self::SceneLock => "scene_lock",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909PatternAdoptionPolicy {
    SupportPulse,
    MainlineDrive,
    TakeoverGrid,
}

impl Tr909PatternAdoptionPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SupportPulse => "support_pulse",
            Self::MainlineDrive => "mainline_drive",
            Self::TakeoverGrid => "takeover_grid",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909PhraseVariationPolicy {
    PhraseAnchor,
    PhraseLift,
    PhraseDrive,
    PhraseRelease,
}

impl Tr909PhraseVariationPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PhraseAnchor => "phrase_anchor",
            Self::PhraseLift => "phrase_lift",
            Self::PhraseDrive => "phrase_drive",
            Self::PhraseRelease => "phrase_release",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tr909RenderPolicyProjection {
    pub mode: Tr909RenderModePolicy,
    pub routing: Tr909RenderRoutingPolicy,
    pub source_support_profile: Option<Tr909SourceSupportProfilePolicy>,
    pub source_support_context: Option<Tr909SourceSupportContextPolicy>,
    pub takeover_profile: Option<Tr909TakeoverRenderProfilePolicy>,
    pub pattern_adoption: Option<Tr909PatternAdoptionPolicy>,
    pub phrase_variation: Option<Tr909PhraseVariationPolicy>,
}
