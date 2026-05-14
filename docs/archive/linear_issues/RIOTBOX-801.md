# `RIOTBOX-801` Add musical-quality validator fixture coverage

- Ticket: `RIOTBOX-801`
- Title: `Add musical-quality validator fixture coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-801/add-musical-quality-validator-fixture-coverage`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-801-musical-quality-fixtures`
- Linear branch: `feature/riotbox-801-add-musical-quality-validator-fixture-coverage`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`
- PR: `#796 (https://github.com/marang/riotbox/pull/796)`
- Merge commit: `669ec4392ba078f7dd1c90ac4b66f9a30d4b5dd0`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-801-fixtures-final.log just representative-source-showcase-musical-quality-fixtures`; `scripts/run_compact.sh /tmp/riotbox-801-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-801-pycompile.log python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py`; `scripts/run_compact.sh /tmp/riotbox-801-just-list.log just --list`; `scripts/run_compact.sh /tmp/riotbox-801-fmt-final.log cargo fmt --check`; `git diff --check`; `GitHub Actions Rust CI run 1927 passed on 540fde154c02b49ec520fb564f809709fe6077d3`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-802`

## Why This Ticket Existed

The RIOTBOX-799 musical-quality validator needed compact fixture coverage so future threshold or schema regressions fail without requiring the full local representative showcase.

## What Shipped

- Added valid and invalid representative-showcase musical-quality fixtures, a Just fixture target, and wired the target into audio-qa-ci.

## Notes

- The invalid fixture proves weak generated support is rejected with generated_support_balance_out_of_range.
