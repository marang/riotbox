# Linear Issue Archive

This directory is the long-term archive for completed Linear tickets that no longer need to remain in the active Linear workspace.

Use it to keep Riotbox project history searchable and durable while staying under the Linear free-tier issue cap.

## Canonical Roles

- `Linear`
  active operations: in-progress work, review flow, near-term backlog, recent completions
- `docs/archive/linear_issues/`
  canonical long-term ticket archive
- `MemPalace`
  search and retrieval layer over the archive, not canonical storage

## When To Archive Here

Archive a ticket here before deleting it from Linear when:

- the PR is merged
- the issue is marked done
- the ticket is no longer needed in the active workspace

## Archive Shapes

Use one of these two shapes:

- one file per ticket for architecture, review, decision, or process-heavy tickets
- grouped files for routine feature work when readability stays good

Examples:

- `RIOTBOX-038.md`
- `2026-04.md`

## Minimum Entry Content

Every archived ticket entry should preserve at least:

- ticket id and title
- Linear project
- milestone or phase
- final status such as done, canceled, duplicate, or superseded
- created date
- implementation start date when known
- final status date such as merged, done, canceled, or deleted
- feature branch when one existed
- why the ticket existed
- what shipped
- PR link
- merge commit
- follow-up tickets or bounded open questions

Use [TEMPLATE.md](./TEMPLATE.md) when creating a one-file ticket archive entry.

## Index

Keep [index.md](./index.md) updated when new archive files are added so the history stays easy to browse and easy to mine.
