//! Resolve only reachable typed metadata. Paths and opaque origin strings stay opaque.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    SnapshotRuntimeField as Field, SnapshotRuntimeReference as Reference,
    SnapshotRuntimeReferenceError as Reason, SnapshotRuntimeValidationError as Error, Validator,
};
use crate::{
    action::ActionCommand,
    ids::{CaptureId, SectionId, SourceId},
    session::{GraphStorageMode, Mc202SourcePhrasePlanState},
    source_graph::SourceGraph,
};

impl Validator<'_> {
    pub(super) fn graph_context(&self) -> Result<(), Error> {
        let Some(provided) = self.graph else {
            return Ok(());
        };
        // The supplied context also reaches suffix traversal/resolvers. Refuse
        // disagreement instead of merely preferring embedded data for checks
        // and then allowing a different graph to shape the returned runtime.
        for reference in &self.session.source_graph_refs {
            if reference.source_id == provided.source.source_id
                && reference.storage_mode == GraphStorageMode::Embedded
                && reference
                    .embedded_graph
                    .as_ref()
                    .is_some_and(|embedded| embedded != provided)
            {
                return Err(Self::reference_error(
                    Field::SourceGraphContext,
                    Reference::Graph(reference.source_id.clone()),
                    Reason::IdentityMismatch,
                ));
            }
        }
        Ok(())
    }

    pub(super) fn reference_error(field: Field, reference: Reference, reason: Reason) -> Error {
        Error::Reference {
            field,
            reference,
            reason,
        }
    }

    pub(super) fn source(&self, field: Field, id: &SourceId) -> Result<(), Error> {
        let count = self
            .session
            .source_refs
            .iter()
            .filter(|source| source.source_id == *id)
            .count();
        match count {
            1 => Ok(()),
            _ => Err(Self::reference_error(
                field,
                Reference::Source(id.clone()),
                if count == 0 {
                    Reason::Missing
                } else {
                    Reason::Ambiguous
                },
            )),
        }
    }

    fn graph(&self, field: Field, id: &SourceId) -> Result<&SourceGraph, Error> {
        self.source(field, id)?;
        let mut refs = self
            .session
            .source_graph_refs
            .iter()
            .filter(|reference| reference.source_id == *id);
        let Some(reference) = refs.next() else {
            return Err(Self::reference_error(
                field,
                Reference::Graph(id.clone()),
                Reason::Missing,
            ));
        };
        if refs.next().is_some() {
            return Err(Self::reference_error(
                field,
                Reference::Graph(id.clone()),
                Reason::Ambiguous,
            ));
        }
        let graph = match reference.storage_mode {
            GraphStorageMode::Embedded => reference.embedded_graph.as_ref(),
            // The caller owns external Graph identity admission. Never open a
            // path, use incidental embedded metadata, or override an embedded
            // authority with a different same-SourceId supplied graph.
            GraphStorageMode::External => self.graph.filter(|graph| graph.source.source_id == *id),
        }
        .ok_or_else(|| {
            Self::reference_error(field, Reference::Graph(id.clone()), Reason::Unavailable)
        })?;
        if graph.source.source_id != *id {
            return Err(Self::reference_error(
                field,
                Reference::Graph(id.clone()),
                Reason::IdentityMismatch,
            ));
        }
        Ok(graph)
    }

    pub(super) fn section(
        &self,
        field: Field,
        source: &SourceId,
        section: &SectionId,
    ) -> Result<(), Error> {
        let graph = self.graph(field, source)?;
        let count = graph
            .sections
            .iter()
            .filter(|candidate| candidate.section_id == *section)
            .count();
        match count {
            1 => Ok(()),
            _ => Err(Self::reference_error(
                field,
                Reference::Section {
                    source_id: source.clone(),
                    section_id: section.clone(),
                },
                if count == 0 {
                    Reason::Missing
                } else {
                    Reason::Ambiguous
                },
            )),
        }
    }

    pub(super) fn phrase(
        &self,
        field: Field,
        plan: &Mc202SourcePhrasePlanState,
    ) -> Result<(), Error> {
        self.source(field, &plan.source_id)?;
        if let Some(section) = &plan.source_section_id {
            self.section(field, &plan.source_id, section)?;
        }
        Ok(())
    }

    pub(super) fn capture(&self, field: Field, root: &CaptureId) -> Result<(), Error> {
        let mut index = BTreeMap::new();
        for capture in &self.session.captures {
            index
                .entry(&capture.capture_id)
                .and_modify(|entry| *entry = None)
                .or_insert(Some(capture));
        }
        // An explicit DFS stack bounds stack usage even for maliciously deep lineage.
        let mut stack = vec![(root, field, false)];
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        while let Some((id, field, leaving)) = stack.pop() {
            if leaving {
                visiting.remove(id);
                visited.insert(id);
                continue;
            }
            if visited.contains(id) {
                continue;
            }
            if !visiting.insert(id) {
                return Err(Self::reference_error(
                    field,
                    Reference::Capture(id.clone()),
                    Reason::Cycle,
                ));
            }
            let capture = match index.get(id) {
                None => {
                    return Err(Self::reference_error(
                        field,
                        Reference::Capture(id.clone()),
                        Reason::Missing,
                    ));
                }
                Some(None) => {
                    return Err(Self::reference_error(
                        field,
                        Reference::Capture(id.clone()),
                        Reason::Ambiguous,
                    ));
                }
                Some(Some(capture)) => *capture,
            };
            if let Some(owner) = capture.created_from_action {
                let action = self.action(Field::CaptureProducer, owner)?;
                self.owner(
                    Field::CaptureProducer,
                    action,
                    matches!(
                        action.command,
                        ActionCommand::CaptureNow
                            | ActionCommand::CaptureLoop
                            | ActionCommand::CaptureBarGroup
                            | ActionCommand::W30CaptureToPad
                            | ActionCommand::W30LoopFreeze
                            | ActionCommand::PromoteResample
                    ),
                )?;
            }
            if let Some(window) = &capture.source_window {
                self.source(Field::CaptureSourceWindow, &window.source_id)?;
            }
            stack.push((id, field, true));
            stack.extend(
                capture
                    .lineage_capture_refs
                    .iter()
                    .rev()
                    .map(|id| (id, Field::CaptureLineage, false)),
            );
        }
        Ok(())
    }
}
