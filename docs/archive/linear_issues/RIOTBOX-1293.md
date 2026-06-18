# `RIOTBOX-1293` Require source-derived TR-909 support evidence before product claims

- Ticket: `RIOTBOX-1293`
- Title: `Require source-derived TR-909 support evidence before product claims`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1293/require-source-derived-tr-909-support-evidence-before-product-claims`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1293-tr909-source-support-evidence`
- Linear branch: `feature/riotbox-1293-require-source-derived-tr-909-support-evidence-before`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1267 (https://github.com/marang/riotbox/pull/1267)`
- Merge commit: `5da8a4bb803127ed409162a63fc13f9cfbf59a6a`
- Deleted from Linear: `2026-06-18`
- Verification: `python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_automated_musical_fitness.py scripts/route_weak_output_fixes.py; just representative-source-showcase-musical-quality-fixtures; just automated-musical-fitness-fixtures; just syncopated-source-showcase-smoke; just full-grid-export-reproducibility-smoke; just weak-output-fix-routing-fixtures; git diff --check; just ci; GitHub rust-ci pass`
- Docs touched: `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

TR-909 support could still pass higher-level quality gates with low-band/applied proof but without source-derived support evidence.

## What Shipped

- Representative showcase and automated musical fitness now require source-derived TR-909 kick-pressure evidence, source profile reason, enough anchors, and source-derived accent dynamics; generated pack smokes assert the same manifest fields; weak-output routing keeps source-masking as mix-bus primary when stricter TR-909 failures are secondary.

## Notes

- None
