#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Mc202LaneState {
    pub role: Option<Mc202RoleState>,
    pub phrase_ref: Option<String>,
    #[serde(default)]
    pub phrase_variant: Option<Mc202PhraseVariantState>,
    #[serde(default)]
    pub source_phrase_plan: Option<Mc202SourcePhrasePlanState>,
}

impl Mc202LaneState {
    #[must_use]
    pub fn role_label(&self) -> Option<&'static str> {
        self.role.map(Mc202RoleState::label)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mc202SourcePhrasePlanState {
    pub source_id: SourceId,
    pub phrase_slot: Mc202SourcePhraseSlotState,
    #[serde(default)]
    pub source_expression: Option<Mc202SourcePhraseExpressionState>,
    pub role: Mc202RoleState,
    pub rhythm_cells: [Option<i8>; 16],
    pub note_budget: Mc202SourcePhraseNoteBudgetState,
    pub touch: f32,
    pub confidence: f32,
    #[serde(default)]
    pub candidate_family: Option<Mc202SourcePhraseCandidateFamilyState>,
    #[serde(default)]
    pub candidate_count: u8,
    #[serde(default)]
    pub rejected_candidate_count: u8,
    #[serde(default)]
    pub candidate_provenance_refs: Vec<String>,
    #[serde(default)]
    pub candidate_scorecards: Vec<Mc202SourcePhraseCandidateScoreState>,
    #[serde(default)]
    pub phrase_memory_distance: f32,
    #[serde(default)]
    pub fallback_reason: Option<String>,
}

impl Mc202SourcePhrasePlanState {
    #[must_use]
    pub fn is_source_derived(&self) -> bool {
        let family_is_source_derived = self
            .candidate_family
            .is_none_or(Mc202SourcePhraseCandidateFamilyState::is_source_derived);
        self.fallback_reason.is_none() && family_is_source_derived
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mc202SourcePhraseExpressionState {
    pub low_pressure_contour: f32,
    pub bass_pressure: f32,
    pub transient_backbeat: f32,
    pub offbeat_answer_space: f32,
    pub phrase_density: f32,
    pub hook_restraint: f32,
    pub stab_bite: f32,
    pub stay_out_pressure: f32,
    pub confidence: f32,
    #[serde(default)]
    pub provenance_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mc202SourcePhraseCandidateScoreState {
    pub family: Mc202SourcePhraseCandidateFamilyState,
    pub selected: bool,
    pub total_score: f32,
    pub low_end_impact: f32,
    pub source_grid_lock: f32,
    pub answer_contrast: f32,
    pub hook_avoidance: f32,
    pub phrase_memory: f32,
    pub destructive_usefulness: f32,
    pub role_fit: f32,
    #[serde(default)]
    pub rejection_reason: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202SourcePhraseCandidateFamilyState {
    SubPressureShove,
    SparseOffbeatAnswer,
    CallBackStab,
    HookRestraintGhostAnswer,
    FillPickupInstigator,
    StayOut,
    FallbackControl,
}

impl Mc202SourcePhraseCandidateFamilyState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SubPressureShove => "sub_pressure_shove",
            Self::SparseOffbeatAnswer => "sparse_offbeat_answer",
            Self::CallBackStab => "call_back_stab",
            Self::HookRestraintGhostAnswer => "hook_restraint_ghost_answer",
            Self::FillPickupInstigator => "fill_pickup_instigator",
            Self::StayOut => "stay_out",
            Self::FallbackControl => "fallback_control",
        }
    }

    #[must_use]
    pub const fn is_source_derived(self) -> bool {
        !matches!(self, Self::StayOut | Self::FallbackControl)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Mc202SourcePhraseSlotState {
    pub phrase_index: u32,
    pub start_bar: u32,
    pub end_bar: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202SourcePhraseNoteBudgetState {
    Sparse,
    Balanced,
    Push,
    Wide,
}

impl Mc202SourcePhraseNoteBudgetState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sparse => "sparse",
            Self::Balanced => "balanced",
            Self::Push => "push",
            Self::Wide => "wide",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202RoleState {
    Leader,
    Follower,
    Answer,
    Pressure,
    Instigator,
}

impl Mc202RoleState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Leader => "leader",
            Self::Follower => "follower",
            Self::Answer => "answer",
            Self::Pressure => "pressure",
            Self::Instigator => "instigator",
        }
    }

    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "leader" => Some(Self::Leader),
            "follower" => Some(Self::Follower),
            "answer" => Some(Self::Answer),
            "pressure" => Some(Self::Pressure),
            "instigator" => Some(Self::Instigator),
            _ => None,
        }
    }

    #[must_use]
    pub const fn default_touch(self) -> f32 {
        match self {
            Self::Leader => 0.85,
            Self::Follower => 0.78,
            Self::Answer => 0.82,
            Self::Pressure => 0.84,
            Self::Instigator => 0.90,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202PhraseVariantState {
    MutatedDrive,
}

impl Mc202PhraseVariantState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MutatedDrive => "mutated_drive",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202PhraseIntentState {
    Base,
    MutatedDrive,
}

impl Mc202PhraseIntentState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::MutatedDrive => "mutated_drive",
        }
    }

    #[must_use]
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "base" => Some(Self::Base),
            "mutated_drive" => Some(Self::MutatedDrive),
            _ => None,
        }
    }

    #[must_use]
    pub const fn from_phrase_variant(variant: Option<Mc202PhraseVariantState>) -> Self {
        match variant {
            Some(Mc202PhraseVariantState::MutatedDrive) => Self::MutatedDrive,
            None => Self::Base,
        }
    }

    #[must_use]
    pub const fn phrase_variant(self) -> Option<Mc202PhraseVariantState> {
        match self {
            Self::Base => None,
            Self::MutatedDrive => Some(Mc202PhraseVariantState::MutatedDrive),
        }
    }
}
