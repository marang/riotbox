use std::{fs, path::PathBuf};

use riotbox_core::{action::ActionCommand, action::ActionStatus, queue::ActionQueue};
use tempfile::{TempDir, tempdir};

use crate::jam_app::{
    JamAppState,
    product_export::{ProductMixCheckpoint, with_product_mix_checkpoint},
    tests::{
        fixtures::{
            session_source::sample_graph, session_source::sample_session,
            source_io::write_pcm16_wave,
        },
        p016_product_export_action::{sha256_bytes, write_product_export_proof},
    },
};

#[test]
fn changed_handoff_artifact_cannot_commit_a_mismatched_export() {
    let temp = tempdir().expect("tempdir");
    let proof_dir = temp.path().join("handoff");
    fs::create_dir(&proof_dir).expect("handoff directory");
    let artifact_path = proof_dir.join("full_grid_mix.wav");
    fs::write(&artifact_path, b"validated artifact A").expect("artifact A");
    let proof_path = proof_dir.join("proof.json");
    write_product_export_proof(
        &proof_path,
        "full_grid_mix.wav",
        &sha256_bytes(b"validated artifact A"),
    );
    let destination = temp.path().join("export");
    let graph = sample_graph();
    let mut state =
        JamAppState::from_parts(sample_session(&graph), Some(graph), ActionQueue::new());

    let result = with_product_mix_checkpoint(
        ProductMixCheckpoint::BeforeArtifactSnapshot,
        move || fs::write(artifact_path, b"different artifact B").expect("replace handoff"),
        || state.commit_product_mix_export_from_proof(&proof_path, &destination, 900),
    );

    let error = result.expect_err("changed copied bytes must reject before successful commit");
    assert!(error.to_string().contains("export artifact hash mismatch"));
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportProductMix)
    );
    let action = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportProductMix)
        .expect("rejected export history");
    assert_eq!(action.status, ActionStatus::Rejected);
    assert!(!destination.join("full_grid_mix.wav").exists());
    assert!(!destination.join("product_export_proof.json").exists());
}

struct Handoff {
    _temp: TempDir,
    proof: PathBuf,
    artifact: PathBuf,
    destination: PathBuf,
    artifact_bytes: Vec<u8>,
    proof_bytes: Vec<u8>,
}

impl Handoff {
    fn new() -> Self {
        let temp = tempdir().expect("tempdir");
        let handoff = temp.path().join("handoff");
        fs::create_dir(&handoff).expect("handoff directory");
        let artifact = handoff.join("full_grid_mix.wav");
        // More than one copy-buffer chunk, still a small synthetic fixture.
        write_pcm16_wave(&artifact, 44_100, 1, 1.0);
        let artifact_bytes = fs::read(&artifact).expect("generated artifact");
        let proof = handoff.join("proof.json");
        write_product_export_proof(&proof, "full_grid_mix.wav", &sha256_bytes(&artifact_bytes));
        let proof_bytes = fs::read(&proof).expect("proof snapshot");
        let destination = temp.path().join("export");
        Self {
            _temp: temp,
            proof,
            artifact,
            destination,
            artifact_bytes,
            proof_bytes,
        }
    }

    fn state(&self) -> JamAppState {
        let graph = sample_graph();
        JamAppState::from_parts(sample_session(&graph), Some(graph), ActionQueue::new())
    }
}

#[test]
fn changed_proof_path_publishes_only_the_proof_snapshot_that_was_validated() {
    let fixture = Handoff::new();
    let mut state = fixture.state();
    let proof_path = fixture.proof.clone();
    let receipt = with_product_mix_checkpoint(
        ProductMixCheckpoint::BeforeArtifactSnapshot,
        move || fs::write(proof_path, b"{").expect("replace proof after validation"),
        || state.commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 900),
    )
    .expect("unchanged admitted snapshot succeeds");

    assert_eq!(
        fs::read(fixture.destination.join("product_export_proof.json")).expect("published proof"),
        fixture.proof_bytes
    );
    assert_eq!(
        receipt.artifact_set[1].sha256,
        sha256_bytes(&fixture.proof_bytes)
    );
    assert_eq!(receipt.export_hash, sha256_bytes(&fixture.artifact_bytes));
}

#[test]
fn changed_handoff_after_staging_cannot_retarget_published_artifact() {
    let fixture = Handoff::new();
    let mut state = fixture.state();
    let artifact = fixture.artifact.clone();
    let receipt = with_product_mix_checkpoint(
        ProductMixCheckpoint::AfterArtifactSnapshot,
        move || fs::write(artifact, b"replaced after staging").expect("replace source path"),
        || state.commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 900),
    )
    .expect("publish validated descriptor, not changed source path");

    assert_eq!(
        fs::read(fixture.destination.join("full_grid_mix.wav")).expect("published artifact"),
        fixture.artifact_bytes
    );
    assert_eq!(receipt.export_hash, sha256_bytes(&fixture.artifact_bytes));
}

#[test]
fn complete_identical_bundle_is_idempotent_and_preserves_unrelated_files() {
    let fixture = Handoff::new();
    fs::create_dir(&fixture.destination).expect("existing empty destination");
    let sentinel = fixture.destination.join("unrelated.txt");
    fs::write(&sentinel, b"leave this alone").expect("sentinel");
    let mut state = fixture.state();
    let first = state
        .commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 900)
        .expect("existing directory and unrelated file admitted");
    let second = state
        .commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 901)
        .expect("identical bundle admitted");
    assert_eq!(first.export_hash, second.export_hash);
    assert_ne!(first.created_by_action, second.created_by_action);
    assert_eq!(
        fs::read(sentinel).expect("sentinel unchanged"),
        b"leave this alone"
    );
    assert_eq!(
        fs::read(fixture.destination.join("full_grid_mix.wav")).expect("artifact unchanged"),
        fixture.artifact_bytes
    );
    assert_eq!(
        fs::read(fixture.destination.join("product_export_proof.json")).expect("proof unchanged"),
        fixture.proof_bytes
    );
}

#[test]
fn incomplete_or_different_bundles_reject_without_replacing_existing_files() {
    for (artifact, proof) in [(true, false), (false, true), (true, true)] {
        let fixture = Handoff::new();
        fs::create_dir(&fixture.destination).expect("existing destination");
        let artifact_path = fixture.destination.join("full_grid_mix.wav");
        let proof_path = fixture.destination.join("product_export_proof.json");
        if artifact {
            fs::write(&artifact_path, b"pre-existing artifact").expect("existing artifact");
        }
        if proof {
            fs::write(&proof_path, b"pre-existing proof").expect("existing proof");
        }
        let mut state = fixture.state();
        state
            .commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 900)
            .expect_err("incomplete/different bundle rejects");
        assert!(state.session.export_receipts.is_empty());
        assert!(state.queue.pending_actions().is_empty());
        if artifact {
            assert_eq!(
                fs::read(artifact_path).expect("artifact preserved"),
                b"pre-existing artifact"
            );
        } else {
            assert!(!artifact_path.exists());
        }
        if proof {
            assert_eq!(
                fs::read(proof_path).expect("proof preserved"),
                b"pre-existing proof"
            );
        } else {
            assert!(!proof_path.exists());
        }
    }
}

#[test]
fn late_artifact_creation_collision_preserves_the_unowned_file_and_rolls_back_proof() {
    let fixture = Handoff::new();
    let mut state = fixture.state();
    let collision = fixture.destination.join("full_grid_mix.wav");
    let collision_for_hook = collision.clone();
    let result = with_product_mix_checkpoint(
        ProductMixCheckpoint::BeforeArtifactPublication,
        move || {
            fs::write(collision_for_hook, b"other writer's artifact").expect("inject collision")
        },
        || state.commit_product_mix_export_from_proof(&fixture.proof, &fixture.destination, 900),
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read(collision).expect("unowned file retained"),
        b"other writer's artifact"
    );
    assert!(
        !fixture
            .destination
            .join("product_export_proof.json")
            .exists()
    );
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
}
