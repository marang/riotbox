use riotbox_core::source_graph::{EnergyClass, SectionLabelHint};

pub(crate) fn scene_label_hint(label: &str) -> SectionLabelHint {
    match label {
        "intro" => SectionLabelHint::Intro,
        "build" => SectionLabelHint::Build,
        "drop" => SectionLabelHint::Drop,
        "break" => SectionLabelHint::Break,
        "verse" => SectionLabelHint::Verse,
        "chorus" => SectionLabelHint::Chorus,
        "bridge" => SectionLabelHint::Bridge,
        "outro" => SectionLabelHint::Outro,
        _ => SectionLabelHint::Unknown,
    }
}

pub(crate) fn scene_energy_for_label(label: &str) -> EnergyClass {
    match label {
        "drop" | "chorus" => EnergyClass::High,
        "break" | "outro" => EnergyClass::Low,
        "intro" | "build" | "verse" | "bridge" => EnergyClass::Medium,
        _ => EnergyClass::Unknown,
    }
}
