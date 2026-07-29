use serde::{Deserialize, Serialize};

/// Performer-owned domain for a committed W-30 damage gesture.
///
/// The intent constrains source-derived realization; it never authorizes a
/// synthetic replacement, invented transient, or trigger grid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum W30HardIntent {
    /// Compatibility route for Session v1 actions written before intent was typed.
    LegacyAuto,
    /// Request source-owned transient/body impact.
    Impact,
    /// Request continuous source-owned roughness without a trigger grid.
    Texture,
}

impl W30HardIntent {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LegacyAuto => "legacy_auto",
            Self::Impact => "impact",
            Self::Texture => "texture",
        }
    }

    #[must_use]
    pub const fn is_performer_request(self) -> bool {
        matches!(self, Self::Impact | Self::Texture)
    }
}

/// Product projection result for a requested W-30 Hard domain.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30HardIntentOutcome {
    #[default]
    Inactive,
    LegacyAuto,
    RealizedImpact,
    RealizedTexture,
    SourceMismatch,
    SourceUnavailable,
}

impl W30HardIntentOutcome {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::LegacyAuto => "legacy_auto",
            Self::RealizedImpact => "realized_impact",
            Self::RealizedTexture => "realized_texture",
            Self::SourceMismatch => "source_mismatch",
            Self::SourceUnavailable => "source_unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::W30HardIntent;

    #[test]
    fn hard_intent_has_stable_session_labels() {
        for (intent, label) in [
            (W30HardIntent::LegacyAuto, "legacy_auto"),
            (W30HardIntent::Impact, "impact"),
            (W30HardIntent::Texture, "texture"),
        ] {
            assert_eq!(intent.label(), label);
            assert_eq!(
                serde_json::to_string(&intent).expect("serialize hard intent"),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<W30HardIntent>(&format!("\"{label}\""))
                    .expect("deserialize hard intent"),
                intent
            );
        }
        assert!(!W30HardIntent::LegacyAuto.is_performer_request());
        assert!(W30HardIntent::Impact.is_performer_request());
        assert!(W30HardIntent::Texture.is_performer_request());
    }
}
