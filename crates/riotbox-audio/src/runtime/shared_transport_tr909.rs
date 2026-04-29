impl AudioRuntimeShell {
    pub fn start_default_output() -> Result<Self, AudioRuntimeError> {
        Self::start_default_output_with_render_states(
            Tr909RenderState::default(),
            Mc202RenderState::default(),
            W30PreviewRenderState::default(),
            W30ResampleTapState::default(),
        )
    }

    pub fn start_default_output_with_tr909(
        render_state: Tr909RenderState,
    ) -> Result<Self, AudioRuntimeError> {
        Self::start_default_output_with_render_states(
            render_state,
            Mc202RenderState::default(),
            W30PreviewRenderState::default(),
            W30ResampleTapState::default(),
        )
    }

    pub fn start_default_output_with_render_states(
        tr909_render_state: Tr909RenderState,
        mc202_render_state: Mc202RenderState,
        w30_preview_render_state: W30PreviewRenderState,
        w30_resample_tap_state: W30ResampleTapState,
    ) -> Result<Self, AudioRuntimeError> {
        let host = cpal::default_host();
        let host_name = format!("{:?}", host.id());

        let Some(device) = host.default_output_device() else {
            return Err(AudioRuntimeError::NoDefaultOutputDevice { host_name });
        };

        #[allow(deprecated)]
        let device_name = device
            .name()
            .unwrap_or_else(|_| "<unknown-device>".to_string());

        let supported_output_config_count = device
            .supported_output_configs()
            .ok()
            .map(|configs| configs.count());

        let default_config = device.default_output_config().map_err(|error| {
            AudioRuntimeError::DefaultOutputConfig {
                host_name: host_name.clone(),
                device_name: device_name.clone(),
                reason: error.to_string(),
            }
        })?;

        let output = AudioOutputInfo {
            host_name: host_name.clone(),
            device_name: device_name.clone(),
            sample_format: format!("{:?}", default_config.sample_format()),
            sample_rate: default_config.sample_rate(),
            channel_count: default_config.channels(),
            buffer_size: format!("{:?}", default_config.buffer_size()),
            supported_output_config_count,
        };

        let telemetry = Arc::new(RuntimeTelemetry::new());
        let transport = Arc::new(SharedTransportTimingState::new(
            tr909_render_state.is_transport_running,
            tr909_render_state.tempo_bpm,
            tr909_render_state.position_beats,
        ));
        let tr909_render = Arc::new(SharedTr909RenderState::new(&tr909_render_state));
        let mc202_render = Arc::new(SharedMc202RenderState::new(&mc202_render_state));
        let w30_preview = Arc::new(SharedW30PreviewRenderState::new(&w30_preview_render_state));
        let w30_resample_tap = Arc::new(SharedW30ResampleTapState::new(&w30_resample_tap_state));
        let stream_config = default_config.config();
        let start = Instant::now();

        let stream = match default_config.sample_format() {
            cpal::SampleFormat::F32 => build_silent_output_stream::<f32>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            cpal::SampleFormat::I16 => build_silent_output_stream::<i16>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            cpal::SampleFormat::U16 => build_silent_output_stream::<u16>(
                &device,
                &stream_config,
                AudioRuntimeSharedState {
                    telemetry: Arc::clone(&telemetry),
                    transport: Arc::clone(&transport),
                    tr909_render: Arc::clone(&tr909_render),
                    mc202_render: Arc::clone(&mc202_render),
                    w30_preview: Arc::clone(&w30_preview),
                    w30_resample_tap: Arc::clone(&w30_resample_tap),
                },
                start,
            ),
            sample_format => {
                return Err(AudioRuntimeError::UnsupportedSampleFormat {
                    host_name,
                    device_name,
                    sample_format: format!("{sample_format:?}"),
                });
            }
        }
        .map_err(|error| AudioRuntimeError::BuildStream {
            host_name: host_name.clone(),
            device_name: device_name.clone(),
            reason: error.to_string(),
        })?;

        stream
            .play()
            .map_err(|error| AudioRuntimeError::PlayStream {
                host_name,
                device_name,
                reason: error.to_string(),
            })?;

        Ok(Self {
            lifecycle: AudioRuntimeLifecycle::Running,
            output: Some(output),
            telemetry,
            transport,
            tr909_render,
            mc202_render,
            w30_preview,
            w30_resample_tap,
            stream: Some(stream),
        })
    }

    #[must_use]
    pub fn lifecycle(&self) -> AudioRuntimeLifecycle {
        self.lifecycle
    }

    #[must_use]
    pub fn health_snapshot(&self) -> AudioRuntimeHealth {
        let telemetry = self.telemetry.snapshot();

        let lifecycle = if telemetry.stream_error_count > 0
            && matches!(self.lifecycle, AudioRuntimeLifecycle::Running)
        {
            AudioRuntimeLifecycle::Faulted
        } else {
            self.lifecycle
        };

        AudioRuntimeHealth {
            lifecycle,
            output: self.output.clone(),
            callback_count: telemetry.callback_count,
            max_callback_gap_micros: telemetry.max_callback_gap_micros,
            stream_error_count: telemetry.stream_error_count,
            last_stream_error: telemetry.last_stream_error,
        }
    }

    #[must_use]
    pub fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        self.telemetry.timing_snapshot()
    }

    pub fn update_transport_state(
        &self,
        is_transport_running: bool,
        tempo_bpm: f32,
        position_beats: f64,
    ) {
        self.transport
            .update(is_transport_running, tempo_bpm, position_beats);
    }

    pub fn update_tr909_render_state(&self, render_state: &Tr909RenderState) {
        self.tr909_render.update(render_state);
    }

    pub fn update_mc202_render_state(&self, render_state: &Mc202RenderState) {
        self.mc202_render.update(render_state);
    }

    pub fn update_w30_preview_render_state(&self, render_state: &W30PreviewRenderState) {
        self.w30_preview.update(render_state);
    }

    pub fn update_w30_resample_tap_state(&self, render_state: &W30ResampleTapState) {
        self.w30_resample_tap.update(render_state);
    }

    pub fn stop(&mut self) {
        self.stream.take();
        self.lifecycle = AudioRuntimeLifecycle::Stopped;
    }

    #[cfg(test)]
    fn from_test_parts(parts: AudioRuntimeShellTestParts) -> Self {
        Self {
            lifecycle: parts.lifecycle,
            output: parts.output,
            telemetry: parts.telemetry,
            transport: parts.transport,
            tr909_render: parts.tr909_render,
            mc202_render: parts.mc202_render,
            w30_preview: parts.w30_preview,
            w30_resample_tap: parts.w30_resample_tap,
            stream: None,
        }
    }
}

#[cfg(test)]
struct AudioRuntimeShellTestParts {
    lifecycle: AudioRuntimeLifecycle,
    output: Option<AudioOutputInfo>,
    telemetry: Arc<RuntimeTelemetry>,
    transport: Arc<SharedTransportTimingState>,
    tr909_render: Arc<SharedTr909RenderState>,
    mc202_render: Arc<SharedMc202RenderState>,
    w30_preview: Arc<SharedW30PreviewRenderState>,
    w30_resample_tap: Arc<SharedW30ResampleTapState>,
}

impl Drop for AudioRuntimeShell {
    fn drop(&mut self) {
        self.stop();
    }
}

fn build_silent_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared: AudioRuntimeSharedState,
    start: Instant,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let callback_telemetry = Arc::clone(&shared.telemetry);
    let error_telemetry = Arc::clone(&shared.telemetry);
    let callback_transport = Arc::clone(&shared.transport);
    let mut render_state = Tr909CallbackState::default();
    let mut transport_state = TransportTimingCallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::default();
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut mix_buffer = Vec::<f32>::new();
    let sample_rate = config.sample_rate;
    let channel_count = usize::from(config.channels.max(1));

    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            if mix_buffer.len() != data.len() {
                mix_buffer.resize(data.len(), 0.0);
            }

            let frame_count = data.len() / channel_count.max(1);
            let callback_timing = advance_transport_timing(
                &callback_transport.snapshot(),
                &mut transport_state,
                sample_rate,
                frame_count,
            );
            let mut tr909_render_state = shared.tr909_render.snapshot();
            tr909_render_state.is_transport_running = callback_timing.is_transport_running;
            tr909_render_state.tempo_bpm = callback_timing.tempo_bpm;
            tr909_render_state.position_beats = callback_timing.render_position_beats;
            let mut mc202_render_state = shared.mc202_render.snapshot();
            mc202_render_state.is_transport_running = callback_timing.is_transport_running;
            mc202_render_state.tempo_bpm = callback_timing.tempo_bpm;
            mc202_render_state.position_beats = callback_timing.render_position_beats;
            let mut w30_preview_render_state = shared.w30_preview.snapshot();
            w30_preview_render_state.is_transport_running = callback_timing.is_transport_running;
            w30_preview_render_state.tempo_bpm = callback_timing.tempo_bpm;
            w30_preview_render_state.position_beats = callback_timing.render_position_beats;
            let mut w30_resample_render_state = shared.w30_resample_tap.snapshot();
            w30_resample_render_state.is_transport_running = callback_timing.is_transport_running;

            render_mix_buffer(
                &mut mix_buffer,
                sample_rate,
                channel_count,
                &tr909_render_state,
                &mc202_render_state,
                &mut render_state,
                &mut W30MixRenderState {
                    preview_render: &w30_preview_render_state,
                    preview_state: &mut w30_preview_state,
                    resample_render: &w30_resample_render_state,
                    resample_state: &mut w30_resample_state,
                },
            );
            for (output, sample) in data.iter_mut().zip(mix_buffer.iter().copied()) {
                *output = T::from_sample(sample);
            }

            let now = start.elapsed().as_micros() as u64;
            callback_telemetry.record_callback_at(now, &callback_timing);
        },
        move |error| {
            error_telemetry.record_stream_error(error.to_string());
        },
        None,
    )
}

#[derive(Clone)]
struct AudioRuntimeSharedState {
    telemetry: Arc<RuntimeTelemetry>,
    transport: Arc<SharedTransportTimingState>,
    tr909_render: Arc<SharedTr909RenderState>,
    mc202_render: Arc<SharedMc202RenderState>,
    w30_preview: Arc<SharedW30PreviewRenderState>,
    w30_resample_tap: Arc<SharedW30ResampleTapState>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeTransportTimingState {
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

struct SharedTransportTimingState {
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedTransportTimingState {
    fn new(is_transport_running: bool, tempo_bpm: f32, position_beats: f64) -> Self {
        Self {
            is_transport_running: AtomicBool::new(is_transport_running),
            tempo_bpm_bits: AtomicU32::new(tempo_bpm.to_bits()),
            position_beats_bits: AtomicU64::new(position_beats.to_bits()),
        }
    }

    fn update(&self, is_transport_running: bool, tempo_bpm: f32, position_beats: f64) {
        self.is_transport_running
            .store(is_transport_running, Ordering::Relaxed);
        self.tempo_bpm_bits
            .store(tempo_bpm.to_bits(), Ordering::Relaxed);
        self.position_beats_bits
            .store(position_beats.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RealtimeTransportTimingState {
        RealtimeTransportTimingState {
            is_transport_running: self.is_transport_running.load(Ordering::Relaxed),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Relaxed)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RealtimeTr909RenderState {
    mode: Tr909RenderMode,
    routing: Tr909RenderRouting,
    source_support_profile: Option<Tr909SourceSupportProfile>,
    source_support_context: Option<Tr909SourceSupportContext>,
    pattern_adoption: Option<Tr909PatternAdoption>,
    phrase_variation: Option<Tr909PhraseVariation>,
    takeover_profile: Option<Tr909TakeoverRenderProfile>,
    drum_bus_level: f32,
    slam_intensity: f32,
    is_transport_running: bool,
    tempo_bpm: f32,
    position_beats: f64,
}

struct SharedTr909RenderState {
    mode: AtomicU32,
    routing: AtomicU32,
    source_support_profile: AtomicU32,
    source_support_context: AtomicU32,
    pattern_adoption: AtomicU32,
    phrase_variation: AtomicU32,
    takeover_profile: AtomicU32,
    drum_bus_level_bits: AtomicU32,
    slam_intensity_bits: AtomicU32,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

