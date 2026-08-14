# `RIOTBOX-1102` P016: Add stem-package manifest normalized JSON proof helper

- Ticket: `RIOTBOX-1102`
- Title: `P016: Add stem-package manifest normalized JSON proof helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1102/p016-add-stem-package-manifest-normalized-json-proof-helper`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1102-manifest-normalized-json-proof`
- Linear branch: `feature/riotbox-1102-p016-add-stem-package-manifest-normalized-json-proof-helper`
- Assignee: `Markus`
- Labels: None
- PR: `#1083 (https://github.com/marang/riotbox/pull/1083)`
- Merge commit: `563429b93443ad9f122a97cfd6686a5f3aea970d`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1083; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Bounded P016 manifest/proof slice.

Goal:

Add a deterministic normalized JSON representation for `StemPackageManifest` suitable for future proof hashing, without adding a writer or filesystem side effect.

Acceptance:

* Core helper serializes stem-package manifest values into deterministic pretty or canonical JSON.
* Tests prove equivalent manifest values produce stable bytes and changed artifact identity changes the serialized proof input.
* Docs clarify the helper prepares future proof hashing and does not write files.

Why it matters:

Software needs stable proof bytes before package files are written. Musicians later get reproducible package manifests whose identity does not depend on temp paths or incidental serializer choices.

## What Shipped

- Closed the bounded scope: P016: Add stem-package manifest normalized JSON proof helper.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
