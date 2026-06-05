# `RIOTBOX-1214` Promote source-backed human passes into demo-bank entries

- Ticket: `RIOTBOX-1214`
- Title: `Promote source-backed human passes into demo-bank entries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1214/promote-source-backed-human-passes-into-demo-bank-entries`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1214-promote-source-backed-human-passes-into-demo-bank-entries`
- Linear branch: `feature/riotbox-1214-promote-source-backed-human-passes-into-demo-bank-entries`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1189 (https://github.com/marang/riotbox/pull/1189)`
- Merge commit: `5aafca0a0589e5d1854a7c757e55f78a66b4ef3f`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/promote_listening_review_to_demo_bank.py; just demo-bank-promotion-fixtures; just release-grade-demo-bank-fixtures; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed on PR #1189.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Connect structured human listening reviews to release-grade demo-bank entries without letting unverified or stale artifacts become product claims.

## What Shipped

- Added promote_listening_review_to_demo_bank.py for checked demo-bank upserts from professional listening reviews; added fixture coverage for human pass promotion, weak not-demo-ready preservation, unverified rejection, and stale hash rejection; wired demo-bank promotion fixtures into audio-qa-ci and documented the promotion seam.

## Notes

- None
