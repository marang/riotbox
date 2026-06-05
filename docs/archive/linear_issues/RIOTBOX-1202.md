# `RIOTBOX-1202` Define 10/10 sound-product readiness rubric

- Ticket: `RIOTBOX-1202`
- Title: `Define 10/10 sound-product readiness rubric`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1202/define-1010-sound-product-readiness-rubric`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1202-define-1010-sound-product-readiness-rubric`
- Linear branch: `feature/riotbox-1202-define-1010-sound-product-readiness-rubric`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1184 (https://github.com/marang/riotbox/pull/1184)`
- Merge commit: `53a088692c63a987acc247baaa149ee3fb2e7a9a`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/validate_sound_product_readiness_rubric.py; just sound-product-readiness-rubric-fixtures; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed for PR #1184.`
- Docs touched: `docs/specs/sound_product_readiness_rubric_spec.md; docs/README.md; docs/benchmarks/README.md; docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Define a 10/10 sound-product readiness rubric so Riotbox separates valid audio, diagnostic evidence, automated promise, human verdicts, demo readiness, and release readiness without a hidden taste oracle.

## What Shipped

- Added sound-product readiness spec, versioned JSON rubric, strict validator, negative claim-boundary fixtures, audio-qa-ci wiring, and docs/roadmap/benchmark indexing.

## Notes

- Hardcoded/scripted output remains smoke, regression, or diagnostic only; missing human listening stays human_verdict: unverified; agent_promising is not a musical pass.
