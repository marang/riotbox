use serde::{Deserialize, Serialize};

/// Identity of the complete encoded WAV, not decoded samples or source audio.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureAudioIdentity {
    pub sha256: String,
    pub provenance: CaptureAudioIdentityProvenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_requires_canonical_hash_and_explicit_legacy_provenance() {
        let mut identity = CaptureAudioIdentity {
            sha256: format!("sha256:{}", "ab".repeat(32)),
            provenance: CaptureAudioIdentityProvenance::CreatedFromEncodedBytesV1,
        };
        assert!(identity.is_valid());
        let encoded = serde_json::to_vec(&identity).unwrap();
        assert_eq!(
            serde_json::from_slice::<CaptureAudioIdentity>(&encoded).unwrap(),
            identity
        );
        identity.provenance = CaptureAudioIdentityProvenance::AdoptedLegacyV1 {
            adopted_at: "".into(),
        };
        assert!(!identity.is_valid());
        identity.provenance = CaptureAudioIdentityProvenance::AdoptedLegacyV1 {
            adopted_at: "unix_ms:1".into(),
        };
        assert!(identity.is_valid());
        for hash in [
            "sha256:abc".to_owned(),
            format!("sha256:{}", "AB".repeat(32)),
            "ab".repeat(32),
        ] {
            identity.sha256 = hash;
            assert!(!identity.is_valid());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureAudioIdentityProvenance {
    CreatedFromEncodedBytesV1,
    /// Explicitly accepted current bytes; historical authenticity is unknown.
    AdoptedLegacyV1 {
        adopted_at: String,
    },
}

impl CaptureAudioIdentity {
    pub fn is_valid(&self) -> bool {
        self.sha256.strip_prefix("sha256:").is_some_and(|hash| {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) && match &self.provenance {
            CaptureAudioIdentityProvenance::CreatedFromEncodedBytesV1 => true,
            CaptureAudioIdentityProvenance::AdoptedLegacyV1 { adopted_at } => {
                !adopted_at.trim().is_empty()
            }
        }
    }
}
