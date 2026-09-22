//! Preserve legacy Session spelling while sharing the Graph's typed vocabulary.
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::source_graph::DecodeProfile;

pub(super) fn serialize<S: Serializer>(
    profile: &DecodeProfile,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    validate_name(profile).map_err(serde::ser::Error::custom)?;
    match profile {
        DecodeProfile::Native => serializer.serialize_str("native"),
        DecodeProfile::NormalizedStereo => serializer.serialize_str("normalized_stereo"),
        DecodeProfile::NormalizedMono => serializer.serialize_str("normalized_mono"),
        DecodeProfile::Custom(name) => {
            // Old writers flattened Custom("native") to "native", losing its
            // kind. Keep that exceptional name explicit in new Sessions.
            if matches!(
                name.as_str(),
                "native" | "normalized_stereo" | "normalized_mono"
            ) {
                profile.serialize(serializer)
            } else {
                serializer.serialize_str(name)
            }
        }
    }
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<DecodeProfile, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Wire {
        Legacy(String),
        Explicit(DecodeProfile),
    }
    let profile = match Wire::deserialize(deserializer)? {
        Wire::Legacy(name) => match name.as_str() {
            "native" => DecodeProfile::Native,
            "normalized_stereo" => DecodeProfile::NormalizedStereo,
            "normalized_mono" => DecodeProfile::NormalizedMono,
            _ => DecodeProfile::Custom(name),
        },
        Wire::Explicit(profile) => profile,
    };
    validate_name(&profile).map_err(D::Error::custom)?;
    Ok(profile)
}

fn validate_name(profile: &DecodeProfile) -> Result<(), &'static str> {
    if let DecodeProfile::Custom(name) = profile
        && (name.trim().is_empty() || name.chars().any(char::is_control))
    {
        return Err("invalid legacy/custom source decode profile name");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{session::SourceRef, source_graph::DecodeProfile};
    use serde_json::json;

    #[test]
    fn legacy_standard_and_custom_profiles_roundtrip_without_changing_spelling() {
        for (wire, expected) in [
            (json!("native"), DecodeProfile::Native),
            (json!("normalized_stereo"), DecodeProfile::NormalizedStereo),
            (json!("normalized_mono"), DecodeProfile::NormalizedMono),
            (
                json!("my-decoder-v1"),
                DecodeProfile::Custom("my-decoder-v1".into()),
            ),
        ] {
            let source = json!({
                "source_id": "source-a", "path_hint": "metadata-only.wav",
                "content_hash": "synthetic", "duration_seconds": 8.0, "decode_profile": wire,
            });
            let parsed: SourceRef = serde_json::from_value(source.clone()).unwrap();
            assert_eq!(parsed.decode_profile, expected);
            assert_eq!(serde_json::to_value(parsed).unwrap(), source);
        }
        // The Graph spelling is intentionally not globally changed by this adapter.
        assert_eq!(
            serde_json::to_value(DecodeProfile::NormalizedStereo).unwrap(),
            json!("NormalizedStereo")
        );
    }

    #[test]
    fn reserved_custom_names_are_explicit_and_malformed_profiles_are_rejected() {
        let mut source = json!({
            "source_id": "source-a", "path_hint": "metadata-only.wav",
            "content_hash": "synthetic", "duration_seconds": 8.0,
            "decode_profile": {"Custom": "native"},
        });
        let parsed: SourceRef = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            parsed.decode_profile,
            DecodeProfile::Custom("native".into())
        );
        assert_eq!(serde_json::to_value(parsed).unwrap(), source);
        for invalid in [
            json!(""),
            json!("\n"),
            json!(17),
            json!({"Unknown": "profile"}),
        ] {
            source["decode_profile"] = invalid;
            assert!(serde_json::from_value::<SourceRef>(source.clone()).is_err());
        }
    }
}
