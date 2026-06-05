# `RIOTBOX-1204` Create release-grade musician demo bank

- Ticket: `RIOTBOX-1204`
- Title: `Create release-grade musician demo bank`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1204/create-release-grade-musician-demo-bank`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1204-create-release-grade-musician-demo-bank`
- Linear branch: `feature/riotbox-1204-create-release-grade-musician-demo-bank`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1185 (https://github.com/marang/riotbox/pull/1185)`
- Merge commit: `e20f9cce19011f8be8a5c1e7ce746b0631bc6f92`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/validate_release_grade_demo_bank.py; just release-grade-demo-bank-fixtures; git diff --check; just audio-qa-ci; just ci; GitHub rust-ci passed on PR #1185.`
- Docs touched: `docs/specs/release_grade_musician_demo_bank_spec.md; docs/README.md; docs/benchmarks/README.md; docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Create a release-grade musician demo-bank contract so demo evidence is curated, typed, and cannot promote unverified generated artifacts as product quality.

## What Shipped

- Added release-grade demo-bank spec, validator, valid/invalid fixtures, audio-qa-ci wiring, and docs/roadmap/benchmark indexes. The bank distinguishes pass, weak, fail, and unverified examples across dense-break and non-dense source families.

## Notes

- Fixture hashes are contract identities, not committed public demo audio; unverified entries must remain unverified and cannot claim quality.
