use crate::cli::controls::AudioRuntimeRefreshAction;
use crate::cli::controls::replace_app_state_after_refresh;
use crate::cli::terminal::audio_start_failure_health;
use crate::jam_app::JamAppState;
use crate::ui::JamShellState;
use crate::ui::ShellLaunchMode;
use riotbox_audio::runtime::AudioRuntimeError;
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::queue::ActionQueue;
use riotbox_core::session::SessionFile;

#[test]
fn audio_start_failure_becomes_persistent_faulted_runtime_health() {
    let error = AudioRuntimeError::BuildStream {
        host_name: "Alsa".into(),
        device_name: "default".into(),
        reason: "device busy".into(),
    };

    let health = audio_start_failure_health(&error);

    assert_eq!(health.lifecycle, AudioRuntimeLifecycle::Faulted);
    assert_eq!(health.stream_error_count, 1);
    assert_eq!(health.callback_count, 0);
    assert!(health.output.is_none());
    let message = error.to_string();
    assert_eq!(health.last_stream_error.as_deref(), Some(message.as_str()));
}

#[test]
fn refresh_without_audio_runtime_preserves_fault_health_and_requests_retry() {
    let error = AudioRuntimeError::BuildStream {
        host_name: "Alsa".into(),
        device_name: "default".into(),
        reason: "device busy".into(),
    };
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("before-refresh", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    shell
        .app
        .set_audio_health(audio_start_failure_health(&error));
    let refreshed = JamAppState::from_parts(
        SessionFile::new("after-refresh", "0.1.0", "2026-07-15T00:00:01Z"),
        None,
        ActionQueue::new(),
    );

    assert_eq!(
        replace_app_state_after_refresh(&mut shell, refreshed, false),
        AudioRuntimeRefreshAction::RetryUnavailable
    );

    let health = shell.app.runtime.audio.as_ref().expect("fault health");
    let error_message = error.to_string();
    assert_eq!(health.lifecycle, AudioRuntimeLifecycle::Faulted);
    assert_eq!(health.stream_error_count, 1);
    assert_eq!(
        health.last_stream_error.as_deref(),
        Some(error_message.as_str())
    );
    assert_eq!(shell.app.runtime_view.audio_status, "faulted");
    assert!(
        shell
            .app
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning == "audio runtime faulted")
    );
}

#[test]
fn refresh_with_live_audio_requests_stream_restart_for_new_source_cache() {
    let source_a =
        SourceAudioCache::from_interleaved_samples("source-a.wav", 48_000, 2, vec![0.10; 960])
            .expect("source A cache");
    let source_b =
        SourceAudioCache::from_interleaved_samples("source-b.wav", 48_000, 2, vec![0.75; 960])
            .expect("source B cache");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("source-a", "0.1.0", "2026-07-15T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    shell.app.source_audio_cache = Some(source_a);
    let mut refreshed = JamAppState::from_parts(
        SessionFile::new("source-b", "0.1.0", "2026-07-15T00:00:01Z"),
        None,
        ActionQueue::new(),
    );
    refreshed.source_audio_cache = Some(source_b);

    assert_eq!(
        replace_app_state_after_refresh(&mut shell, refreshed, true),
        AudioRuntimeRefreshAction::Restart
    );
    let render = shell.app.source_monitor_render_state();
    assert_eq!(
        shell
            .app
            .source_audio_cache
            .as_ref()
            .map(|cache| cache.path.as_path()),
        Some(std::path::Path::new("source-b.wav"))
    );
    assert_eq!(
        render
            .source
            .as_ref()
            .map(|source| source.interleaved_samples()[0]),
        Some(0.75)
    );
}
