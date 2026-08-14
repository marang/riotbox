# `RIOTBOX-1082` P016: Attach source graph refs to export artifacts

- Ticket: `RIOTBOX-1082`
- Title: `P016: Attach source graph refs to export artifacts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1082/p016-attach-source-graph-refs-to-export-artifacts`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `Unknown`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-p016-export-source-graph-ref`
- Linear branch: `feature/riotbox-1082-p016-attach-source-graph-refs-to-export-artifacts`
- Assignee: `Unassigned`
- Labels: None
- PR: `#1062 (https://github.com/marang/riotbox/pull/1062)`
- Merge commit: `7635b778d9f137b6c35cda13a0a40d56bbf2a570`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1062; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Shipped in PR #1062.

What changed:

* Product-mix artifact-set entries now attach `source_graph_ref` when the Session has a source graph reference.
* The reference preserves source id, graph version, and graph hash without copying embedded graph data into receipts.

Why it matters:

Software can trace a saved export back to Session/Core graph identity, and musicians can diagnose which analyzed source produced the exported mix.

## What Shipped

- Closed the bounded scope: P016: Attach source graph refs to export artifacts.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
