use riotbox_audio::w30::{
    W30_RESAMPLE_HARD_SLICE_COUNT, W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY,
    W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE, W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE,
    W30ResampleLowImpactDecision, W30ResampleLowImpactPlan, W30ResampleLowImpactRecipe,
    W30ResampleLowImpactRole,
};

use super::{bandpass_window, rms};

const LOW_HZ: f32 = 45.0;
const HIGH_HZ: f32 = 180.0;
const MIN_RMS: f32 = 1.0e-6;
const MIN_ATTACK_SECONDS: f32 = 0.02;
const MAX_ATTACK_SECONDS: f32 = 0.08;
const MIN_BODY_SECONDS: f32 = 0.04;
const MAX_BODY_SECONDS: f32 = 0.12;

#[derive(Copy, Clone)]
struct LowImpactCandidate {
    slot: u8,
    onset_cursor: u16,
    attack_frames: u16,
    body_frames: u16,
    attack_share: f32,
    attack_over_body: f32,
    attack_over_source: f32,
    score: f32,
}

pub(super) fn derive_w30_resample_low_impact(
    proxy: &[f32],
    proxy_sample_rate: f32,
    trigger_mask: u8,
    onset_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    attack_lengths: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
) -> W30ResampleLowImpactPlan {
    if proxy.len() < 2
        || !proxy_sample_rate.is_finite()
        || proxy_sample_rate <= HIGH_HZ * 2.0
        || trigger_mask == 0
    {
        return W30ResampleLowImpactPlan::default();
    }

    let low_proxy = bandpass_window(proxy, proxy_sample_rate, LOW_HZ, HIGH_HZ);
    let source_rms = rms(proxy).max(MIN_RMS);
    let min_attack_frames = (proxy_sample_rate * MIN_ATTACK_SECONDS).round().max(1.0) as usize;
    let max_attack_frames = (proxy_sample_rate * MAX_ATTACK_SECONDS)
        .round()
        .max(min_attack_frames as f32) as usize;
    let min_body_frames = (proxy_sample_rate * MIN_BODY_SECONDS).round().max(1.0) as usize;
    let max_body_frames = (proxy_sample_rate * MAX_BODY_SECONDS)
        .round()
        .max(min_body_frames as f32) as usize;
    let mut candidate_count = 0_u8;
    let mut selected: Option<LowImpactCandidate> = None;
    for slot in 0..W30_RESAMPLE_HARD_SLICE_COUNT {
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = usize::from(onset_cursors[slot]).min(proxy.len() - 1);
        let attack_len =
            usize::from(attack_lengths[slot].max(1)).clamp(min_attack_frames, max_attack_frames);
        let attack_end = onset.saturating_add(attack_len).min(proxy.len());
        if attack_end <= onset {
            continue;
        }
        let body_len = attack_len
            .saturating_mul(2)
            .clamp(min_body_frames, max_body_frames);
        let body_end = attack_end.saturating_add(body_len).min(proxy.len());
        if body_end.saturating_sub(attack_end) < body_len.div_ceil(2) {
            continue;
        }
        candidate_count = candidate_count.saturating_add(1);
        let low_attack_rms = rms(&low_proxy[onset..attack_end]);
        let full_attack_rms = rms(&proxy[onset..attack_end]).max(MIN_RMS);
        let low_body_rms = rms(&low_proxy[attack_end..body_end]).max(MIN_RMS);
        let attack_share = (low_attack_rms / full_attack_rms).min(1.0);
        let attack_over_body = low_attack_rms / low_body_rms;
        let attack_over_source = low_attack_rms / source_rms;
        let score = (attack_share / W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE)
            .min(attack_over_body / W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY)
            .min(attack_over_source / W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE);
        let candidate = LowImpactCandidate {
            slot: slot as u8,
            onset_cursor: onset_cursors[slot],
            attack_frames: attack_end.saturating_sub(onset).min(usize::from(u16::MAX)) as u16,
            body_frames: body_end
                .saturating_sub(attack_end)
                .min(usize::from(u16::MAX)) as u16,
            attack_share,
            attack_over_body,
            attack_over_source,
            score,
        };
        if selected.is_none_or(|current| candidate.outranks(current)) {
            selected = Some(candidate);
        }
    }

    let Some(selected) = selected else {
        return W30ResampleLowImpactPlan {
            decision: W30ResampleLowImpactDecision::NoCompleteCandidateWindow,
            ..W30ResampleLowImpactPlan::default()
        };
    };
    selected.into_plan(candidate_count)
}

impl LowImpactCandidate {
    fn outranks(self, current: Self) -> bool {
        self.score
            .total_cmp(&current.score)
            .then_with(|| self.attack_over_body.total_cmp(&current.attack_over_body))
            .then_with(|| current.slot.cmp(&self.slot))
            .is_gt()
    }

    fn into_plan(self, candidate_count: u8) -> W30ResampleLowImpactPlan {
        let share_margin = self.attack_share / W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_SHARE;
        let body_margin = self.attack_over_body / W30_RESAMPLE_HIT_SHAPER_MIN_ATTACK_OVER_BODY;
        let source_margin =
            self.attack_over_source / W30_RESAMPLE_LOW_IMPACT_MIN_ATTACK_OVER_SOURCE;
        let (recipe, role, decision) =
            if share_margin >= 1.0 && body_margin >= 1.0 && source_margin >= 1.0 {
                (
                    W30ResampleLowImpactRecipe::SourceHitShaperV3,
                    W30ResampleLowImpactRole::TransientLowBody,
                    W30ResampleLowImpactDecision::SourceHitSelected,
                )
            } else if share_margin <= body_margin && share_margin <= source_margin {
                (
                    W30ResampleLowImpactRecipe::Unavailable,
                    W30ResampleLowImpactRole::Unassigned,
                    W30ResampleLowImpactDecision::InsufficientAttackShare,
                )
            } else if body_margin <= source_margin {
                (
                    W30ResampleLowImpactRecipe::Unavailable,
                    W30ResampleLowImpactRole::Unassigned,
                    W30ResampleLowImpactDecision::InsufficientAttackOverBody,
                )
            } else {
                (
                    W30ResampleLowImpactRecipe::Unavailable,
                    W30ResampleLowImpactRole::Unassigned,
                    W30ResampleLowImpactDecision::InsufficientAttackOverSource,
                )
            };
        W30ResampleLowImpactPlan {
            recipe,
            role,
            decision,
            candidate_count,
            selected_slot: self.slot,
            selected_onset_cursor: self.onset_cursor,
            attack_window_proxy_frames: self.attack_frames,
            body_window_proxy_frames: self.body_frames,
            low_band_attack_share: self.attack_share,
            low_band_attack_over_body: self.attack_over_body,
            low_band_attack_over_source: self.attack_over_source,
        }
    }
}

#[cfg(test)]
mod tests {
    use riotbox_audio::{
        source_audio::SourceAudioCache,
        w30::{
            W30_RESAMPLE_HARD_SLICE_COUNT, W30ResampleLowImpactDecision,
            W30ResampleLowImpactRecipe, W30ResampleLowImpactRole,
        },
    };

    use super::{super::project_resample_source_from_interleaved, derive_w30_resample_low_impact};

    #[test]
    fn selects_one_robust_source_hit_without_averaging_in_sustained_slots() {
        let sample_rate = 8_000.0;
        let mut proxy = (0..6_400)
            .map(|index| 0.32 * (std::f32::consts::TAU * 90.0 * index as f32 / sample_rate).sin())
            .collect::<Vec<_>>();
        for (local_index, sample) in proxy[800..1_120].iter_mut().enumerate() {
            let envelope = if local_index < 160 { 0.92 } else { 0.04 };
            *sample =
                envelope * (std::f32::consts::TAU * 90.0 * local_index as f32 / sample_rate).sin();
        }
        let mut onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        onsets[1] = 800;
        let plan = derive_w30_resample_low_impact(
            &proxy,
            sample_rate,
            0b0000_0011,
            onsets,
            [160; W30_RESAMPLE_HARD_SLICE_COUNT],
        );

        assert_eq!(plan.recipe, W30ResampleLowImpactRecipe::SourceHitShaperV3);
        assert_eq!(plan.role, W30ResampleLowImpactRole::TransientLowBody);
        assert_eq!(
            plan.decision,
            W30ResampleLowImpactDecision::SourceHitSelected
        );
        assert_eq!(plan.candidate_count, 2);
        assert_eq!(plan.selected_slot, 1);
        assert_eq!(plan.selected_onset_cursor, 800);
        assert_eq!(plan.attack_window_proxy_frames, 160);
        assert_eq!(plan.body_window_proxy_frames, 320);
    }

    #[test]
    fn selection_is_deterministic_for_identical_candidates() {
        let sample_rate = 8_000.0;
        let mut proxy = vec![0.0; 2_400];
        for onset in [400, 1_200] {
            for local_index in 0..480 {
                let envelope = if local_index < 160 { 0.9 } else { 0.04 };
                proxy[onset + local_index] = envelope
                    * (std::f32::consts::TAU * 90.0 * local_index as f32 / sample_rate).sin();
            }
        }
        let mut onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        onsets[0] = 400;
        onsets[1] = 1_200;

        let first = derive_w30_resample_low_impact(
            &proxy,
            sample_rate,
            0b0000_0011,
            onsets,
            [160; W30_RESAMPLE_HARD_SLICE_COUNT],
        );
        let repeated = derive_w30_resample_low_impact(
            &proxy,
            sample_rate,
            0b0000_0011,
            onsets,
            [160; W30_RESAMPLE_HARD_SLICE_COUNT],
        );

        assert_eq!(first, repeated);
        assert_eq!(first.selected_slot, 0);
    }

    #[test]
    fn reports_when_no_onset_has_a_complete_following_body() {
        let mut onsets = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
        onsets[0] = 790;
        let plan = derive_w30_resample_low_impact(
            &[0.2; 800],
            8_000.0,
            1,
            onsets,
            [160; W30_RESAMPLE_HARD_SLICE_COUNT],
        );

        assert_eq!(plan.recipe, W30ResampleLowImpactRecipe::Unavailable);
        assert_eq!(
            plan.decision,
            W30ResampleLowImpactDecision::NoCompleteCandidateWindow
        );
        assert_eq!(plan.candidate_count, 0);
    }

    #[test]
    #[ignore = "requires the local ignored RIOTBOX-1423 development corpus"]
    fn local_development_matrix_reports_cross_family_selection() {
        let cases = [
            (
                "golden_path_dense_break",
                "data/test_audio/examples/Beat03_130BPM(Full).wav",
            ),
            (
                "dense_full_mix",
                "data/test_audio/external/RIOTBOX-1423/wav/stress_oga_bertsz_dnb.wav",
            ),
            (
                "dense_break",
                "data/test_audio/external/RIOTBOX-1423/wav/dense_oga_cinameng_can_be_so_beautiful.wav",
            ),
            (
                "sparse_drums",
                "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_marwan_cinematic_percussion.wav",
            ),
            (
                "tonal_riff",
                "data/test_audio/external/RIOTBOX-1423/wav/tonal_oga_fupi_plimplom.wav",
            ),
            (
                "pad_noise",
                "data/test_audio/external/RIOTBOX-1423/wav/pad_oga_isaiah658_ambient.wav",
            ),
            (
                "weak_source",
                "data/test_audio/external/RIOTBOX-1423/wav/weak_oga_killerfishred_short_synth.wav",
            ),
            (
                "bad_timing",
                "data/test_audio/external/RIOTBOX-1423/wav/bad_timing_oga_laleksic_tap_water.wav",
            ),
        ];
        let mut selected_families = Vec::new();
        let mut selection_signatures = std::collections::BTreeSet::new();
        for (family, path) in cases {
            let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path);
            let source =
                SourceAudioCache::load_pcm_wav(source_path).expect("load local development WAV");
            let projection = project_resample_source_from_interleaved(
                source.interleaved_samples(),
                usize::from(source.channel_count),
                source.sample_rate,
                1.0,
                1.0,
            )
            .expect("project development source");
            let repeated = project_resample_source_from_interleaved(
                source.interleaved_samples(),
                usize::from(source.channel_count),
                source.sample_rate,
                1.0,
                1.0,
            )
            .expect("repeat development source projection");
            assert_eq!(projection.hard_policy, repeated.hard_policy);
            assert_eq!(projection.hard_low_impact, repeated.hard_low_impact);
            let plan = projection.hard_low_impact;
            eprintln!(
                "{family}: policy={} recipe={} role={} decision={} candidates={} slot={} onset={} attack={} body={} share={:.6} over_body={:.6} over_source={:.6}",
                projection.hard_policy.label(),
                plan.recipe.label(),
                plan.role.label(),
                plan.decision.label(),
                plan.candidate_count,
                plan.selected_slot,
                plan.selected_onset_cursor,
                plan.attack_window_proxy_frames,
                plan.body_window_proxy_frames,
                plan.low_band_attack_share,
                plan.low_band_attack_over_body,
                plan.low_band_attack_over_source,
            );
            selection_signatures.insert((
                projection.hard_policy.label(),
                plan.decision.label(),
                plan.selected_slot,
                plan.selected_onset_cursor,
                plan.attack_window_proxy_frames,
                plan.body_window_proxy_frames,
            ));
            if plan.recipe == W30ResampleLowImpactRecipe::SourceHitShaperV3 {
                selected_families.push(family);
            }
        }
        assert!(
            selected_families.contains(&"golden_path_dense_break")
                && selected_families.contains(&"dense_full_mix"),
            "existing exact development sources must stay reachable: {selected_families:?}"
        );
        assert!(
            selected_families.contains(&"sparse_drums")
                && selected_families.contains(&"tonal_riff"),
            "two independent development families must newly reach the exact recipe: {selected_families:?}"
        );
        assert!(
            selection_signatures.len() >= 5,
            "cross-source selection must not collapse: {selection_signatures:?}"
        );
    }
}
