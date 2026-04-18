use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionDraft, ActionParams, ActorType, Quantization, TargetScope},
    ids::SceneId,
};

use super::{JamAppState, QueueControlResult};

impl JamAppState {
    pub fn queue_scene_mutation(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: self.session.runtime_state.scene_state.active_scene.clone(),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: self.session.runtime_state.macro_state.chaos,
            target_id: self
                .session
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string),
        };
        draft.explanation = Some("mutate current scene on next bar".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    fn scene_transition_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::SceneLaunch | ActionCommand::SceneRestore
            )
        })
    }

    fn next_scene_candidate(&self) -> Option<SceneId> {
        let scenes = &self.session.runtime_state.scene_state.scenes;
        let current_scene = self
            .session
            .runtime_state
            .scene_state
            .active_scene
            .clone()
            .or_else(|| self.session.runtime_state.transport.current_scene.clone());

        if scenes.is_empty() {
            return None;
        }

        if let Some(current_scene) = current_scene
            && let Some(index) = scenes
                .iter()
                .position(|scene_id| *scene_id == current_scene)
        {
            return Some(scenes[(index + 1) % scenes.len()].clone());
        }

        scenes.first().cloned()
    }

    fn restorable_scene_target(&self) -> Option<SceneId> {
        let current_scene = self
            .session
            .runtime_state
            .scene_state
            .active_scene
            .clone()
            .or_else(|| self.session.runtime_state.transport.current_scene.clone());

        self.session
            .runtime_state
            .scene_state
            .restore_scene
            .clone()
            .filter(|scene_id| current_scene.as_ref() != Some(scene_id))
    }

    pub fn queue_scene_select(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.scene_transition_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let Some(scene_id) = self.next_scene_candidate() else {
            return QueueControlResult::AlreadyInState;
        };

        let current_scene = self
            .session
            .runtime_state
            .scene_state
            .active_scene
            .clone()
            .or_else(|| self.session.runtime_state.transport.current_scene.clone());
        if current_scene.as_ref() == Some(&scene_id)
            && self.session.runtime_state.scene_state.scenes.len() <= 1
        {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::SceneLaunch,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(scene_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Scene {
            scene_id: Some(scene_id.clone()),
        };
        draft.explanation = Some(format!("launch scene {scene_id} on next bar"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_scene_restore(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.scene_transition_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let Some(scene_id) = self.restorable_scene_target() else {
            return QueueControlResult::AlreadyInState;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::SceneRestore,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(scene_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Scene {
            scene_id: Some(scene_id.clone()),
        };
        draft.explanation = Some(format!("restore scene {scene_id} on next bar"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
