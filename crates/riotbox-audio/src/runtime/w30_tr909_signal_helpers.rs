use super::*;

pub(super) fn w30_render_gain(
    render: &RealtimeW30PreviewRenderState,
    transport_running: bool,
) -> f32 {
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

pub(super) fn w30_envelope_decay(render: &RealtimeW30PreviewRenderState) -> f32 {
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

pub(super) fn w30_preview_idle_bpm(render: &RealtimeW30PreviewRenderState) -> f32 {
    render.tempo_bpm.max(92.0)
}

pub(super) const fn render_subdivision(render: &RealtimeTr909RenderState) -> u32 {
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

pub(super) fn should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
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

pub(super) fn trigger_envelope(render: &RealtimeTr909RenderState) -> f32 {
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
        Tr909RenderMode::Fill => 0.12,
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

pub(super) fn trigger_frequency(render: &RealtimeTr909RenderState, step: i64) -> f32 {
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
                Some(Tr909SourceSupportProfile::SteadyPulse) | None => 58.0,
                Some(Tr909SourceSupportProfile::BreakLift) => 104.0,
                Some(Tr909SourceSupportProfile::DropDrive) => 44.0,
            };
            base + accent + phrase_pitch + slam
        }
        Tr909RenderMode::Fill => 112.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::BreakReinforce => 88.0 + accent + phrase_pitch + slam,
        Tr909RenderMode::Takeover => {
            let base = match render.takeover_profile {
                Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 92.0,
                Some(Tr909TakeoverRenderProfile::SceneLock) => 108.0,
            };
            base + accent + phrase_pitch + slam
        }
    }
}

pub(super) fn tr909_step_waveform(render: &RealtimeTr909RenderState, step: i64, phase: f32) -> f32 {
    let balance = tr909_voice_balance(render, step);
    let kick = (std::f32::consts::TAU * phase).sin();
    let snare = tr909_deterministic_noise(phase, step);
    let hat_phase = phase.mul_add(11.0, step as f32 * 0.073);
    let hat = (std::f32::consts::TAU * hat_phase).sin()
        * (std::f32::consts::TAU * hat_phase * 0.37).cos().abs();

    ((kick * balance.kick) + (snare * balance.snare) + (hat * balance.hat)).clamp(-1.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tr909VoiceBalance {
    kick: f32,
    snare: f32,
    hat: f32,
}

fn tr909_voice_balance(render: &RealtimeTr909RenderState, step: i64) -> Tr909VoiceBalance {
    let subdivision = i64::from(render_subdivision(render)).max(1);
    let bar_steps = subdivision * 4;
    let step_in_bar = step.rem_euclid(bar_steps);
    let on_beat = step_in_bar % subdivision == 0;
    let downbeat = step_in_bar == 0;
    let backbeat = step_in_bar == subdivision * 2;
    let offbeat = !on_beat;

    let mut balance = Tr909VoiceBalance {
        kick: if downbeat {
            1.0
        } else if on_beat {
            0.46
        } else {
            0.12
        },
        snare: if backbeat {
            0.86
        } else if offbeat {
            0.28
        } else {
            0.12
        },
        hat: if offbeat { 0.72 } else { 0.24 },
    };

    if matches!(render.mode, Tr909RenderMode::SourceSupport) {
        match render.source_support_profile {
            Some(Tr909SourceSupportProfile::DropDrive) => {
                balance.kick *= 1.65;
                balance.snare *= 0.55;
                balance.hat *= 0.55;
            }
            Some(Tr909SourceSupportProfile::BreakLift) => {
                balance.kick *= 0.74;
                balance.snare *= 1.68;
                balance.hat *= 1.48;
            }
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => {
                balance.kick *= 1.0;
                balance.snare *= 0.82;
                balance.hat *= 0.64;
            }
        }
    }

    match render.pattern_adoption {
        Some(Tr909PatternAdoption::MainlineDrive) => {
            balance.kick *= 1.12;
            balance.hat *= 0.92;
        }
        Some(Tr909PatternAdoption::TakeoverGrid) => {
            balance.snare *= 1.16;
            balance.hat *= 1.22;
        }
        Some(Tr909PatternAdoption::SupportPulse) | None => {}
    }

    match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseLift) => {
            balance.snare *= 1.14;
            balance.hat *= 1.16;
        }
        Some(Tr909PhraseVariation::PhraseDrive) => {
            if matches!(render.mode, Tr909RenderMode::SourceSupport) {
                balance.kick *= 1.22;
                balance.snare *= 0.92;
                balance.hat *= 0.96;
            } else {
                balance.kick *= 1.10;
                balance.hat *= 1.06;
            }
        }
        Some(Tr909PhraseVariation::PhraseRelease) => {
            if matches!(render.mode, Tr909RenderMode::SourceSupport) {
                balance.kick *= 0.46;
                balance.snare *= 1.04;
                balance.hat *= 1.18;
            } else {
                balance.kick *= 0.80;
                balance.snare *= 0.72;
                balance.hat *= 0.54;
            }
        }
        Some(Tr909PhraseVariation::PhraseAnchor) | None => {}
    }

    match render.mode {
        Tr909RenderMode::Fill => {
            balance.snare *= 1.40;
            balance.hat *= 1.55;
        }
        Tr909RenderMode::BreakReinforce => {
            balance.snare *= 1.22;
            balance.hat *= 1.08;
        }
        Tr909RenderMode::Takeover => {
            balance.kick *= 1.28;
            balance.snare *= 1.24;
            balance.hat *= 1.18;
        }
        Tr909RenderMode::Idle | Tr909RenderMode::SourceSupport => {}
    }

    let sum = balance.kick + balance.snare + balance.hat;
    if sum > 2.20 {
        let scale = 2.20 / sum;
        balance.kick *= scale;
        balance.snare *= scale;
        balance.hat *= scale;
    }
    balance
}

fn tr909_deterministic_noise(phase: f32, step: i64) -> f32 {
    let seeded = (phase * 12_989.0 + step as f32 * 78.233).sin() * 43_758.547;
    ((seeded - seeded.floor()) * 2.0) - 1.0
}

pub(super) fn render_gain(render: &RealtimeTr909RenderState) -> f32 {
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
    let source_profile_gain = match (render.mode, render.source_support_profile) {
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportProfile::DropDrive)) => 1.42,
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportProfile::BreakLift)) => 1.28,
        (Tr909RenderMode::SourceSupport, Some(Tr909SourceSupportProfile::SteadyPulse) | None) => {
            1.05
        }
        _ => 1.0,
    };
    let mode_gain = match render.mode {
        Tr909RenderMode::Fill => 1.75,
        Tr909RenderMode::BreakReinforce => 1.12,
        Tr909RenderMode::Takeover => match render.takeover_profile {
            Some(Tr909TakeoverRenderProfile::SceneLock) => 1.15,
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => 1.36,
        },
        Tr909RenderMode::Idle | Tr909RenderMode::SourceSupport => 1.0,
    };
    (routing_gain
        * pattern_gain
        * phrase_gain
        * context_gain
        * source_profile_gain
        * mode_gain
        * render.drum_bus_level.clamp(0.0, 1.0))
    .clamp(0.0, 0.25)
}
