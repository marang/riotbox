# `RIOTBOX-1086` P016: Add product export artifact audio metrics evidence

- Ticket: `RIOTBOX-1086`
- Title: `P016: Add product export artifact audio metrics evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1086/p016-add-product-export-artifact-audio-metrics-evidence`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1086-product-export-audio-metrics`
- Linear branch: `feature/riotbox-1086-p016-add-product-export-artifact-audio-metrics-evidence`
- Assignee: `Markus`
- Labels: None
- PR: `#1067 (https://github.com/marang/riotbox/pull/1067)`
- Merge commit: `795a2707fbb7db1889ccadeeca74da1a8a620d0f`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1067; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P016 product-export slice.

Goal:

Populate current product-mix export artifact-set entries with deterministic audio metrics for the full-grid WAV where the existing local export path can compute them safely outside realtime audio.

Acceptance:

* Metrics live on `ExportArtifactSetEntry.audio_metrics` for the product-mix WAV.
* Older receipts remain compatible when metrics are absent.
* Observer lifecycle surfaces the metrics through the existing receipt projection.
* Proof does not claim stem/live/DAW readiness.
* Validation includes focused unit/integration tests plus `just ci` or the relevant compact gate.

Why it matters:

Software gets a stronger receipt-level output proof than hashes alone, and musicians can inspect that the saved mix was not silently empty or fallback-collapsed.

Implementation:

PR #1067 attaches PCM-WAV peak/RMS/silence/frame/duration/format evidence to the FullGridMix artifact-set entry after the product-export proof/hash gate succeeds. Observer lifecycle now serializes the same receipt evidence. Local verification passed: focused product export test, focused observer test, and compact `just ci`.

## What Shipped

- Closed the bounded scope: P016: Add product export artifact audio metrics evidence.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
