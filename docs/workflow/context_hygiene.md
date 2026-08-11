# Context And Token Hygiene

Parent: [Riotbox Workflow Conventions](../workflow_conventions.md)

Use for repo-history searches, noisy commands, CI polling, and long sessions;
keep archives, generated artifacts, and full logs out of context by default.

## Context Hygiene For Agents

Riotbox keeps long-term history and generated evidence in the repo, but normal
implementation work should not load all of it into agent context.

Default behavior:

- respect `.rgignore` for broad searches
- search live source, specs, reviews, and workflow docs first
- avoid broad searches through `docs/archive/linear_issues/`, generated
  `artifacts/`, local `data/`, and raw planning transcripts unless the current
  task needs those paths
- prefer
  `rg "..." crates docs/specs docs/reviews docs/workflow_conventions.md docs/workflow AGENTS.md`
  for architecture or implementation questions
- prefer exact archive searches such as
  `rg --no-ignore "RIOTBOX-123" docs/archive/linear_issues` when ticket history
  is needed
- prefer exact audio/manifest searches under `artifacts/audio_qa/...` only when
  validating a specific generated proof

Do not paste large archive batches, generated manifests, raw transcript files,
or long audio probe outputs into a PR description or agent summary. Summarize
the relevant finding and link the file path instead.

## Token-Bounded Command Output

Long command output is an operational cost. It can consume enough agent context
to slow down implementation or force an avoidable session reset.

Default behavior:

- redirect long CI, QA, cargo, clippy, observer/audio, and generated-pack output to `/tmp/...log`
- if a command is expected to print more than roughly 100 lines, run it through
  `scripts/run_compact.sh` or redirect it before starting it
- report only whether the command passed, plus the relevant final lines or error excerpt
- when a command fails, show the failing command, exit status, and the smallest useful log excerpt
- avoid streaming full `just ci`, full `cargo test`, full Linear JSON, full
  GitHub JSON, large manifests, or generated evidence into the agent context
- use `GIT_EDITOR=true` for non-interactive rebase/commit continuation when the existing commit message is sufficient
- set shell tool output limits deliberately; do not request large output unless the task needs it
- if a tool call unexpectedly emits too much output, switch to compact execution
  for the next attempt and record the workflow gap if it is recurring

Preferred pattern:

```bash
scripts/run_compact.sh /tmp/riotbox-ci.log just ci
```

Manual fallback:

```bash
just ci >/tmp/riotbox-ci.log 2>&1
status=$?
if [ "$status" -ne 0 ]; then
  tail -n 80 /tmp/riotbox-ci.log
fi
exit "$status"
```

If the log matters for review or later debugging, keep it in `/tmp` for the
current session and summarize the important finding in Linear or the PR. Do not
commit transient command logs.

## Token Budget Discipline

The autonomous loop must stay token-efficient. Repeatedly re-reading stable
workflow docs, streaming large tool payloads, and writing verbose status updates
can consume more context than the implementation itself.

Default behavior:

- treat `AGENTS.md`, the workflow core, each focused workflow module already
  loaded for the current operation, and the active skill as stable after they
  have been read in the current session; re-read only exact line ranges when the
  user asks about workflow, the files changed, or a concrete rule must be quoted
- do not run broad `rg` queries across `AGENTS.md`, the full workflow set, skill
  files, archives, and specs together unless diagnosing documentation drift;
  search the smallest relevant file set first
- never read `docs/research_decision_log.md` wholesale during normal
  implementation work; it is a large canonical log and should be queried by
  `just decision-search "query"`, exact `rg` terms, or targeted line ranges
- when an exact decision ID is known, use a bounded lookup such as
  `just decision-search "RBX-252"`; the helper returns only that matching
  decision block
- otherwise use `just decision-search "query"` for bounded term retrieval over
  `docs/research_decision_log.md`; the same helper has no semantic-memory dependency
- keep Linear issue descriptions, PR bodies, archive notes, and issue comments
  concise for tiny QA/docs slices; list verification commands once and avoid
  repeating the same long prose across Linear, PR, archive, and chat
- poll GitHub Actions with run/job summaries only; fetch full job logs only for
  failed, cancelled, or suspicious runs
- while waiting for CI, do not send standalone progress updates more often than
  necessary; prefer one short update when a gate changes state or a new action starts
- use `git diff --stat`, `git diff --name-only`, and targeted `git diff -- <file>` before asking for large diffs
- avoid creating stacked follow-up tickets that touch the same files while an
  earlier PR is still open unless the dependency is intentional and worth the
  extra context cost

If token use becomes a concern during a session, finish the active ticket, then
pause new feature work and run a short token-hygiene pass before continuing.
That pass should identify the largest context sources and update this module
when the fix is durable.

## Bounded Subagent Context

Delegate one concrete, independent deliverable with exact file and mutation
boundaries. Prefer the shortest history fork that still carries the user's
request and applicable safety rules; do not fork the full conversation merely
for convenience. Tell the subagent which canonical entry document to load and
let its router select detail instead of pasting large specs into the task.

For read-only audits, say so explicitly. For parallel edits, assign disjoint
files and require a concise result with findings, changed paths, and validation.
The coordinating agent remains responsible for cross-file consistency and must
review the integrated diff rather than treating subagent completion as proof.
