//! Existing neutral direct writers remain independent of queued transport work.
use std::{fs, path::Path};

use riotbox_core::{
    action::{ActionCommand, Quantization},
    replay::build_committed_replay_plan,
    session::{ExportArtifactRole, ReplayPolicy},
};

use super::{JamAppState, enqueue, loaded_app};

#[test]
fn neutral_direct_writer_does_not_drain_unsafe_transport_and_keeps_history_reloadable() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let path = owner.path().join(format!("{policy:?}-writer.json"));
        let destination = owner.path().join(format!("{policy:?}-generated-ci-stems"));
        let mut app = loaded_app(&path, policy);
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::Immediate,
        );
        let pending = app.queue.pending_actions()[0].clone();
        let saved = fs::read(&path).unwrap();
        let receipt = app
            .commit_stem_package_export_local_ci_package(
                &destination,
                110,
                vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
            )
            .unwrap();
        assert_eq!(app.queue.pending_actions(), [&pending]);
        assert_eq!(app.session.action_log.actions.len(), 1);
        assert_eq!(
            app.session.action_log.actions[0].command,
            ActionCommand::ExportStemPackage
        );
        assert_eq!(app.session.export_receipts, [receipt]);
        assert_eq!(app.session.action_log.replay_policy, policy);
        assert_eq!(app.runtime.transport.position_beats, 12.0);
        build_committed_replay_plan(&app.session.action_log).unwrap();
        assert_eq!(fs::read(&path).unwrap(), saved);
        app.save().unwrap();
        let loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        assert_eq!(loaded.session, app.session);
    }
}
