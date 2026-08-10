use super::*;

const CASE_ID: &str = "synthetic_registered_source";
const LOGICAL_PATH: &str = "registered/development/source.wav";

fn expectation(
    raw_sha256: &str,
    sample_rate_hz: u32,
    channels: u16,
    sample_width_bits: u16,
) -> StageAAuthorizedSourceAccess {
    authorize_synthetic_source_for_test(expectation_record(
        raw_sha256,
        sample_rate_hz,
        channels,
        sample_width_bits,
    ))
    .expect("valid synthetic source-blind authorization")
}

fn expectation_record(
    raw_sha256: &str,
    sample_rate_hz: u32,
    channels: u16,
    sample_width_bits: u16,
) -> StageARegistryPcmExpectation {
    StageARegistryPcmExpectation {
        case_id: CASE_ID.to_owned(),
        logical_source_path: LOGICAL_PATH.to_owned(),
        expected_raw_wav_sha256: raw_sha256.to_owned(),
        format: StageARegistryPcmFormat {
            sample_rate_hz,
            channels,
            sample_width_bits,
            maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
        },
    }
}

fn valid_safe_access_receipt() -> StageASafeAccessGateReceipt<'static> {
    StageASafeAccessGateReceipt {
        registry_path: STAGE_A_SOURCE_REGISTRY_PATH,
        registry_schema: STAGE_A_SOURCE_REGISTRY_SCHEMA,
        registry_sha256: STAGE_A_SOURCE_REGISTRY_SHA256,
        partition: DEVELOPMENT_PARTITION,
        session_kind: STAGE_A_QUALIFICATION_SESSION_KIND,
        session_id: "fixture-stage-a-session",
        access_log_schema: STAGE_A_DEVELOPMENT_ACCESS_LOG_SCHEMA,
        access_log_path: "/tmp/fixture-stage-a-access.json",
        access_record_index: 0,
        access_record_status: VERIFIED_AND_DELIVERED_ACCESS_STATUS,
        case_id: "oga_cinameng_can_be_so_beautiful",
        logical_source_path: concat!(
            "data/test_audio/external/RIOTBOX-1423/wav/",
            "dense_oga_cinameng_can_be_so_beautiful.wav"
        ),
        expected_raw_wav_sha256: "bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f",
        accessed_raw_wav_sha256: "bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f",
    }
}

#[test]
fn safe_gate_alone_mints_exact_registry_session_access_and_source_provenance() {
    let authorization = authorize_stage_a_development_access(valid_safe_access_receipt())
        .expect("exact frozen development receipt");
    assert_eq!(
        authorization.expectation.case_id,
        "oga_cinameng_can_be_so_beautiful"
    );
    assert_eq!(
        authorization.provenance.registry.sha256,
        STAGE_A_SOURCE_REGISTRY_SHA256
    );
    assert_eq!(
        authorization.provenance.session.session_id,
        "fixture-stage-a-session"
    );
    assert_eq!(authorization.provenance.access.access_record_index, 0);

    for (index, case_id, logical_source_path, raw_sha256, expected_format) in [
        (
            0,
            "oga_cinameng_can_be_so_beautiful",
            "data/test_audio/external/RIOTBOX-1423/wav/dense_oga_cinameng_can_be_so_beautiful.wav",
            "bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f",
            (48_000, 2, 16),
        ),
        (
            1,
            "oga_marwan_cinematic_percussion",
            "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_marwan_cinematic_percussion.wav",
            "9373f577cf09135e2b7e3ce0e946ce5af6ea333f5a7462ab9126f6802f9986f3",
            (48_000, 2, 16),
        ),
        (
            2,
            "oga_william_hector_horde_war_drums",
            "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_william_hector_horde_war_drums.wav",
            "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417",
            (44_100, 2, 24),
        ),
        (
            3,
            "oga_frosty_ham_osdrums",
            "data/test_audio/external/RIOTBOX-1423/wav/sparse_oga_frosty_ham_osdrums.wav",
            "7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2",
            (44_100, 2, 16),
        ),
    ] {
        let mut receipt = valid_safe_access_receipt();
        receipt.access_record_index = index;
        receipt.case_id = case_id;
        receipt.logical_source_path = logical_source_path;
        receipt.expected_raw_wav_sha256 = raw_sha256;
        receipt.accessed_raw_wav_sha256 = raw_sha256;
        let authorized = authorize_stage_a_development_access(receipt)
            .expect("one exact frozen Stage-A development identity");
        assert_eq!(
            (
                authorized.expectation.format.sample_rate_hz,
                authorized.expectation.format.channels,
                authorized.expectation.format.sample_width_bits,
            ),
            expected_format
        );
    }

    let mut forged = valid_safe_access_receipt();
    forged.registry_sha256 = "0";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::AccessProvenanceMismatch {
            field: "registry_sha256"
        })
    );

    let mut forged = valid_safe_access_receipt();
    forged.partition = "holdout_a";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::AccessProvenanceMismatch { field: "partition" })
    );

    let mut forged = valid_safe_access_receipt();
    forged.session_kind = "StageAPrequalificationSession";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::AccessProvenanceMismatch {
            field: "session_kind"
        })
    );

    let mut forged = valid_safe_access_receipt();
    forged.session_id = " invalid ";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::InvalidQualificationSessionId)
    );

    let mut forged = valid_safe_access_receipt();
    forged.access_log_schema = "riotbox.source_holdout_development_access_log.v1";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::AccessProvenanceMismatch {
            field: "access_log_schema"
        })
    );

    let mut forged = valid_safe_access_receipt();
    forged.access_record_status = "opened";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::AccessProvenanceMismatch {
            field: "access_record_status"
        })
    );

    let mut forged = valid_safe_access_receipt();
    forged.access_record_index = 1;
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::UnauthorizedDevelopmentSourceIdentity)
    );

    let mut forged = valid_safe_access_receipt();
    forged.accessed_raw_wav_sha256 = "0";
    assert_eq!(
        authorize_stage_a_development_access(forged),
        Err(StageAQualificationPcmError::UnauthorizedDevelopmentSourceIdentity)
    );
}

#[test]
fn pcm16_integer_goldens_bind_exact_lsb_and_full_source_hash() {
    let integers = [i16::MIN, -1, 0, 1, i16::MAX];
    let data = integers
        .into_iter()
        .flat_map(i16::to_le_bytes)
        .collect::<Vec<_>>();
    let wave = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, data);
    let raw_hash = sha256_hex(&wave);

    let bound = bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &wave)
        .expect("strict synthetic PCM16 WAV");

    assert_eq!(bound.case_id, CASE_ID);
    assert_eq!(bound.logical_source_path, LOGICAL_PATH);
    assert_eq!(bound.raw_wav_sha256, raw_hash);
    assert_eq!(bound.frame_count, 5);
    assert_eq!(bound.format.encoding, StageAPcmEncoding::SignedPcm16);
    assert_eq!(bound.format.container_bits, 16);
    assert_eq!(bound.format.valid_bits, 16);
    assert_eq!(bound.format.input_lsb, 2.0_f64.powi(-15));
    assert_eq!(
        bound.format.maximum_duration_seconds,
        STAGE_A_MAXIMUM_DURATION_SECONDS
    );
    assert_eq!(
        bound.qualification_provenance.registry.sha256,
        STAGE_A_SOURCE_REGISTRY_SHA256
    );
    assert_eq!(
        bound.interleaved_samples,
        vec![
            -1.0,
            -1.0 / 32_768.0,
            0.0,
            1.0 / 32_768.0,
            f32::from(i16::MAX) / 32_768.0,
        ]
    );
    assert_eq!(
        bound.pcm_f32le_sha256,
        "be2abff76fe003c0c2471d736606215695ea777e75049ec1e920aa0d0ae57f2d"
    );
}

#[test]
fn pcm24_integer_goldens_bind_exact_lsb_without_clamping() {
    let integers = [-8_388_608, -1, 0, 1, 8_388_607];
    let data = integers
        .into_iter()
        .flat_map(pcm24_le_bytes)
        .collect::<Vec<_>>();
    let wave = pcm_wave(PCM_FORMAT_TAG, 44_100, 1, 24, data);
    let raw_hash = sha256_hex(&wave);

    let bound = bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 44_100, 1, 24), &wave)
        .expect("strict synthetic PCM24 WAV");

    assert_eq!(bound.frame_count, 5);
    assert_eq!(bound.format.encoding, StageAPcmEncoding::SignedPcm24);
    assert_eq!(bound.format.container_bits, 24);
    assert_eq!(bound.format.valid_bits, 24);
    assert_eq!(bound.format.input_lsb, 2.0_f64.powi(-23));
    assert_eq!(
        bound.interleaved_samples,
        vec![
            -1.0,
            -1.0 / 8_388_608.0,
            0.0,
            1.0 / 8_388_608.0,
            8_388_607.0 / 8_388_608.0,
        ]
    );
    assert_eq!(
        bound.pcm_f32le_sha256,
        "14240b34f81c3fb475f52f9afa2473705c81259e2b2461022b16e5e654868537"
    );
}

#[test]
fn rejects_rifx_and_malformed_riff_sizes() {
    let mut rifx = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![0, 0]);
    rifx[..4].copy_from_slice(b"RIFX");
    let rifx_hash = sha256_hex(&rifx);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&rifx_hash, 48_000, 1, 16), &rifx),
        Err(StageAQualificationPcmError::InvalidRiffWave(
            "container is not little-endian RIFF"
        ))
    );

    let mut wrong_size = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![0, 0]);
    wrong_size[4..8].copy_from_slice(&0_u32.to_le_bytes());
    let wrong_size_hash = sha256_hex(&wrong_size);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&wrong_size_hash, 48_000, 1, 16), &wrong_size,),
        Err(StageAQualificationPcmError::InvalidRiffWave(
            "declared RIFF size does not equal byte length"
        ))
    );
}

#[test]
fn rejects_extensible_float_and_compressed_format_tags() {
    for tag in [0xfffe, 3, 6] {
        let wave = pcm_wave(tag, 48_000, 1, 16, vec![0, 0]);
        let raw_hash = sha256_hex(&wave);
        assert_eq!(
            bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &wave),
            Err(StageAQualificationPcmError::UnsupportedFormatTag(tag))
        );
    }
}

#[test]
fn rejects_duplicate_fmt_and_data_chunks() {
    let format = pcm_format_chunk(PCM_FORMAT_TAG, 48_000, 1, 16);
    let duplicate_fmt = riff_wave(vec![
        (*b"fmt ", format.clone()),
        (*b"fmt ", format.clone()),
        (*b"data", vec![0, 0]),
    ]);
    let raw_hash = sha256_hex(&duplicate_fmt);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &duplicate_fmt,),
        Err(StageAQualificationPcmError::DuplicateChunk("fmt"))
    );

    let duplicate_data = riff_wave(vec![
        (*b"fmt ", format),
        (*b"data", vec![0, 0]),
        (*b"data", vec![0, 0]),
    ]);
    let raw_hash = sha256_hex(&duplicate_data);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &duplicate_data,),
        Err(StageAQualificationPcmError::DuplicateChunk("data"))
    );
}

#[test]
fn rejects_noncanonical_fmt_length_and_data_before_fmt() {
    let mut extended_format = pcm_format_chunk(PCM_FORMAT_TAG, 48_000, 1, 16);
    extended_format.extend_from_slice(&0_u16.to_le_bytes());
    let extended_fmt = riff_wave(vec![(*b"fmt ", extended_format), (*b"data", vec![0, 0])]);
    let raw_hash = sha256_hex(&extended_fmt);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &extended_fmt),
        Err(StageAQualificationPcmError::InvalidRiffWave(
            "fmt chunk must be exactly 16 bytes"
        ))
    );

    let data_before_fmt = riff_wave(vec![
        (*b"data", vec![0, 0]),
        (*b"fmt ", pcm_format_chunk(PCM_FORMAT_TAG, 48_000, 1, 16)),
    ]);
    let raw_hash = sha256_hex(&data_before_fmt);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &data_before_fmt),
        Err(StageAQualificationPcmError::InvalidRiffWave(
            "data chunk precedes fmt chunk"
        ))
    );
}

#[test]
fn rejects_registry_format_mismatch_without_repair() {
    let wave = pcm_wave(PCM_FORMAT_TAG, 48_000, 2, 16, vec![0, 0, 0, 0]);
    let raw_hash = sha256_hex(&wave);
    for (expected, field, expected_value, actual_value) in [
        (
            StageARegistryPcmFormat {
                sample_rate_hz: 44_100,
                channels: 2,
                sample_width_bits: 16,
                maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
            },
            "sample_rate_hz",
            44_100,
            48_000,
        ),
        (
            StageARegistryPcmFormat {
                sample_rate_hz: 48_000,
                channels: 1,
                sample_width_bits: 16,
                maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
            },
            "channels",
            1,
            2,
        ),
        (
            StageARegistryPcmFormat {
                sample_rate_hz: 48_000,
                channels: 2,
                sample_width_bits: 24,
                maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
            },
            "sample_width_bits",
            24,
            16,
        ),
    ] {
        let actual = bind_stage_a_registry_pcm_wav(
            authorize_synthetic_source_for_test(StageARegistryPcmExpectation {
                case_id: CASE_ID.to_owned(),
                logical_source_path: LOGICAL_PATH.to_owned(),
                expected_raw_wav_sha256: raw_hash.clone(),
                format: expected,
            })
            .unwrap(),
            &wave,
        );
        assert_eq!(
            actual,
            Err(StageAQualificationPcmError::RegistryFormatMismatch {
                field,
                expected: expected_value,
                actual: actual_value,
            })
        );
    }
}

#[test]
fn rejects_wrong_byte_rate_block_align_empty_and_misaligned_data() {
    let mut wrong_byte_rate = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![0, 0]);
    wrong_byte_rate[28..32].copy_from_slice(&1_u32.to_le_bytes());
    let raw_hash = sha256_hex(&wrong_byte_rate);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &wrong_byte_rate,),
        Err(StageAQualificationPcmError::InvalidByteRate {
            actual: 1,
            expected: 96_000,
        })
    );

    let mut wrong_block_align = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![0, 0]);
    wrong_block_align[32..34].copy_from_slice(&1_u16.to_le_bytes());
    let raw_hash = sha256_hex(&wrong_block_align);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &wrong_block_align,),
        Err(StageAQualificationPcmError::InvalidBlockAlign {
            actual: 1,
            expected: 2,
        })
    );

    let empty = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![]);
    let raw_hash = sha256_hex(&empty);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &empty),
        Err(StageAQualificationPcmError::EmptyPcmData)
    );

    let misaligned = pcm_wave(PCM_FORMAT_TAG, 48_000, 2, 16, vec![0, 0]);
    let raw_hash = sha256_hex(&misaligned);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 2, 16), &misaligned),
        Err(StageAQualificationPcmError::MisalignedPcmData {
            byte_count: 2,
            block_align: 4,
        })
    );
}

#[test]
fn rejects_pcm_exceeding_the_exact_registry_duration() {
    let maximum_frame_count = 48_000_u64 * u64::from(STAGE_A_MAXIMUM_DURATION_SECONDS) + 1;
    let excessive_frame_count = usize::try_from(maximum_frame_count + 1).unwrap();
    let wave = pcm_wave(
        PCM_FORMAT_TAG,
        48_000,
        1,
        16,
        vec![0; excessive_frame_count * 2],
    );
    let raw_hash = sha256_hex(&wave);
    assert_eq!(
        bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 1, 16), &wave),
        Err(StageAQualificationPcmError::PcmDurationExceedsRegistry {
            frame_count: excessive_frame_count,
            maximum_frame_count,
        })
    );
}

#[test]
fn bound_f3_wrapper_derives_encoding_and_lsb_without_caller_override() {
    let (wave, region) = f3_step_body_pcm16_wave();
    let raw_hash = sha256_hex(&wave);
    let bound = bind_stage_a_registry_pcm_wav(expectation(&raw_hash, 48_000, 2, 16), &wave)
        .expect("strict source-blind step/body PCM16 WAV");

    let rendered = render_f3_from_stage_a_bound_pcm_v2(&bound, region)
        .expect("bound F3-v2 render derives encoding from PCM provenance");
    assert_eq!(rendered.policy.pcm_encoding, F3PcmEncoding::SignedPcm16);
    assert_eq!(rendered.policy.pcm_valid_bits, 16);
    assert_eq!(rendered.policy.normalized_input_lsb, bound.format.input_lsb);
}

#[test]
fn rejects_raw_hash_identity_and_expected_format_contract_failures() {
    let wave = pcm_wave(PCM_FORMAT_TAG, 48_000, 1, 16, vec![0, 0]);
    assert!(matches!(
        bind_stage_a_registry_pcm_wav(expectation(&"0".repeat(64), 48_000, 1, 16), &wave),
        Err(StageAQualificationPcmError::RawWavSha256Mismatch { .. })
    ));
    assert_eq!(
        authorize_synthetic_source_for_test(expectation_record("ABC", 48_000, 1, 16)),
        Err(StageAQualificationPcmError::InvalidExpectedRawSha256)
    );
    let raw_hash = sha256_hex(&wave);
    let mut unsafe_path = expectation_record(&raw_hash, 48_000, 1, 16);
    unsafe_path.logical_source_path = "../source.wav".to_owned();
    assert_eq!(
        authorize_synthetic_source_for_test(unsafe_path),
        Err(StageAQualificationPcmError::UnsafeLogicalSourcePath)
    );
    assert_eq!(
        authorize_synthetic_source_for_test(expectation_record(&raw_hash, 31_999, 1, 16)),
        Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "sample_rate_hz",
            value: 31_999,
        })
    );
    let mut wrong_duration = expectation_record(&raw_hash, 48_000, 1, 16);
    wrong_duration.format.maximum_duration_seconds = 15;
    assert_eq!(
        authorize_synthetic_source_for_test(wrong_duration),
        Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "maximum_duration_seconds",
            value: 15,
        })
    );
}

fn pcm_wave(
    format_tag: u16,
    sample_rate_hz: u32,
    channels: u16,
    bits_per_sample: u16,
    data: Vec<u8>,
) -> Vec<u8> {
    riff_wave(vec![
        (
            *b"fmt ",
            pcm_format_chunk(format_tag, sample_rate_hz, channels, bits_per_sample),
        ),
        (*b"data", data),
    ])
}

fn pcm_format_chunk(
    format_tag: u16,
    sample_rate_hz: u32,
    channels: u16,
    bits_per_sample: u16,
) -> Vec<u8> {
    let bytes_per_sample = bits_per_sample / 8;
    let block_align = channels * bytes_per_sample;
    let byte_rate = sample_rate_hz * u32::from(block_align);
    let mut format = Vec::with_capacity(16);
    format.extend_from_slice(&format_tag.to_le_bytes());
    format.extend_from_slice(&channels.to_le_bytes());
    format.extend_from_slice(&sample_rate_hz.to_le_bytes());
    format.extend_from_slice(&byte_rate.to_le_bytes());
    format.extend_from_slice(&block_align.to_le_bytes());
    format.extend_from_slice(&bits_per_sample.to_le_bytes());
    format
}

fn riff_wave(chunks: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    for (id, payload) in chunks {
        bytes.extend_from_slice(&id);
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        if !payload.len().is_multiple_of(2) {
            bytes.push(0);
        }
    }
    let riff_size = u32::try_from(bytes.len() - 8).unwrap();
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    bytes
}

fn pcm24_le_bytes(value: i32) -> [u8; 3] {
    let encoded = value.to_le_bytes();
    [encoded[0], encoded[1], encoded[2]]
}

fn f3_step_body_pcm16_wave() -> (Vec<u8>, FrozenEventRegion) {
    let frame_count = 4_608;
    let onset_frame = 1_152;
    let high_end_frame = onset_frame + 192;
    let attack_end_frame = onset_frame + 384;
    let body_end_frame = onset_frame + 2_304;
    let mut data = Vec::with_capacity(frame_count * 4);
    for frame in 0..frame_count {
        let amplitude = if frame < onset_frame {
            1.0 / 32.0
        } else if frame < high_end_frame {
            3.0 / 8.0
        } else if frame < body_end_frame {
            3.0 / 16.0
        } else {
            1.0 / 32.0
        };
        let phase = std::f64::consts::TAU * frame as f64 / 64.0;
        for sample in [amplitude * phase.cos(), amplitude * phase.sin()] {
            let integer = (sample * 32_768.0).round().clamp(-32_768.0, 32_767.0) as i16;
            data.extend_from_slice(&integer.to_le_bytes());
        }
    }
    (
        pcm_wave(PCM_FORMAT_TAG, 48_000, 2, 16, data),
        FrozenEventRegion {
            onset_frame,
            attack_end_frame,
            body_end_frame,
        },
    )
}
