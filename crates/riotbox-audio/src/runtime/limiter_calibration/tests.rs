use super::{CalibrationInputError, Policy, compare, render_sequence};
use crate::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState, apply_master_bus_soft_limiter_with_report,
        master_bus_limiter_ceiling, master_bus_limiter_threshold,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report, signal_metrics,
    },
    source_audio::SourceAudioCache,
    tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
};
use riotbox_core::action::SourceMonitorMode;

fn bits(samples: &[f32]) -> Vec<u32> {
    samples.iter().map(|sample| sample.to_bits()).collect()
}

#[test]
fn policies_bind_the_inherited_baseline_and_only_two_declared_alternatives() {
    assert_eq!(master_bus_limiter_threshold().to_bits(), 0.92_f32.to_bits());
    assert_eq!(master_bus_limiter_ceiling().to_bits(), 0.985_f32.to_bits());
    for (policy, (threshold, ceiling)) in
        Policy::ALL
            .into_iter()
            .zip([(0.92_f32, 0.985_f32), (0.9525, 0.985), (0.92, 0.9525)])
    {
        assert_eq!(policy.threshold().to_bits(), threshold.to_bits());
        assert_eq!(policy.ceiling().to_bits(), ceiling.to_bits());
    }
}

#[test]
fn baseline_preserves_original_operations_bits_reports_and_actual_write_count() {
    let knee = 0.92_f32;
    let mut input = vec![
        0.0,
        -0.0,
        knee,
        -knee,
        f32::from_bits(knee.to_bits() + 1),
        -f32::from_bits(knee.to_bits() + 1),
    ];
    input.extend((-2_500..=2_500).map(|value| value as f32 / 1_024.0));
    // Independent, frozen v1 formula oracle, deliberately confined to this test.
    let mut expected = input.clone();
    let mut writes = 0;
    for sample in &mut expected {
        let magnitude = sample.abs();
        let shaped = if magnitude <= 0.92 {
            *sample
        } else {
            let width = 0.985_f32 - 0.92_f32;
            let excess = ((magnitude - 0.92) / width).tanh();
            sample.signum() * (0.92 + width * excess).min(0.985)
        };
        if (shaped - *sample).abs() > f32::EPSILON {
            *sample = shaped;
            writes += 1;
        }
    }
    let mut production = input.clone();
    let production_report = apply_master_bus_soft_limiter_with_report(&mut production);
    let [baseline, _, _] = compare(&input).unwrap();
    assert_eq!(bits(&baseline.samples), bits(&expected));
    assert_eq!(bits(&production), bits(&expected));
    assert_eq!(baseline.limiter, production_report);
    assert_eq!(baseline.limiter.limited_sample_count, writes);
    assert_eq!(baseline.limiter.pre, signal_metrics(&input));
    assert_eq!(baseline.limiter.post, signal_metrics(&baseline.samples));
}

#[test]
fn immediate_above_knee_neighbors_do_not_become_false_writes() {
    for policy in Policy::ALL {
        let knee = policy.threshold();
        let above = f32::from_bits(knee.to_bits() + 1);
        let input = [0.0, -0.0, knee, -knee, above, -above];
        let output = compare(&input)
            .unwrap()
            .into_iter()
            .find(|output| output.policy == policy)
            .unwrap();
        assert!(above > knee);
        assert_eq!(bits(&output.samples), bits(&input));
        assert!(!output.limiter.applied);
        assert_eq!(output.limiter.limited_sample_count, 0);
        assert_eq!(output.limiter.pre, output.limiter.post);
    }
}

#[test]
fn alternatives_are_independent_bounded_odd_monotone_and_never_boost() {
    let input = [0.0, 0.25, 0.92, 0.94, 0.9525, 0.98, 0.985, 1.0, 1.25, 4.0];
    let original_bits = bits(&input);
    let outputs = compare(&input).unwrap();
    let negatives = compare(&input.map(|sample| -sample)).unwrap();
    assert_eq!(bits(&input), original_bits);
    assert_eq!(outputs[1].samples[3].to_bits(), input[3].to_bits());
    assert_ne!(outputs[0].samples[3].to_bits(), input[3].to_bits());
    assert_ne!(
        outputs[2].samples[3].to_bits(),
        outputs[0].samples[3].to_bits()
    );
    for (output, negative) in outputs.iter().zip(&negatives) {
        assert_eq!(output.limiter.pre, signal_metrics(&input));
        assert_eq!(output.limiter.post.clip_count, 0);
        assert!(output.samples.windows(2).all(|pair| pair[0] <= pair[1]));
        for ((before, after), opposite) in input.iter().zip(&output.samples).zip(&negative.samples)
        {
            assert!(after.is_finite());
            assert!(after.abs() <= output.policy.ceiling());
            assert!(after.abs() <= before.abs());
            assert_eq!((-after).to_bits(), opposite.to_bits());
        }
        let writes = input
            .iter()
            .zip(&output.samples)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(output.limiter.limited_sample_count, writes);
        assert_eq!(output.limiter.applied, writes > 0);
    }
}

#[test]
fn comparison_preserves_empty_control_and_rejects_nonfinite_input() {
    for output in compare(&[]).unwrap() {
        assert!(output.samples.is_empty());
        assert!(!output.limiter.applied);
        assert_eq!(output.limiter.limited_sample_count, 0);
        assert_eq!(output.limiter.pre, output.limiter.post);
    }
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            compare(&[0.0, invalid]),
            Err(CalibrationInputError::NonFiniteSample { sample_index: 1 })
        );
    }
}

#[test]
fn policy_output_and_write_counts_are_partition_and_repeat_identical() {
    let input = [0.0, -0.0, 0.92, -0.94, 0.9525, -0.985, 1.25, -1.25].repeat(193);
    let expected = compare(&input).unwrap();
    assert_eq!(expected, compare(&input).unwrap());
    for chunk_size in [1, 127, 128, 257] {
        let mut samples: [Vec<f32>; 3] = std::array::from_fn(|_| Vec::new());
        let mut writes = [0; 3];
        for chunk in input.chunks(chunk_size) {
            for (index, output) in compare(chunk).unwrap().into_iter().enumerate() {
                samples[index].extend(output.samples);
                writes[index] += output.limiter.limited_sample_count;
            }
        }
        for (index, expected) in expected.iter().enumerate() {
            assert_eq!(bits(&samples[index]), bits(&expected.samples));
            assert_eq!(writes[index], expected.limiter.limited_sample_count);
        }
    }
}

#[test]
fn frozen_baseline_v1_stateless_catalog_reports_all_three_policies() {
    // Match the owning v1 buffers exactly. Its five-owner composite remains a
    // separate unchanged A-only control, not an ABC catalog-completeness claim.
    let knee = 0.92_f32;
    let ceiling = 0.985_f32;
    let mut quiet = vec![0.0, -0.0, 0.000_05, -0.000_05];
    for neighbor in [
        f32::from_bits(knee.to_bits() - 1),
        knee,
        f32::from_bits(knee.to_bits() + 1),
    ] {
        quiet.extend([neighbor, -neighbor]);
    }
    let positive_ladder = [0.0, 0.25, knee, 0.94, ceiling, 1.0, 1.2, 4.0];
    let ladder = positive_ladder
        .iter()
        .flat_map(|value| [*value, -*value])
        .collect();
    let mut transient = vec![0.0; 256];
    transient[63] = 1.25;
    transient[64] = -1.25;
    let partition = [
        0.0, 0.25, -0.25, knee, -knee, 0.94, -0.94, 1.25, -1.25, 4.0, -4.0,
    ]
    .repeat(193);
    for (name, input, expected_writes, expected_clips) in [
        ("empty", Vec::new(), [0; 3], 0),
        ("quiet_knee_neighbors", quiet, [0; 3], 0),
        ("signed_overload_ladder", ladder, [10, 8, 10], 6),
        ("sparse_transient", transient, [2; 3], 2),
        (
            "sustained_overload",
            [4.0, -4.0].repeat(512),
            [1_024; 3],
            1_024,
        ),
        (
            "partition_repeat",
            partition,
            [6 * 193, 4 * 193, 6 * 193],
            4 * 193,
        ),
    ] {
        let outputs = compare(&input).unwrap();
        for output in &outputs {
            eprintln!(
                "baseline_v1_abc {name} {:?}: {:?}",
                output.policy, output.limiter
            );
        }
        let mut production = input.clone();
        let production_report = apply_master_bus_soft_limiter_with_report(&mut production);
        assert_eq!(bits(&outputs[0].samples), bits(&production));
        assert_eq!(outputs[0].limiter, production_report);
        assert_eq!(outputs, compare(&input).unwrap());
        for (output, writes) in outputs.iter().zip(expected_writes) {
            assert_eq!(output.samples.len(), input.len());
            assert_eq!(output.limiter.pre, signal_metrics(&input));
            assert_eq!(output.limiter.post, signal_metrics(&output.samples));
            assert_eq!(output.limiter.pre.clip_count, expected_clips);
            assert_eq!(output.limiter.limited_sample_count, writes);
            assert_eq!(output.limiter.applied, writes > 0);
            assert_eq!(output.limiter.post.clip_count, 0);
            assert!(output.limiter.post.peak_abs <= output.policy.ceiling());
            for (before, after) in input.iter().zip(&output.samples) {
                assert!(after.is_finite());
                assert!(after.abs() <= before.abs());
                if writes == 0 || *before == 0.0 {
                    assert_eq!(after.to_bits(), before.to_bits());
                }
            }
        }
        for chunk_size in [1, 127, 128, 257] {
            let mut partitioned: [Vec<f32>; 3] = std::array::from_fn(|_| Vec::new());
            let mut writes = [0; 3];
            for chunk in input.chunks(chunk_size) {
                for (index, output) in compare(chunk).unwrap().into_iter().enumerate() {
                    partitioned[index].extend(output.samples);
                    writes[index] += output.limiter.limited_sample_count;
                }
            }
            for (index, expected) in outputs.iter().enumerate() {
                assert_eq!(bits(&partitioned[index]), bits(&expected.samples));
                assert_eq!(writes[index], expected.limiter.limited_sample_count);
            }
        }
    }
}

fn monitor_plan(channel_count: u16, hot_drums: bool) -> RuntimeMixRenderPlan {
    let samples = (0..2_048)
        .flat_map(|_| (0..channel_count).map(|channel| if channel == 0 { 1.3 } else { -0.65 }))
        .collect();
    let cache = SourceAudioCache::from_interleaved_samples(
        "synthetic-calibration-no-file.wav",
        48_000,
        channel_count,
        samples,
    )
    .unwrap();
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        source_monitor_render: SourceMonitorRenderState {
            source_anchor_seconds: Some(0.0),
            ..SourceMonitorRenderState::from_source_cache(
                if hot_drums {
                    SourceMonitorMode::Blend
                } else {
                    SourceMonitorMode::Source
                },
                Some(&cache),
            )
        },
        tr909_render: if hot_drums {
            Tr909RenderState {
                mode: Tr909RenderMode::Fill,
                routing: Tr909RenderRouting::DrumBusSupport,
                drum_bus_level: 4.0,
                slam_intensity: 2.5,
                ..Tr909RenderState::default()
            }
        } else {
            Tr909RenderState::default()
        },
        ..RuntimeMixRenderPlan::default()
    }
}

#[test]
fn retained_pre_pcm_and_baseline_match_the_existing_mix_for_channels_and_steps() {
    assert!(render_sequence(&[], 48_000, 2, 128).is_empty());
    for channels in [1, 2] {
        for hot_drums in [false, true] {
            let plan = monitor_plan(channels, hot_drums);
            let steps = [
                RuntimeMixRenderSequenceStep::new(&plan, 0),
                RuntimeMixRenderSequenceStep::new(&plan, 257),
                RuntimeMixRenderSequenceStep::new(&plan, 511),
            ];
            for partition in [1, 127, 128, 257] {
                let existing =
                    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
                        &steps, 48_000, channels, partition,
                    );
                let observed = render_sequence(&steps, 48_000, channels, partition);
                assert_eq!(observed.len(), steps.len());
                for ((evidence, existing), step) in observed.iter().zip(existing).zip(steps) {
                    assert_eq!(
                        evidence.pre_samples.len(),
                        step.frame_count * usize::from(channels)
                    );
                    assert_eq!(bits(&evidence.baseline.samples), bits(&existing.samples));
                    assert_eq!(evidence.baseline.limiter, existing.limiter);
                    let [baseline, _, _] = compare(&evidence.pre_samples).unwrap();
                    assert_eq!(bits(&baseline.samples), bits(&existing.samples));
                    assert_eq!(baseline.limiter, existing.limiter);
                }
                assert!(observed[1].baseline.limiter.applied || hot_drums);
            }
        }
    }
}

#[test]
fn stereo_monitor_channels_and_pre_pcm_survive_callback_partitions_exactly() {
    let plan = monitor_plan(2, false);
    let steps = [RuntimeMixRenderSequenceStep::new(&plan, 1_024)];
    let expected = render_sequence(&steps, 48_000, 2, 128).pop().unwrap();
    for frame in expected.pre_samples.as_chunks::<2>().0 {
        assert!(frame[0] > 1.0);
        assert!(frame[1] < 0.0 && frame[1].abs() < 0.92);
    }
    for partition in [1, 127, 257, 1_024] {
        let observed = render_sequence(&steps, 48_000, 2, partition).pop().unwrap();
        assert_eq!(bits(&observed.pre_samples), bits(&expected.pre_samples));
        assert_eq!(
            bits(&observed.baseline.samples),
            bits(&expected.baseline.samples)
        );
        assert_eq!(observed.baseline.limiter, expected.baseline.limiter);
    }
}
