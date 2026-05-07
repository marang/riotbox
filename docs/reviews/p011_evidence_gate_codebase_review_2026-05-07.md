# P011 Evidence Gate Codebase Review - 2026-05-07

Scope:

- `Justfile` P011 evidence and audio QA recipes
- `scripts/run_p011_exit_evidence_category.py`
- `docs/benchmarks/README.md`
- `docs/phase_definition_of_done.md`
- `docs/specs/audio_qa_workflow_spec.md`

Context:

- Review triggered by the workflow cadence after the P011 evidence-gate slices promoted replay, recovery, export reproducibility, and stage-style stability into executable category gates.

## Findings

### Fixed: aggregate gate selected non-bounded categories

- Location: `scripts/run_p011_exit_evidence_category.py`
- Category: scope
- Severity: minor
- Description: The aggregate `all` mode was documented as the bounded P011 evidence gate, but it selected every manifest category regardless of category `status`. The current manifest only contains `bounded_supported` categories, so this was not a current failure. It would become unsafe if a future `open` or `partial` category were added for indexing before it was CI-safe.
- Resolution: `all` now selects only `bounded_supported` categories. Per-category recipes remain available for targeted local checks.

## Residual Risks

- `just ci` still runs some proof paths twice: once through `just p011-exit-evidence-gate` and again through `just audio-qa-ci`. This preserves standalone `audio-qa-ci` coverage, but it increases CI runtime. Track this as a follow-up only if runtime becomes a practical blocker.
- The aggregate gate executes repo-local `just` and `cargo test` proof commands using `shlex.split` without shell expansion. This is intentional for safety, but future proof commands that require environment assignment or shell operators should be represented as explicit `just` recipes instead of inline shell.

## Verification

- `python3 -m py_compile scripts/run_p011_exit_evidence_category.py`
- `just p011-exit-evidence-category-gate-fixtures`
- `just p011-exit-evidence-gate`
- `just ci`
