#[derive(Clone, Debug, PartialEq)]
struct W30SourceSliceChoicePlan {
    offsets: Vec<usize>,
    proof: W30SourceSliceChoiceProof,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct W30SourceSliceChoiceProof {
    applied: bool,
    candidate_count: usize,
    unique_source_offset_count: usize,
    selected_offset_span_samples: usize,
    min_selected_offset_samples: usize,
    max_selected_offset_samples: usize,
    reason: &'static str,
}

#[derive(Serialize)]
struct ManifestW30SourceSliceChoiceProof {
    applied: bool,
    candidate_count: usize,
    unique_source_offset_count: usize,
    selected_offset_span_samples: usize,
    min_selected_offset_samples: usize,
    max_selected_offset_samples: usize,
    reason: &'static str,
}

const W30_SOURCE_SLICE_CHOICE_CANDIDATE_COUNT: usize = 8;
const W30_SOURCE_SLICE_CHOICE_MIN_UNIQUE_OFFSETS: usize = 4;

fn manifest_w30_source_slice_choice_proof(
    proof: W30SourceSliceChoiceProof,
) -> ManifestW30SourceSliceChoiceProof {
    ManifestW30SourceSliceChoiceProof {
        applied: proof.applied,
        candidate_count: proof.candidate_count,
        unique_source_offset_count: proof.unique_source_offset_count,
        selected_offset_span_samples: proof.selected_offset_span_samples,
        min_selected_offset_samples: proof.min_selected_offset_samples,
        max_selected_offset_samples: proof.max_selected_offset_samples,
        reason: proof.reason,
    }
}

fn w30_source_slice_choice_plan(preview: &W30PreviewSampleWindow) -> W30SourceSliceChoicePlan {
    let sample_count = preview.sample_count.min(W30_PREVIEW_SAMPLE_WINDOW_LEN);
    if sample_count == 0 {
        return W30SourceSliceChoicePlan {
            offsets: vec![0],
            proof: W30SourceSliceChoiceProof {
                applied: false,
                candidate_count: 0,
                unique_source_offset_count: 1,
                selected_offset_span_samples: 0,
                min_selected_offset_samples: 0,
                max_selected_offset_samples: 0,
                reason: "source_slice_choice_empty_preview",
            },
        };
    }

    let mut candidates = source_slice_choice_candidates(&preview.samples[..sample_count]);
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut offsets = candidates
        .iter()
        .take(W30_SOURCE_SLICE_CHOICE_MIN_UNIQUE_OFFSETS + 2)
        .map(|candidate| candidate.offset_samples)
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    offsets.dedup();
    if offsets.is_empty() {
        offsets.push(0);
    }

    let min_selected_offset_samples = offsets.iter().copied().min().unwrap_or(0);
    let max_selected_offset_samples = offsets.iter().copied().max().unwrap_or(0);
    let selected_offset_span_samples =
        max_selected_offset_samples.saturating_sub(min_selected_offset_samples);
    let unique_source_offset_count = offsets.len();
    let applied = unique_source_offset_count >= W30_SOURCE_SLICE_CHOICE_MIN_UNIQUE_OFFSETS
        && selected_offset_span_samples > sample_count / 4;

    W30SourceSliceChoicePlan {
        offsets,
        proof: W30SourceSliceChoiceProof {
            applied,
            candidate_count: candidates.len(),
            unique_source_offset_count,
            selected_offset_span_samples,
            min_selected_offset_samples,
            max_selected_offset_samples,
            reason: if applied {
                "source_analyzed_slice_choice_variation"
            } else {
                "source_slice_choice_too_static"
            },
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct W30SourceSliceChoiceCandidate {
    offset_samples: usize,
    score: f32,
}

fn source_slice_choice_candidates(samples: &[f32]) -> Vec<W30SourceSliceChoiceCandidate> {
    let slice_len = (samples.len() / W30_SOURCE_SLICE_CHOICE_CANDIDATE_COUNT).max(1);
    let mut candidates = Vec::with_capacity(W30_SOURCE_SLICE_CHOICE_CANDIDATE_COUNT);
    for index in 0..W30_SOURCE_SLICE_CHOICE_CANDIDATE_COUNT {
        let start = index
            .saturating_mul(slice_len)
            .min(samples.len().saturating_sub(1));
        let end = (start + slice_len).min(samples.len());
        let slice = &samples[start..end];
        let score = rms(slice) + positive_abs_delta(slice) * 0.65 + peak_abs(slice) * 0.08;
        candidates.push(W30SourceSliceChoiceCandidate {
            offset_samples: start,
            score,
        });
    }
    candidates
}

impl W30SourceSliceChoicePlan {
    fn offset_for_stride(&self, stride: usize) -> usize {
        self.offsets[stride % self.offsets.len()]
    }
}
