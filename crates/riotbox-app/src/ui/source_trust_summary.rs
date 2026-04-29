fn source_candidate_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let scorecard = &shell.app.jam_view.source.feral_scorecard;
            let best_loop = graph
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.candidate_type
                        == riotbox_core::source_graph::CandidateType::LoopCandidate
                })
                .max_by(|left, right| left.score.total_cmp(&right.score));
            let best_hook = graph
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.candidate_type
                        == riotbox_core::source_graph::CandidateType::HookCandidate
                })
                .max_by(|left, right| left.score.total_cmp(&right.score));

            vec![
                Line::from(format!(
                    "feral {} | break {}",
                    scorecard.readiness, scorecard.break_rebuild_potential
                )),
                Line::from(format!(
                    "quote risk {} | support {}",
                    scorecard.quote_risk_count, scorecard.break_support_count
                )),
                Line::from(format!(
                    "hooks {} | capture {}",
                    scorecard.hook_fragment_count, scorecard.capture_candidate_count
                )),
                Line::from(format!("use {}", scorecard.top_reason)),
                Line::from(format!(
                    "feral warn {}",
                    if scorecard.warnings.is_empty() {
                        "none".into()
                    } else {
                        scorecard.warnings.join(", ")
                    }
                )),
                Line::from(format!(
                    "loops {} | hooks {}",
                    graph.loop_candidate_count(),
                    graph.hook_candidate_count()
                )),
                Line::from(format!(
                    "best loop {}",
                    best_loop
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!(
                    "best hook {}",
                    best_hook
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!("assets {}", graph.assets.len())),
            ]
        }
        None => vec![Line::from("no candidate information available")],
    }
}

fn source_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("sidecar {}", graph.provenance.sidecar_version)),
            Line::from(format!(
                "providers {}",
                if graph.provenance.provider_set.is_empty() {
                    "none".into()
                } else {
                    graph.provenance.provider_set.join(", ")
                }
            )),
            Line::from(format!("seed {}", graph.provenance.analysis_seed)),
            Line::from(format!("generated {}", graph.provenance.generated_at)),
        ],
        None => vec![Line::from("no provenance available")],
    }
}

fn source_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.analysis_summary.warnings.is_empty() => graph
            .analysis_summary
            .warnings
            .iter()
            .take(4)
            .flat_map(|warning| {
                [
                    Line::from(format!("{}: {}", warning.code, warning.message)),
                    Line::from(""),
                ]
            })
            .collect(),
        Some(_) => vec![Line::from("no source-graph warnings")],
        None => vec![Line::from("no warnings because no source graph is loaded")],
    }
}

fn source_confidence_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "overall {:.2} | break potential {}",
                graph.analysis_summary.overall_confidence,
                quality_label(&graph.analysis_summary.break_rebuild_potential)
            )),
            Line::from(format!(
                "timing {} | section quality {}",
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
            Line::from(format!(
                "summary loops {} | hooks {}",
                graph.analysis_summary.loop_candidate_count,
                graph.analysis_summary.hook_candidate_count
            )),
            Line::from(format!("jam trust {}", trust_summary(shell).headline)),
        ],
        None => vec![Line::from("no confidence summary available")],
    }
}

struct TrustSummary {
    headline: &'static str,
    overall_confidence: f32,
    warning_count: usize,
    timing_quality: &'static str,
    section_quality: &'static str,
}

fn trust_summary(shell: &JamShellState) -> TrustSummary {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let overall = graph.analysis_summary.overall_confidence;
            let headline = if overall >= 0.8 {
                "strong"
            } else if overall >= 0.62 {
                "usable"
            } else {
                "tentative"
            };

            TrustSummary {
                headline,
                overall_confidence: overall,
                warning_count: graph.analysis_summary.warnings.len(),
                timing_quality: quality_label(&graph.analysis_summary.timing_quality),
                section_quality: quality_label(&graph.analysis_summary.section_quality),
            }
        }
        None => TrustSummary {
            headline: "unknown",
            overall_confidence: 0.0,
            warning_count: 0,
            timing_quality: "unknown",
            section_quality: "unknown",
        },
    }
}

fn quality_label(quality: &QualityClass) -> &'static str {
    match quality {
        QualityClass::Low => "low",
        QualityClass::Medium => "medium",
        QualityClass::High => "high",
        QualityClass::Unknown => "unknown",
    }
}

fn energy_label(section: &Section) -> &'static str {
    match section.energy_class {
        EnergyClass::Low => "low",
        EnergyClass::Medium => "medium",
        EnergyClass::High => "high",
        EnergyClass::Peak => "peak",
        EnergyClass::Unknown => "unknown",
    }
}
