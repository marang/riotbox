# `RIOTBOX-1205` Add weak-output failure-to-fix routing

- Ticket: `RIOTBOX-1205`
- Title: `Add weak-output failure-to-fix routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1205/add-weak-output-failure-to-fix-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1205-add-weak-output-failure-to-fix-routing`
- Linear branch: `feature/riotbox-1205-add-weak-output-failure-to-fix-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1183 (https://github.com/marang/riotbox/pull/1183)`
- Merge commit: `e34e8e6f7df38bc7b2838321477d22feb8de6a2d`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/route_weak_output_fixes.py; just weak-output-fix-routing-fixtures; git diff --check; just audio-qa-ci; just ci; GitHub Actions rust-ci passed on PR #1183`
- Docs touched: `docs/README.md; docs/benchmarks/README.md; docs/benchmarks/weak_output_fix_routing_v1_2026-06-05.md`
- Follow-ups: `None`

## Why This Ticket Existed

Weak audio reports needed to become concrete production work instead of vague scores or implied musical approval.

## What Shipped

- Added weak-output fix routing from existing failure codes/listening tags to source selection, chop policy, drum pressure, bass movement, mix bus, destructive gesture, fixture threshold, and UI cue categories; added negative fixtures for weak hook/pressure, source loss, flat stutter, hookless tonal chop, weak sparse-bass pressure, and source-masked generated support; wired weak-output-fix-routing-fixtures into audio-qa-ci while preserving human_verdict unverified, quality_proof false, and automated_musical_approval false.

## Notes

- None
