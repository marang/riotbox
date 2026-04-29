#[test]
fn signal_metrics_reports_shape_metrics_beyond_level() {
    let metrics = signal_metrics(&[-0.5, 0.25, -0.25, 0.5]);

    assert_eq!(metrics.active_samples, 4);
    assert_eq!(metrics.peak_abs, 0.5);
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
