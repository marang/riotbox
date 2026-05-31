# P016 Export Spine Broad Review - 2026-05-31

Scope:

- `crates/riotbox-core/src/session/export_types.rs`
- `crates/riotbox-core/src/export_qa.rs`
- `crates/riotbox-core/src/replay/export_receipt.rs`
- `crates/riotbox-app/src/jam_app/product_export.rs`
- `crates/riotbox-app/src/observer/export.rs`
- `crates/riotbox-app/src/bin/riotbox-app/event_loop.rs`

Reason:

- Broad review cadence after the recent P016 feature-branch run.
- Focused on the export spine touched by RIOTBOX-1066 through RIOTBOX-1074.

## Findings

### 1. Export recovery preflight still validates legacy receipt paths only

- Location: `crates/riotbox-app/src/jam_app/product_export.rs:270`
- Category: scope
- Severity: major
- Title: Recovery preflight is not artifact-set aware
- Description: `preflight_export_receipt_artifacts` resolves and validates
  `artifact_path` and `proof_path`. That remains correct for the current
  product-mix receipt, but future stem, DAW, or live export receipts will rely
  on `artifact_set[]` entries with multiple roles or URI identities. If wider
  scopes land before this preflight learns `artifact_set_or_legacy()`, recovery
  can report a receipt ready while ignoring role-specific local artifacts.
- Suggestion: RIOTBOX-1076 should add artifact-set-aware validation for
  local-path entries and keep URI entries identity-only until a fetch/cache
  contract exists.

### 2. Export QA gate file is close to the Rust review budget

- Location: `crates/riotbox-core/src/export_qa.rs:1`
- Category: scope
- Severity: minor
- Title: Next QA expansion should split tests first
- Description: `export_qa.rs` is still cohesive and under the soft 500-line
  budget, but it is already at 464 lines after the stem non-silence gate.
  Adding fallback-collapse or lineage behavior in the same file will likely
  push it over budget and make future reviews harder.
- Suggestion: RIOTBOX-1077 should extract stem QA tests or fixtures into a
  semantic module before adding another gate behavior.

## No Findings

- Core export receipt state remains the single persisted export truth.
- Observer export lifecycle derives from ActionCommand / queue / Session
  receipts, not a second observer-only export model.
- Current product-mix export still writes only the bounded full-grid artifact
  and proof receipt.
- Stem package export, DAW session export, live recording export, and
  fallback-collapse claims remain unimplemented and explicitly deferred.

## Follow-Up Tickets

- RIOTBOX-1076: Make export recovery preflight artifact-set aware before wider
  scopes.
- RIOTBOX-1077: Split export QA tests before next gate expansion.
