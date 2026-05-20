# `RIOTBOX-809` Guard representative showcase output deletion

- Ticket: `RIOTBOX-809`
- Title: `Guard representative showcase output deletion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-809/guard-representative-showcase-output-deletion`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-809-showcase-output-guard`
- Linear branch: `feature/riotbox-809-guard-representative-showcase-output-deletion`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`
- PR: `#804 (https://github.com/marang/riotbox/pull/804)`
- Merge commit: `cb1673846c96eb9c0ed66635edbd914ad2baf5c1`
- Verification: `Rust CI run 1951 passed; local bash -n, git diff --check, unsafe-path rejection, and /tmp/riotbox-* representative showcase smoke passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-808 found that the representative showcase generator reset arbitrary output paths with rm -rf before rendering local review artifacts.

## What Shipped

- Added a guard that allows only repo-local artifacts/audio_qa paths and /tmp/riotbox-* review paths before resetting representative showcase output.

## Notes

- Workflow safety slice only; generated artifact behavior remains unchanged for normal just representative-source-showcase and /tmp/riotbox-* review commands.
- Linear deletion was not performed during archive generation because `LINEAR_API_TOKEN` was not present in the local environment.
