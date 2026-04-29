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
