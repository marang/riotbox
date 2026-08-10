#!/usr/bin/env python3
"""Fail-closed metadata-only validator for RIOTBOX-1428 Stage A.

This program intentionally reads five named JSON contracts and nothing else. It
does not enumerate directories, resolve source paths, hash WAVs, or inspect any
audio. Source and holdout identities are compared only as manifest metadata.
"""

from __future__ import annotations

import hashlib
import json
import struct
from pathlib import Path
from typing import Any, Iterable


PROTOCOL_REL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v1.json")
MATRIX_REL = Path("docs/benchmarks/percussive_force_development_matrix_v2.json")
MATRIX_V1_REL = Path("docs/benchmarks/percussive_force_development_matrix_v1.json")
REGISTRY_V1_REL = Path("docs/benchmarks/source_holdout_rotation_v1.json")
REGISTRY_V2_REL = Path("docs/benchmarks/source_holdout_rotation_v2.json")

EXPECTED_PROTOCOL_SHA256 = "35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840"
EXPECTED_MATRIX_V1_SHA256 = "3290011471bb1ae0fc66e54c8bb4e1382f82ceee6266245a44755d8f62f1f970"
EXPECTED_MATRIX_V2_SHA256 = "aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6"
EXPECTED_REGISTRY_V1_SHA256 = "dd017080f311dcb2a8eda2fac63d8da372a356f0fc2cc33d5c97d3fd2ea34cfc"
EXPECTED_REGISTRY_V2_SHA256 = "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812"
EXPECTED_PROTOCOL_SEMANTIC_SHA256 = "7681ab68a9fe2261c97b7499e298e9e6dcb7cbe60df64355dceff577d7fc0848"
EXPECTED_MATRIX_V2_SEMANTIC_SHA256 = "57edb217b8dd17166826274d96ef091bd6fd2a88a9688e37b3ef0b7a6d27e94b"
CHANGE_RULE = "stage_a_protocol_v2_plus_relevant_component_bump_plus_decision_log_before_recompute"

EXPECTED_COMPONENT_VERSIONS = {
    "prequalification": "riotbox.percussive_force_prequalification.v2",
    "impact_role": "riotbox.impact_role.v1",
    "event_detector": "riotbox.percussive_event_detector.v1",
    "event_anatomy": "riotbox.percussive_event_anatomy.v1",
    "source_contrast": "riotbox.percussive_source_contrast.v1",
    "rhythmic_location_proxy": "riotbox.rhythmic_location_proxy.v1",
    "event_ordinal_policy": "riotbox.event_ordinal_policy.v1",
    "exact_three_band_analysis": "riotbox.exact_complementary_three_band_analysis.v1",
    "f1": "f1_ab_energy_redistribution_v1",
    "f2": "f2_exact_complementary_three_band_v1",
    "f3": "f3_causal_envelope_contrast_dynamic_residual_v2",
    "f3_source_response_diversity": "riotbox.f3_source_response_diversity.v1",
    "level_matcher": "event_train_rms_attenuation_match_v1",
    "blind_order": "riotbox.percussive_force_blind.v1",
}

# These are the preregistration's scientific and operational values. The exact
# canonical-byte hash protects the remaining passports and rationale prose.
EXPECTED_VALUES = {
    "input_channel_counts": [1, 2],
    "input_sample_rate_range_hz": [32000, 192000],
    "analysis_epsilon_lsb_squared": 1,
    "minimum_signal_peak_lsb": 16,
    "frame_rounding_offset": 0.5,
    "periodic_hann_coefficients": [0.5, -0.5],
    "detector_window_ms": 8,
    "detector_hop_ms": 1,
    "detector_log_energy_lag_ms": 4,
    "detector_log_energy_lag_hops": 4,
    "detector_median_hops": 3,
    "detector_baseline_radius_ms": 500,
    "detector_baseline_exclusion_ms": 25,
    "detector_minimum_baseline_hops": 200,
    "mad_consistency_scale": 1.4826,
    "detector_mad_multiplier": 3,
    "detector_zero_mad_delta": 0.000001,
    "detector_coarse_rms_floor_ratio": 4,
    "detector_local_rms_percentile": 20,
    "detector_peak_search_radius_ms": 6,
    "detector_nms_ms": 30,
    "analysis_band_edges_hz": [20, 160, 2500, 12000],
    "rms_envelope_windows_ms": [1, 8, 20],
    "anchor_search_ms": [-20, 5],
    "lookbehind_ms": 20,
    "onset_fraction_above_baseline": 0.1,
    "onset_persistence_ms": 1,
    "anatomy_peak_baseline_ratio": 4,
    "lookbehind_peak_ratio_max": 0.25,
    "attack_peak_search_ms": 40,
    "attack_turnover_fraction": 0.9,
    "attack_turnover_persistence_ms": 3,
    "body_baseline_multiplier": 2,
    "body_peak_fraction": 0.1,
    "body_minimum_ms": 20,
    "body_below_floor_ms": 10,
    "body_maximum_ms": 250,
    "tail_baseline_multiplier": 1.5,
    "tail_peak_fraction": 0.05,
    "tail_minimum_ms": 10,
    "tail_below_floor_ms": 20,
    "tail_maximum_ms": 500,
    "diagnostic_window_edges_ms": [0, 10, 40, 120, 250],
    "composite_fusion_ms": 12,
    "event_valley_peak_fraction": 0.5,
    "event_valley_persistence_ms": 2,
    "rhythmic_proxy_window_ms": 60,
    "rhythmic_proxy_quantile": 0.5,
    "candidate_onset_tolerance_ms": 1.0,
    "candidate_rhythmic_proxy_tolerance_ms": 5.0,
    "source_welch_window_ms": 32,
    "source_welch_hop_ms": 16,
    "source_minimum_onsets": 3,
    "source_minimum_resolved_body_events": 2,
    "normalization_density_scale_per_second": 4,
    "normalization_ioi_scale_ms": 250,
    "normalization_ioi_cv_scale": 1,
    "normalization_duration_scale_ms": 250,
    "source_distinct_distance_min": 0.25,
    "source_changed_domain_min_delta": 0.1,
    "source_changed_domain_minimum_count": 2,
    "positive_source_count": 4,
    "positive_author_count": 4,
    "positive_family_count": 3,
    "source_distance_domain_count": 5,
    "minimum_source_clusters": 3,
    "four_source_partition_count": 15,
    "valid_source_partition_count": 1,
    "minimum_events_per_source": 2,
    "maximum_frozen_events_per_source": 3,
    "development_event_ordinals": [1, 2],
    "confirmation_event_ordinal": 3,
    "golden_event_ordinal": 1,
    "mask_crossfade_divisor": 8,
    "mask_crossfade_endpoint_offset": 1,
    "f1_masked_energy_allocation_multiplier": 2,
    "f1_body_energy_retention_min": 0.5,
    "floating_comparison_epsilon_multiplier": 64,
    "f2_minimum_split_separation_bins": 2,
    "f2_attack_spectrum_quantiles": [0.25, 0.75],
    "f2_band_trust_lookbehind_ratio": 4,
    "f2_required_trusted_bands": 2,
    "f2_preflight_sample_rates_hz": [44100, 48000, 96000],
    "f2_preflight_frame_count": 1024,
    "f2_preflight_impulse": {"frame": 256, "amplitude": 0.8},
    "f2_preflight_dc_amplitude": 0.21,
    "f2_preflight_noise_lcg": {
        "seed_u64": "7",
        "multiplier_u64": "6364136223846793005",
        "addend_u64": "1",
        "output": "upper_32_bits_divided_by_u32_max_mapped_to_minus1_plus1",
        "gain": 0.35,
    },
    "f2_preflight_bounded_tones": [
        {"amplitude": 0.31, "frequency_hz": 430},
        {"amplitude": 0.17, "frequency_hz": 3700},
    ],
    "f2_preflight_cutoffs_hz": [900, 5500],
    "f2_quantile_golden": {
        "sample_rate_hz": 48000,
        "frame_count": 128,
        "tone_bins": [8, 24],
        "tone_amplitude_each": 0.25,
        "expected_f25_bin": 8,
        "expected_f75_bin": 24,
    },
    "f2_reconstruction_rms_tolerance": 0.000001,
    "f3_v2_attack_up_ms": 1,
    "f3_v2_attack_down_ms": 8,
    "f3_v2_body_up_ms": 8,
    "f3_v2_body_down_ms": 20,
    "f3_v2_branch_scale": 1,
    "f3_v2_output_factor_range": [1, 2],
    "f3_v2_branch_contribution_min": 0.05,
    "f3_v2_preflight_sample_rates_hz": [44100, 48000, 96000],
    "f3_v2_preflight_alignment_block_frames": 64,
    "f3_v2_preflight_duration_ms": 96,
    "f3_v2_preflight_onset_ms": 24,
    "f3_v2_preflight_high_duration_ms": 4,
    "f3_v2_preflight_attack_duration_ms": 8,
    "f3_v2_preflight_body_duration_ms": 48,
    "f3_v2_preflight_constant_amplitude": 0.25,
    "f3_v2_preflight_step_amplitudes": [0.03125, 0.375, 0.1875, 0.03125],
    "f3_v2_preflight_low_carrier_cycles_per_sample": 0.015625,
    "f3_v2_preflight_high_carrier_cycles_per_sample": 0.125,
    "f3_v2_preflight_amplitude_scale": 0.5,
    "f3_v2_preflight_pcm_valid_bits": 16,
    "f3_v2_preflight_input_lsb": 0.000030517578125,
    "f3_source_response_preflight_q_goldens": {
        "44100": [2, 9, 3, 8, 4, 8, 2, 4],
        "48000": [2, 9, 3, 7, 4, 8, 2, 3],
        "96000": [2, 9, 3, 7, 4, 8, 2, 3],
    },
    "control_gain_db": 3,
    "control_global_rate_ratio": [4, 5],
    "control_delay_ms": 25,
    "control_duplicate_gain_db": -6,
    "control_body_attenuation_db": -12,
    "control_distortion_drive": 2,
    "matcher_ceiling_dbfs": -1,
    "output_peak_absolute_max": 1.0,
    "train_event_maximum_ms": 500,
    "train_duration_ms": 2300,
    "train_event_count": 4,
    "train_onsets_ms": [50, 600, 1150, 1700],
    "train_event_complete_by_ms": 2200,
    "train_terminal_silence_ms": 100,
    "pair_inter_train_silence_ms": 250,
    "pair_duration_ms": 4850,
    "control_train_event_maximum_ms": 650,
    "control_train_duration_ms": 2900,
    "control_train_onsets_ms": [50, 750, 1450, 2150],
    "control_train_event_complete_by_ms": 2800,
    "control_train_terminal_silence_ms": 100,
    "control_pair_duration_ms": 6050,
    "near_identity_delta_rms_min": 0.05,
    "identity_correlation_min": 0.8,
    "attack_spectral_band_count": 24,
    "attack_spectral_cosine_min": 0.75,
    "body_energy_ratio_range": [0.5, 2.0],
    "gain_only_fit_residual_min": 0.01,
    "static_eq_fit_residual_min": 0.03,
    "static_distortion_fit_residual_min": 0.03,
    "f3_source_response_field_count": 8,
    "f3_source_response_quantized_max": 20,
    "f3_source_response_minimum_quantized_component_distance": 2,
    "minimum_actionable_diversity_hashes_per_family": 2,
    "minimum_clusters_with_distinct_actionable_identity_sets": 2,
    "boundary_discontinuity_neighborhood_ms": 1,
    "maximum_review_ready_generations_per_family": 2,
    "maximum_structural_revisions_per_family": 1,
}

EXPECTED_F3_V1_RECORD = {
    "record_schema": "riotbox.percussive_force_synthetic_preflight_record.v1",
    "family_version": "f3_os4_onset_residual_v1",
    "probe_id": "three_tone_0071_0193_0317_v1",
    "input": "mono_f64",
    "frame_count": 4096,
    "formula": "x[n]=sum_components(amplitude*sin(2*pi*cycles_per_base_sample*n+phase_radians))",
    "components": [
        {"amplitude": 0.44, "cycles_per_base_sample": 0.071, "phase_radians": 0.0},
        {"amplitude": 0.29, "cycles_per_base_sample": 0.193, "phase_radians": 0.31},
        {"amplitude": 0.17, "cycles_per_base_sample": 0.317, "phase_radians": 0.79},
    ],
    "input_processing": "no_normalization_no_envelope_no_dither",
    "drive": 3.0,
    "path_peak_rule": "Each 4x and 8x path independently uses P=max(abs(interpolated_x)) in the same phi formula.",
    "alignment": "Both filtered and decimated residuals use the exact 64-base-frame total delay.",
    "comparison_interval_frames": [512, 3584],
    "comparison_interval_semantics": "half_open",
    "metric": "10*log10(sum((r4-r8)^2)/sum(r8^2))_without_epsilon_or_floor",
    "rejected_family_definition": {
        "candidate_oversampling_factor": 4,
        "candidate_filter_taps": 257,
        "candidate_filter_midpoint": 128,
        "oracle_oversampling_factor": 8,
        "oracle_filter_taps": 513,
        "oracle_filter_midpoint": 256,
        "cutoff_cycles_per_high_rate_sample": "0.45/L",
        "blackman_coefficients": [0.42, -0.5, 0.08],
        "window_denominators": [256, 512],
        "base_rate_total_delay_frames": 64,
        "drive": 3.0,
    },
    "required_error_db_max": -60.0,
    "measured_error_db": -43.38236728008191,
    "pass": False,
    "immutable": True,
    "source_audio_accessed": False,
    "consequence": "Reject and freeze f3_os4_onset_residual_v1 before any source use; do not retune any value. A replacement requires a separately versioned structural family.",
}

EXPECTED_PASSPORT_METADATA = {'input_channel_counts': ('channels', 'registered_pcm_input', 'exact_supported_set'),
 'input_sample_rate_range_hz': ('hertz_inclusive', 'registered_pcm_input', 'min_max_supported_native_rate'),
 'analysis_epsilon_lsb_squared': ('input_lsb_squared',
                                  'all_log_and_energy_denominators',
                                  'epsilon_equals_one_input_lsb_squared'),
 'minimum_signal_peak_lsb': ('input_lsb',
                             'source_event_and_band_signal_guards',
                             'strict_minimum_peak_or_mean_square_reference'),
 'frame_rounding_offset': ('sample_frame',
                           'all_millisecond_to_frame_conversions',
                           'positive_duration_and_signed_offset_nearest_frame_rules'),
 'periodic_hann_coefficients': ('dimensionless', 'detector_STFT_and_source_Welch', 'w[n]=a0+a1*cos(2*pi*n/N)'),
 'detector_window_ms': ('milliseconds', 'detector_periodic_hann', 'analysis_window_length'),
 'detector_hop_ms': ('milliseconds', 'detector', 'hop_length'),
 'detector_log_energy_lag_ms': ('milliseconds', 'bandwise_novelty', 'previous_hop_lag'),
 'detector_log_energy_lag_hops': ('detector_hops', 'bandwise_novelty', 'exact_previous_valid_hop_index_delta'),
 'detector_median_hops': ('hops', 'novelty_smoothing', 'centered_median_width'),
 'detector_baseline_radius_ms': ('milliseconds', 'novelty_baseline', 'symmetric_radius'),
 'detector_baseline_exclusion_ms': ('milliseconds', 'novelty_baseline', 'symmetric_anchor_exclusion'),
 'detector_minimum_baseline_hops': ('valid_hops', 'novelty_baseline', 'greater_than_or_equal'),
 'mad_consistency_scale': ('dimensionless',
                           'detector_and_anatomy_robust_baselines',
                           'mad_to_normal_consistency_multiplier'),
 'detector_mad_multiplier': ('scaled_mad', 'novelty_threshold', 'strictly_above_median_plus_value_times_scaled_mad'),
 'detector_zero_mad_delta': ('log_energy_novelty', 'zero_mad_novelty_threshold', 'strict_additional_delta'),
 'detector_coarse_rms_floor_ratio': ('ratio',
                                     'coarse_broadband_guard',
                                     'rms_greater_than_or_equal_local_20th_percentile_times_value'),
 'detector_local_rms_percentile': ('percent', 'coarse_broadband_guard', 'local_reference_percentile'),
 'detector_peak_search_radius_ms': ('milliseconds', 'coarse_peak', 'earliest_plateau_local_max_radius'),
 'detector_nms_ms': ('milliseconds',
                     'coarse_peak_nonmaximum_suppression',
                     'keep_larger_novelty_then_earlier_on_equality'),
 'analysis_band_edges_hz': ('hertz',
                            'detector_source_vector_and_band_metrics',
                            'three_half_open_bands_with_final_inclusive_edge'),
 'rms_envelope_windows_ms': ('milliseconds', 'forward_tagged_anatomy_envelopes', 'exact_window_set'),
 'anchor_search_ms': ('milliseconds_relative_to_coarse_anchor', 'physical_onset_search', 'inclusive_search_bounds'),
 'lookbehind_ms': ('milliseconds', 'event_anatomy', 'frozen_pre_onset_context'),
 'onset_fraction_above_baseline': ('fraction_of_peak_minus_baseline', 'physical_onset', 'earliest_r1_crossing'),
 'onset_persistence_ms': ('milliseconds', 'physical_onset', 'minimum_continuous_crossing'),
 'anatomy_peak_baseline_ratio': ('ratio', 'physical_onset', 'peak_greater_than_or_equal_baseline_times_value'),
 'lookbehind_peak_ratio_max': ('ratio', 'lookbehind_mask_guard', 'lookbehind_rms_over_peak_less_than_or_equal'),
 'attack_peak_search_ms': ('milliseconds', 'attack_peak_and_turnover', 'maximum_time_after_onset'),
 'attack_turnover_fraction': ('fraction_of_attack_peak', 'attack_end', 'r1_less_than_or_equal_for_persistence'),
 'attack_turnover_persistence_ms': ('milliseconds', 'attack_end', 'continuous_below_threshold'),
 'body_baseline_multiplier': ('ratio', 'body_floor', 'max_component'),
 'body_peak_fraction': ('fraction_of_attack_peak', 'body_floor', 'max_component'),
 'body_minimum_ms': ('milliseconds', 'body_end', 'minimum_duration_before_floor_search'),
 'body_below_floor_ms': ('milliseconds', 'body_end', 'continuous_r8_below_floor'),
 'body_maximum_ms': ('milliseconds_after_physical_onset',
                     'body_resolution',
                     'body_end_less_than_or_equal_onset_plus_value'),
 'tail_baseline_multiplier': ('ratio', 'tail_floor', 'max_component'),
 'tail_peak_fraction': ('fraction_of_attack_peak', 'tail_floor', 'max_component'),
 'tail_minimum_ms': ('milliseconds', 'tail_end', 'minimum_duration_before_floor_search'),
 'tail_below_floor_ms': ('milliseconds', 'tail_end', 'continuous_r20_below_floor'),
 'tail_maximum_ms': ('milliseconds_after_physical_onset',
                     'tail_resolution',
                     'tail_end_less_than_or_equal_onset_plus_value'),
 'diagnostic_window_edges_ms': ('milliseconds_after_onset',
                                'fixed_diagnostic_windows',
                                'half_open_edges_clipped_to_event'),
 'composite_fusion_ms': ('milliseconds',
                         'source_micropeak_classification',
                         'refined_onsets_within_or_equal_form_one_cluster'),
 'event_valley_peak_fraction': ('fraction_of_smaller_adjacent_peak', 'separate_event_test', 'r1_less_than_or_equal'),
 'event_valley_persistence_ms': ('milliseconds', 'separate_event_test', 'continuous_valley_duration'),
 'rhythmic_proxy_window_ms': ('milliseconds', 'rhythmic_location_proxy', 'source_frozen_onset_to_min_body_end_window'),
 'rhythmic_proxy_quantile': ('cumulative_weight_fraction', 'rhythmic_location_proxy', 'first_cumulative_crossing'),
 'candidate_onset_tolerance_ms': ('milliseconds',
                                  'all_candidate_integrity_screens',
                                  'absolute_movement_less_than_or_equal'),
 'candidate_rhythmic_proxy_tolerance_ms': ('milliseconds',
                                           'all_candidate_integrity_screens',
                                           'absolute_movement_less_than_or_equal'),
 'source_welch_window_ms': ('milliseconds', 'whole_source_spectrum', 'periodic_hann_window'),
 'source_welch_hop_ms': ('milliseconds', 'whole_source_spectrum', 'half_window_hop'),
 'source_minimum_onsets': ('detected_onsets', 'source_vector', 'greater_than_or_equal'),
 'source_minimum_resolved_body_events': ('events', 'source_vector', 'greater_than_or_equal'),
 'normalization_density_scale_per_second': ('onsets_per_second', 'source_vector', 'P_scale'),
 'normalization_ioi_scale_ms': ('milliseconds', 'source_vector', 'one_minus_P_scale'),
 'normalization_ioi_cv_scale': ('population_cv', 'source_vector', 'P_scale'),
 'normalization_duration_scale_ms': ('milliseconds', 'source_vector', 'P_scale'),
 'source_distinct_distance_min': ('five_domain_rms_distance', 'source_contrast', 'greater_than_or_equal'),
 'source_changed_domain_min_delta': ('normalized_domain_distance', 'source_contrast', 'greater_than_or_equal'),
 'source_changed_domain_minimum_count': ('domains', 'source_contrast', 'greater_than_or_equal'),
 'positive_source_count': ('registered_sources', 'isolated_stage_a', 'exact'),
 'positive_author_count': ('distinct_authors', 'isolated_stage_a', 'exact'),
 'positive_family_count': ('registered_percussive_families', 'isolated_stage_a', 'exact'),
 'source_distance_domain_count': ('normalized_domains', 'source_contrast', 'exact_mean_denominator'),
 'minimum_source_clusters': ('clusters', 'source_contrast', 'greater_than_or_equal'),
 'four_source_partition_count': ('set_partitions', 'source_contrast', 'enumerate_all_and_require_exactly_one_valid'),
 'valid_source_partition_count': ('partitions', 'source_contrast', 'exact'),
 'minimum_events_per_source': ('eligible_events', 'event_catalog', 'greater_than_or_equal'),
 'maximum_frozen_events_per_source': ('events', 'event_catalog', 'less_than_or_equal'),
 'development_event_ordinals': ('mechanism_blind_event_ordinals', 'development_cross_product', 'exact_partition'),
 'confirmation_event_ordinal': ('mechanism_blind_event_ordinal', 'within_source_confirmation', 'exact_when_present'),
 'golden_event_ordinal': ('mechanism_blind_event_ordinal', 'preregistered_golden_path', 'exact_if_eligible'),
 'mask_crossfade_divisor': ('region_fraction_denominator',
                            'attack_body_masks',
                            'round_min_region_length_divided_by_value'),
 'mask_crossfade_endpoint_offset': ('frame_index_offset', 'attack_body_masks', 'theta_index_and_denominator_guard'),
 'f1_masked_energy_allocation_multiplier': ('dimensionless_K',
                                            'F1_masked_resolver',
                                            'energy_allocation_multiplier_not_measured_ratio_target'),
 'f1_body_energy_retention_min': ('source_body_energy_ratio', 'F1', 'greater_than_or_equal'),
 'floating_comparison_epsilon_multiplier': ('machine_epsilon_multiples',
                                            'F1_ratio_and_F2_max_reconstruction',
                                            'numerical_only_tolerance'),
 'f2_minimum_split_separation_bins': ('dft_bins', 'F2', 'greater_than_or_equal'),
 'f2_attack_spectrum_quantiles': ('cumulative_attack_power_fraction', 'F2', 'first_crossing_f25_and_f75'),
 'f2_band_trust_lookbehind_ratio': ('mean_square_ratio', 'F2', 'attack_and_body_strictly_above_lookbehind_times_value'),
 'f2_required_trusted_bands': ('bands', 'F2', 'greater_than_or_equal'),
 'f2_preflight_sample_rates_hz': ('hertz', 'F2_filter_bank_preflight', 'exact_test_set'),
 'f2_preflight_frame_count': ('mono_frames', 'F2_filter_bank_preflight', 'exact_for_each_signal_and_rate'),
 'f2_preflight_impulse': ('frame_and_normalized_sample', 'F2_filter_bank_preflight', 'exact_impulse'),
 'f2_preflight_dc_amplitude': ('normalized_sample', 'F2_filter_bank_preflight', 'exact_constant_sequence'),
 'f2_preflight_noise_lcg': ('unsigned_wrapping_LCG_and_normalized_sample',
                            'F2_filter_bank_preflight',
                            'exact_deterministic_noise'),
 'f2_preflight_bounded_tones': ('normalized_sample_and_hertz', 'F2_filter_bank_preflight', 'sum_of_zero_phase_sines'),
 'f2_preflight_cutoffs_hz': ('hertz', 'F2_reconstruction_only_preflight', 'exact_low_and_mid_split'),
 'f2_quantile_golden': ('mono_native_rate_frames_DFT_bins_normalized_sample',
                        'F2_split_estimator_preflight',
                        'exact_known_bin_periodic_Hann_golden'),
 'f2_reconstruction_rms_tolerance': ('normalized_rms',
                                     'F2_filter_bank_preflight_and_internal_event_reconstruction',
                                     'less_than_or_equal'),
 'f3_v2_attack_up_ms': ('milliseconds_63_2_percent_time_constant',
                        'F3_v2_attack_controller',
                        'exact_rising_smoother_tau'),
 'f3_v2_attack_down_ms': ('milliseconds_63_2_percent_time_constant',
                          'F3_v2_attack_controller',
                          'exact_falling_smoother_tau'),
 'f3_v2_body_up_ms': ('milliseconds_63_2_percent_time_constant', 'F3_v2_body_controller', 'exact_rising_smoother_tau'),
 'f3_v2_body_down_ms': ('milliseconds_63_2_percent_time_constant',
                        'F3_v2_body_controller',
                        'exact_falling_smoother_tau'),
 'f3_v2_branch_scale': ('linear_gain', 'F3_v2_both_residual_branches', 'exact_no_target_no_caller_knob'),
 'f3_v2_output_factor_range': ('multiplicative_sample_factor', 'F3_v2_affected_support', 'inclusive_range'),
 'f3_v2_branch_contribution_min': ('weighted_delta_rms_over_weighted_source_rms',
                                   'F3_v2_each_branch',
                                   'greater_than_or_equal'),
 'f3_v2_preflight_sample_rates_hz': ('hertz', 'F3_v2_synthetic_preflight', 'exact_test_set'),
 'f3_v2_preflight_alignment_block_frames': ('frames', 'F3_v2_synthetic_preflight', 'M64_rounding_multiple'),
 'f3_v2_preflight_duration_ms': ('milliseconds_before_M64_rounding', 'F3_v2_synthetic_preflight', 'exact_duration'),
 'f3_v2_preflight_onset_ms': ('milliseconds_before_M64_rounding', 'F3_v2_synthetic_preflight', 'exact_onset'),
 'f3_v2_preflight_high_duration_ms': ('milliseconds_before_M64_rounding',
                                      'F3_v2_step_body_probe',
                                      'exact_high_plateau_duration'),
 'f3_v2_preflight_attack_duration_ms': ('milliseconds_before_M64_rounding',
                                        'F3_v2_synthetic_preflight',
                                        'exact_attack_region_duration'),
 'f3_v2_preflight_body_duration_ms': ('milliseconds_before_M64_rounding',
                                      'F3_v2_synthetic_preflight',
                                      'exact_onset_relative_body_end'),
 'f3_v2_preflight_constant_amplitude': ('normalized_sample', 'F3_v2_constant_quadrature_probe', 'exact'),
 'f3_v2_preflight_step_amplitudes': ('normalized_sample_pre_high_body_post',
                                     'F3_v2_step_body_probe',
                                     'exact_piecewise_amplitudes'),
 'f3_v2_preflight_low_carrier_cycles_per_sample': ('cycles_per_sample', 'F3_v2_quadrature_probes', 'exact_one_over_64'),
 'f3_v2_preflight_high_carrier_cycles_per_sample': ('cycles_per_sample',
                                                    'F3_v2_carrier_invariance_probe',
                                                    'exact_one_over_8'),
 'f3_v2_preflight_amplitude_scale': ('linear_amplitude_ratio', 'F3_v2_scale_equivariance_probe', 'exact'),
 'f3_v2_preflight_pcm_valid_bits': ('signed_PCM_valid_bits',
                                    'F3_v2_synthetic_preflight',
                                    'input_lsb_equals_two_power_minus_15'),
 'f3_v2_preflight_input_lsb': ('normalized_sample', 'F3_v2_synthetic_preflight', 'exact_two_power_minus_15'),
 'f3_source_response_preflight_q_goldens': ('ordered_u8_controller_response_codes',
                                            'F3_v2_source_response_preflight',
                                            'exact_within_rate_golden_and_cross_rate_nondiversity_evidence'),
 'control_gain_db': ('decibels', 'gain_and_brightness_controls', 'linear_gain_10_power_db_over_20'),
 'control_global_rate_ratio': ('exact_rational_output_over_input_rate', 'global_rate_control', 'exact_four_fifths'),
 'control_delay_ms': ('milliseconds', 'delayed_duplicate_control', 'exact_delay'),
 'control_duplicate_gain_db': ('decibels', 'delayed_duplicate_control', 'duplicate_linear_gain'),
 'control_body_attenuation_db': ('decibels', 'detached_click_control', 'body_and_tail_linear_gain'),
 'control_distortion_drive': ('dimensionless', 'distortion_only_control', 'exact_tanh_drive'),
 'matcher_ceiling_dbfs': ('decibels_full_scale', 'event_train_matcher', 'common_attenuation_ceiling'),
 'output_peak_absolute_max': ('normalized_full_scale',
                              'all_candidate_and_control_output',
                              'absolute_peak_strictly_less_than'),
 'train_event_maximum_ms': ('milliseconds', 'review_train', 'less_than_or_equal'),
 'train_duration_ms': ('milliseconds', 'review_train', 'exact'),
 'train_event_count': ('event_instances', 'review_train', 'exact'),
 'train_onsets_ms': ('milliseconds', 'review_train', 'exact'),
 'train_event_complete_by_ms': ('milliseconds', 'review_train', 'less_than_or_equal'),
 'train_terminal_silence_ms': ('milliseconds', 'review_train', 'greater_than_or_equal_minimum'),
 'pair_inter_train_silence_ms': ('milliseconds', 'A_B_review_pair', 'exact'),
 'pair_duration_ms': ('milliseconds', 'A_B_review_pair', 'exact'),
 'control_train_event_maximum_ms': ('milliseconds', 'false_positive_control_review_train', 'less_than_or_equal'),
 'control_train_duration_ms': ('milliseconds', 'false_positive_control_review_train', 'exact'),
 'control_train_onsets_ms': ('milliseconds', 'false_positive_control_review_train', 'exact'),
 'control_train_event_complete_by_ms': ('milliseconds', 'false_positive_control_review_train', 'less_than_or_equal'),
 'control_train_terminal_silence_ms': ('milliseconds',
                                       'false_positive_control_review_train',
                                       'greater_than_or_equal_minimum'),
 'control_pair_duration_ms': ('milliseconds', 'false_positive_control_A_B_pair', 'exact'),
 'near_identity_delta_rms_min': ('dimensionless_ratio_or_unit_interval_controller_resolution',
                                 'candidate_reject_screen_and_F3_source_response_quantization',
                                 'greater_than_or_equal_candidate_delta_and_exact_F3_quantization_divisor'),
 'identity_correlation_min': ('normalized_waveform_correlation', 'candidate_reject_screen', 'greater_than_or_equal'),
 'attack_spectral_band_count': ('log_frequency_bands', 'attack_identity_screen', 'exact'),
 'attack_spectral_cosine_min': ('cosine_similarity', 'attack_identity_screen', 'greater_than_or_equal'),
 'body_energy_ratio_range': ('candidate_over_source_energy', 'raw_and_matched_body_screen', 'inclusive_range'),
 'gain_only_fit_residual_min': ('normalized_rms_residual', 'confound_reject_screen', 'greater_than_or_equal'),
 'static_eq_fit_residual_min': ('normalized_rms_residual', 'confound_reject_screen', 'greater_than_or_equal'),
 'static_distortion_fit_residual_min': ('normalized_rms_residual', 'confound_reject_screen', 'greater_than_or_equal'),
 'f3_source_response_field_count': ('ordered_controller_summary_fields',
                                    'riotbox.f3_source_response_diversity.v1',
                                    'exact_raw_and_quantized_vector_length'),
 'f3_source_response_quantized_max': ('inclusive_u8_code',
                                      'riotbox.f3_source_response_diversity.v1',
                                      'floor_one_over_near_identity_resolution_plus_one_half'),
 'f3_source_response_minimum_quantized_component_distance': ('quantized_controller_bins',
                                                              'F3_actionable_diversity_pair',
                                                              'maximum_absolute_component_distance_greater_than_or_equal'),
 'minimum_actionable_diversity_hashes_per_family': ('distinct_family_typed_actionable_identity_hashes',
                                                     'complete_four_by_two_development_set_per_family',
                                                     'greater_than_or_equal'),
 'minimum_clusters_with_distinct_actionable_identity_sets': ('feature_clusters',
                                                              'anti_hardcoding',
                                                              'greater_than_or_equal_when_declared_causal_inputs_differ'),
 'boundary_discontinuity_neighborhood_ms': ('milliseconds_per_side',
                                            'processing_boundary_rejector',
                                            'inclusive_local_delta_step_context_excluding_boundary'),
 'maximum_review_ready_generations_per_family': ('generations', 'human_review_stop_rule', 'less_than_or_equal'),
 'maximum_structural_revisions_per_family': ('revisions', 'stage_a_family', 'less_than_or_equal')}

EXPECTED_POSITIVE_SOURCES = {
    "oga_cinameng_can_be_so_beautiful": {
        "source_family": "dense_break",
        "author": "cinameng",
        "source_pack_id": "oga_can_be_so_beautiful",
        "source_path": "data/test_audio/external/RIOTBOX-1423/wav/dense_oga_cinameng_can_be_so_beautiful.wav",
        "sha256": "bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f",
    },
    "oga_marwan_cinematic_percussion": {
        "source_family": "sparse_drums",
        "author": "Marwan Antonios",
        "source_pack_id": "oga_cinematic_percussion_loop",
        "source_path": "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_marwan_cinematic_percussion.wav",
        "sha256": "9373f577cf09135e2b7e3ce0e946ce5af6ea333f5a7462ab9126f6802f9986f3",
    },
    "oga_william_hector_horde_war_drums": {
        "source_family": "dense_break",
        "author": "William Hector",
        "source_pack_id": "oga_horde_war_drums_loop",
        "source_path": "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_william_hector_horde_war_drums.wav",
        "sha256": "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417",
    },
    "oga_frosty_ham_osdrums": {
        "source_family": "electronic_drums",
        "author": "frosty ham",
        "source_pack_id": "oga_drumming",
        "source_path": "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_frosty_ham_osdrums.wav",
        "sha256": "7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2",
    },
}

EXPECTED_HOLDOUTS = {
    (
        "oga_ruok_160bpm",
        "holdout_a",
        "data/test_audio/external/RIOTBOX-1423/wav/dense_oga_ruok_160bpm.wav",
        "2d674ceb618f38076be09a83b4100803d41b74c6dc99c64f6197c37abc3cef2d",
    ),
    (
        "oga_bart_getequipped",
        "holdout_a",
        "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_bart_getequipped.wav",
        "c625239bec045308449f6f6b1787a9233e3884affbfddb34038c38f5a2baef60",
    ),
    (
        "oga_rami99_electronic",
        "holdout_a",
        "data/test_audio/external/RIOTBOX-1423/wav/tonal_oga_rami99_electronic.wav",
        "02d694f652df414a16c15921a6130c329de838705a9e243c035b9696f0f95b52",
    ),
    (
        "oga_tinyworlds_cyberworld",
        "holdout_a",
        "data/test_audio/external/RIOTBOX-1423/wav/pad_oga_tinyworlds_cyberworld.wav",
        "4909849f68ab24b589199e1d3974676dd80689db68165ceaaac625ee682f5b4d",
    ),
    (
        "oga_illin_robotic",
        "holdout_b",
        "data/test_audio/external/RIOTBOX-1423/wav/dense_oga_illin_robotic.wav",
        "626d337e170f67a696f760a8ec82880d5326c364457296b5f7cc47de6a837a4a",
    ),
    (
        "oga_bretbernhoft_beatloops",
        "holdout_b",
        "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_bretbernhoft_beatloops.wav",
        "7e950adad640fee4c763a9aad508fac49a988c1f34c9cbafba56a73d818dcce4",
    ),
    (
        "oga_akikazer_menu",
        "holdout_b",
        "data/test_audio/external/RIOTBOX-1423/wav/tonal_oga_akikazer_menu.wav",
        "41bad327af79ff60b4cd301c69b5c17e202e7fac586e0d4b80b65c255d151b57",
    ),
    (
        "oga_srg774_airy",
        "holdout_b",
        "data/test_audio/external/RIOTBOX-1423/wav/pad_oga_srg774_airy.wav",
        "cc46ed25aa9d86c8812077c5728460f406800781ac8efbc764c05c829e3d134e",
    ),
    (
        "oga_laleksic_grind_metal",
        "holdout_b",
        "data/test_audio/external/RIOTBOX-1423/wav/bad_timing_oga_laleksic_grind_metal.wav",
        "900f077fb4a86c72a1e1e52ec7a8efbccc2500d796a5d66302b73755ecdc427d",
    ),
}

RESULT_KEYS = {
    "actual",
    "candidate",
    "computed",
    "event_records",
    "feature_results",
    "gate_result",
    "human_verdict",
    "measurement",
    "policy_results",
    "qualified",
    "render",
    "result",
    "survivor",
    "verdict",
}


class ContractError(ValueError):
    """Raised when the preregistration fails closed."""


def _fail(path: str, message: str) -> None:
    raise ContractError(f"{path}: {message}")


def _strict_equal(actual: Any, expected: Any) -> bool:
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(
            _strict_equal(actual[key], expected[key]) for key in expected
        )
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(
            _strict_equal(a, e) for a, e in zip(actual, expected, strict=True)
        )
    return actual == expected


def _expect(path: str, actual: Any, expected: Any) -> None:
    if not _strict_equal(actual, expected):
        _fail(path, f"expected {expected!r}, got {actual!r}")


def _contains(path: str, actual: Any, token: str) -> None:
    if not isinstance(actual, str) or token not in actual:
        _fail(path, f"must contain {token!r}")


def _mapping(path: str, value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        _fail(path, "must be an object")
    return value


def _list(path: str, value: Any) -> list[Any]:
    if not isinstance(value, list):
        _fail(path, "must be an array")
    return value


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _f3_source_response_digest(q_values: list[int]) -> str:
    domain = b"riotbox.f3_source_response_diversity.v1"
    family_id = b"f3_causal_envelope_contrast_dynamic_residual_v2"
    payload = (
        len(domain).to_bytes(4, "big")
        + domain
        + len(family_id).to_bytes(4, "big")
        + family_id
        + struct.pack(">d", 1.0)
        + struct.pack(">d", 1.0)
        + bytes(q_values)
    )
    return _sha256(payload)


def _semantic_sha256(value: Any) -> str:
    try:
        encoded = json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError) as exc:
        _fail("semantic JSON", f"cannot canonicalize: {exc}")
    return _sha256(encoded)


def _reject_duplicate_object_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ContractError(f"JSON duplicate object key: {key!r}")
        result[key] = value
    return result


def _load_named_json(repo_root: Path, relative: Path) -> tuple[dict[str, Any], str]:
    allowed = {PROTOCOL_REL, MATRIX_REL, MATRIX_V1_REL, REGISTRY_V1_REL, REGISTRY_V2_REL}
    if relative not in allowed:
        _fail(str(relative), "validator may read only named JSON contracts")
    path = repo_root / relative
    if path.suffix != ".json" or not path.is_file():
        _fail(str(relative), "named JSON contract is missing or not a regular file")
    raw = path.read_bytes()
    try:
        decoded = json.loads(raw, object_pairs_hook=_reject_duplicate_object_keys)
    except ContractError as exc:
        _fail(str(relative), str(exc))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        _fail(str(relative), f"invalid JSON: {exc}")
    return _mapping(str(relative), decoded), _sha256(raw)


def _ensure_no_result_keys(value: Any, path: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in RESULT_KEYS or key.endswith("_result") or key.endswith("_verdict"):
                _fail(f"{path}.{key}", "result/evidence fields are forbidden in preregistration")
            _ensure_no_result_keys(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _ensure_no_result_keys(child, f"{path}[{index}]")


def _validate_train_arithmetic(
    path: str,
    *,
    onsets: list[int],
    event_count: int,
    event_maximum_ms: int,
    event_complete_by_ms: int,
    train_duration_ms: int,
    terminal_silence_minimum_ms: int,
    inter_train_silence_ms: int,
    pair_duration_ms: int,
) -> None:
    _expect(f"{path}.event_count", len(onsets), event_count)
    if onsets != sorted(onsets) or len(set(onsets)) != len(onsets):
        _fail(f"{path}.onsets", "must be strictly increasing and unique")
    if any(b - a < event_maximum_ms for a, b in zip(onsets, onsets[1:])):
        _fail(f"{path}.spacing", "adjacent onsets truncate or overlap the maximum event")
    _expect(f"{path}.event_complete_by", event_complete_by_ms, onsets[-1] + event_maximum_ms)
    if train_duration_ms - event_complete_by_ms < terminal_silence_minimum_ms:
        _fail(f"{path}.terminal_silence", "minimum terminal silence is not preserved")
    _expect(
        f"{path}.pair_duration",
        pair_duration_ms,
        2 * train_duration_ms + inter_train_silence_ms,
    )


def _manifest_holdouts(manifest: dict[str, Any]) -> set[tuple[str, str, str, str]]:
    rows = set()
    for entry in _list("registry.entries", manifest.get("entries")):
        entry = _mapping("registry.entries[]", entry)
        if entry.get("partition") in {"holdout_a", "holdout_b"}:
            rows.add(
                (
                    entry.get("case_id"),
                    entry.get("partition"),
                    entry.get("source_path"),
                    entry.get("sha256"),
                )
            )
    return rows


def validate_protocol(protocol: dict[str, Any], *, canonical_sha256: str | None = None) -> None:
    _ensure_no_result_keys(protocol)
    _expect("protocol.schema", protocol.get("schema"), "riotbox.percussive_force_stage_a_protocol.v1")
    _expect("protocol.schema_version", protocol.get("schema_version"), 1)
    _expect("protocol.owner_ticket", protocol.get("owner_ticket"), "RIOTBOX-1428")
    _expect(
        "protocol.execution_state",
        protocol.get("execution_state"),
        "preregistered_no_source_qualification_or_candidate_render",
    )
    _expect("protocol.quality_proof", protocol.get("quality_proof"), False)
    _expect("protocol.product_path_proof", protocol.get("product_path_proof"), False)
    _contains("protocol.directly_enabled_outcome", protocol.get("directly_enabled_outcome"), "source-backed isolated Stage-A raw and attenuation-matched renders")
    _contains("protocol.directly_enabled_outcome", protocol.get("directly_enabled_outcome"), "bounded blinded human force-direction and desirable-to-trigger comparison")
    _contains("protocol.directly_enabled_outcome", protocol.get("directly_enabled_outcome"), "RuntimeMix and TUI promotion remain Stage-B-only after a unique human pass")
    boundary = _mapping("protocol.stage_boundary", protocol.get("stage_boundary"))
    _expect(
        "protocol.stage_boundary.forbidden_before_this_contract_and_matrix_validate",
        boundary.get("forbidden_before_this_contract_and_matrix_validate"),
        [
            "source_feature_computation",
            "wav_or_pcm_reading",
            "source_event_qualification",
            "candidate_or_control_rendering",
            "generated_audio_artifacts",
            "human_candidate_playback",
            "holdout_audio_access",
        ],
    )
    _expect(
        "protocol.stage_boundary.forbidden_result_fields",
        set(boundary.get("forbidden_result_fields", [])),
        RESULT_KEYS,
    )
    _contains(
        "protocol.stage_boundary.source_independent_preflight_exception",
        boundary.get("source_independent_preflight_exception"),
        "can only reject a family",
    )
    _expect("protocol.component_versions", protocol.get("component_versions"), EXPECTED_COMPONENT_VERSIONS)
    preflight_records = _list(
        "protocol.immutable_source_independent_preflight_records",
        protocol.get("immutable_source_independent_preflight_records"),
    )
    _expect("protocol immutable preflight record count", len(preflight_records), 1)
    f3_record = _mapping("protocol.immutable_source_independent_preflight_records[0]", preflight_records[0])
    _expect("f3 immutable rejected record", f3_record, EXPECTED_F3_V1_RECORD)
    _expect("protocol.change_control.scalar_retuning_after_results", protocol["change_control"].get("scalar_retuning_after_results"), False)
    _expect("protocol.change_control.source_specific_or_filename_specific_constants", protocol["change_control"].get("source_specific_or_filename_specific_constants"), False)
    _contains("protocol.change_control.single_rule", protocol["change_control"].get("single_rule"), "stage_a_protocol.v2")
    _contains("protocol.change_control.single_rule", protocol["change_control"].get("single_rule"), "relevant component-version bump")
    _contains("protocol.change_control.single_rule", protocol["change_control"].get("single_rule"), "invalidates all earlier Stage-A results")
    _contains("protocol.change_control.prequalification_specific_rule", protocol["change_control"].get("prequalification_specific_rule"), "prequalification.v2 to v3")

    passport_contract = _mapping("protocol.numeric_passport_contract", protocol.get("numeric_passport_contract"))
    _expect("protocol.numeric_passport_contract.change_rule_token", passport_contract.get("change_rule_token"), CHANGE_RULE)
    required_passport_fields = set(_list("protocol.numeric_passport_contract.required_fields", passport_contract.get("required_fields")))
    _expect(
        "protocol.numeric_passport_contract.required_fields",
        required_passport_fields,
        {"value", "unit", "scope", "comparator_or_formula_role", "rationale", "change_rule"},
    )
    passports = _mapping("protocol.numeric_passports", protocol.get("numeric_passports"))
    _expect("protocol passport metadata key set", set(EXPECTED_PASSPORT_METADATA), set(EXPECTED_VALUES))
    if set(passports) != set(EXPECTED_VALUES):
        missing = sorted(set(EXPECTED_VALUES) - set(passports))
        extra = sorted(set(passports) - set(EXPECTED_VALUES))
        _fail("protocol.numeric_passports", f"exact passport set required; missing={missing}, extra={extra}")
    for name, expected in EXPECTED_VALUES.items():
        passport = _mapping(f"protocol.numeric_passports.{name}", passports[name])
        if set(passport) != required_passport_fields:
            _fail(f"protocol.numeric_passports.{name}", "passport fields must match required_fields exactly")
        _expect(f"protocol.numeric_passports.{name}.value", passport.get("value"), expected)
        _expect(f"protocol.numeric_passports.{name}.change_rule", passport.get("change_rule"), CHANGE_RULE)
        _expect(
            f"protocol.numeric_passports.{name}.metadata",
            (
                passport.get("unit"),
                passport.get("scope"),
                passport.get("comparator_or_formula_role"),
            ),
            EXPECTED_PASSPORT_METADATA[name],
        )
        if not isinstance(passport.get("rationale"), str) or not passport["rationale"].strip():
            _fail(f"protocol.numeric_passports.{name}.rationale", "must be non-empty rationale metadata")
    _expect(
        "protocol.numeric_passports.f3_source_response_quantized_max.derivation",
        passports["f3_source_response_quantized_max"]["value"],
        int(1.0 / passports["near_identity_delta_rms_min"]["value"] + 0.5),
    )
    _expect(
        "protocol.numeric_passports.f3_source_response_field_count.value",
        passports["f3_source_response_field_count"]["value"],
        8,
    )
    _contains(
        "protocol.numeric_passports.near_identity_delta_rms_min.rationale",
        passports["near_identity_delta_rms_min"].get("rationale"),
        "not a perceptual hardness threshold",
    )
    _contains(
        "protocol.numeric_passports.f3_source_response_minimum_quantized_component_distance.rationale",
        passports["f3_source_response_minimum_quantized_component_distance"].get("rationale"),
        "one fixed-horizon controller summary",
    )
    _contains(
        "protocol.numeric_passports.f3_source_response_minimum_quantized_component_distance.rationale",
        passports["f3_source_response_minimum_quantized_component_distance"].get("rationale"),
        "does not claim 0.10 raw separation",
    )
    q_goldens = passports["f3_source_response_preflight_q_goldens"]["value"]
    for rate, q_values in q_goldens.items():
        _expect(
            f"protocol.numeric_passports.f3_source_response_preflight_q_goldens.{rate}.length",
            len(q_values),
            passports["f3_source_response_field_count"]["value"],
        )
        if any(
            not isinstance(value, int)
            or isinstance(value, bool)
            or value < 0
            or value > passports["f3_source_response_quantized_max"]["value"]
            for value in q_values
        ):
            _fail(
                f"protocol.numeric_passports.f3_source_response_preflight_q_goldens.{rate}",
                "every code must be an integer inside the frozen quantized range",
            )
    q_items = list(q_goldens.items())
    for left_index, (left_rate, left_q) in enumerate(q_items):
        for right_rate, right_q in q_items[left_index + 1 :]:
            _expect(
                f"protocol F3 cross-rate q nondiversity {left_rate}/{right_rate}",
                max(abs(left - right) for left, right in zip(left_q, right_q, strict=True))
                < passports[
                    "f3_source_response_minimum_quantized_component_distance"
                ]["value"],
                True,
            )
    _expect(
        "protocol.numeric_passports.detector_coarse_rms_floor_ratio.comparator_or_formula_role",
        passports["detector_coarse_rms_floor_ratio"]["comparator_or_formula_role"],
        "rms_greater_than_or_equal_local_20th_percentile_times_value",
    )
    _expect(
        "protocol.numeric_passports.anatomy_peak_baseline_ratio.comparator_or_formula_role",
        passports["anatomy_peak_baseline_ratio"]["comparator_or_formula_role"],
        "peak_greater_than_or_equal_baseline_times_value",
    )
    _expect(
        "protocol.numeric_passports.output_peak_absolute_max.metadata",
        EXPECTED_PASSPORT_METADATA["output_peak_absolute_max"],
        ("normalized_full_scale", "all_candidate_and_control_output", "absolute_peak_strictly_less_than"),
    )

    pre = _mapping("protocol.prequalification", protocol.get("prequalification"))
    _expect(
        "protocol.prequalification.purpose",
        pre.get("purpose"),
        "Mechanism-blind rejection and partitioning only; no hardness score and no algorithm-family selection.",
    )
    _contains("protocol.prequalification.input.analysis_signal", pre["input"].get("analysis_signal"), "mu_c is the f64 arithmetic mean")
    _contains("protocol.prequalification.input.analysis_signal", pre["input"].get("analysis_signal"), "p[n]=(1/C)*sum_c(x_prime[c,n]^2)")
    _contains("protocol.prequalification.input.analysis_signal", pre["input"].get("analysis_signal"), "q[n]=sqrt(p[n])")
    _contains("protocol.prequalification.input.analysis_signal", pre["input"].get("analysis_signal"), "never mono-sum channels")
    _expect("protocol.prequalification.input.audio", pre["input"].get("audio"), "registered_PCM_at_native_sample_rate_without_analysis_resampling")
    _expect("protocol.prequalification.input.renderer_signal", pre["input"].get("renderer_signal"), "untouched_registered_PCM")
    _expect(
        "protocol.prequalification.input.input_lsb_provenance",
        pre["input"].get("input_lsb_provenance"),
        "Use registered_format_binding exactly. Only verified signed RIFF/WAVE PCM16 format-tag 1 with valid_bits=16 and input_lsb=2^-15, or verified signed RIFF/WAVE PCM24 format-tag 1 with valid_bits=24 and input_lsb=2^-23, is admitted. Container width alone is insufficient. Missing, mismatched, nonfinite, or nonpositive LSB provenance refuses and is never a caller-tunable knob.",
    )
    format_binding = _mapping(
        "protocol.prequalification.input.registered_format_binding",
        pre["input"].get("registered_format_binding"),
    )
    _contains("format_binding.inherited", format_binding.get("inherited_v1_without_source_format"), "sample_rate_hz=48000, channels=2")
    _contains("format_binding.inherited", format_binding.get("inherited_v1_without_source_format"), "format_tag=1 signed little-endian integer PCM16")
    _contains("format_binding.explicit", format_binding.get("explicit_v2_source_format"), "sample_width_bits exactly 16 or 24")
    _contains("format_binding.explicit", format_binding.get("explicit_v2_source_format"), "container_bits=valid_bits=24")
    _contains("format_binding.binder", format_binding.get("qualification_binder"), "parse the RIFF/WAVE fmt chunk before PCM decoding")
    _contains("format_binding.binder", format_binding.get("qualification_binder"), "Reject WAVE_FORMAT_EXTENSIBLE, IEEE float, big-endian RIFX")
    _contains("format_binding.policy", format_binding.get("typed_policy"), "caller override is impossible")

    roles = pre["impact_roles"]
    _expect(
        "protocol.prequalification.impact_roles.eligible.roles",
        [item.get("role") for item in roles.get("eligible", [])],
        ["body_bearing_single_percussive", "body_bearing_fused_composite_percussive"],
    )
    _expect(
        "protocol.prequalification.impact_roles.refusal_reasons",
        roles.get("refusal_reasons"),
        [
            "edge_only_impulse",
            "multi_event_or_flam",
            "slow_or_sustained",
            "lookbehind_masked",
            "attack_turnover_unresolved",
            "body_unresolved",
            "tail_unresolved",
            "overlapped_event",
            "insufficient_signal",
            "unsupported_format",
            "nonfinite_analysis",
        ],
    )
    _contains("protocol.prequalification.impact_roles.ownership", roles.get("ownership"), "never selects F1, F2, or F3")

    detector = pre["event_detector"]
    _expect(
        "detector.frame_conversion",
        detector.get("frame_conversion"),
        {
            "positive_duration": "For ms>0, N(ms)=max(1,floor(fs*ms/1000+frame_rounding_offset)).",
            "signed_or_zero_offset": "O(ms)=sign(ms)*floor(fs*abs(ms)/1000+frame_rounding_offset), with sign(0)=0 and O(0)=0.",
            "comparison_domain": "Convert every tolerance, NMS radius, fusion distance, search bound, and persistence duration to integer frames first; compare integer frame indices or deltas only.",
        },
    )
    _contains("detector.band_novelty", detector.get("band_novelty"), "k-detector_log_energy_lag_hops")
    _contains("detector.threshold", detector.get("threshold"), "strict N > median")
    _contains("detector.window", detector.get("window"), "0.5-0.5*cos(2*pi*n/N)")
    _expect("detector.frame_timestamp", detector.get("frame_timestamp"), "start_frame+floor(N/2)")
    _contains("detector.band_membership", detector.get("band_membership"), "top band alone also includes")
    _contains("detector.band_energy", detector.get("band_energy"), "divided by sum(window^2)")
    _contains("detector.smoothing", detector.get("smoothing"), "k-1,k,k+1")
    _contains("detector.order_statistics", detector.get("order_statistics"), "even-count median is the arithmetic mean")
    _contains("detector.baseline", detector.get("baseline"), "exactly the same set of valid complete-window hop indices")
    _contains("detector.broadband_guard", detector.get("broadband_guard"), "unweighted DC-subtracted RMS")
    _contains("detector.coarse_peak", detector.get("coarse_peak"), "inclusive")
    _contains("detector.nms", detector.get("nms"), "strictly less than detector_nms_ms")
    _contains("detector.nms", detector.get("nms"), "then earlier anchor on equality")
    _contains("detector.envelopes", detector.get("envelopes"), "causal right-aligned")
    _contains("detector.envelopes", detector.get("envelopes"), "standard phase-safe multichannel")
    _contains("detector.envelopes", detector.get("envelopes"), "does not use fourth-moment weighting")
    _expect("detector.pre_nms_peaks_retained_for_composite_analysis", detector.get("pre_nms_peaks_retained_for_composite_analysis"), True)

    anatomy = pre["event_anatomy"]
    for key, token in {
        "baseline": "For each tested s independently",
        "local_peak": "For each tested s",
        "physical_onset": "Freeze that s, b_s, and peak_s",
        "lookbehind_guard": "lookbehind_peak_ratio_max",
        "candidate_lookbehind": "f32-bit-identical source copy for F1, F2, and F3",
        "attack_end": "following N(attack_turnover_persistence_ms)-1",
        "body_end": "entire qualifying persistence run to finish",
        "tail_end": "entire qualifying persistence run to finish",
        "comparison_windows": "source-frozen windows",
    }.items():
        _contains(f"anatomy.{key}", anatomy.get(key), token)

    composite = pre["composite_policy"]
    _contains("composite.separate_event", composite.get("separate_event"), "event_valley_persistence_ms")
    _contains("composite.candidate_requirement", composite.get("candidate_requirement"), "candidate full-source event-level cluster count must be <= the source count")
    rhythmic = pre["rhythmic_location_proxy"]
    _expect("rhythmic.candidate_onset_movement", rhythmic.get("candidate_onset_movement"), "absolute integer-frame delta <= N(candidate_onset_tolerance_ms)")
    _expect("rhythmic.candidate_proxy_movement", rhythmic.get("candidate_proxy_movement"), "absolute integer-frame delta <= N(candidate_rhythmic_proxy_tolerance_ms)")
    _expect("rhythmic.perceptual_claim", rhythmic.get("perceptual_claim"), "not_a_PAT_or_P_centre_measure")

    contrast = pre["source_contrast"]
    _expect("source_contrast.metadata_input_allowed", contrast.get("metadata_input_allowed"), False)
    _expect("source_contrast.overall_distance", contrast.get("overall_distance"), "D=sqrt(sum_over_source_distance_domain_count(domain_delta^2)/source_distance_domain_count)")
    _contains("source_contrast.partitioning", contrast.get("partitioning"), "Require exactly valid_source_partition_count valid partition")
    _contains("source_contrast.requirements", contrast.get("requirements"), "no imputation")
    _contains("source_contrast.welch", contrast.get("welch"), "Only complete windows")
    _contains("source_contrast.onset_sequence", contrast.get("onset_sequence"), "refined event-level physical onsets")
    _contains("source_contrast.articulation_energy", contrast.get("articulation_energy"), "per-channel mean power")
    _contains("source_contrast.comparison_convention", contrast.get("comparison_convention"), "inclusive as declared")
    ordinals = pre["event_ordinal_policy"]
    _expect("ordinals.development_ordinals", ordinals.get("development_ordinals"), [1, 2])
    _expect("ordinals.confirmation_ordinal", ordinals.get("confirmation_ordinal"), 3)
    _expect("ordinals.golden_case", ordinals.get("golden_case"), {"case_id": "oga_cinameng_can_be_so_beautiful", "event_ordinal": 1, "automatic_fallback": False, "eligibility": "must_pass_identical_mechanism_blind_qualification"})

    precandidate = _mapping("protocol.precandidate", protocol.get("precandidate"))
    _expect("precandidate.numeric_representation", precandidate.get("numeric_representation"), {"analysis_and_policy": "f64", "render_output": "f32"})
    _expect("precandidate.source_frozen_regions_only", precandidate.get("source_frozen_regions_only"), True)
    wrapper = precandidate.get("qualification_wrapper", {})
    _contains("qualification_wrapper.single_evidence_path", wrapper.get("single_evidence_path"), "Only the versioned Stage-A qualification wrapper")
    _contains("qualification_wrapper.single_evidence_path", wrapper.get("single_evidence_path"), "unqualified diagnostic primitives")
    required_bindings = wrapper.get("required_bindings", [])
    for token in ("source-registry v2", "valid_bits", "event-catalog artifact SHA-256", "impact role", "native sample-rate range", "family version", "preflight status"):
        if not any(token in value for value in required_bindings):
            _fail("qualification_wrapper.required_bindings", f"missing binding containing {token!r}")
    _contains("qualification_wrapper.caller_knobs", wrapper.get("caller_knobs"), "cannot supply or override")
    _expect(
        "qualification_wrapper.output_gate",
        wrapper.get("output_gate"),
        "After f32 rendering and before any qualified serialization, require finite non-silent output and strict abs_peak<output_peak_absolute_max plus every reject-only screen.",
    )
    _contains("qualification_wrapper.direct_renderer_boundary", wrapper.get("direct_renderer_boundary"), "qualification_state=unqualified_diagnostic")
    _contains("qualification_wrapper.direct_renderer_boundary", wrapper.get("direct_renderer_boundary"), "cannot enter a blinded review pack")
    _contains("precandidate.energy_definitions.region_energy", precandidate.get("energy_definitions", {}).get("region_energy"), "without division by channel count")
    _contains("precandidate.energy_definitions.weighted_region_mean_square", precandidate.get("energy_definitions", {}).get("weighted_region_mean_square"), "channel_count*sum_n(wR[n])")
    _contains("masks.crossfade_frames", precandidate["attack_body_masks"].get("crossfade_frames"), "mask_crossfade_divisor+0.5")
    _contains("masks.attack_to_body_crossfade", precandidate["attack_body_masks"].get("attack_to_body_crossfade"), "start=attack_end-floor(m/2)")
    _contains("masks.attack_to_body_crossfade", precandidate["attack_body_masks"].get("attack_to_body_crossfade"), "m+mask_crossfade_endpoint_offset")
    _contains("masks.body_end_fade", precandidate["attack_body_masks"].get("body_end_fade"), "[body_end-m,body_end)")
    _contains("masks.index_validation", precandidate["attack_body_masks"].get("index_validation"), "refuse inconsistent boundaries")
    _contains("masks.frozen_signal_floor", precandidate["attack_body_masks"].get("frozen_signal_floor"), "strict MS_R>(minimum_signal_peak_lsb*input_lsb)^2")
    _contains("masks.frozen_signal_floor", precandidate["attack_body_masks"].get("frozen_signal_floor"), "never a free caller value")
    _contains("masks.ablation_rule", precandidate["attack_body_masks"].get("ablation_rule"), "combined candidate's resolved policy values")

    three_band = _mapping(
        "precandidate.exact_complementary_three_band_analysis",
        precandidate.get("exact_complementary_three_band_analysis"),
    )
    _expect("three_band.id", three_band.get("id"), "riotbox.exact_complementary_three_band_analysis.v1")
    _expect(
        "three_band.shared_consumers",
        three_band.get("shared_consumers"),
        ["F2", "static_eq_fit_all_families", "dark_onepole_f25_v1", "bright_exact_split_f75_v1"],
    )
    _contains("three_band.split_estimation", three_band.get("split_estimation"), "Use raw registered PCM, not whole-source-DC-subtracted")
    _contains("three_band.split_estimation", three_band.get("split_estimation"), "sum squared magnitudes across channels per bin without mono summing")
    _contains("three_band.filter_bank", three_band.get("filter_bank"), "H=x-L-M")
    _contains("three_band.cross_family_failure", three_band.get("cross_family_failure"), "rejects F1, F2, and F3 for that event")
    _contains("three_band.cross_family_failure", three_band.get("cross_family_failure"), "fails control sanity")

    families = _list("precandidate.algorithm_families", precandidate.get("algorithm_families"))
    _expect("precandidate.algorithm_families.ids", [f.get("id") for f in families], ["f1_ab_energy_redistribution_v1", "f2_exact_complementary_three_band_v1", "f3_causal_envelope_contrast_dynamic_residual_v2"])
    f1, f2, f3 = families
    _contains("F1.resolver", f1.get("resolver"), "K=f1_masked_energy_allocation_multiplier")
    _contains("F1.resolver", f1.get("resolver"), "gA^2=K*(EA+EB)/(K*EA+EB)")
    _contains("F1.render_formula", f1.get("render_formula"), "y[c,n]=x[c,n]*g[n]")
    _expect("F1.outside_body", f1.get("outside_body"), "bit_identical_dry_before_onset_and_from_body_end")
    _expect("F1.ablation_keys", set(f1.get("ablations", {})), {"attack_only", "body_only", "combined"})
    f1_falsifiers = "\n".join(f1.get("family_falsifiers", []))
    _contains("F1.falsifier.conservation", f1_falsifiers, "floating_comparison_epsilon_multiplier*f32_epsilon")
    _contains("F1.falsifier.conservation", f1_falsifiers, "max(E_source,E_candidate)")
    _contains("F1.falsifier.ratio", f1_falsifiers, "r_candidate>r_source+tol")
    _contains("F1.falsifier.ratio", f1_falsifiers, "f64_epsilon")

    _expect("F2.analysis_component", f2.get("analysis_component"), "riotbox.exact_complementary_three_band_analysis.v1")
    _contains("F2.trusted_band_rule", f2.get("trusted_band_rule"), "strictly above max(f2_band_trust_lookbehind_ratio*per-band frozen-lookbehind mean-square,(minimum_signal_peak_lsb*input_lsb)^2)")
    _contains("F2.trusted_band_rule", f2.get("trusted_band_rule"), "channel_count*sum(mask)")
    _contains("F2.resolver", f2.get("resolver"), "channel-summed masked band energies without channel averaging")
    _contains("F2.resolver", f2.get("resolver"), "Kj=2-pj")
    _expect("F2.outside_body", f2.get("outside_body"), "bit_identical_source_copy_before_physical_onset_and_from_body_end; filtered reconstruction is inserted only inside the frozen affected event")
    _expect("F2.preflight.required_before_any_source", f2["preflight"].get("required_before_any_source"), True)
    _expect("F2.preflight.signals", f2["preflight"].get("signals"), ["impulse", "dc", "deterministic_noise", "bounded_real_sequence"])
    _contains("F2.preflight.signal_definitions", f2["preflight"].get("signal_definitions"), "f2_preflight_noise_lcg")
    _contains("F2.preflight.normalized_reconstruction_formula", f2["preflight"].get("normalized_reconstruction_formula"), "sum_over_all_frames((L+M+H-x)^2)")
    _contains("F2.preflight.maximum_reconstruction_error", f2["preflight"].get("maximum_reconstruction_error"), "f32_epsilon")
    _contains("F2.preflight.known_bin_quantile_golden", f2["preflight"].get("known_bin_quantile_golden"), "numeric_passports.f2_quantile_golden")
    _contains("F2.preflight.known_bin_quantile_golden", f2["preflight"].get("known_bin_quantile_golden"), "1:4:1 weights")
    _contains("F2.preflight.known_bin_quantile_golden", f2["preflight"].get("known_bin_quantile_golden"), "raw, not DC-subtracted")

    _expect("F3.caller_knobs", f3.get("caller_knobs"), "none")
    _expect(
        "F3.source_response_diversity_component",
        f3.get("source_response_diversity_component"),
        "riotbox.f3_source_response_diversity.v1",
    )
    _contains("F3.predecessor_boundary", f3.get("predecessor_boundary"), "inherits no oversampling factor, taps, window, drive, residual target, or scale")
    _contains("F3.input_and_provenance", f3.get("input_and_provenance"), "Signed PCM16 means input_lsb=2^-15")
    _contains("F3.phase_safe_envelopes", f3.get("phase_safe_envelopes"), "R_t[n]=sqrt(mean(q^2))")
    _contains("F3.raw_contrast", f3.get("raw_contrast"), "a0[n]=sqrt(D(R1[n],R8[n])*D(R8[n],R20[n]))")
    _contains("F3.causal_smoother", f3.get("causal_smoother"), "63.2-percent step-response time-constant")
    _contains("F3.branch_contribution", f3.get("branch_contribution"), "MS_branch_R=sum_c,n(rR[c,n]^2)/(C*S_R)")
    _contains("F3.branch_contribution", f3.get("branch_contribution"), "(minimum_signal_peak_lsb*input_lsb)^2")
    _contains("F3.render_formula", f3.get("render_formula"), "attack_only=x+rA, body_only=x+rB, and combined=x+rA+rB")
    _expect("F3.controller_hash.labels", f3["controller_hashes"].get("labels_in_exact_order"), ["a0", "b0", "attack_state", "body_state"])
    _expect(
        "F3.controller_hash.evidence_role",
        f3["controller_hashes"].get("evidence_role"),
        "provenance_only_not_actionable_diversity",
    )
    _expect("F3.controller_hash.domain", f3["controller_hashes"].get("domain"), "riotbox.f3_causal_envelope_contrast_dynamic_residual_v2.controller.v1")
    _contains("F3.controller_hash.preimage", f3["controller_hashes"].get("preimage"), "concat_each_f64_to_bits_u64_be")
    _contains("F3.controller_hash.array_contract", f3["controller_hashes"].get("array_contract"), "rejects negative zero or nonfinite")
    _expect(
        "F3.direction",
        f3["family_falsifiers"].get("direction"),
        "Recompute candidate R1/R8/R20 with the exact source-frozen DC means and F. Q_attack(z)=sum_n(wA[n]*R1_z[n]^2)/sum_n(wA[n]*R20_z[n]^2); require attack_only Q_attack > source Q_attack by floating_comparison_epsilon_multiplier*f64_epsilon*max(1,abs(values)). Q_body(z)=sum_n(wB[n]*R1_z[n]^2)/sum_n(wB[n]*max(R8_z[n]^2,R20_z[n]^2)); require body_only Q_body > source Q_body by the same tolerance. Zero or nonfinite denominator refuses. Combined Q values are recorded diagnostically and are not direction gates because independent branch interaction is not an isolated ablation. These checks establish controller integrity only, never hardness.",
    )
    _expect("F3.preflight.required_before_any_source", f3["preflight"].get("required_before_any_source"), True)
    _contains("F3.preflight.frame_helper", f3["preflight"].get("frame_helper"), "M64(t_ms)")
    _expect(
        "F3.preflight.signal_common",
        f3["preflight"].get("signal_common"),
        "Stereo quadrature xL[n]=amplitude[n]*cos(2*pi*carrier*n), xR[n]=amplitude[n]*sin(2*pi*carrier*n), PCM16 valid_bits and exact input_lsb from passports, complete production per-channel whole-source DC means/removal, full production envelopes/controller/masks/render/gates, and raw peak strictly below 1. Every run hashes raw input, a0, b0, attack_state, body_state, ordinary policy, and typed outcome. Every successfully rendered run additionally hashes attack-only, body-only, and combined outputs plus its source-response diversity identity. The expected constant_quadrature_v1 refusal has no output hash and no source-response diversity identity. Repeated runs on the same build/platform must match their applicable hashes; cross-platform conformance uses formulas and tolerances, not brittle hash equality.",
    )
    _contains("F3.preflight.constant", f3["preflight"].get("constant_quadrature_v1"), "missing_attack_and_body_dynamic_contrast")
    _contains("F3.preflight.step", f3["preflight"].get("step_body_quadrature_v1"), "first A>0 exactly at onset")
    _contains("F3.preflight.scale", f3["preflight"].get("amplitude_scale_equivariance_v1"), "Recompute frozen anatomy baseline b")
    _expect(
        "F3.preflight.source_response_diversity_identity",
        f3["preflight"].get("source_response_diversity_identity_v1"),
        "For every sample rate, step_body_quadrature_v1 must produce the riotbox.f3_source_response_diversity.v1 quantized vector and hash; amplitude_scale_equivariance_v1, polarity_equivariance_v1, and carrier_invariance_v1 must each produce the exact same vector and hash at that rate, and the vector must equal numeric_passports.f3_source_response_preflight_q_goldens for that rate. Across rates, record the corresponding vectors and hashes but do not require hash equality because M64 alignment changes physical duration; require every corresponding q-component absolute difference strictly less than f3_source_response_minimum_quantized_component_distance, so sample-rate/alignment drift is not diversity-separated and cannot earn F3 diversity. constant_quadrature_v1 is typed refusal provenance, produces no actionable diversity identity, and cannot count as diversity evidence.",
    )
    _expect(
        "F3.preflight.source_response_diversity_hash_goldens",
        f3["preflight"].get("source_response_diversity_hash_goldens_v1"),
        {
            rate: _f3_source_response_digest(q_values)
            for rate, q_values in q_goldens.items()
        },
    )
    _contains("F3.preflight.failure", f3["preflight"].get("failure"), "reject_entire_F3_v2_family")

    controls = _list("precandidate.false_positive_controls", precandidate.get("false_positive_controls"))
    _expect(
        "controls.ids",
        [control.get("control_id") for control in controls],
        [
            "hidden_exact_a_a_v1",
            "gain_plus_3db_v1",
            "global_rate_four_fifths_v1",
            "dark_onepole_f25_v1",
            "bright_exact_split_f75_v1",
            "distortion_os4_d2_v1",
            "delayed_duplicate_25ms_minus6db_v1",
            "detached_click_minus12db_body_v1",
        ],
    )
    if any(control.get("can_earn_force") is not False for control in controls):
        _fail("controls.can_earn_force", "every false-positive control must be reject-only")
    distortion_control = controls[5]
    _expect("controls.distortion.failed_candidate_alias_preflight_acknowledged", distortion_control.get("failed_candidate_alias_preflight_acknowledged"), True)
    _expect("controls.distortion.promotable_candidate_path", distortion_control.get("promotable_candidate_path"), False)
    _expect("controls.distortion.reuse_boundary", distortion_control.get("reuse_boundary"), "non_promotable_negative_confound_only_never_candidate_reference_safety_or_quality_evidence")
    _contains("controls.rate.renderer", controls[2].get("renderer"), "M=ceil((N-1)*5/4)+1")
    _contains("controls.rate.renderer", controls[2].get("renderer"), "p=min(4*m,5*(N-1))")
    _contains("controls.rate.renderer", controls[2].get("renderer"), "typed playback_rate=[4,5]")
    _contains("controls.dark.renderer", controls[3].get("renderer"), "riotbox.exact_complementary_three_band_analysis.v1")
    _contains("controls.bright.renderer", controls[4].get("renderer"), "riotbox.exact_complementary_three_band_analysis.v1")
    _contains("controls.detached.renderer", controls[7].get("renderer"), "[tail_end-m,tail_end)")
    _contains("controls.detached.renderer", controls[7].get("renderer"), "attenuating the physical body and tail")
    _contains("control_sanity_rule", precandidate.get("control_sanity_rule"), "requires a versioned protocol revision")

    matcher = precandidate["level_matcher"]
    _expect("matcher.id", matcher.get("id"), "event_train_rms_attenuation_match_v1")
    _contains("matcher.target", matcher.get("target"), "T=min(RA,RB)")
    _contains("matcher.headroom", matcher.get("headroom"), "H=10^(matcher_ceiling_dbfs/20)")
    _expect("matcher.forbidden", matcher.get("forbidden"), ["boost", "limiter", "frequency_weighting", "window_selection"])
    _expect("matcher.raw_view_preserved", matcher.get("raw_view_preserved"), True)

    train = _mapping("review_train", precandidate.get("review_train"))
    _expect(
        "review_train",
        train,
        {
            "candidate_train": {
                "single_event_maximum": "train_event_maximum_ms",
                "train_duration": "train_duration_ms",
                "event_count": 4,
                "onsets": "numeric_passports.train_onsets_ms",
                "event_complete_by": "train_event_complete_by_ms",
                "terminal_silence_minimum": "train_terminal_silence_ms",
                "pair_duration": "pair_duration_ms",
            },
            "false_positive_control_train": {
                "single_event_maximum": "control_train_event_maximum_ms",
                "train_duration": "control_train_duration_ms",
                "event_count": 4,
                "onsets": "numeric_passports.control_train_onsets_ms",
                "event_complete_by": "control_train_event_complete_by_ms",
                "terminal_silence_minimum": "control_train_terminal_silence_ms",
                "pair_duration": "control_pair_duration_ms",
            },
            "pair_layout": "train_A then exactly pair_inter_train_silence_ms silence then train_B",
            "views": ["raw", "event_train_rms_attenuation_match_v1"],
        },
    )
    _validate_train_arithmetic(
        "review_train.candidate_train",
        onsets=EXPECTED_VALUES["train_onsets_ms"],
        event_count=EXPECTED_VALUES["train_event_count"],
        event_maximum_ms=EXPECTED_VALUES["train_event_maximum_ms"],
        event_complete_by_ms=EXPECTED_VALUES["train_event_complete_by_ms"],
        train_duration_ms=EXPECTED_VALUES["train_duration_ms"],
        terminal_silence_minimum_ms=EXPECTED_VALUES["train_terminal_silence_ms"],
        inter_train_silence_ms=EXPECTED_VALUES["pair_inter_train_silence_ms"],
        pair_duration_ms=EXPECTED_VALUES["pair_duration_ms"],
    )
    _validate_train_arithmetic(
        "review_train.false_positive_control_train",
        onsets=EXPECTED_VALUES["control_train_onsets_ms"],
        event_count=EXPECTED_VALUES["train_event_count"],
        event_maximum_ms=EXPECTED_VALUES["control_train_event_maximum_ms"],
        event_complete_by_ms=EXPECTED_VALUES["control_train_event_complete_by_ms"],
        train_duration_ms=EXPECTED_VALUES["control_train_duration_ms"],
        terminal_silence_minimum_ms=EXPECTED_VALUES["control_train_terminal_silence_ms"],
        inter_train_silence_ms=EXPECTED_VALUES["pair_inter_train_silence_ms"],
        pair_duration_ms=EXPECTED_VALUES["control_pair_duration_ms"],
    )
    blind = precandidate["blinding"]
    _expect("blinding.allowed_views", blind.get("allowed_views"), ["raw", "event_train_rms_attenuation_match_v1"])
    _expect("blinding.allowed_repetition_ids", blind.get("allowed_repetition_ids"), ["primary", "reversed"])
    _contains("blinding.field_encoding", blind.get("field_encoding"), "u32_be(byte_length)")
    _contains("blinding.base_pair_id", blind.get("base_pair_id"), "repetition_id is intentionally excluded")
    _expect("blinding.artifact_id", blind.get("artifact_id"), "artifact_id=pair_id+'.'+repetition_id+'.wav'; these are the only two artifact IDs for a base pair and filenames never disclose orientation.")
    _contains("blinding.primary_orientation_seed", blind.get("primary_orientation_seed"), "digest[31]&1")
    _contains("blinding.reversed_orientation", blind.get("reversed_orientation"), "do not hash them or derive a second orientation")
    _contains("blinding.block_order", blind.get("block_order"), "lexical digest bytes")

    construction = _mapping("candidate_event_construction", precandidate.get("candidate_event_construction"))
    _contains("construction.full_length_copy", construction.get("full_length_copy"), "complete registered source PCM shape")
    _contains("construction.analysis_reuse", construction.get("analysis_reuse"), "source-frozen per-channel whole-source DC means")
    _contains("construction.rhythmic_proxy", construction.get("rhythmic_proxy"), "source-frozen b")
    _contains("construction.event_count_rule", construction.get("event_count_rule"), "candidate event-level cluster count must be <= the source event-level cluster count")
    _contains("construction.non_silent", construction.get("non_silent"), "strictly greater than minimum_signal_peak_lsb*input_lsb")
    _contains("construction.candidate_pcm_hash", construction.get("candidate_pcm_hash"), "riotbox.percussive_force_pcm_f32le.v1")
    _contains("construction.candidate_pcm_hash", construction.get("candidate_pcm_hash"), "frame-major interleaved channel-order exact f32.to_bits().to_le_bytes()")
    _expect("construction.screen_views", set(construction.get("screen_views", {})), {"raw", "event_train_rms_attenuation_match_v1"})

    metrics = precandidate["mechanical_metric_definitions"]
    _contains("metrics.correlation", metrics.get("zero_lag_identity_correlation"), "per-channel arithmetic mean")
    _contains("metrics.correlation", metrics.get("zero_lag_identity_correlation"), "rho=sum_c,n")
    _contains("metrics.attack_spectral_cosine", metrics.get("attack_spectral_cosine"), "e_j=20*(U/20)^(j/K)")
    _contains("metrics.attack_spectral_cosine", metrics.get("attack_spectral_cosine"), "Do not take logarithms")
    _contains("metrics.least_squares_common", metrics.get("least_squares_common"), "no intercept")
    _contains("metrics.least_squares_common", metrics.get("least_squares_common"), "deterministic partial pivoting")
    _contains("metrics.static_eq_fit", metrics.get("static_eq_fit"), "riotbox.exact_complementary_three_band_analysis.v1")
    _contains("metrics.static_eq_fit", metrics.get("static_eq_fit"), "rejects every family for that event")
    _contains("metrics.boundary_discontinuity", metrics.get("boundary_discontinuity"), "Remove physical_onset from the set")
    _contains("metrics.boundary_discontinuity", metrics.get("boundary_discontinuity"), "excluding every delta-step whose right-hand index is any member of the complete declared boundary set")
    _contains("metrics.boundary_discontinuity", metrics.get("boundary_discontinuity"), "j>local_max+tol")
    _contains("metrics.global_rate_or_pitch", metrics.get("global_rate_or_pitch"), "playback_rate=[1,1]")

    screens = precandidate["reject_only_mechanical_screens"]
    _expect(
        "screens.semantics",
        screens.get("semantics"),
        "These checks may reject unsafe, collapsed, confounded, or identity-destroying output. No metric or aggregate may award perceived hardness.",
    )
    _expect(
        "screens.format_and_safety",
        screens.get("format_and_safety"),
        "On each declared screen view require finite output; same sample rate, channels, and full-source frames as source except global_rate_four_fifths_v1; strict abs_peak<output_peak_absolute_max; and candidate_event_construction.non_silent. On the raw view additionally require the exact candidate PCM hash/byte inequality except hidden A/A.",
    )
    _expect(
        "screens.body",
        screens.get("body"),
        "For raw and event_train_rms_attenuation_match_v1 views separately, sum x[c,n]^2 over all channels and the exact disjoint source-frozen half-open B=[attack_end,body_end). Require finite positive source denominator and inclusive candidate/source energy ratio inside body_energy_ratio_range.",
    )
    for key, token in {
        "untouched_regions": "F1/F2/F3 render, full-length candidate samples are f32-bit-identical",
        "event_integrity": "candidate full-source event-level cluster count <= source count",
        "near_identity": "near_identity_delta_rms_min",
        "identity": "attack_spectral_cosine_min",
        "gain_confound": "gain_only_fit_residual_min",
        "eq_confound": "static_eq_fit_residual_min",
        "distortion_confound": "static_distortion_fit_residual_min",
        "rate_or_pitch_confound": "do not estimate pitch",
    }.items():
        _contains(f"screens.{key}", screens.get(key), token)

    anti = precandidate["anti_hardcoding"]
    _expect("anti.missing_required_feature", anti.get("missing_required_feature"), "typed_refusal_without_fallback")
    _contains("anti.name_and_path_invariance", anti.get("name_and_path_invariance"), "Byte-identical")
    _contains("anti.feature_omission", anti.get("feature_omission"), "at least one such field")
    policy_hash = _mapping("anti.resolved_policy_hash", anti.get("resolved_policy_hash"))
    _expect("anti.policy_hash.domain", policy_hash.get("domain"), "riotbox.percussive_force_actionable_policy.v1")
    _expect("anti.policy_hash.f1", policy_hash.get("f1_field_order"), ["gA_f64", "gB_f64"])
    _expect(
        "anti.policy_hash.f2",
        policy_hash.get("f2_field_order"),
        ["f25_hz_f64", "f75_hz_f64", "trusted_L_bool", "trusted_M_bool", "trusted_H_bool", "gA_L_f64", "gA_M_f64", "gA_H_f64", "gB_L_f64", "gB_M_f64", "gB_H_f64", "a25_f64", "a75_f64"],
    )
    _expect(
        "anti.policy_hash.f3",
        policy_hash.get("f3_field_order"),
        ["sA_f64", "sB_f64"],
    )
    _expect(
        "anti.policy_hash.excluded_fields",
        policy_hash.get("excluded_fields"),
        [
            "case_id",
            "source_path",
            "source_sha256",
            "filename",
            "title",
            "author",
            "raw_EA",
            "raw_EB",
            "absolute_region_indices",
            "mask_shapes",
            "anatomy_lengths",
            "diagnostic_or_outcome_measurements",
            "verdicts",
            "artifacts",
        ],
    )
    _expect("anti.policy_hash.excludes_masks", "mask_shapes" in policy_hash.get("excluded_fields", []), True)
    _expect("anti.policy_hash.excludes_anatomy", "anatomy_lengths" in policy_hash.get("excluded_fields", []), True)
    _expect(
        "anti.policy_hash.excludes_generic_measurements_token",
        "measurements" in policy_hash.get("excluded_fields", []),
        False,
    )
    _contains(
        "anti.policy_hash.scope",
        policy_hash.get("scope"),
        "provenance controller hashes",
    )

    response_identity = _mapping(
        "anti.f3_source_response_diversity",
        anti.get("f3_source_response_diversity"),
    )
    _expect(
        "anti.f3_response.component",
        response_identity.get("component"),
        "riotbox.f3_source_response_diversity.v1",
    )
    _expect(
        "anti.f3_response.evidence_role",
        response_identity.get("evidence_role"),
        "actionable_controller_response_identity_input_not_generic_diagnostic_measurement_not_hardness_evidence",
    )
    _expect(
        "anti.f3_response.horizon",
        response_identity.get("horizon"),
        "Exact half-open [physical_onset,physical_onset+N(body_minimum_ms)); require the complete horizon inside the resolved event or refuse without an identity. This relative fixed physical-time horizon never uses attack_end, body_end, mask boundaries, absolute source indices, or source/array length in identity computation.",
    )
    response_fields = [
        "mean_a0",
        "max_a0",
        "mean_b0",
        "max_b0",
        "mean_A",
        "max_A",
        "mean_B",
        "max_B",
    ]
    _expect(
        "anti.f3_response.raw_field_order",
        response_identity.get("raw_field_order"),
        response_fields,
    )
    _expect(
        "anti.f3_response.raw_field_count",
        len(response_fields),
        passports["f3_source_response_field_count"]["value"],
    )
    _expect(
        "anti.f3_response.raw_statistics",
        response_identity.get("raw_statistics"),
        "For each controller in field order, traverse the exact horizon in increasing relative-frame order; mean is the f64 sum divided by N(body_minimum_ms), and max is the ordinary maximum. Require every raw field finite, in [0,1], and canonical +0.0; reject negative zero.",
    )
    _expect(
        "anti.f3_response.quantization_resolution_ref",
        response_identity.get("quantization_resolution_ref"),
        "numeric_passports.near_identity_delta_rms_min",
    )
    _expect(
        "anti.f3_response.quantization",
        response_identity.get("quantization"),
        "For each raw field v, q(v)=floor(v/near_identity_delta_rms_min+0.5)=floor(20*v+0.5), encoded as one u8 in raw_field_order. Require each q integer in [0,f3_source_response_quantized_max]. The reused 0.05 resolution is numerical anti-collapse evidence, not a perceptual hardness threshold.",
    )
    _expect(
        "anti.f3_response.domain",
        response_identity.get("domain"),
        "riotbox.f3_source_response_diversity.v1",
    )
    _expect(
        "anti.f3_response.family_id",
        response_identity.get("family_id"),
        "f3_causal_envelope_contrast_dynamic_residual_v2",
    )
    _expect(
        "anti.f3_response.preimage",
        response_identity.get("preimage"),
        "u32_be(domain_utf8_byte_length)||domain_utf8||u32_be(family_id_utf8_byte_length)||family_id_utf8||sA_f64_to_bits_u64_be||sB_f64_to_bits_u64_be||q_mean_a0_u8||q_max_a0_u8||q_mean_b0_u8||q_max_b0_u8||q_mean_A_u8||q_max_A_u8||q_mean_B_u8||q_max_B_u8. Require finite canonical +0.0 sA/sB. No sample rate, channel count, frame count, absolute or relative region index, array length, mask, anatomy length, source identity, path, filename, title, author, or provenance controller hash enters the preimage.",
    )
    _expect(
        "anti.f3_response.digest",
        response_identity.get("digest"),
        "lowercase_sha256",
    )
    _expect(
        "anti.f3_response.record",
        response_identity.get("record"),
        "Record the raw eight-vector, quantized eight-vector, and digest. These preregistered fixed-horizon controller-response properties directly determine F3 branch modulation and are typed actionable identity inputs, not excluded generic diagnostic or outcome measurements; they remain reject-only and never award hardness.",
    )
    _expect(
        "anti.f3_response.robust_pair_separation",
        response_identity.get("robust_pair_separation"),
        "A pair of distinct F3 identity hashes may satisfy the family minimum only when max_i(abs(q_left[i]-q_right[i]))>=f3_source_response_minimum_quantized_component_distance. Distinct hashes separated only by a one-bin boundary cannot count.",
    )
    _expect(
        "anti.actionable_diversity_identity",
        anti.get("actionable_diversity_identity"),
        {
            "F1": "resolved_policy_hash",
            "F2": "resolved_policy_hash",
            "F3": "f3_source_response_diversity",
        },
    )
    _contains(
        "anti.diversity",
        anti.get("diversity"),
        "minimum_actionable_diversity_hashes_per_family",
    )
    _contains(
        "anti.diversity",
        anti.get("diversity"),
        "f3_source_response_minimum_quantized_component_distance",
    )
    _contains(
        "anti.diversity",
        anti.get("diversity"),
        "minimum_clusters_with_distinct_actionable_identity_sets",
    )
    _contains(
        "anti.diversity",
        anti.get("diversity"),
        "family-typed actionable identity hash sets",
    )
    cross = precandidate["required_cross_product"]
    _expect("cross.families", cross.get("families"), ["F1", "F2", "F3"])
    _expect("cross.events", cross.get("events"), [1, 2])
    _expect("cross.formula", cross.get("formula"), "3_families_times_4_sources_times_2_events_equals_24_candidate_event_conditions")
    _expect("cross.failed_positive_source_fails_family", cross.get("failed_positive_source_fails_family"), True)
    _expect("cross.aggregate_average_may_rescue_failure", cross.get("aggregate_average_may_rescue_failure"), False)
    _expect("cross.preference_tie", cross.get("preference_tie"), "block_Stage_B_preserve_results_no_retune_or_metric_tiebreak")

    human = precandidate["human_promotion_gate"]
    _expect("human.mechanical_gate_grants", human.get("mechanical_gate_grants"), "listening_request_only")
    fields = human.get("required_directional_fields", {})
    for field in (
        "more_forcefully_struck_direction",
        "clearly_different",
        "recognizable_source_identity",
        "one_audible_event_without_flam_echo_or_detached_duplicate",
        "same_rhythmic_place_and_perceived_timing",
        "physical_body_retained",
        "source_related_bite_retained",
        "desirable_to_trigger",
        "reversed_order_directional_agreement",
        "hidden_exact_a_a_heard_as_same",
    ):
        _expect(f"human.required_directional_fields.{field}", fields.get(field), True)
    _expect("human.confidence", fields.get("confidence"), ["medium", "high"])
    _expect("human.different_but_not_harder", human.get("different_but_not_harder"), "reject_and_freeze_family_recipe")
    _expect("human.aggregate_vote_may_rescue_failed_source", human.get("aggregate_vote_may_rescue_failed_source"), False)

    scope = protocol["readiness_scope"]
    _contains("readiness_scope.stage_a_floor", scope.get("stage_a_floor"), "four legal positive sources")
    _contains("readiness_scope.broader_source_aware_readiness", scope.get("broader_source_aware_readiness"), "at least five development sources spanning at least four source families")
    _expect("readiness_scope.runtime_mix_or_tui_claim", scope.get("runtime_mix_or_tui_claim"), False)
    _expect("readiness_scope.loop_or_instrument_quality_claim", scope.get("loop_or_instrument_quality_claim"), False)

    _expect(
        "protocol exact semantic contract SHA-256",
        _semantic_sha256(protocol),
        EXPECTED_PROTOCOL_SEMANTIC_SHA256,
    )

    if canonical_sha256 is not None:
        _expect("protocol canonical SHA-256", canonical_sha256, EXPECTED_PROTOCOL_SHA256)


def validate_matrix(
    matrix: dict[str, Any],
    *,
    protocol_sha256: str,
    matrix_v1_sha256: str,
    registry_v1: dict[str, Any],
    registry_v1_sha256: str,
    registry_v2: dict[str, Any],
    registry_v2_sha256: str,
    canonical_sha256: str | None = None,
) -> None:
    _expect("matrix.schema", matrix.get("schema"), "riotbox.percussive_force_development_matrix.v2")
    _expect("matrix.schema_version", matrix.get("schema_version"), 2)
    _expect("matrix.owner_ticket", matrix.get("owner_ticket"), "RIOTBOX-1428")
    _expect(
        "matrix.execution_state",
        matrix.get("execution_state"),
        "preregistered_no_source_qualification_or_candidate_render",
    )
    _expect("matrix.quality_proof", matrix.get("quality_proof"), False)
    _expect("matrix.product_path_proof", matrix.get("product_path_proof"), False)
    _contains("matrix.directly_enabled_followup.outcome", matrix.get("directly_enabled_followup", {}).get("outcome"), "source-backed isolated raw and attenuation-matched renders")
    _contains("matrix.directly_enabled_followup.outcome", matrix.get("directly_enabled_followup", {}).get("outcome"), "bounded blinded human force-direction and desirable-to-trigger comparison")
    _expect(
        "matrix.predecessor",
        matrix.get("predecessor"),
        {
            "path": str(MATRIX_V1_REL),
            "schema": "riotbox.percussive_force_development_matrix.v1",
            "sha256": EXPECTED_MATRIX_V1_SHA256,
        },
    )
    _expect("matrix predecessor bytes", matrix_v1_sha256, EXPECTED_MATRIX_V1_SHA256)
    _expect(
        "matrix.protocol",
        matrix.get("protocol"),
        {
            "path": str(PROTOCOL_REL),
            "schema": "riotbox.percussive_force_stage_a_protocol.v1",
            "sha256": EXPECTED_PROTOCOL_SHA256,
            "state": "frozen_before_source_qualification_or_candidate_render",
        },
    )
    _expect("matrix protocol bytes", protocol_sha256, EXPECTED_PROTOCOL_SHA256)
    _expect(
        "matrix.source_registry",
        matrix.get("source_registry"),
        {
            "path": str(REGISTRY_V2_REL),
            "schema": "riotbox.source_holdout_rotation.v2",
            "sha256": EXPECTED_REGISTRY_V2_SHA256,
            "predecessor_path": str(REGISTRY_V1_REL),
            "predecessor_schema": "riotbox.source_holdout_rotation.v1",
            "predecessor_sha256": EXPECTED_REGISTRY_V1_SHA256,
        },
    )
    _expect("registry v1 bytes", registry_v1_sha256, EXPECTED_REGISTRY_V1_SHA256)
    _expect("registry v2 bytes", registry_v2_sha256, EXPECTED_REGISTRY_V2_SHA256)
    _expect("registry v1 schema", registry_v1.get("schema"), "riotbox.source_holdout_rotation.v1")
    _expect("registry v2 schema", registry_v2.get("schema"), "riotbox.source_holdout_rotation.v2")
    _expect("registry v2 owner", registry_v2.get("owner_ticket"), "RIOTBOX-1428")

    transition = matrix.get("source_registry_transition", {})
    _expect("matrix.source_registry_transition.state", transition.get("state"), "completed_metadata_transition_before_event_qualification")
    _expect("matrix.source_registry_transition.predecessor_holdout_union_unchanged", transition.get("predecessor_holdout_union_unchanged"), True)
    _expect("matrix.source_registry_transition.exact_development_additions", transition.get("exact_development_additions"), ["oga_william_hector_horde_war_drums", "oga_frosty_ham_osdrums"])
    _expect("matrix.source_registry_transition.new_family", transition.get("new_family"), "electronic_drums")
    _expect("matrix.source_registry_transition.source_qualification_started", transition.get("source_qualification_started"), False)
    _expect("matrix.source_registry_transition.candidate_render_started", transition.get("candidate_render_started"), False)
    evidence_boundary = matrix.get("qualification_evidence_boundary", {})
    _expect("matrix.qualification_evidence_boundary.only_path", evidence_boundary.get("only_path"), "protocol.precandidate.qualification_wrapper")
    _expect("matrix.qualification_evidence_boundary.direct_family_renderer_can_qualify", evidence_boundary.get("direct_family_renderer_can_qualify"), False)
    _expect("matrix.qualification_evidence_boundary.caller_supplied_event_lsb_lookbehind_or_policy_allowed", evidence_boundary.get("caller_supplied_event_lsb_lookbehind_or_policy_allowed"), False)
    _expect("matrix.qualification_evidence_boundary.strict_candidate_peak_gate", evidence_boundary.get("strict_candidate_peak_gate"), "abs_peak_strictly_less_than_1.0_before_serialization")

    registry_v1_holdouts = _manifest_holdouts(registry_v1)
    registry_v2_holdouts = _manifest_holdouts(registry_v2)
    _expect("registry v1 active holdout metadata", registry_v1_holdouts, EXPECTED_HOLDOUTS)
    _expect("registry v2 active holdout metadata", registry_v2_holdouts, EXPECTED_HOLDOUTS)
    _expect("registry holdout transition", registry_v2_holdouts, registry_v1_holdouts)
    matrix_holdouts = {
        (row.get("case_id"), row.get("partition"), row.get("source_path"), row.get("sha256"))
        for row in _list("matrix.active_holdout_union", matrix.get("active_holdout_union"))
    }
    _expect("matrix active_holdout_union", matrix_holdouts, EXPECTED_HOLDOUTS)
    _expect("matrix active_holdout_union count", len(matrix.get("active_holdout_union", [])), 9)

    positive_rows = _list("matrix.positive_sources", matrix.get("positive_sources"))
    _expect("matrix positive source count", len(positive_rows), 4)
    positive_by_id: dict[str, dict[str, Any]] = {}
    for row in positive_rows:
        row = _mapping("matrix.positive_sources[]", row)
        case_id = row.get("case_id")
        if case_id in positive_by_id:
            _fail("matrix.positive_sources", f"duplicate {case_id!r}")
        positive_by_id[case_id] = row
    _expect("matrix positive source IDs", set(positive_by_id), set(EXPECTED_POSITIVE_SOURCES))
    registry_v2_by_id = {entry.get("case_id"): entry for entry in registry_v2.get("entries", [])}
    for case_id, expected in EXPECTED_POSITIVE_SOURCES.items():
        row = positive_by_id[case_id]
        for key, value in expected.items():
            _expect(f"matrix.positive_sources.{case_id}.{key}", row.get(key), value)
            _expect(f"registry_v2.entries.{case_id}.{key}", registry_v2_by_id.get(case_id, {}).get(key), value)
        _expect(f"matrix.positive_sources.{case_id}.partition", row.get("partition"), "development")
        _expect(f"matrix.positive_sources.{case_id}.matrix_role", row.get("matrix_role"), "positive_qualification_source")
        _expect(f"matrix.positive_sources.{case_id}.event_qualification", row.get("event_qualification"), "not_started")
        _expect(f"matrix.positive_sources.{case_id}.candidate_render", row.get("candidate_render"), "not_started")
        _expect(f"registry_v2.entries.{case_id}.partition", registry_v2_by_id.get(case_id, {}).get("partition"), "development")
    _expect("matrix positive authors", len({row.get("author") for row in positive_rows}), 4)
    _expect("matrix positive families", {row.get("source_family") for row in positive_rows}, {"dense_break", "sparse_drums", "electronic_drums"})

    source_set = matrix.get("positive_source_set", {})
    _expect("matrix.positive_source_set.exact_source_count", source_set.get("exact_source_count"), 4)
    _expect("matrix.positive_source_set.exact_author_count", source_set.get("exact_author_count"), 4)
    _expect("matrix.positive_source_set.exact_family_count", source_set.get("exact_family_count"), 3)
    _expect("matrix.positive_source_set.required_families", source_set.get("required_families"), ["dense_break", "sparse_drums", "electronic_drums"])
    _expect("matrix.positive_source_set.event_qualification_state", source_set.get("event_qualification_state"), "not_started")

    catalog = matrix.get("event_catalog", {})
    _expect("matrix.event_catalog.state", catalog.get("state"), "pending_mechanism_blind_qualification")
    _expect("matrix.event_catalog.minimum_events_per_source", catalog.get("minimum_events_per_source"), 2)
    _expect("matrix.event_catalog.maximum_frozen_events_per_source", catalog.get("maximum_frozen_events_per_source"), 3)
    _expect("matrix.event_catalog.development_ordinals", catalog.get("development_ordinals"), [1, 2])
    _expect("matrix.event_catalog.confirmation_ordinal", catalog.get("confirmation_ordinal"), 3)
    if "event_records" in catalog:
        _fail("matrix.event_catalog.event_records", "result field must be absent before qualification")

    golden = matrix.get("stage_a_golden_path", {})
    _expect("matrix.golden.case_id", golden.get("case_id"), "oga_cinameng_can_be_so_beautiful")
    _expect("matrix.golden.event_ordinal", golden.get("event_ordinal"), 1)
    _expect("matrix.golden.automatic_fallback", golden.get("automatic_fallback"), False)

    cross = matrix.get("required_cross_product", {})
    _expect("matrix.cross.families", cross.get("families"), ["F1", "F2", "F3"])
    _expect(
        "matrix.cross.family_versions",
        cross.get("family_versions"),
        [
            "f1_ab_energy_redistribution_v1",
            "f2_exact_complementary_three_band_v1",
            "f3_causal_envelope_contrast_dynamic_residual_v2",
        ],
    )
    _expect(
        "matrix.cross.f3_actionable_diversity_component",
        cross.get("f3_actionable_diversity_component"),
        "riotbox.f3_source_response_diversity.v1",
    )
    _expect("matrix.cross.source_count", cross.get("source_count"), 4)
    _expect("matrix.cross.event_ordinals", cross.get("event_ordinals"), [1, 2])
    _expect("matrix.cross.candidate_event_condition_count", cross.get("candidate_event_condition_count"), 24)
    _expect("matrix.cross.failed_source_fails_family", cross.get("failed_source_fails_family"), True)
    _expect("matrix.cross.aggregate_rescue", cross.get("aggregate_rescue"), False)
    _expect("matrix.cross.execution", cross.get("execution"), "not_started")

    shared_analysis = _mapping(
        "matrix.shared_analysis_components", matrix.get("shared_analysis_components")
    )
    _expect(
        "matrix.shared_analysis.exact_three_band_analysis",
        shared_analysis.get("exact_three_band_analysis"),
        "riotbox.exact_complementary_three_band_analysis.v1",
    )
    _expect(
        "matrix.shared_analysis.consumers",
        shared_analysis.get("consumers"),
        ["F2", "static_eq_fit_all_families", "dark_onepole_f25_v1", "bright_exact_split_f75_v1"],
    )
    _contains(
        "matrix.shared_analysis.unavailable_event_semantics",
        shared_analysis.get("unavailable_event_semantics"),
        "rejects_every_family",
    )
    _expect(
        "matrix.rejected_family_history",
        matrix.get("rejected_family_history"),
        {
            "family_version": "f3_os4_onset_residual_v1",
            "state": "immutable_source_independent_preflight_reject_only",
            "active_candidate_or_reference": False,
            "control_reuse": "distortion_os4_d2_v1_non_promotable_negative_confound_only",
        },
    )

    control_contract = matrix.get("false_positive_control_contract", {})
    _expect(
        "matrix.false_positive_control_contract.required_ids",
        control_contract.get("required_ids"),
        [
            "hidden_exact_a_a_v1",
            "gain_plus_3db_v1",
            "global_rate_four_fifths_v1",
            "dark_onepole_f25_v1",
            "bright_exact_split_f75_v1",
            "distortion_os4_d2_v1",
            "delayed_duplicate_25ms_minus6db_v1",
            "detached_click_minus12db_body_v1",
        ],
    )
    _expect("matrix.false_positive_control_contract.may_earn_force", control_contract.get("may_earn_force"), False)
    _contains("matrix.false_positive_control_contract.distortion_os4_boundary", control_contract.get("distortion_os4_boundary"), "non-promotable negative confound only")
    _contains("matrix.false_positive_control_contract.distortion_os4_boundary", control_contract.get("distortion_os4_boundary"), "cannot provide candidate, reference, safety, or quality evidence")

    natural = matrix.get("natural_directional_reference_controls", {})
    _expect("matrix.natural.algorithm_selection_allowed", natural.get("algorithm_selection_allowed"), False)
    _expect("matrix.natural.filename_dynamics_are_ground_truth", natural.get("filename_dynamics_are_ground_truth"), False)
    _expect(
        "matrix.natural.control_set_ids",
        [value.get("control_set_id") for value in natural.get("sets", [])],
        ["philharmonia_snare_with_snares_025", "philharmonia_whip_struck_together_025"],
    )

    refusal = matrix.get("refusal_and_stress_sources", [])
    _expect(
        "matrix.refusal_and_stress_sources.ids",
        [value.get("case_id") for value in refusal],
        [
            "oga_bertsz_dnb",
            "oga_fupi_plimplom",
            "oga_isaiah658_ambient",
            "oga_killerfishred_short_synth",
            "oga_laleksic_tap_water",
        ],
    )
    if any(value.get("positive_coverage") is not False for value in refusal):
        _fail("matrix.refusal_and_stress_sources.positive_coverage", "stress/refusal sources cannot satisfy positive coverage")

    protection = matrix.get("holdout_protection", {})
    _expect("matrix.holdout.reject_partitions", protection.get("reject_partitions"), ["holdout_a", "holdout_b"])
    for key in (
        "read_audio",
        "hash_audio",
        "render_audio",
        "classify_audio",
        "play_audio",
        "glob_or_directory_discovery",
    ):
        _expect(f"matrix.holdout_protection.{key}", protection.get(key), False)
    _expect("matrix.holdout.dynamic_metadata_rejection", protection.get("dynamic_metadata_rejection"), ["case_id", "source_path", "sha256"])

    promotion = matrix.get("promotion", {})
    _expect("matrix.promotion.mechanical_gate_grants", promotion.get("mechanical_gate_grants"), "human_listening_request_only")
    _expect("matrix.promotion.desirable_to_trigger_required", promotion.get("desirable_to_trigger_required"), True)
    _expect("matrix.promotion.unique_candidate_required", promotion.get("unique_candidate_required"), True)
    _expect("matrix.promotion.tie_blocks", promotion.get("tie_blocks"), True)
    _expect("matrix.promotion.runtime_mix_integration_allowed_now", promotion.get("runtime_mix_integration_allowed_now"), False)
    if "human_candidate_evidence" in promotion:
        _fail("matrix.promotion.human_candidate_evidence", "result field must be absent before listening")

    readiness = matrix.get("readiness_scope", {})
    _expect("matrix.readiness_scope.stage_a_positive_floor", readiness.get("stage_a_positive_floor"), "four_sources_four_authors_three_families_isolated_mechanism_gate")
    _contains("matrix.readiness_scope.broader_floor", readiness.get("broader_floor"), "five development sources and four source families")
    _expect("matrix.readiness_scope.broader_floor_claimed", readiness.get("broader_floor_claimed"), False)
    _expect("matrix.readiness_scope.instrument_quality_claimed", readiness.get("instrument_quality_claimed"), False)
    _expect("matrix.readiness_scope.product_path_claimed", readiness.get("product_path_claimed"), False)

    _ensure_no_result_keys(matrix)
    _expect(
        "matrix v2 exact semantic contract SHA-256",
        _semantic_sha256(matrix),
        EXPECTED_MATRIX_V2_SEMANTIC_SHA256,
    )
    if canonical_sha256 is not None:
        _expect("matrix v2 canonical SHA-256", canonical_sha256, EXPECTED_MATRIX_V2_SHA256)


def validate_repository(repo_root: Path) -> dict[str, str]:
    protocol, protocol_sha = _load_named_json(repo_root, PROTOCOL_REL)
    matrix, matrix_sha = _load_named_json(repo_root, MATRIX_REL)
    matrix_v1, matrix_v1_sha = _load_named_json(repo_root, MATRIX_V1_REL)
    registry_v1, registry_v1_sha = _load_named_json(repo_root, REGISTRY_V1_REL)
    registry_v2, registry_v2_sha = _load_named_json(repo_root, REGISTRY_V2_REL)
    _expect("matrix v1 schema", matrix_v1.get("schema"), "riotbox.percussive_force_development_matrix.v1")
    validate_protocol(protocol, canonical_sha256=protocol_sha)
    validate_matrix(
        matrix,
        protocol_sha256=protocol_sha,
        matrix_v1_sha256=matrix_v1_sha,
        registry_v1=registry_v1,
        registry_v1_sha256=registry_v1_sha,
        registry_v2=registry_v2,
        registry_v2_sha256=registry_v2_sha,
        canonical_sha256=matrix_sha,
    )
    return {
        "protocol_sha256": protocol_sha,
        "matrix_sha256": matrix_sha,
        "matrix_v1_sha256": matrix_v1_sha,
        "registry_v1_sha256": registry_v1_sha,
        "registry_v2_sha256": registry_v2_sha,
    }


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    try:
        hashes = validate_repository(repo_root)
    except (ContractError, OSError) as exc:
        print(f"FAIL: {exc}")
        return 1
    print("PASS: RIOTBOX-1428 Stage-A preregistration is frozen and metadata-only")
    for name, value in hashes.items():
        print(f"{name}={value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
