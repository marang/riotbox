# `RIOTBOX-35` Fix pending versus committed TR-909 fill state across save and reload

- Ticket: `RIOTBOX-35`
- Title: `Fix pending versus committed TR-909 fill state across save and reload`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-35/fix-pending-versus-committed-tr-909-fill-state-across-save-and-reload`
- Project: `P005 | TR-909 MVP`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-13`
- Finished: `2026-04-13`
- Branch: `lemonsterizoone/riotbox-35-fix-pending-versus-committed-tr-909-fill-state-across-save`
- PR: `#28`
- Merge commit: `249ca1d`
- Follow-ups: `RIOTBOX-38`

## Why This Ticket Existed

The periodic review found that queued TR-909 fill intent was leaking into persisted committed state.

## What Shipped

- Derived the armed fill cue from queue state instead of mutating lane state early.

## Notes

- This preserved the pending-versus-committed contract without changing the action model.
