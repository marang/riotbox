# `RIOTBOX-116` Operationalize audio QA workflow spec in agent and PR workflow

- Ticket: `RIOTBOX-116`
- Title: `Operationalize audio QA workflow spec in agent and PR workflow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-116/operationalize-audio-qa-workflow-spec-in-agent-and-pr-workflow`
- Project: `Riotbox MVP Buildout`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-116-audio-qa-workflow`
- Linear branch: `feature/riotbox-116-operationalize-audio-qa-workflow-spec-in-agent-and-pr`
- Assignee: `Markus`
- Labels: `None`
- PR: `#109`
- Merge commit: `adfce94433e2ee0cad05ab4f3ef681f823e37ba9`
- Deleted from Linear: `Not deleted`
- Verification: `git diff --check`, GitHub Actions `Rust CI` run `#309`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/README.md`, `docs/workflow_conventions.md`, `AGENTS.md`
- Follow-ups: `None`

## Why This Ticket Existed

The repo had gained a useful audio QA north-star document, but it still lived only as indexed documentation. Riotbox needed one bounded workflow slice that turned the new spec into an active working rule for audio-producing changes without pretending the fuller offline WAV and listening-pack harnesses already existed.

## What Shipped

- added the audio QA workflow spec to the docs index and active source-of-truth list
- added an explicit `Audio-producing slices` rule in `AGENTS.md`
- added a matching `Audio-Producing Slice Check` section in `docs/workflow_conventions.md`
- kept the rule honest about current repo state by requiring the strongest real checks available today while explicitly calling stronger listening-pack gates future work

## Notes

- this was a documentation and workflow slice only; it did not introduce the missing offline render or listening-pack harnesses themselves
- the spec is now part of the repo’s real operating surface instead of a passive reference
