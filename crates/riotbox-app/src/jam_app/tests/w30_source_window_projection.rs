#[test]
fn resample_source_projection_keeps_the_complete_phrase_in_the_bounded_window() {
    let frame_count = W30_RESAMPLE_SOURCE_WINDOW_LEN * 3;
    let mut samples = vec![0.0_f32; frame_count * 2];
    let transient_start = W30_RESAMPLE_SOURCE_WINDOW_LEN * 2;
    for frame in transient_start..frame_count {
        let phase = (frame - transient_start) as f32 / 11.0;
        let sample = phase.sin() * 0.72;
        samples[frame * 2] = sample;
        samples[frame * 2 + 1] = sample;
    }

    let projected = super::projection::resample_source_from_interleaved(&samples, 2, 48_000)
        .expect("transient source projection");

    assert_eq!(projected.source_start_frame, 0);
    assert_eq!(projected.source_frame_count, frame_count as u64);
    assert_eq!(projected.sample_count, W30_RESAMPLE_SOURCE_WINDOW_LEN);
    assert_eq!(projected.samples[0], 0.0);
    assert!(projected.samples.iter().any(|sample| sample.abs() > 0.5));
}

#[test]
fn resample_source_revision_is_stable_for_identical_pcm_and_changes_with_pcm() {
    let mut samples = (0..8_192)
        .flat_map(|frame| {
            let sample = (frame as f32 / 17.0).sin() * 0.4;
            [sample, sample]
        })
        .collect::<Vec<_>>();
    let first = super::projection::resample_source_from_interleaved(&samples, 2, 48_000)
        .expect("first source projection");
    let repeated = super::projection::resample_source_from_interleaved(&samples, 2, 48_000)
        .expect("repeated source projection");
    samples[4_096] += 0.125;
    samples[4_097] += 0.125;
    let changed = super::projection::resample_source_from_interleaved(&samples, 2, 48_000)
        .expect("changed source projection");

    assert_ne!(first.source_revision, 0);
    assert_eq!(first.source_revision, repeated.source_revision);
    assert_ne!(first.source_revision, changed.source_revision);
}

#[test]
fn resample_source_projection_rejects_invalid_audio_metadata() {
    let samples = [0.25_f32; 32];

    assert!(super::projection::resample_source_from_interleaved(&samples, 0, 48_000).is_none());
    assert!(super::projection::resample_source_from_interleaved(&samples, 2, 0).is_none());
    assert!(super::projection::resample_source_from_interleaved(&[], 2, 48_000).is_none());
    assert!(
        super::projection::resample_source_from_interleaved(&samples[..31], 2, 48_000).is_none()
    );
    let non_finite = [0.0_f32, f32::NAN];
    assert!(
        super::projection::resample_source_from_interleaved(&non_finite, 2, 48_000).is_none()
    );
}

#[test]
fn resample_hard_policy_separates_transient_chops_from_sustained_texture() {
    let sample_rate = 48_000_u32;
    let frame_count = sample_rate as usize * 2;
    let mut transient = vec![0.0_f32; frame_count * 2];
    for onset_frame in [0, frame_count / 4, frame_count * 5 / 8] {
        for offset in 0..960 {
            let envelope = 1.0 - offset as f32 / 960.0;
            let sample = (offset as f32 / 5.0).sin() * envelope * 0.8;
            transient[(onset_frame + offset) * 2] = sample;
            transient[(onset_frame + offset) * 2 + 1] = sample;
        }
    }
    let sustained = (0..frame_count)
        .flat_map(|frame| {
            let sample =
                (frame as f32 * 220.0 * std::f32::consts::TAU / sample_rate as f32).sin() * 0.12;
            [sample, sample]
        })
        .collect::<Vec<_>>();

    let (transient_policy, transient_mask, transient_cursors, transient_contrast) =
        super::projection::analyze_w30_resample_hard_policy(&transient, 2, sample_rate);
    let (texture_policy, texture_mask, texture_cursors, texture_contrast) =
        super::projection::analyze_w30_resample_hard_policy(&sustained, 2, sample_rate);

    assert_eq!(
        transient_policy,
        riotbox_audio::w30::W30ResampleTapHardPolicy::SourceTransientChop
    );
    assert!(transient_mask.count_ones() >= 2);
    assert!(transient_cursors.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        transient_contrast
            >= riotbox_audio::w30::W30_RESAMPLE_TRANSIENT_CHOP_MIN_RISE_TO_MEAN
    );
    assert_eq!(
        texture_policy,
        riotbox_audio::w30::W30ResampleTapHardPolicy::SourceTextureBite
    );
    assert_eq!(texture_mask, 0);
    assert!(texture_cursors.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        texture_contrast < riotbox_audio::w30::W30_RESAMPLE_TRANSIENT_CHOP_MIN_RISE_TO_MEAN
    );
}

#[test]
fn near_mean_envelope_rise_does_not_claim_transient_chop() {
    let sample_rate = 48_000_u32;
    let window_frames = sample_rate as usize * 20 / 1_000;
    let samples = (0..window_frames * 10)
        .flat_map(|frame| {
            let amplitude = if frame < window_frames { 0.1 } else { 1.0 };
            [amplitude, amplitude]
        })
        .collect::<Vec<_>>();

    let (policy, trigger_mask, _, transient_contrast) =
        super::projection::analyze_w30_resample_hard_policy(&samples, 2, sample_rate);

    assert!((0.9..1.0).contains(&transient_contrast));
    assert_eq!(
        policy,
        riotbox_audio::w30::W30ResampleTapHardPolicy::SourceTextureBite
    );
    assert_eq!(trigger_mask, 0);
}

#[test]
fn resample_attack_lengths_follow_source_decay_instead_of_one_fixed_gate() {
    let sample_rate = 48_000_u32;
    let frame_count = sample_rate as usize;
    let source = |decay_frames: usize| {
        let mut samples = vec![0.0_f32; frame_count * 2];
        for frame in 0..decay_frames {
            let envelope = 1.0 - frame as f32 / decay_frames as f32;
            let sample = (frame as f32 / 3.0).sin() * envelope * 0.8;
            samples[frame * 2] = sample;
            samples[frame * 2 + 1] = sample;
        }
        samples
    };
    let short = source(sample_rate as usize * 6 / 1_000);
    let long = source(sample_rate as usize * 60 / 1_000);
    let onset_cursors = [0; W30_RESAMPLE_HARD_SLICE_COUNT];
    let short_lengths = super::projection::derive_w30_resample_attack_lengths(
        &short,
        2,
        sample_rate,
        W30_RESAMPLE_SOURCE_WINDOW_LEN,
        onset_cursors,
    );
    let long_lengths = super::projection::derive_w30_resample_attack_lengths(
        &long,
        2,
        sample_rate,
        W30_RESAMPLE_SOURCE_WINDOW_LEN,
        onset_cursors,
    );

    assert!(short_lengths.iter().all(|length| *length > 0));
    assert!(long_lengths[0] > short_lengths[0] * 2);
}

#[test]
fn resample_attack_lengths_bound_a_late_source_onset_to_available_audio() {
    let sample_rate = 48_000_u32;
    let frame_count = sample_rate as usize;
    let mut samples = vec![0.0_f32; frame_count * 2];
    let last_frame = frame_count - 1;
    samples[last_frame * 2] = 0.8;
    samples[last_frame * 2 + 1] = 0.8;
    let last_proxy_cursor = (W30_RESAMPLE_SOURCE_WINDOW_LEN - 1) as u16;

    let lengths = super::projection::derive_w30_resample_attack_lengths(
        &samples,
        2,
        sample_rate,
        W30_RESAMPLE_SOURCE_WINDOW_LEN,
        [last_proxy_cursor; W30_RESAMPLE_HARD_SLICE_COUNT],
    );

    assert_eq!(lengths, [1; W30_RESAMPLE_HARD_SLICE_COUNT]);
}

#[test]
fn resample_transient_trigger_mask_follows_source_onset_positions() {
    let sample_rate = 48_000_u32;
    let frame_count = sample_rate as usize * 2;
    let source = |onset_slots: [usize; 3]| {
        let mut samples = vec![0.0_f32; frame_count * 2];
        for slot in onset_slots {
            let onset_frame = slot * frame_count / 8;
            for offset in 0..960 {
                let envelope = 1.0 - offset as f32 / 960.0;
                let sample = (offset as f32 / 4.0).sin() * envelope * 0.8;
                samples[(onset_frame + offset) * 2] = sample;
                samples[(onset_frame + offset) * 2 + 1] = sample;
            }
        }
        samples
    };

    let (_, first_mask, _, _) =
        super::projection::analyze_w30_resample_hard_policy(&source([1, 3, 5]), 2, sample_rate);
    let (_, second_mask, _, _) =
        super::projection::analyze_w30_resample_hard_policy(&source([2, 4, 7]), 2, sample_rate);

    assert_ne!(first_mask, second_mask);
    assert_ne!(first_mask & 0b0010_1010, 0);
    assert_ne!(second_mask & 0b1001_0100, 0);
}

#[test]
fn resample_transient_cursors_target_local_onsets_instead_of_slot_boundaries() {
    let sample_rate = 48_000_u32;
    let frame_count = sample_rate as usize * 2;
    let slot_frames = frame_count / W30_RESAMPLE_HARD_SLICE_COUNT;
    let local_offset_frames = sample_rate as usize * 20 / 1_000;
    let mut samples = vec![0.0_f32; frame_count * 2];
    for slot in 0..W30_RESAMPLE_HARD_SLICE_COUNT {
        let onset_frame = slot * slot_frames + local_offset_frames;
        for offset in 0..960 {
            let envelope = 1.0 - offset as f32 / 960.0;
            let sample = (offset as f32 / 4.0).sin() * envelope * 0.8;
            samples[(onset_frame + offset) * 2] = sample;
            samples[(onset_frame + offset) * 2 + 1] = sample;
        }
    }

    let (_, _, cursors, _) =
        super::projection::analyze_w30_resample_hard_policy(&samples, 2, sample_rate);
    let proxy_len = W30_RESAMPLE_SOURCE_WINDOW_LEN.min(frame_count);
    let expected_offset = local_offset_frames * (proxy_len - 1) / (frame_count - 1);
    for (slot, cursor) in cursors.into_iter().enumerate() {
        let slot_boundary = slot * slot_frames * (proxy_len - 1) / (frame_count - 1);
        assert!(
            usize::from(cursor) >= slot_boundary + expected_offset.saturating_sub(64),
            "slot {slot} cursor {cursor} did not advance to the local source onset"
        );
    }
}

#[test]
fn raw_capture_audition_projects_source_window_preview_samples() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 1.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 1.0;
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.w30.preview_mode =
        Some(W30PreviewModeState::RawCaptureAudition);
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_frame: 0,
        end_frame: 48_000,
    });
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);

    state.refresh_view();

    let preview = state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("source-window preview");
    assert_eq!(preview.source_start_frame, 0);
    assert_eq!(preview.source_end_frame, 48_000);
    assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
fn captured_source_window_promotes_to_pad_and_auditions_source_preview() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 8.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 8.0;
    let mut session = sample_session(&graph);
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);
    state.refresh_view();

    state.queue_capture_bar(300);
    let committed_capture = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 0,
            bar_index: 1,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed_capture.len(), 1);
    assert_eq!(state.session.captures.len(), 1);
    assert_eq!(
        state.session.captures[0]
            .source_window
            .as_ref()
            .map(|source_window| source_window.start_frame),
        Some(0)
    );
    assert!(state.session.captures[0].source_window.is_some());

    assert!(state.queue_promote_last_capture(410));
    let committed_promotion = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 4,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        500,
    );

    assert_eq!(committed_promotion.len(), 1);
    assert_eq!(
        state.session.captures[0].assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        })
    );

    assert_eq!(
        state.queue_w30_audition(520),
        Some(QueueControlResult::Enqueued)
    );
    let committed_audition = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 3,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        600,
    );

    assert_eq!(committed_audition.len(), 1);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::PromotedAudition
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    let preview = state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("source-backed promoted audition preview");
    assert_eq!(preview.source_start_frame, 0);
    assert!(preview.source_end_frame > preview.source_start_frame);
    assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
fn promoted_and_recall_w30_previews_project_source_window_preview_samples() {
    for preview_mode in [
        W30PreviewModeState::PromotedAudition,
        W30PreviewModeState::LiveRecall,
    ] {
        let tempdir = tempdir().expect("create source audio tempdir");
        let source_path = tempdir.path().join("source.wav");
        write_pcm16_wave(&source_path, 48_000, 2, 1.0);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.duration_seconds = 1.0;
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.w30.preview_mode = Some(preview_mode);
        session.captures[0].source_window = Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.0,
            end_seconds: 1.0,
            start_frame: 0,
            end_frame: 48_000,
        });
        let source_audio_cache =
            SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_audio_cache);

        state.refresh_view();

        let preview = state
            .runtime
            .w30_preview
            .source_window_preview
            .as_ref()
            .expect("source-window preview");
        assert_eq!(preview.source_start_frame, 0);
        assert_eq!(preview.source_end_frame, 48_000);
        assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
        assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
    }
}
