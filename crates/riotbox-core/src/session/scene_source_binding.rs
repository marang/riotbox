//! Scene-to-source identity belongs to Session, not to display labels or app state.
use serde::{Deserialize, Serialize};

use super::{SceneState, SessionFile};
use crate::{
    action::{Action, ActionParams},
    ids::{SceneId, SectionId, SourceId},
    source_graph::{Section, SectionLabelHint, SourceGraph, sorted_sections},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneSourceBinding {
    pub scene_id: SceneId,
    pub source_id: SourceId,
    pub section_id: SectionId,
}

impl SceneSourceBinding {
    fn section<'a>(&self, graph: &'a SourceGraph) -> Option<&'a Section> {
        if [&self.scene_id.0, &self.source_id.0, &self.section_id.0]
            .into_iter()
            .any(|id| id.trim().is_empty() || id.chars().any(char::is_control))
            || self.source_id != graph.source.source_id
        {
            return None;
        }
        let mut matches = graph
            .sections
            .iter()
            .filter(|section| section.section_id == self.section_id);
        let section = matches.next()?;
        matches.next().is_none().then_some(section)
    }
}

/// Keep established generated Scene IDs at the creation boundary only. Their
/// labels are presentation; consumers follow the accompanying SectionId.
#[must_use]
pub fn projected_scene_bindings(graph: &SourceGraph) -> Vec<SceneSourceBinding> {
    sorted_sections(graph)
        .into_iter()
        .enumerate()
        .map(|(index, section)| SceneSourceBinding {
            scene_id: SceneId::from(format!(
                "scene-{:02}-{}",
                index + 1,
                label_slug(section.label_hint)
            )),
            source_id: graph.source.source_id.clone(),
            section_id: section.section_id.clone(),
        })
        .collect()
}

impl SceneState {
    /// Explicit malformed/dangling bindings must not be repaired through labels.
    pub fn validate_source_bindings(
        &self,
        graph: &SourceGraph,
    ) -> Result<(), SceneSourceBindingError> {
        let Some(bindings) = &self.source_bindings else {
            return Ok(());
        };
        let mut seen = std::collections::HashSet::new();
        for binding in bindings {
            if !seen.insert(&binding.scene_id) {
                return Err(SceneSourceBindingError::DuplicateScene(
                    binding.scene_id.clone(),
                ));
            }
            if binding.section(graph).is_none() {
                return Err(SceneSourceBindingError::InvalidTarget(binding.clone()));
            }
        }
        Ok(())
    }

    /// Materialize the legacy adapter exactly once. Never repair, infer, or
    /// overwrite an explicit binding list when the graph/labels subsequently change.
    pub fn migrate_source_bindings(&mut self, graph: &SourceGraph) {
        self.migrate_source_bindings_with_refs(graph, std::iter::empty());
    }

    fn migrate_source_bindings_with_refs<'a>(
        &'a mut self,
        graph: &SourceGraph,
        extra_scenes: impl Iterator<Item = &'a SceneId>,
    ) {
        if self.source_bindings.is_some() {
            return;
        }
        let mut bindings = projected_scene_bindings(graph);
        let mut seen = bindings
            .iter()
            .map(|binding| binding.scene_id.clone())
            .collect::<std::collections::HashSet<_>>();
        let sections = sorted_sections(graph);
        for scene_id in self
            .scenes
            .iter()
            .chain(self.active_scene.iter())
            .chain(self.restore_scene.iter())
            .chain(extra_scenes)
        {
            if seen.insert(scene_id.clone())
                && let Some(binding) = legacy_scene_binding_in_sections(graph, &sections, scene_id)
            {
                bindings.push(binding);
            }
        }
        self.source_bindings = Some(bindings);
    }

    #[must_use]
    pub fn source_section<'a>(
        &self,
        graph: &'a SourceGraph,
        scene_id: &SceneId,
    ) -> Option<&'a Section> {
        match &self.source_bindings {
            Some(bindings) => {
                let mut matches = bindings
                    .iter()
                    .filter(|binding| binding.scene_id == *scene_id);
                let binding = matches.next()?;
                if matches.next().is_some() {
                    return None;
                }
                binding.section(graph)
            }
            // Read-only Core consumers also accept unmigrated V1 Sessions.
            // This is the sole legacy decoder, never a fallback for explicit refs.
            None => legacy_scene_binding(graph, scene_id)?.section(graph),
        }
    }
}

impl SessionFile {
    pub fn migrate_scene_source_bindings(&mut self, graph: &SourceGraph) {
        self.migrate_scene_source_bindings_with_refs(graph, std::iter::empty());
    }

    pub(crate) fn migrate_scene_source_bindings_with_refs<'a>(
        &'a mut self,
        graph: &SourceGraph,
        extra_scenes: impl Iterator<Item = &'a SceneId>,
    ) {
        let historical_scenes = self
            .action_log
            .actions
            .iter()
            .flat_map(action_scene_refs)
            .chain(
                self.action_log
                    .commit_records
                    .iter()
                    .filter_map(|record| record.boundary.scene_id.as_ref()),
            )
            .chain(self.runtime_state.transport.current_scene.iter())
            .chain(extra_scenes);
        self.runtime_state
            .scene_state
            .migrate_source_bindings_with_refs(graph, historical_scenes);
    }
}

pub(crate) fn action_scene_refs(action: &Action) -> impl Iterator<Item = &SceneId> {
    action.target.scene_id.iter().chain(match &action.params {
        ActionParams::Scene { scene_id } => scene_id.as_ref(),
        _ => None,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneSourceBindingError {
    DuplicateScene(SceneId),
    InvalidTarget(SceneSourceBinding),
}

impl std::fmt::Display for SceneSourceBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateScene(id) => write!(f, "duplicate source binding for Scene {id}"),
            Self::InvalidTarget(binding) => write!(
                f,
                "invalid source binding for Scene {}: Source {}, Section {}",
                binding.scene_id, binding.source_id, binding.section_id
            ),
        }
    }
}

impl std::error::Error for SceneSourceBindingError {}

fn legacy_scene_binding(graph: &SourceGraph, scene_id: &SceneId) -> Option<SceneSourceBinding> {
    legacy_scene_binding_in_sections(graph, &sorted_sections(graph), scene_id)
}

fn legacy_scene_binding_in_sections(
    graph: &SourceGraph,
    sections: &[&Section],
    scene_id: &SceneId,
) -> Option<SceneSourceBinding> {
    let mut parts = scene_id.as_str().splitn(3, '-');
    let (Some("scene"), Some(index), Some(label)) = (parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    if label.is_empty() || label.chars().any(char::is_control) {
        return None;
    }
    let index = index.parse::<usize>().ok()?.checked_sub(1)?;
    let section = *sections.get(index)?;
    Some(SceneSourceBinding {
        scene_id: scene_id.clone(),
        source_id: graph.source.source_id.clone(),
        section_id: section.section_id.clone(),
    })
}

const fn label_slug(label: SectionLabelHint) -> &'static str {
    match label {
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

#[cfg(test)]
#[path = "scene_source_binding_tests.rs"]
mod tests;
