# `RIOTBOX-369` Add audio QA manifest envelope compatibility smoke

- Ticket: `RIOTBOX-369`
- Title: `Add audio QA manifest envelope compatibility smoke`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-369/add-audio-qa-manifest-envelope-compatibility-smoke`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-369-add-audio-qa-manifest-envelope-compatibility-smoke`
- Linear branch: `feature/riotbox-369-add-audio-qa-manifest-envelope-compatibility-smoke`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`
- PR: `#357`
- Merge commit: `75e6139c120055b51918732d57785c627ef65e3b`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-audio listening_manifest`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/benchmarks/listening_manifest_schema_policy_2026-04-29.md`
- Follow-ups: `RIOTBOX-370`

## Why This Ticket Existed

Generated audio QA manifests needed a shared v1 envelope validator so future automation could reject unstable top-level manifest shapes before relying on pack-specific metrics.

## What Shipped

- added shared listening manifest v1 envelope validation
- covered current producer shapes in tests
- documented the manifest schema compatibility policy

## Notes

- this kept pack-specific metrics flexible while freezing the stable envelope fields
