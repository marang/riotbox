use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use riotbox_audio::percussive_force::{
    F3PcmEncoding, FrozenEventInput, FrozenEventRegion, render_f1_ab_energy_redistribution_v1,
    render_f2_exact_complementary_three_band_v1,
    render_f3_causal_envelope_contrast_dynamic_residual_v2,
    render_f4_source_native_body_sustain_v1,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MATRIX_SCHEMA_V6: &str = "riotbox.percussive_force_development_matrix.v6";
const MATRIX_SCHEMA_V7: &str = "riotbox.percussive_force_development_matrix.v7";
const RESULT_SCHEMA_V1: &str = "riotbox.percussive_force_development_matrix_result.v1";
const RESULT_SCHEMA_V2: &str = "riotbox.percussive_force_development_matrix_result.v2";
const PCM_HASH_DOMAIN: &str = "riotbox.percussive_force_pcm_f32le.v1";

#[derive(Debug, Deserialize)]
struct Matrix {
    schema: String,
    qualification_artifact_sha256: String,
    condition_count: usize,
    condition_ids: Vec<String>,
    selected_sources: Vec<MatrixSource>,
}

#[derive(Debug, Deserialize)]
struct MatrixSource {
    case_id: String,
    source_family: String,
    source_path: String,
    source_sha256: String,
    sample_rate_hz: u32,
    channel_count: usize,
    sample_width_bits: u16,
    events: Vec<MatrixEvent>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct MatrixEvent {
    ordinal: usize,
    physical_onset_frame: usize,
    attack_end_frame: usize,
    body_end_frame: usize,
}

struct DecodedWave {
    samples: Vec<f32>,
    sample_rate_hz: u32,
    channel_count: usize,
    sample_width_bits: u16,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (matrix_path, qualification_path, output_dir) = parse_args()?;
    if output_dir.exists() {
        return Err(format!("output directory already exists: {}", output_dir.display()).into());
    }
    let matrix_bytes = fs::read(&matrix_path)?;
    let matrix: Matrix = serde_json::from_slice(&matrix_bytes)?;
    let (families, expected_condition_count, result_schema, matrix_label): (
        &[&str],
        usize,
        &str,
        &str,
    ) = match matrix.schema.as_str() {
        MATRIX_SCHEMA_V6 => (&["F1", "F2", "F3"], 24, RESULT_SCHEMA_V1, "Matrix-v6"),
        MATRIX_SCHEMA_V7 => (&["F4"], 8, RESULT_SCHEMA_V2, "Matrix-v7"),
        _ => return Err("unsupported Stage-A matrix schema".into()),
    };
    if matrix.condition_count != expected_condition_count
        || matrix.condition_ids.len() != expected_condition_count
        || matrix.selected_sources.len() != 4
    {
        return Err(format!("{matrix_label} identity or cardinality changed").into());
    }
    let qualification_bytes = fs::read(&qualification_path)?;
    if sha256_hex(&qualification_bytes) != matrix.qualification_artifact_sha256 {
        return Err("qualification artifact SHA-256 changed".into());
    }
    fs::create_dir_all(&output_dir)?;

    let mut results = Vec::with_capacity(expected_condition_count);
    let mut source_access_log = Vec::with_capacity(matrix.selected_sources.len());
    for source in &matrix.selected_sources {
        if source.events.len() != 2 {
            return Err(format!("{} must bind exactly two events", source.case_id).into());
        }
        let source_bytes = fs::read(&source.source_path)?;
        let actual_source_sha256 = sha256_hex(&source_bytes);
        if actual_source_sha256 != source.source_sha256 {
            return Err(format!("source SHA-256 changed: {}", source.case_id).into());
        }
        let decoded = decode_pcm_wave(&source_bytes)?;
        if decoded.sample_rate_hz != source.sample_rate_hz
            || decoded.channel_count != source.channel_count
            || decoded.sample_width_bits != source.sample_width_bits
        {
            return Err(format!("source format changed: {}", source.case_id).into());
        }
        source_access_log.push(json!({
            "case_id": source.case_id,
            "source_path": source.source_path,
            "expected_sha256": source.source_sha256,
            "actual_sha256": actual_source_sha256,
            "byte_count": source_bytes.len(),
            "sample_rate_hz": decoded.sample_rate_hz,
            "channel_count": decoded.channel_count,
            "sample_width_bits": decoded.sample_width_bits,
            "access": "one_exact_registered_development_file_read"
        }));
        let source_pcm_sha256 = pcm_f32le_sha256(
            &decoded.samples,
            decoded.sample_rate_hz,
            decoded.channel_count,
        );
        for event in &source.events {
            if !matches!(event.ordinal, 1 | 2) {
                return Err("matrix event ordinal must be 1 or 2".into());
            }
            for &family in families {
                let condition_id = format!(
                    "{}_{}_event{}",
                    family.to_ascii_lowercase(),
                    source.case_id,
                    event.ordinal
                );
                let input = FrozenEventInput {
                    interleaved_samples: &decoded.samples,
                    sample_rate_hz: decoded.sample_rate_hz,
                    channel_count: decoded.channel_count,
                    region: FrozenEventRegion {
                        onset_frame: event.physical_onset_frame,
                        attack_end_frame: event.attack_end_frame,
                        body_end_frame: event.body_end_frame,
                    },
                };
                let rendered = render_family(
                    family,
                    input,
                    decoded.sample_width_bits,
                    event.physical_onset_frame,
                );
                match rendered {
                    Ok((candidate, policy)) => {
                        let metrics = basic_metrics(
                            &decoded.samples,
                            &candidate,
                            decoded.channel_count,
                            *event,
                        )?;
                        let candidate_pcm_sha256 = pcm_f32le_sha256(
                            &candidate,
                            decoded.sample_rate_hz,
                            decoded.channel_count,
                        );
                        let passed = metrics["passed"].as_bool().unwrap_or(false)
                            && candidate_pcm_sha256 != source_pcm_sha256;
                        let output_path = output_dir.join(format!("{condition_id}.wav"));
                        let output_sha256 = if passed {
                            let wav = encode_float32_wave(
                                &candidate,
                                decoded.sample_rate_hz,
                                decoded.channel_count,
                            )?;
                            write_new(&output_path, &wav)?;
                            Some(sha256_hex(&wav))
                        } else {
                            None
                        };
                        results.push(json!({
                            "condition_id": condition_id,
                            "family": family,
                            "case_id": source.case_id,
                            "source_family": source.source_family,
                            "event_ordinal": event.ordinal,
                            "event": event_json(*event),
                            "render_state": if passed { "rendered_basic_screens_passed" } else { "rejected_basic_screen" },
                            "source_pcm_f32le_sha256": source_pcm_sha256,
                            "candidate_pcm_f32le_sha256": candidate_pcm_sha256,
                            "output_path": if passed { Some(output_path.to_string_lossy().into_owned()) } else { None },
                            "output_wav_sha256": output_sha256,
                            "policy": policy,
                            "basic_metrics": metrics,
                            "advanced_mechanical_screens": "pending_python_full_source_reanalysis",
                            "quality_proof": false,
                            "hardness_proof": false,
                            "human_verdict": "unverified"
                        }));
                    }
                    Err(reason) => results.push(json!({
                        "condition_id": condition_id,
                        "family": family,
                        "case_id": source.case_id,
                        "source_family": source.source_family,
                        "event_ordinal": event.ordinal,
                        "event": event_json(*event),
                        "render_state": "renderer_refused",
                        "refusal": reason,
                        "advanced_mechanical_screens": "not_run",
                        "quality_proof": false,
                        "hardness_proof": false,
                        "human_verdict": "unverified"
                    })),
                }
            }
        }
    }
    if results.len() != expected_condition_count {
        return Err(format!(
            "matrix did not execute exactly {expected_condition_count} conditions"
        )
        .into());
    }
    let actual_condition_ids = results
        .iter()
        .map(|item| item["condition_id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    if actual_condition_ids != matrix.condition_ids {
        return Err("matrix condition identity or order changed".into());
    }
    let rendered_count = results
        .iter()
        .filter(|item| item["render_state"] == "rendered_basic_screens_passed")
        .count();
    let result = json!({
        "schema": result_schema,
        "matrix_path": matrix_path,
        "matrix_sha256": sha256_hex(&matrix_bytes),
        "qualification_artifact_path": qualification_path,
        "qualification_artifact_sha256": matrix.qualification_artifact_sha256,
        "condition_count": expected_condition_count,
        "rendered_basic_screen_pass_count": rendered_count,
        "source_access_log": source_access_log,
        "conditions": results,
        "candidate_render_started": true,
        "advanced_mechanical_screens_complete": false,
        "holdout_audio_accessed": false,
        "commercial_reference_accessed": false,
        "quality_proof": false,
        "hardness_proof": false,
        "human_verdict": "unverified"
    });
    let result_path = output_dir.join("matrix-result.json");
    write_new(&result_path, &serde_json::to_vec_pretty(&result)?)?;
    println!("PASS: {matrix_label} rendered {expected_condition_count} conditions");
    println!("rendered_basic_screen_pass_count={rendered_count}");
    println!("result={}", result_path.display());
    println!("human_verdict=unverified");
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut matrix = None;
    let mut qualification = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {arg}"))?;
        match arg.as_str() {
            "--matrix" => matrix = Some(PathBuf::from(value)),
            "--qualification" => qualification = Some(PathBuf::from(value)),
            "--output-dir" => output = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    Ok((
        matrix.ok_or("--matrix is required")?,
        qualification.ok_or("--qualification is required")?,
        output.ok_or("--output-dir is required")?,
    ))
}

fn render_family(
    family: &str,
    input: FrozenEventInput<'_>,
    sample_width_bits: u16,
    onset_frame: usize,
) -> Result<(Vec<f32>, Value), String> {
    match family {
        "F1" => render_f1_ab_energy_redistribution_v1(input)
            .map(|rendered| {
                let policy = &rendered.policy;
                (
                    rendered.combined,
                    json!({
                        "version_id": policy.version_id,
                        "attack_energy_multiplier": policy.attack_energy_multiplier,
                        "attack_gain": policy.attack_gain,
                        "body_gain": policy.body_gain,
                        "attack_body_crossfade_frames": policy.attack_body_crossfade_frames,
                        "body_fade_frames": policy.body_fade_frames
                    }),
                )
            })
            .map_err(|error| error.to_string()),
        "F2" => {
            let lookbehind_frames = input.sample_rate_hz as usize * 20 / 1000;
            if onset_frame < lookbehind_frames {
                return Err("frozen 20ms lookbehind unavailable".to_owned());
            }
            let quantization_lsb = 2.0_f64.powi(-(i32::from(sample_width_bits) - 1));
            render_f2_exact_complementary_three_band_v1(input, lookbehind_frames, quantization_lsb)
                .map(|rendered| {
                    let policy = &rendered.policy;
                    let bands = policy
                        .bands
                        .iter()
                        .map(|band| {
                            json!({
                                "role": format!("{:?}", band.role),
                                "trusted": band.trusted,
                                "attack_gain": band.attack_gain,
                                "body_gain": band.body_gain
                            })
                        })
                        .collect::<Vec<_>>();
                    (
                        rendered.combined,
                        json!({
                            "version_id": policy.version_id,
                            "f25_hz": policy.f25_hz,
                            "f75_hz": policy.f75_hz,
                            "lookbehind_frames": policy.lookbehind_frames,
                            "bands": bands,
                            "attack_body_crossfade_frames": policy.attack_body_crossfade_frames,
                            "body_fade_frames": policy.body_fade_frames
                        }),
                    )
                })
                .map_err(|error| error.to_string())
        }
        "F3" => {
            let encoding = match sample_width_bits {
                16 => F3PcmEncoding::SignedPcm16,
                24 => F3PcmEncoding::SignedPcm24,
                _ => return Err("unsupported frozen F3 PCM encoding".to_owned()),
            };
            render_f3_causal_envelope_contrast_dynamic_residual_v2(input, encoding)
                .map(|rendered| {
                    let policy = &rendered.policy;
                    (
                        rendered.combined,
                        json!({
                            "version_id": policy.version_id,
                            "pcm_valid_bits": policy.pcm_valid_bits,
                            "lookbehind_frames": policy.lookbehind_frames,
                            "attack_contribution_ratio": policy.attack_contribution_ratio,
                            "body_contribution_ratio": policy.body_contribution_ratio,
                            "attack_source_ratio": policy.source_attack_fast_to_slow_ratio,
                            "attack_only_ratio": policy.attack_only_fast_to_slow_ratio,
                            "body_source_ratio": policy.source_body_fast_to_context_ratio,
                            "body_only_ratio": policy.body_only_fast_to_context_ratio,
                            "controller_hashes": {
                                "raw_attack": policy.controller_hashes.raw_attack_sha256,
                                "raw_body": policy.controller_hashes.raw_body_sha256,
                                "attack_state": policy.controller_hashes.attack_state_sha256,
                                "body_state": policy.controller_hashes.body_state_sha256
                            },
                            "attack_body_crossfade_frames": policy.attack_body_crossfade_frames,
                            "body_fade_frames": policy.body_fade_frames
                        }),
                    )
                })
                .map_err(|error| error.to_string())
        }
        "F4" => {
            let lookbehind_frames = input.sample_rate_hz as usize * 20 / 1_000;
            if onset_frame < lookbehind_frames {
                return Err("frozen 20ms lookbehind unavailable".to_owned());
            }
            let quantization_lsb = 2.0_f64.powi(-(i32::from(sample_width_bits) - 1));
            render_f4_source_native_body_sustain_v1(input, lookbehind_frames, quantization_lsb)
                .map(|rendered| {
                    let policy = &rendered.policy;
                    (
                        rendered.combined,
                        json!({
                            "version_id": policy.version_id,
                            "selected_band": format!("{:?}", policy.selected_band),
                            "selected_band_index": policy.selected_band_index,
                            "band_edges_hz": policy.band_edges_hz,
                            "trusted_bands": policy.trusted_bands,
                            "selected_band_score": policy.selected_band_score,
                            "lookbehind_frames": policy.lookbehind_frames,
                            "body_envelope_frames": policy.body_envelope_frames,
                            "body_entry_frames": policy.body_entry_frames,
                            "body_exit_frames": policy.body_exit_frames,
                            "maximum_additional_band_gain": policy.maximum_additional_band_gain,
                            "maximum_resolved_additional_gain": policy.maximum_resolved_additional_gain,
                            "body_energy_ratio": policy.body_energy_ratio,
                            "output_peak": policy.output_peak,
                            "attack_bit_identical": policy.attack_bit_identical,
                            "playback_rate": [
                                policy.playback_rate_numerator,
                                policy.playback_rate_denominator
                            ]
                        }),
                    )
                })
                .map_err(|error| error.to_string())
        }
        _ => Err(format!("unknown matrix family: {family}")),
    }
}

fn basic_metrics(
    source: &[f32],
    candidate: &[f32],
    channels: usize,
    event: MatrixEvent,
) -> Result<Value, Box<dyn std::error::Error>> {
    if source.len() != candidate.len() || !source.len().is_multiple_of(channels) {
        return Err("candidate shape changed".into());
    }
    let onset = event.physical_onset_frame * channels;
    let attack_end = event.attack_end_frame * channels;
    let body_end = event.body_end_frame * channels;
    if source[..onset] != candidate[..onset] || source[body_end..] != candidate[body_end..] {
        return Err("candidate changed samples outside frozen event support".into());
    }
    let finite = candidate.iter().all(|value| value.is_finite());
    let peak = candidate
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f32, f32::max) as f64;
    let mut source_energy = 0.0;
    let mut delta_energy = 0.0;
    for index in onset..body_end {
        let x = f64::from(source[index]);
        let y = f64::from(candidate[index]);
        source_energy += x * x;
        delta_energy += (y - x) * (y - x);
    }
    if source_energy <= 0.0 {
        return Err("zero source event energy".into());
    }
    let near_identity_delta = (delta_energy / source_energy).sqrt();
    let identity_correlation = correlation(source, candidate, channels, onset, body_end)?;
    let source_body_energy = source[attack_end..body_end]
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>();
    let candidate_body_energy = candidate[attack_end..body_end]
        .iter()
        .map(|value| f64::from(*value).powi(2))
        .sum::<f64>();
    if source_body_energy <= 0.0 {
        return Err("zero source body energy".into());
    }
    let body_energy_ratio = candidate_body_energy / source_body_energy;
    let passed = finite
        && peak < 1.0
        && near_identity_delta >= 0.05
        && identity_correlation >= 0.8
        && (0.5..=2.0).contains(&body_energy_ratio);
    Ok(json!({
        "finite": finite,
        "absolute_peak": peak,
        "absolute_peak_strict_maximum": 1.0,
        "near_identity_delta": near_identity_delta,
        "near_identity_minimum": 0.05,
        "zero_lag_identity_correlation": identity_correlation,
        "identity_correlation_minimum": 0.8,
        "body_energy_ratio": body_energy_ratio,
        "body_energy_ratio_range": [0.5, 2.0],
        "untouched_regions_bit_identical": true,
        "playback_rate": [1, 1],
        "frame_count_unchanged": true,
        "sample_rate_unchanged": true,
        "passed": passed
    }))
}

fn correlation(
    source: &[f32],
    candidate: &[f32],
    channels: usize,
    start: usize,
    end: usize,
) -> Result<f64, Box<dyn std::error::Error>> {
    let frame_count = (end - start) / channels;
    let mut source_means = vec![0.0; channels];
    let mut candidate_means = vec![0.0; channels];
    for index in start..end {
        let channel = index % channels;
        source_means[channel] += f64::from(source[index]);
        candidate_means[channel] += f64::from(candidate[index]);
    }
    for channel in 0..channels {
        source_means[channel] /= frame_count as f64;
        candidate_means[channel] /= frame_count as f64;
    }
    let mut dot = 0.0;
    let mut source_norm = 0.0;
    let mut candidate_norm = 0.0;
    for index in start..end {
        let channel = index % channels;
        let x = f64::from(source[index]) - source_means[channel];
        let y = f64::from(candidate[index]) - candidate_means[channel];
        dot += x * y;
        source_norm += x * x;
        candidate_norm += y * y;
    }
    let denominator = (source_norm * candidate_norm).sqrt();
    if denominator <= 0.0 || !denominator.is_finite() {
        return Err("identity correlation undefined".into());
    }
    Ok(dot / denominator)
}

fn event_json(event: MatrixEvent) -> Value {
    json!({
        "ordinal": event.ordinal,
        "physical_onset_frame": event.physical_onset_frame,
        "attack_end_frame": event.attack_end_frame,
        "body_end_frame": event.body_end_frame
    })
}

fn decode_pcm_wave(bytes: &[u8]) -> Result<DecodedWave, Box<dyn std::error::Error>> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("source is not RIFF/WAVE".into());
    }
    if u32::from_le_bytes(bytes[4..8].try_into()?) as usize != bytes.len() - 8 {
        return Err("source RIFF size changed".into());
    }
    let mut format = None;
    let mut data = None;
    let mut offset = 12;
    while offset < bytes.len() {
        if offset + 8 > bytes.len() {
            return Err("truncated source chunk".into());
        }
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into()?) as usize;
        let start = offset + 8;
        let end = start.checked_add(size).ok_or("source chunk overflow")?;
        let padded = end + (size & 1);
        if padded > bytes.len() {
            return Err("source chunk exceeds RIFF".into());
        }
        match &bytes[offset..offset + 4] {
            b"fmt " => {
                if format.is_some() || size != 16 {
                    return Err("source PCM fmt chunk changed".into());
                }
                format = Some((
                    u16::from_le_bytes(bytes[start..start + 2].try_into()?),
                    u16::from_le_bytes(bytes[start + 2..start + 4].try_into()?),
                    u32::from_le_bytes(bytes[start + 4..start + 8].try_into()?),
                    u16::from_le_bytes(bytes[start + 14..start + 16].try_into()?),
                ));
            }
            b"data" => {
                if format.is_none() || data.is_some() {
                    return Err("source data chunk order changed".into());
                }
                data = Some(&bytes[start..end]);
            }
            _ => {}
        }
        offset = padded;
    }
    let (tag, channels, sample_rate_hz, bits) = format.ok_or("source fmt missing")?;
    let data = data.ok_or("source data missing")?;
    if tag != 1 || !matches!(bits, 16 | 24) || !matches!(channels, 1 | 2) {
        return Err("unsupported source PCM format".into());
    }
    let width = usize::from(bits / 8);
    if data.len() % width != 0 {
        return Err("source PCM alignment changed".into());
    }
    let samples = if bits == 16 {
        data.chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect()
    } else {
        data.chunks_exact(3)
            .map(|chunk| {
                let unsigned =
                    i32::from(chunk[0]) | (i32::from(chunk[1]) << 8) | (i32::from(chunk[2]) << 16);
                let signed = if unsigned & 0x80_0000 != 0 {
                    unsigned - 0x100_0000
                } else {
                    unsigned
                };
                signed as f32 / 8_388_608.0
            })
            .collect()
    };
    Ok(DecodedWave {
        samples,
        sample_rate_hz,
        channel_count: usize::from(channels),
        sample_width_bits: bits,
    })
}

fn encode_float32_wave(
    samples: &[f32],
    sample_rate_hz: u32,
    channels: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let channels_u16 = u16::try_from(channels)?;
    let data_size = u32::try_from(samples.len().checked_mul(4).ok_or("WAV size overflow")?)?;
    let riff_size = 36_u32.checked_add(data_size).ok_or("RIFF size overflow")?;
    let byte_rate = sample_rate_hz
        .checked_mul(u32::from(channels_u16))
        .and_then(|value| value.checked_mul(4))
        .ok_or("WAV byte rate overflow")?;
    let block_align = channels_u16
        .checked_mul(4)
        .ok_or("WAV block align overflow")?;
    let mut output = Vec::with_capacity(data_size as usize + 44);
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&riff_size.to_le_bytes());
    output.extend_from_slice(b"WAVEfmt ");
    output.extend_from_slice(&16_u32.to_le_bytes());
    output.extend_from_slice(&3_u16.to_le_bytes());
    output.extend_from_slice(&channels_u16.to_le_bytes());
    output.extend_from_slice(&sample_rate_hz.to_le_bytes());
    output.extend_from_slice(&byte_rate.to_le_bytes());
    output.extend_from_slice(&block_align.to_le_bytes());
    output.extend_from_slice(&32_u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        output.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(output)
}

fn pcm_f32le_sha256(samples: &[f32], sample_rate_hz: u32, channels: usize) -> String {
    let mut digest = Sha256::new();
    digest.update((PCM_HASH_DOMAIN.len() as u32).to_le_bytes());
    digest.update(PCM_HASH_DOMAIN.as_bytes());
    digest.update(sample_rate_hz.to_le_bytes());
    digest.update((channels as u32).to_le_bytes());
    digest.update(((samples.len() / channels) as u64).to_le_bytes());
    for sample in samples {
        digest.update(sample.to_bits().to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
