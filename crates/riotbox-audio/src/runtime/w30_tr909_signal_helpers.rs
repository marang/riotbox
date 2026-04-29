fn w30_preview_frequency(render: &RealtimeW30PreviewRenderState, step: i64) -> f32 {
    let base = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 196.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 261.63,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 293.66,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 220.0,
        Some(W30PreviewSourceProfile::PromotedAudition) => 329.63,
    };
    let step_offset = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => {
            if step.rem_euclid(2) == 0 {
                -8.0
            } else {
                0.0
            }
        }
        Some(W30PreviewSourceProfile::PromotedRecall) | None => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 7.0,
            2 => 12.0,
            _ => 7.0,
        },
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => match step.rem_euclid(3) {
            0 => 0.0,
            1 => 5.0,
            _ => 10.0,
        },
        Some(W30PreviewSourceProfile::RawCaptureAudition) => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 3.0,
            2 => 10.0,
            _ => 5.0,
        },
        Some(W30PreviewSourceProfile::PromotedAudition) => match step.rem_euclid(4) {
            0 => 0.0,
            1 => 12.0,
            2 => 19.0,
            _ => 7.0,
        },
    };
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 28.0;
    base + step_offset + grit_offset
}

fn w30_preview_waveform(phase: f32, grit_level: f32) -> f32 {
    let sine = (std::f32::consts::TAU * phase).sin();
    let overtone = (std::f32::consts::TAU * phase * 2.0).sin();
    let grit = grit_level.clamp(0.0, 1.0);
    let blended = sine * (1.0 - grit * 0.45) + overtone * (0.18 + grit * 0.3);
    let quant_steps = (24.0 - grit * 18.0).max(4.0);
    ((blended * quant_steps).round() / quant_steps).clamp(-1.0, 1.0)
}

fn w30_render_gain(render: &RealtimeW30PreviewRenderState, transport_running: bool) -> f32 {
    let base = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.12,
        W30PreviewRenderMode::RawCaptureAudition => 0.15,
        W30PreviewRenderMode::PromotedAudition => 0.18,
    };
    let profile_gain = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 1.0,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 1.08,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => 1.12,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => 1.16,
        Some(W30PreviewSourceProfile::PromotedAudition) => 1.2,
    };
    let transport_gain = if transport_running { 1.0 } else { 0.72 };
    (base
        * profile_gain
        * transport_gain
        * render.music_bus_level.clamp(0.0, 1.0)
        * (1.0 + render.grit_level.clamp(0.0, 1.0) * 0.2))
        .clamp(0.0, 0.28)
}

fn w30_envelope_decay(render: &RealtimeW30PreviewRenderState) -> f32 {
    let base = match render.mode {
        W30PreviewRenderMode::Idle => 0.0,
        W30PreviewRenderMode::LiveRecall => 0.99983,
        W30PreviewRenderMode::RawCaptureAudition => 0.99978,
        W30PreviewRenderMode::PromotedAudition => 0.99972,
    };
    let profile_offset = match render.source_profile {
        Some(W30PreviewSourceProfile::PinnedRecall) => 0.00002,
        Some(W30PreviewSourceProfile::PromotedRecall) | None => 0.0,
        Some(W30PreviewSourceProfile::SlicePoolBrowse) => -0.00001,
        Some(W30PreviewSourceProfile::RawCaptureAudition) => -0.00002,
        Some(W30PreviewSourceProfile::PromotedAudition) => -0.00003,
    };
    let grit_offset = render.grit_level.clamp(0.0, 1.0) * 0.00008;
    (base + profile_offset - grit_offset).clamp(0.0, 1.0)
}

fn w30_preview_idle_bpm(render: &RealtimeW30PreviewRenderState) -> f32 {
    render.tempo_bpm.max(92.0)
}

const fn render_subdivision(render: &RealtimeTr909RenderState) -> u32 {
    let base = if let Some(adoption) = render.pattern_adoption {
        match adoption {
            Tr909PatternAdoption::SupportPulse => 1,
            Tr909PatternAdoption::MainlineDrive => 2,
            Tr909PatternAdoption::TakeoverGrid => 4,
        }
    } else {
        match render.mode {
            Tr909RenderMode::Idle => 1,
            Tr909RenderMode::SourceSupport => match render.source_support_profile {
                Some(
                    Tr909SourceSupportProfile::BreakLift | Tr909SourceSupportProfile::DropDrive,
                ) => 2,
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 1,
            },
            Tr909RenderMode::Fill | Tr909RenderMode::BreakReinforce | Tr909RenderMode::Takeover => {
                2
            }
        }
    };

    match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => base,
        Some(Tr909PhraseVariation::PhraseLift) => {
            if base < 2 {
                2
            } else {
                base
            }
        }
        Some(Tr909PhraseVariation::PhraseDrive) => {
            if base < 4 {
                4
            } else {
                base
            }
        }
        Some(Tr909PhraseVariation::PhraseRelease) => {
            if base > 2 {
                2
            } else {
                base
            }
        }
    }
}

fn should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
    let base = if let Some(adoption) = render.pattern_adoption {
        match adoption {
            Tr909PatternAdoption::SupportPulse => step % 2 == 0,
            Tr909PatternAdoption::MainlineDrive => true,
            Tr909PatternAdoption::TakeoverGrid => !matches!(step.rem_euclid(4), 1),
        }
    } else {
        match render.mode {
            Tr909RenderMode::Idle => false,
            Tr909RenderMode::SourceSupport => match render.source_support_profile {
                Some(Tr909SourceSupportProfile::BreakLift) => step % 2 == 0,
                Some(Tr909SourceSupportProfile::DropDrive) => true,
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => true,
            },
            Tr909RenderMode::Fill => true,
            Tr909RenderMode::BreakReinforce => true,
            Tr909RenderMode::Takeover => match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::SceneLock) => step % 4 != 3,
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => true,
            },
        }
    };

    match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => base,
        Some(Tr909PhraseVariation::PhraseLift) => base || step.rem_euclid(8) == 7,
        Some(Tr909PhraseVariation::PhraseDrive) => base || matches!(step.rem_euclid(4), 1 | 3),
        Some(Tr909PhraseVariation::PhraseRelease) => base && step.rem_euclid(4) == 0,
    }
}

fn trigger_envelope(render: &RealtimeTr909RenderState) -> f32 {
    let base = match render.routing {
        Tr909RenderRouting::SourceOnly => 0.0,
        Tr909RenderRouting::DrumBusSupport => 0.22,
        Tr909RenderRouting::DrumBusTakeover => 0.34,
    };
    let profile_boost = match render.mode {
        Tr909RenderMode::SourceSupport => match render.source_support_profile {
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => 0.0,
            Some(Tr909SourceSupportProfile::BreakLift) => 0.03,
            Some(Tr909SourceSupportProfile::DropDrive) => 0.08,
        },
        Tr909RenderMode::Takeover => match render.takeover_profile {
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 0.06,
            Some(Tr909TakeoverRenderProfile::SceneLock) => 0.1,
        },
        Tr909RenderMode::Fill => 0.04,
        Tr909RenderMode::BreakReinforce => 0.02,
        Tr909RenderMode::Idle => 0.0,
    };
    let pattern_boost = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 0.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 0.04,
        Some(Tr909PatternAdoption::TakeoverGrid) => 0.07,
    };
    let phrase_boost = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => 0.03,
        Some(Tr909PhraseVariation::PhraseDrive) => 0.06,
        Some(Tr909PhraseVariation::PhraseRelease) => -0.05,
    };
    let context_boost = match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => 0.035,
        _ => 0.0,
    };
    (base
        + profile_boost
        + pattern_boost
        + phrase_boost
        + context_boost
        + (render.slam_intensity * 0.2))
        .clamp(0.0, 0.8)
}

fn trigger_frequency(render: &RealtimeTr909RenderState, step: i64) -> f32 {
    let accent = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => {
            if step % 2 == 0 {
                0.0
            } else {
                14.0
            }
        }
        Some(Tr909PatternAdoption::MainlineDrive) => {
            if step.rem_euclid(4) == 3 {
                18.0
            } else {
                6.0
            }
        }
        Some(Tr909PatternAdoption::TakeoverGrid) => match step.rem_euclid(4) {
            0 => 22.0,
            2 => 10.0,
            _ => 4.0,
        },
    };
    let phrase_pitch = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 0.0,
        Some(Tr909PhraseVariation::PhraseLift) => 6.0,
        Some(Tr909PhraseVariation::PhraseDrive) => 12.0,
        Some(Tr909PhraseVariation::PhraseRelease) => -8.0,
    };
    let slam = render.slam_intensity.clamp(0.0, 1.0) * 18.0;
    match render.mode {
        Tr909RenderMode::Idle => 0.0,
        Tr909RenderMode::SourceSupport => {
            let base = match render.source_support_profile {
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 52.0,
                Some(Tr909SourceSupportProfile::BreakLift) => 66.0,
                Some(Tr909SourceSupportProfile::DropDrive) => 78.0,
            };
            base + accent + phrase_pitch + slam
        }
        Tr909RenderMode::Fill => 78.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::BreakReinforce => 64.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::Takeover => {
            let base = match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 92.0,
                Some(Tr909TakeoverRenderProfile::SceneLock) => 108.0,
            };
            base + accent + phrase_pitch + slam
        }
    }
}

fn render_gain(render: &RealtimeTr909RenderState) -> f32 {
    let routing_gain = match render.routing {
        Tr909RenderRouting::SourceOnly => 0.0,
        Tr909RenderRouting::DrumBusSupport => 0.12,
        Tr909RenderRouting::DrumBusTakeover => 0.18,
    };
    let pattern_gain = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) | None => 1.0,
        Some(Tr909PatternAdoption::MainlineDrive) => 1.08,
        Some(Tr909PatternAdoption::TakeoverGrid) => 1.16,
    };
    let phrase_gain = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseAnchor) | None => 1.0,
        Some(Tr909PhraseVariation::PhraseLift) => 1.06,
        Some(Tr909PhraseVariation::PhraseDrive) => 1.14,
        Some(Tr909PhraseVariation::PhraseRelease) => 0.72,
    };
    let context_gain = match (render.mode, render.source_support_context) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportContext::SceneTarget)) => 1.08,
        _ => 1.0,
    };
    (routing_gain
        * pattern_gain
        * phrase_gain
        * context_gain
        * render.drum_bus_level.clamp(0.0, 1.0))
    .clamp(0.0, 0.25)
}

