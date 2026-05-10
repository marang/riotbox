#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Mc202LaneState {
    pub role: Option<String>,
    pub phrase_ref: Option<String>,
    #[serde(default)]
    pub phrase_variant: Option<Mc202PhraseVariantState>,
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
