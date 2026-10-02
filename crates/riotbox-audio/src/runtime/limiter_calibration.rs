//! Opt-in, in-memory diagnostics for the three frozen limiter comparisons.
//!
//! This module does not open sources, authorize execution, select product policy, or
//! change the live callback. Callers own admission, format/alignment and resource
//! bounds, and must distinguish clean regressions from diagnostic overload controls.

use std::{error::Error, fmt};

use super::{
    public_api_shell::{
        MasterBusLimiterReport, apply_master_bus_soft_limiter_with_bounds,
        master_bus_limiter_ceiling, master_bus_limiter_threshold, signal_metrics,
    },
    runtime_mix_parity::{
        RuntimeMixRenderOutput, RuntimeMixRenderSequenceStep, render_sequence_with_pre_limiter_sink,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    BaselineA,
    LaterKneeB,
    LowerCeilingC,
}

impl Policy {
    pub const ALL: [Self; 3] = [Self::BaselineA, Self::LaterKneeB, Self::LowerCeilingC];

    #[must_use]
    pub fn threshold(self) -> f32 {
        match self {
            Self::BaselineA | Self::LowerCeilingC => master_bus_limiter_threshold(),
            Self::LaterKneeB => 0.9525,
        }
    }

    #[must_use]
    pub fn ceiling(self) -> f32 {
        match self {
            Self::BaselineA | Self::LaterKneeB => master_bus_limiter_ceiling(),
            Self::LowerCeilingC => 0.9525,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolicyOutput {
    pub policy: Policy,
    pub samples: Vec<f32>,
    pub limiter: MasterBusLimiterReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalibrationInputError {
    NonFiniteSample { sample_index: usize },
}

impl fmt::Display for CalibrationInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteSample { sample_index } => {
                write!(
                    formatter,
                    "limiter comparison input sample {sample_index} is not finite"
                )
            }
        }
    }
}

impl Error for CalibrationInputError {}

/// Applies A, B and C independently to the exact same unmodified input.
///
/// Empty input is retained for the separate baseline control. This function does not
/// scale stress inputs, normalize output, validate a channel layout, or grant a pass.
/// Metrics retain the existing f32 semantics, including possible overflow for extreme
/// finite inputs; a runner must enforce its frozen input bounds and metric validity.
pub fn compare(pre_samples: &[f32]) -> Result<[PolicyOutput; 3], CalibrationInputError> {
    if let Some(sample_index) = pre_samples.iter().position(|sample| !sample.is_finite()) {
        return Err(CalibrationInputError::NonFiniteSample { sample_index });
    }
    let pre = signal_metrics(pre_samples);
    Ok(Policy::ALL.map(|policy| {
        let mut samples = pre_samples.to_vec();
        let threshold = policy.threshold();
        let ceiling = policy.ceiling();
        let limited_sample_count =
            apply_master_bus_soft_limiter_with_bounds(&mut samples, threshold, ceiling);
        let post = signal_metrics(&samples);
        PolicyOutput {
            policy,
            samples,
            limiter: MasterBusLimiterReport {
                applied: limited_sample_count > 0,
                threshold,
                ceiling,
                limited_sample_count,
                pre,
                post,
            },
        }
    }))
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMixEvidence {
    pub pre_samples: Vec<f32>,
    pub baseline: RuntimeMixRenderOutput,
}

/// Retains each step's already-computed pre-limiter PCM and unchanged product output.
///
/// This uses the existing offline sequence implementation and its input semantics.
/// Callers must admit positive rates/channel counts and bounded frame/partition sizes
/// before invoking it; the ordinary renderer's compatibility normalization is retained.
#[must_use]
pub fn render_sequence(
    steps: &[RuntimeMixRenderSequenceStep<'_>],
    sample_rate: u32,
    channel_count: u16,
    callback_frame_count: usize,
) -> Vec<RuntimeMixEvidence> {
    let mut pre_buffers = Vec::with_capacity(steps.len());
    let outputs = render_sequence_with_pre_limiter_sink(
        steps,
        sample_rate,
        channel_count,
        callback_frame_count,
        |pre| pre_buffers.push(pre),
    );
    pre_buffers
        .into_iter()
        .zip(outputs)
        .map(|(pre_samples, baseline)| RuntimeMixEvidence {
            pre_samples,
            baseline,
        })
        .collect()
}

#[cfg(test)]
mod tests;
