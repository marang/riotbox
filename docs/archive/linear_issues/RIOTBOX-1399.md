# `RIOTBOX-1399` P023: Split PR validation from broad phase audio-QA

- Ticket: `RIOTBOX-1399`
- Title: `P023: Split PR validation from broad phase audio-QA`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1399/p023-split-pr-validation-from-broad-phase-audio-qa`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-08-26`
- Finished: `2026-08-26`
- Branch: `feature/riotbox-1399-p023-split-pr-validation-from-broad-phase-audio-qa`
- Linear branch: `feature/riotbox-1399-p023-split-pr-validation-from-broad-phase-audio-qa`
- Assignee: `Markus`
- Labels: `Infra`, `benchmark`, `workflow`
- PR: `#1484 (https://github.com/marang/riotbox/pull/1484)`
- Merge commit: `26d7424edb78c006cab822415a897b33af87838a`
- Deleted from Linear: `2026-08-26`
- Verification: `just ci: pass in 158 seconds`; `RIOTBOX_BROAD_AUDIO_QA_ACCESS=registered-development-only just ci-broad: pass in 715 seconds`; `GitHub Rust CI: pass`
- Docs touched: `docs/reviews/riotbox_1399_scoped_pr_validation_2026-08-26.md`, `docs/research_decision_log.md`, `docs/workflow/github_pr_ci.md`
- Follow-ups: `Continue with the next roadmap-aligned P023 product slice; no audio behavior is implied by this workflow closeout.`

## Why This Ticket Existed

The normal PR gate had grown into a broad registered-source phase baseline that delayed feedback, reopened Development inputs during unrelated closeout, and regenerated equivalent professional packs.

## What Shipped

- A source-free normal `just ci` PR gate with closure and mutation protection against broad or direct registered-source generator reachability.
- A guarded Development-only `just ci-broad` phase/release baseline that preserves coverage while generating the professional suite once and reusing its child packs.
- Measured validation evidence: 158-second normal gate, 715-second broad gate, and at least 74% less normal-path time than the incomplete legacy observation.

## Notes

- Process acceleration only; no audio mechanism, musical verdict, source-general proof, Holdout evidence, or release claim changed.
