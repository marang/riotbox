#[test]
fn signal_metrics_reports_shape_metrics_beyond_level() {
    let metrics = signal_metrics(&[-0.5, 0.25, -0.25, 0.5]);

    assert_eq!(metrics.active_samples, 4);
    assert_eq!(metrics.peak_abs, 0.5);
    assert_eq!(metrics.clip_count, 0);
    assert_eq!(metrics.near_clip_count, 0);
    assert_eq!(metrics.headroom_to_full_scale, 0.5);
    assert_eq!(metrics.sum, 0.0);
    assert_eq!(metrics.mean_abs, 0.375);
    assert_eq!(metrics.zero_crossings, 3);
    assert_eq!(metrics.active_sample_ratio, 1.0);
    assert_eq!(metrics.silence_ratio, 0.0);
    assert_eq!(metrics.dc_offset, 0.0);
    assert_eq!(metrics.onset_count, 1);
    assert_eq!(metrics.event_density_per_bar, 0.0);
    assert!((metrics.rms - 0.395_284_7).abs() < 0.000_001);
    assert!((metrics.crest_factor - 1.264_911).abs() < 0.000_001);
}

#[test]
fn signal_metrics_reports_activity_ratio_silence_and_dc_offset() {
    let metrics = signal_metrics(&[0.0, 0.0, 0.2, 0.4]);

    assert_eq!(metrics.active_samples, 2);
    assert_eq!(metrics.active_sample_ratio, 0.5);
    assert_eq!(metrics.silence_ratio, 0.5);
    assert_eq!(metrics.dc_offset, 0.15);
    assert_eq!(metrics.onset_count, 1);
}

#[test]
fn signal_metrics_reports_clip_counts_and_headroom() {
    let metrics = signal_metrics(&[-1.2, -1.0, -0.985, 0.25, 0.99, 1.0, 1.15]);

    assert_eq!(metrics.peak_abs, 1.2);
    assert_eq!(metrics.clip_count, 4);
    assert_eq!(metrics.near_clip_count, 6);
    assert!((metrics.headroom_to_full_scale + 0.2).abs() < 0.000_001);
}

#[test]
fn signal_metrics_with_grid_reports_onsets_and_event_density_per_bar() {
    let samples = [
        0.0, 0.0, 0.4, 0.4, 0.2, 0.2, 0.0, 0.0, 0.35, 0.35, 0.1, 0.1, 0.0, 0.0,
        0.0, 0.0,
    ];
    let metrics = signal_metrics_with_grid(&samples, 8, 2, 120.0, 4);

    assert_eq!(metrics.onset_count, 2);
    assert_eq!(metrics.event_density_per_bar, 4.0);
}

#[test]
fn signal_metrics_with_grid_leaves_density_zero_without_valid_timing_context() {
    let metrics = signal_metrics_with_grid(&[0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.5, 0.5], 0, 2, 120.0, 4);

    assert_eq!(metrics.onset_count, 2);
    assert_eq!(metrics.event_density_per_bar, 0.0);
}

#[test]
fn signal_delta_metrics_reports_audible_difference_shape() {
    let metrics = signal_delta_metrics(&[0.0, 0.5, -0.5], &[0.0, 0.25, 0.5]);

    assert_eq!(metrics.active_samples, 2);
    assert_eq!(metrics.peak_abs, 1.0);
    assert_eq!(metrics.zero_crossings, 1);
    assert!((metrics.rms - 0.595_119).abs() < 0.000_001);
}

#[test]
fn signal_delta_metrics_counts_unmatched_tail_samples() {
    let metrics = signal_delta_metrics(&[0.0, 0.25, -0.75], &[0.0]);

    assert_eq!(metrics.active_samples, 2);
    assert_eq!(metrics.peak_abs, 0.75);
    assert_eq!(metrics.zero_crossings, 1);
}

#[test]
fn master_bus_limiter_controls_clips_without_flattening_transients() {
    let mut samples = [0.0, 0.25, 0.94, 1.20, -1.12, 0.55, -0.97, 0.12];
    let report = apply_master_bus_soft_limiter_with_report(&mut samples);

    assert!(report.applied);
    assert!(report.limited_sample_count >= 4);
    assert_eq!(report.pre.clip_count, 2);
    assert_eq!(report.post.clip_count, 0);
    assert!(report.post.peak_abs <= master_bus_limiter_ceiling() + 0.000_001);
    assert!(report.post.peak_abs > master_bus_limiter_threshold());
    assert!(report.post.rms > report.pre.rms * 0.80);
    assert_eq!(samples[1], 0.25);
}

#[test]
fn master_bus_limiter_does_not_mask_weak_or_silent_output() {
    let mut weak = [0.0, 0.000_05, -0.000_04, 0.000_03];
    let report = apply_master_bus_soft_limiter_with_report(&mut weak);

    assert!(!report.applied);
    assert_eq!(report.limited_sample_count, 0);
    assert_eq!(report.pre.rms, report.post.rms);
    assert_eq!(report.pre.active_samples, report.post.active_samples);
    assert!(report.post.rms < 0.001);
}
