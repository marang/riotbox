use crate::ui::shell_state::JamShellState;
use crate::ui::source_trust_summary::energy_label;
use crate::ui::source_trust_summary::source_timing_clock_line;
use crate::ui::source_trust_summary::source_timing_readiness_line;
use crate::ui::source_trust_summary::source_timing_warning_line;
use crate::ui::source_trust_summary::source_warning_lines;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use riotbox_core::source_graph::DecodeProfile;
use riotbox_core::source_graph::Section;
use riotbox_core::source_graph::SectionLabelHint;

pub(in crate::ui) fn source_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let source = &shell.app.jam_view.source;
    let first_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.first())
        .map(section_compact_label)
        .unwrap_or_else(|| "first none".into());
    let second_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.get(1))
        .map(section_compact_label)
        .unwrap_or_else(|| "next none".into());

    vec![
        Line::from(format!(
            "tempo {:.1} | map {} | {}",
            source.bpm_estimate.unwrap_or(0.0),
            source.source_map.mode.label(),
            source.source_map.trust_label
        )),
        Line::from(format!(
            "sections {} | loops {} | hooks {}",
            source.section_count, source.loop_candidate_count, source.hook_candidate_count
        )),
        Line::from(source_timing_clock_line(shell)),
        source_timing_readiness_line(shell),
        Line::from(source_timing_warning_line(shell)),
        Line::from(first_section),
        Line::from(second_section),
        source_warning_lines(shell)
            .into_iter()
            .next()
            .unwrap_or_else(|| Line::from("warnings clear")),
    ]
}

pub(in crate::ui) fn section_label(section: &Section) -> &'static str {
    match section.label_hint {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "unknown",
    }
}

pub(in crate::ui) fn decode_profile_label(profile: &DecodeProfile) -> String {
    match profile {
        DecodeProfile::Native => "native".into(),
        DecodeProfile::NormalizedStereo => "normalized_stereo".into(),
        DecodeProfile::NormalizedMono => "normalized_mono".into(),
        DecodeProfile::Custom(value) => value.clone(),
    }
}

pub(in crate::ui) fn source_identity_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("source {}", graph.source.source_id)),
            Line::from(graph.source.path.clone()),
            Line::from(format!(
                "{:.2}s | {} Hz | {} ch | {}",
                graph.source.duration_seconds,
                graph.source.sample_rate,
                graph.source.channel_count,
                decode_profile_label(&graph.source.decode_profile)
            )),
            Line::from(format!("hash {}", graph.source.content_hash)),
        ],
        None => vec![Line::from("no source graph loaded")],
    }
}

pub(in crate::ui) fn section_compact_label(section: &Section) -> String {
    format!(
        "{} bars {}-{}",
        section_label_hint_compact(&section.label_hint),
        section.bar_start,
        section.bar_end
    )
}

pub(in crate::ui) fn section_label_hint_compact(label_hint: &SectionLabelHint) -> &'static str {
    match label_hint {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "unknown",
    }
}

pub(in crate::ui) fn source_section_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.sections.is_empty() => graph
            .sections
            .iter()
            .take(6)
            .map(|section| {
                ListItem::new(format!(
                    "{} | bars {}-{} | {:.2}s-{:.2}s | {} | conf {:.2}",
                    section_label(section),
                    section.bar_start,
                    section.bar_end,
                    section.start_seconds,
                    section.end_seconds,
                    energy_label(section),
                    section.confidence
                ))
            })
            .collect(),
        Some(_) => vec![ListItem::new("no sections available")],
        None => vec![ListItem::new("no source graph loaded")],
    }
}
