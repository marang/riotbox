use super::{CallbackPreparationError, prepare_output_callback};
use crate::{
    mc202::Mc202RenderState,
    runtime::{
        live_master_capture::{LiveMasterCaptureRequest, SharedLiveMasterCapture},
        shared_mc202::SharedMc202RenderState,
        shared_transport_tr909::{
            AudioRuntimeSharedState, SharedTr909RenderState, SharedTransportTimingState,
        },
        shared_w30_resample_callback::SharedW30ResampleTapState,
        source_monitor::{
            SharedSourceMonitorRenderState, SourceMonitorAudioSource, SourceMonitorRenderState,
        },
        telemetry::RuntimeTelemetry,
        w30_preview_snapshot::SharedW30PreviewRenderState,
    },
    source_audio::SourceAudioCache,
    test_heap::{HeapCounts, measure},
    tr909::Tr909RenderState,
    w30::{W30PreviewRenderState, W30ResampleTapState},
};
use cpal::Sample;
use riotbox_core::action::SourceMonitorMode;
use std::{process::Command, sync::Arc, time::Instant};

const COLD_TEST: &str = "runtime::shared_transport_tr909::output_callback::tests::cold_first_production_callback_has_no_heap_operations";
const COLD_CHILD: &str = "RIOTBOX_CALLBACK_HEAP_COLD_CHILD";

fn shared() -> AudioRuntimeSharedState {
    AudioRuntimeSharedState {
        telemetry: Arc::new(RuntimeTelemetry::new()),
        transport: Arc::new(SharedTransportTimingState::new(false, 128.0, 0.0)),
        tr909_render: Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default())),
        mc202_render: Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default())),
        w30_preview: Arc::new(SharedW30PreviewRenderState::new(
            &W30PreviewRenderState::default(),
        )),
        w30_resample_tap: Arc::new(SharedW30ResampleTapState::new(
            &W30ResampleTapState::default(),
        )),
        source_monitor: Arc::new(SharedSourceMonitorRenderState::new(
            &SourceMonitorRenderState::default(),
        )),
        live_master_capture: Arc::new(SharedLiveMasterCapture::new()),
    }
}

fn config() -> cpal::StreamConfig {
    cpal::StreamConfig {
        channels: 2,
        sample_rate: 48_000,
        buffer_size: cpal::BufferSize::Fixed(64),
    }
}

fn source(sample: f32) -> SourceMonitorRenderState {
    let cache = SourceAudioCache::from_interleaved_samples(
        "generated-callback-only.wav",
        48_000,
        2,
        vec![sample; 8_192],
    )
    .unwrap();
    SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&cache)),
        ..SourceMonitorRenderState::default()
    }
}

fn request(frames: usize) -> LiveMasterCaptureRequest {
    LiveMasterCaptureRequest {
        target_frame_count: frames,
        channel_count: 2,
        expected_tempo_bpm: 128.0,
        start_position_beats: None,
    }
}

fn isolated(test: &str, case: &str) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env("RIOTBOX_CALLBACK_HEAP_CASE", case)
        .output()
        .expect("isolated generated callback test");
    assert!(
        output.status.success(),
        "{case}:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn cold_source_capture_and_overflow_formats_have_no_heap_operations() {
    const TEST: &str = "runtime::shared_transport_tr909::output_callback::tests::cold_source_capture_and_overflow_formats_have_no_heap_operations";
    let Some(case) = std::env::var_os("RIOTBOX_CALLBACK_HEAP_CASE") else {
        for case in [
            "f32",
            "i16",
            "u16",
            "overflow-f32",
            "overflow-i16",
            "overflow-u16",
            "active-mix",
        ] {
            isolated(TEST, case);
        }
        return;
    };
    match case.to_str().unwrap() {
        "f32" => cold_source_capture::<f32>(false),
        "i16" => cold_source_capture::<i16>(false),
        "u16" => cold_source_capture::<u16>(false),
        "overflow-f32" => cold_source_capture::<f32>(true),
        "overflow-i16" => cold_source_capture::<i16>(true),
        "overflow-u16" => cold_source_capture::<u16>(true),
        "active-mix" => cold_active_mix(),
        _ => panic!("unknown cold test case"),
    }
}

fn cold_active_mix() {
    use crate::mc202::{Mc202RenderMode, Mc202RenderRouting, Mc202SourcePhraseRenderPlan};
    use crate::tr909::{Tr909RenderMode, Tr909RenderRouting};
    use crate::w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30_RESAMPLE_SOURCE_WINDOW_LEN, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewSampleWindow, W30PreviewSourceProfile,
        W30ResampleSourceWindow, W30ResampleTapAvailability, W30ResampleTapMode,
        W30ResampleTapRouting,
    };
    let state = shared();
    state.transport.update(true, 128.0, 0.0);
    state
        .source_monitor
        .update_controls(&SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ));
    state.tr909_render.update(&Tr909RenderState {
        mode: Tr909RenderMode::Takeover,
        routing: Tr909RenderRouting::DrumBusTakeover,
        drum_bus_level: 0.8,
        ..Tr909RenderState::default()
    });
    state.mc202_render.update(&Mc202RenderState {
        mode: Mc202RenderMode::Leader,
        routing: Mc202RenderRouting::MusicBusBass,
        source_phrase_plan: Some(Mc202SourcePhraseRenderPlan {
            active_mask: 0xffff,
            semitones: [-12; 16],
            accent_mask: 1,
            destructive_mask: 0,
            pressure: 0.7,
            contrast: 0.5,
            bass_weight: 0.7,
            stab_bite: 0.3,
            gate_snap: 0.2,
        }),
        ..Mc202RenderState::default()
    });
    state.w30_preview.update(&W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        source_window_preview: Some(W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: [0.2; W30_PREVIEW_SAMPLE_WINDOW_LEN],
        }),
        music_bus_level: 0.6,
        grit_level: 0.4,
        ..W30PreviewRenderState::default()
    });
    state.w30_resample_tap.update(&W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        availability: W30ResampleTapAvailability::SourceAudioReady,
        lineage_capture_count: 1,
        generation_depth: 1,
        source_audio: Some(Box::new(W30ResampleSourceWindow {
            source_start_frame: 0,
            source_sample_rate: 48_000,
            source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
            sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
            samples: [0.1; W30_RESAMPLE_SOURCE_WINDOW_LEN],
        })),
        music_bus_level: 0.5,
        grit_level: 0.3,
        ..W30ResampleTapState::default()
    });
    state
        .live_master_capture
        .begin_after_callback(request(64), None)
        .unwrap();
    let mut callback =
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()).unwrap();
    let first = std::thread::spawn(move || {
        let mut output = [0.0; 128];
        let (_, cold) = measure(|| callback(&mut output));
        let first = output;
        let (_, warm) = measure(|| callback(&mut output));
        assert_eq!(cold, HeapCounts::default());
        assert_eq!(warm, HeapCounts::default());
        println!("cold_active_mix={cold:?} warm={warm:?}");
        first
    })
    .join()
    .unwrap();
    assert!(first.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    assert!(first.iter().any(|s| s.abs() > 0.01));
    assert_eq!(state.live_master_capture.finish().unwrap().samples, first);
    drop(state);
}

fn cold_source_capture<T>(overflow: bool)
where
    T: cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let state = shared();
    state
        .source_monitor
        .replace_source_and_controls(&source(0.25));
    state.transport.update(true, 128.0, 0.0);
    state
        .live_master_capture
        .begin_after_callback(request(64), None)
        .unwrap();
    let mut callback =
        prepare_output_callback::<T>(&config(), state.clone(), Instant::now()).unwrap();
    let device_samples = std::thread::spawn(move || {
        let mut output = [T::from_sample(0.0); 128];
        if overflow {
            let mut oversized = [T::from_sample(1.0); 256];
            let (_, cold) = measure(|| callback(&mut oversized));
            assert_eq!(cold, HeapCounts::default(), "cold overflow callback");
            assert!(oversized.iter().all(|s| f32::from_sample(*s) == 0.0));
            println!("cold_overflow={cold:?}");
        }
        let (_, first) = measure(|| callback(&mut output));
        let captured_device_samples = output.map(f32::from_sample);
        let (_, warm) = measure(|| callback(&mut output));
        assert_eq!(first, HeapCounts::default());
        assert_eq!(warm, HeapCounts::default());
        println!("source_capture_callback={first:?} warm={warm:?}");
        captured_device_samples
    })
    .join()
    .unwrap();
    let outcome = state.live_master_capture.finish().unwrap();
    assert_eq!(outcome.samples.len(), 128);
    assert!(
        outcome
            .samples
            .iter()
            .all(|s| s.is_finite() && *s >= 0.0 && *s <= 0.22 + 1e-6)
    );
    assert!(
        outcome.samples.iter().any(|s| *s > 0.01),
        "real processor must produce generated PCM"
    );
    for (capture, converted) in outcome.samples.iter().zip(device_samples) {
        assert!(
            (*capture - converted).abs() <= 1.0 / 32_768.0 + 1e-6,
            "capture tap must precede only sample-format conversion"
        );
    }
    assert_eq!(
        outcome.progress.callback_scratch_overflow_count,
        u64::from(overflow)
    );
    // A paused test process may legitimately exceed the host-gap threshold;
    // this heap/identity probe is not a device scheduling/deadline gate.
    assert_eq!(
        outcome.progress.fault_count(),
        u64::from(overflow) + outcome.progress.callback_gap_over_threshold_count
    );
    drop(state); // Input/control ownership survives reader/worker teardown.
}

#[test]
fn preparation_rejects_duplicate_consumers_without_render_fallback() {
    let state = shared();
    let first = prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()).unwrap();
    assert!(matches!(
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()),
        Err(CallbackPreparationError::SourceReaderAlreadyTaken)
    ));
    drop(first);
    assert!(matches!(
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()),
        Err(CallbackPreparationError::SourceReaderAlreadyTaken)
    ));

    let other = shared();
    let held = other.live_master_capture.take_callback_reader().unwrap();
    assert!(matches!(
        prepare_output_callback::<f32>(&config(), other.clone(), Instant::now()),
        Err(CallbackPreparationError::CaptureReaderAlreadyTaken)
    ));
    drop(held);
}

#[test]
fn production_callbacks_remain_heap_free_through_publication_capture_and_stop() {
    use std::sync::atomic::Ordering;
    use std::sync::mpsc;
    let state = shared();
    state
        .source_monitor
        .replace_source_and_controls(&source(0.25));
    state.transport.update(true, 128.0, 0.0);
    let mut callback =
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()).unwrap();
    let (commands, commands_in) = mpsc::channel::<bool>();
    let (answers, answers_out) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        while let Ok(overflow) = commands_in.recv() {
            let mut output = [0.0; 256];
            let len = if overflow { 256 } else { 128 };
            let (_, counts) = measure(|| callback(&mut output[..len]));
            answers.send((output, counts)).unwrap(); // Outside measured invocation.
        }
    });
    let tick = |overflow| {
        commands.send(overflow).unwrap();
        let (output, counts) = answers_out.recv().unwrap();
        assert_eq!(counts, HeapCounts::default());
        assert!(output.iter().all(|s| s.is_finite()));
        output
    };
    assert!(tick(false).iter().any(|s| *s > 0.01));
    state
        .live_master_capture
        .begin_after_callback(request(64), None)
        .unwrap();
    let output = tick(false);
    let outcome = state.live_master_capture.finish().unwrap();
    assert_eq!(outcome.samples, output[..128]);
    tick(false); // Fetch detached None; no callback-side payload reclamation.

    state
        .source_monitor
        .replace_source_and_controls(&source(-0.5));
    state
        .source_monitor
        .update_controls(&SourceMonitorRenderState::control_only(
            SourceMonitorMode::Source,
        ));
    assert!(tick(false).iter().any(|s| *s < -0.01));

    // Drive rejected coherent reads in the exact processor. Existing snapshot
    // module tests separately prove field identity; here the gate is heap-free
    // complete-fallback execution, not a new musical/coherence qualification.
    state.w30_preview.revision.fetch_add(1, Ordering::AcqRel);
    state
        .w30_preview
        .music_bus_level_bits
        .store(f32::NAN.to_bits(), Ordering::Relaxed);
    state.tr909_render.revision.fetch_add(1, Ordering::AcqRel);
    state
        .tr909_render
        .drum_bus_level_bits
        .store(f32::NAN.to_bits(), Ordering::Relaxed);
    tick(false);
    state
        .w30_preview
        .music_bus_level_bits
        .store(0.0_f32.to_bits(), Ordering::Relaxed);
    state.w30_preview.revision.fetch_add(1, Ordering::Release);
    state
        .tr909_render
        .drum_bus_level_bits
        .store(0.0_f32.to_bits(), Ordering::Relaxed);
    state.tr909_render.revision.fetch_add(1, Ordering::Release);
    tick(false); // Changed complete snapshots, then unchanged reuse.
    tick(false);

    state
        .live_master_capture
        .begin_after_callback(request(128), None)
        .unwrap();
    tick(true);
    assert_eq!(
        state
            .live_master_capture
            .progress()
            .unwrap()
            .callback_scratch_overflow_count,
        1
    );
    tick(false);
    let aborted = state.live_master_capture.abort().unwrap();
    assert_eq!(aborted.written_sample_count, 128);
    tick(false);
    assert!(state.live_master_capture.progress().is_none());
    state
        .live_master_capture
        .begin_after_callback(request(64), None)
        .unwrap();
    tick(false);
    assert_eq!(
        state
            .live_master_capture
            .finish()
            .unwrap()
            .progress
            .fault_count(),
        0
    );

    state
        .live_master_capture
        .begin_after_callback(
            LiveMasterCaptureRequest {
                start_position_beats: Some(4.0),
                ..request(64)
            },
            None,
        )
        .unwrap();
    tick(false); // Actual processor stays armed before the requested boundary.
    let armed = state.live_master_capture.progress().unwrap();
    assert_eq!(armed.written_sample_count, 0);
    assert_eq!(armed.armed_callback_count, 1);
    assert!(!armed.capture_started);
    state.transport.update(true, 128.0, 4.0);
    tick(false);
    let aligned = state.live_master_capture.finish().unwrap();
    assert_eq!(aligned.captured_start_position_beats, Some(4.0));
    assert_eq!(aligned.samples.len(), 128);
    assert_eq!(
        aligned.progress.fault_count(),
        aligned.progress.callback_gap_over_threshold_count
    );

    state
        .source_monitor
        .replace_source_and_controls(&source(0.25));
    state.transport.update(true, 128.0, 0.0);
    assert!(tick(false).iter().any(|s| *s > 0.01));
    state.transport.update(false, 128.0, 0.0);
    // Preserve the existing 5 ms stop ramp (240 frames at 48 kHz), not an
    // invented first-buffer hard mute. Every ramp invocation is measured.
    for _ in 0..3 {
        tick(false);
    }
    let ramp_end = tick(false); // 256 frames: the final 16 must be silent.
    assert!(ramp_end[96..128].iter().all(|s| *s == 0.0));
    assert!(
        tick(false).iter().all(|s| *s == 0.0),
        "stopped present-source output must be silent after its bounded ramp"
    );

    state
        .source_monitor
        .replace_source_and_controls(&SourceMonitorRenderState::default());
    state.transport.update(true, 128.0, 0.0);
    assert!(
        tick(false).iter().all(|s| *s == 0.0),
        "missing source fails silent"
    );
    state.transport.update(false, 128.0, 0.0);
    assert!(tick(false).iter().all(|s| *s == 0.0));
    drop(commands);
    worker.join().unwrap();
    drop(state);
}

#[test]
fn concurrent_control_publication_and_capture_abort_do_not_reclaim_on_the_callback() {
    use std::sync::Barrier;
    let state = shared();
    let positive = source(0.25);
    let negative = source(-0.5);
    state.source_monitor.replace_source_and_controls(&positive);
    state.transport.update(true, 128.0, 0.0);
    let mut callback =
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()).unwrap();
    let rendezvous = Arc::new(Barrier::new(2));
    let worker_start = Arc::clone(&rendezvous);
    let worker = std::thread::spawn(move || {
        let mut output = [0.0; 128];
        worker_start.wait(); // Synchronization is outside the measured callback.
        for _ in 0..512 {
            let (_, counts) = measure(|| callback(&mut output));
            assert_eq!(counts, HeapCounts::default());
            assert!(output.iter().all(|s| s.is_finite()));
        }
    });
    rendezvous.wait();
    for index in 0..64 {
        state
            .source_monitor
            .replace_source_and_controls(if index % 2 == 0 { &negative } else { &positive });
        state
            .live_master_capture
            .begin_after_callback(request(128), None)
            .unwrap();
        state
            .live_master_capture
            .abort()
            .expect("nonwaiting control abort");
    }
    worker.join().unwrap();
    assert!(state.live_master_capture.progress().is_none());
    // This is bounded two-thread stress, not exhaustive interleaving proof;
    // canonical admission interleavings are separately exercised by Loom.
    drop(state);
}

#[test]
fn cold_first_production_callback_has_no_heap_operations() {
    if std::env::var_os(COLD_CHILD).as_deref() != Some(std::ffi::OsStr::new("1")) {
        // A mere fresh thread can reuse dependency TLS debt nodes left by other
        // tests. A fresh test process with one exact test prevents that masking.
        let output = Command::new(std::env::current_exe().expect("unit test executable"))
            .args(["--exact", COLD_TEST, "--nocapture", "--test-threads=1"])
            .env(COLD_CHILD, "1")
            .output()
            .expect("isolated source-free callback test");
        assert!(
            output.status.success(),
            "isolated cold callback failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    // This is the processor CPAL invokes, prepared on the control thread and
    // transferred to a different worker without any snapshot warmup there.
    let state = shared();
    let mut callback = prepare_output_callback::<f32>(&config(), state.clone(), Instant::now())
        .expect("unique callback preparation");
    std::thread::spawn(move || {
        let mut output = [f32::NAN; 128];
        let (_, cold) = measure(|| callback(&mut output));
        let (_, warm) = measure(|| callback(&mut output));
        println!("cold_production_callback={cold:?} warm_production_callback={warm:?}");
        assert!(
            output.iter().all(|sample| *sample == 0.0),
            "stopped empty-source output must be silent"
        );
        assert_eq!(
            warm,
            HeapCounts::default(),
            "steady callback heap operations"
        );
        assert_eq!(
            cold,
            HeapCounts::default(),
            "first callback heap operations"
        );
    })
    .join()
    .expect("fresh callback worker");
    drop(state); // Control publishers outlive the worker and its unique readers.
}
