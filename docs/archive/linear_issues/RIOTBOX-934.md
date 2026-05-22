# `RIOTBOX-934` Validate manifest downbeat ambiguity evidence fields

- Ticket: `RIOTBOX-934`
- Title: `Validate manifest downbeat ambiguity evidence fields`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-934/validate-manifest-downbeat-ambiguity-evidence-fields`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-934-validate-manifest-downbeat-ambiguity`
- Linear branch: `feature/riotbox-934-validate-manifest-downbeat-ambiguity-evidence-fields`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`
- PR: `#927 (https://github.com/marang/riotbox/pull/927)`
- Merge commit: `28953770884aac5b99a5a0eccf23277eac90e93a`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; GitHub Actions Rust CI run 26292796649 passed`
- Docs touched: `n/a`
- Follow-ups: `RIOTBOX-935 compares observer and manifest downbeat ambiguity evidence.`

## Why This Ticket Existed

Reject malformed manifest-side downbeat ambiguity evidence under strict observer/audio source timing validation.

## What Shipped

- Added strict tests for malformed primary_downbeat_score, primary_downbeat_margin, and alternate_downbeat_phase_count in manifest source_timing evidence.

## Notes

- Fields remain optional for older manifests; no analyzer, UI, Session, ActionCommand, or audio-output behavior changed.
