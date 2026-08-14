# `RIOTBOX-1097` P016: Define stem-package manifest schema types

- Ticket: `RIOTBOX-1097`
- Title: `P016: Define stem-package manifest schema types`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1097/p016-define-stem-package-manifest-schema-types`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-02`
- Started: `2026-06-02`
- Finished: `2026-06-02`
- Branch: `feature/riotbox-1097-stem-package-manifest-types`
- Linear branch: `feature/riotbox-1097-p016-define-stem-package-manifest-schema-types`
- Assignee: `Markus`
- Labels: None
- PR: `#1077 (https://github.com/marang/riotbox/pull/1077)`
- Merge commit: `42a7ae28ded90b3b577993e9b2787ac468c70e91`
- Deleted from Linear: `2026-08-14`
- Verification: `Merged PR #1077; historical closeout metadata recovered from Linear and GitHub.`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Bounded P016 Core contract slice.

Goal:

Define typed stem-package manifest structures for future local package writers without adding a writer or runnable export action.

Acceptance:

* Core has a `stem_package_manifest` type module or equivalent explicit structs for schema id/version, package id, export scope, claimed stem roles, artifact entries, manifest/proof identity, and receipt id/action id references.
* Serialization uses stable snake_case role/scope values already used by Session receipts.
* Tests cover roundtrip JSON and reject/document missing required identity through typed construction rather than stringly maps.
* Docs state this is a manifest contract only and not a package writer.

Why it matters:

Software needs a boring manifest contract before writing stem packages. Musicians later get packages whose file list and proof identity are inspectable instead of inferred from folder names.

## What Shipped

- Closed the bounded scope: P016: Define stem-package manifest schema types.

## Notes

- Historical terminal-ticket cleanup completed on 2026-08-14; archival itself changed no product behavior.
