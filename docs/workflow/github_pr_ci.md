# GitHub, PR, Review, And CI Workflow

Parent: [Riotbox Workflow Conventions](../workflow_conventions.md)

Use for branches, commits, pull requests, review, CI, merge, and local-main
sync. The parent owns Linear-first order, continuation, and stop conditions.

## Branch Naming

Preferred branch pattern:

- `feature/<identifier>-<short-slice-name>`

Examples:

- `feature/riotbox-18-analysis-ingest-slice`
- `feature/riotbox-19-decoded-source-baseline`

Rules:

- keep the name short and human-readable
- keep one branch aligned to one main issue
- do not overload a branch with unrelated slices
- keep the branch under the repo convention even if external tools suggest a different slug

If Linear is configured to generate branch names, it should use the same repo
convention instead of a username-prefixed path.

## Commit Scope

Preferred commit style:

- one coherent slice per commit where possible
- commit message should describe the slice outcome, not just the file touched

Good examples:

- `add first analysis ingest slice`
- `document branch cleanup helper`
- `document PR description guideline`

Avoid mixing unrelated cleanup into the same commit unless it is required for
the slice to pass.

Do not amend commits unless explicitly requested; prefer a new scoped commit for
follow-up changes on an open branch.

## Pull Request Rules

Every completed ticket should normally open a PR.

PR descriptions should include:

- `Why This Matters`
- `Summary`
- `Verification`
- `Drift Check` for non-trivial feature branches

`Why This Matters` must explain:

- what larger phase or milestone the slice belongs to
- what product path or architecture seam it unlocks
- what the change practically gives the software, stated in product-runtime
  terms rather than file names only
- what the change practically gives the musician, stated in one short
  musician-facing sentence
- what remains intentionally bounded, stubbed, or out of scope

`Summary` / `Changes` must describe the practical behavior change, not only a
code or documentation diff. Keep it concise: mention the contract, workflow,
screen, audio path, replay behavior, or QA proof the software gains, and the
musician-facing effect when there is one.

`Drift Check` should answer:

- New `ActionCommand`: yes/no
- Queue path covered: yes/no/n/a
- Commit or side-effect path covered: yes/no/n/a
- Session/replay consequence covered: yes/no/n/a
- User-visible or observer surface covered: yes/no/n/a
- Test/QA proof included: yes/no/n/a
- Added `JamAppState` state: yes/no
- Added or changed audio-producing behavior: yes/no
- Shadow-system risk reviewed: yes/no

Do not write PR descriptions as changelogs only.

## Review Boundary And GitHub Tooling

Once a PR is open for a ticket:

- treat that ticket as being at the review boundary
- inspect the CI / GitHub Actions output explicitly
- if CI is red, treat the branch as still active work until the relevant failures are addressed
- do not let an open or in-flight PR stall other work inside the requested scope
- when explicit cross-ticket continuation is active and the current PR is clean
  locally with CI running or green, start the next bounded Linear-first slice
- re-check open PRs periodically and merge them as soon as their gates are clean
- include CodeRabbit comments in that periodic PR re-check; relevant findings
  must be fixed on the PR or captured as follow-up tickets before merge
- small follow-up fixes on the same PR are fine
- do not silently bundle the next unrelated slice into the same PR

This keeps review history and Linear issue history aligned.

Use the authenticated local `gh` CLI as the primary GitHub PR tool for Riotbox
work. Verify `gh auth status` before relying on it. Use `gh` for PR creation, PR
metadata/status inspection, CI/check summaries, and PR merges when available.
The GitHub connector may be used as a fallback when it is authenticated and
working. For read-only status checks, public GitHub API responses and Linear
diff metadata are acceptable supporting evidence. If neither authenticated
`gh` nor connector auth is available, the agent can push the branch and prepare
the PR URL/body, but cannot complete PR creation itself. If a PR is already
green and merge-ready, SSH `git` may be used as the repository-level merge/push
fallback, followed by an explicit PR state check.

## Branch-Level Review Before PR

Before opening a PR for a finished feature branch:

- run the `code-review` skill when it is available in the current session
- use that review to surface findings, fix them on the same branch, and answer
  review questions before the PR is created
- then do the normal short self-review pass as a final check

Minimum branch-level review expectations:

- correctness and failure paths
- drift against the active specs in `docs/`
- whether new behavior is adequately covered by tests
- whether docs or workflow notes need to move with the code
- whether any Rust file, including tests and bin helpers, grows beyond the repo's roughly 500-line file budget

If the `code-review` skill is not available in the current session:

- state that clearly in the working notes or PR context
- fall back to the normal self-review pass instead of skipping review entirely

### Drift Review Checklist

For each finished feature branch, reviewers should check whether the branch
introduced hidden architecture drift.

Minimum questions:

- Did this add or change an `ActionCommand`? If yes, are queue,
  commit/side-effect, Session/replay, user/observer surface, and QA proof all
  covered?
- Did this add state to `JamAppState`? If yes, why is it app-runtime state
  rather than Session/Core truth?
- Did this add a new lane, Ghost, Feral, capture, replay, source timing, or
  persistence path? If yes, does it reuse the existing contracts instead of
  creating a shadow system?
- Did this claim an audible product change? If yes, where is the audio, metric,
  observer/audio, listening manifest, or reproducibility proof?
- Did this introduce string values that now control behavior? If yes, should
  they become enums or documented contract fields?
- Did this increase repeated queue or side-effect patterns? If yes, is a small
  helper now warranted?
- Did this add broad app-module imports such as `use super::*`? If yes, is the
  dependency surface still reviewable?

Record recurring findings in `docs/reviews/` and promote durable rules into
`AGENTS.md`, the workflow core or affected module, or the relevant spec.

The current detailed guardrail is recorded in
[Riotbox Architecture Drift Guardrails](../reviews/riotbox_drift_guardrails_2026-05-10.md).

### Rust File-Size Budget

[Module Policy](../engineering/module_policy.md) is the canonical module and
textual-include policy. Branch review treats a Rust production or test file over
roughly 500 lines as a soft review-cost signal, never a hard limit. Split only
when semantic ownership improves; mechanical or numbered shards are not a
solution. If a clean extraction does not fit the current slice, record a bounded
follow-up rather than mixing it into unrelated behavior.

## CodeRabbit Reviews

CodeRabbit is configured through [`.coderabbit.yaml`](../../.coderabbit.yaml) as an advisory PR reviewer.
It does not replace the Riotbox branch-level review, local validation, GitHub
Actions, Linear-first issue loop, or ticket closeout workflow.

Initial policy:

- keep CodeRabbit automatic PR review enabled
- keep `request_changes_workflow` disabled until the team explicitly promotes
  CodeRabbit to a merge gate
- do not idle on every PR while waiting for CodeRabbit; if explicit cross-ticket
  continuation is active, clean CI and no blocking review permit the next
  Linear-first slice
- do not forget CodeRabbit after moving on: re-check open PRs for CodeRabbit
  comments during the normal PR polling loop and before merge
- treat CodeRabbit findings like any other review signal: fix clear defects,
  convert useful product/QA findings into tickets or specs, and ignore noise
  only after checking it against [`AGENTS.md`](../../AGENTS.md) and the relevant docs
- if a CodeRabbit finding is relevant but too large or out of scope for the
  current PR, create or link a follow-up Linear ticket before closing the PR
- do not let CodeRabbit comments promote scripted diagnostics, hardcoded
  fallback output, or unverified listening candidates into quality proof

GitHub App installation is an external permission step. A GitHub repository
owner or organization owner must install CodeRabbit for `marang/riotbox` through
the CodeRabbit/GitHub UI and grant access only to the Riotbox repository unless
broader access is intentionally approved.

## External Review Freshness

External reviews are point-in-time evidence. Before creating tickets from an
external review finding:

- verify the finding against current `main`
- check current Linear, including done or archived issues when the finding may already have shipped
- check `docs/reviews/` for newer refreshes or bounded audits
- classify the finding as open, stale, duplicate, superseded, or intentionally deferred

If the review wording is stale but the underlying risk remains valid, create a
new bounded ticket for the current risk instead of reopening the old issue or
copying the stale wording.

## CI Check After PR Open

After opening a PR, explicitly inspect the GitHub Actions / CI status.

Minimum expectation:

- formatter check passes
- test suite passes
- lint / static analysis passes
- any slice-specific workflow required by the repo is checked

Rules:

- do not assume CI is fine just because local checks passed
- if a CI failure is caused by the current slice, fix it on the same branch before treating the ticket as cleanly in review
- mention important CI failures and fixes in the Linear issue update when they happen
- treat CI checks as merge gates, not as a reason to pause other work already
  inside the user's requested scope
- when no event or webhook mechanism is available, poll open PR status
  periodically; when explicit cross-ticket continuation is active, the next
  bounded slice may proceed. Keep polling token-bounded by checking run/job
  summaries first and fetching logs only for failures or unexpected states
- during explicit continuation, do not fall back into standalone status-only
  updates when there is no blocker
- if a progress update is necessary, pair it with the next concrete action already being taken

### Broad Audio-QA Lock

Do not run broad audio-QA gates concurrently when they write shared
`artifacts/audio_qa/local-*` paths. `just audio-qa-ci` is the public broad gate
and must acquire the repo-local `broad-audio-qa` lock through
`scripts/with_audio_qa_lock.sh` before it starts deleting or regenerating local
audio artifacts. `just ci` calls that public gate and therefore inherits the
same protection.

If a second broad audio-QA run is already active, the next run must fail early
with a clear lock message instead of racing on shared artifacts. For concurrent
experiments, run narrower recipes with explicit unique `output=...` arguments
or wait for the broad gate to finish.

Before invoking `just ci` or `just audio-qa-ci`, compare the active ticket's
frozen source-access contract with the recipes in the broad gate. A contract
that forbids further Development or Holdout reads also forbids broad local
audio QA when any included generator opens such audio, even if that generator
belongs to an unrelated historical smoke. In that state, run source-free code
gates (`cargo fmt --check`, `cargo test`, `cargo check --workspace`, and
`cargo clippy --all-targets --all-features -- -D warnings`) plus the exact
source-free validators for the touched contracts. Record the broad gate as
intentionally not run locally because of the access boundary. Do not interpret
the normal preference for `just ci` as authority to reopen a source.

### Audio-Producing Slice Check

For audio-producing changes, also consult
[Audio QA Workflow Spec](../specs/audio_qa_workflow_spec.md),
[`AGENTS.md`](../../AGENTS.md), and
the applicable Riotbox skills. Those sources own product/audio judgment; this
section records its operational PR/CI application.

Current rule:

- do not claim a stronger audio QA process than the repo can actually run today
- use the strongest currently real checks for the affected seam
- note clearly when a desired audio QA layer is still planned rather than operational

Minimum expectation today for an audio-producing slice:

- relevant local formatter, test, and lint checks pass
- relevant audio-facing regression or fixture checks pass when the seam already has them
- action/log/state assertions prove that the intended user action or render state actually landed
- output assertions prove the audible seam is not silent, not fallback-collapsed, and within expected metrics for the affected path
- a local real-session listening pass is done when the slice materially changes behavior that can already be heard
- the PR or working notes say when the slice could not yet use a fuller offline WAV / listening-pack harness because that harness is still future work

Audio-QA selection should be specific before it is broad:

- first run the smallest real checks that cover the changed seam, such as the
  affected fixture validator, pack smoke, promotion/import fixture, or render comparison
- do not treat all audio-QA smokes as the default proof for every small ticket;
  long professional-output and demo-bank smokes rerender multiple real sources
  through `feral_grid_pack` and should be paid for when their surface is touched
  or when branch/merge risk justifies the broader gate
- run `just ci` before PR or merge when the slice changes release gates,
  promotion paths, source-derived quality claims, shared validators, core render
  policy, or other cross-cutting behavior; otherwise document the narrower
  command set that directly proves the slice
- if a broad gate is skipped for a narrow slice, say why in the PR validation
  notes instead of implying the full audio suite ran

Do not close an audio-producing slice with only UI/log proof. If the feature is
supposed to sound different, include a buffer regression, offline render
comparison, source-vs-control metric check, or documented reason why the output
seam is not yet operational.

When manual TUI/audio verification is ambiguous enough that user input timing,
unclear commit feedback, audio-device failure, and fallback-like output cannot
be separated reliably, use the strongest observer path that exists before
guessing. The current first slice is explicit and opt-in:

```bash
cargo run -p riotbox-app --bin riotbox-app -- --source "data/test_audio/examples/Beat08_128BPM(Full).wav" --observer artifacts/audio_qa/local/user-session/events.ndjson
```

That observer records launch, keypress, queue / commit, transport, and runtime
evidence outside the realtime audio callback. It does not yet record raw host
audio or provide a socket-backed monitor; keep those as product/QA work instead
of encoding imaginary behavior into the agent workflow.

As the repo gains the missing audio QA harnesses, tighten this section toward
the stronger release gates defined in the audio QA workflow spec instead of
leaving it as a light note.

## Periodic Whole-Codebase Review

Branch-level review is not enough on its own.

On a regular cadence, run the `review-codebase` skill for a broader whole-repo review.

Default cadence:

- after every 5th substantive feature branch or at an active phase checkpoint,
  whichever comes first
- docs-only, archive-only, fixture-only, and mechanical maintenance branches do
  not advance the counter unless they materially change architecture or product contracts

Purpose:

- catch cross-slice architecture drift
- detect recurring correctness or testing gaps
- find patterns that do not show up clearly in one branch diff

Expected outputs:

- a review artifact under `docs/reviews/`
- important resulting decisions or constraints in `docs/research_decision_log.md`
- workflow or spec updates if the review changes how the repo should be operated

If the `review-codebase` skill is not available in the current session, fall
back to a manual whole-codebase review and record that fact explicitly.

## Direct Push To `main`

Normal slice work should not go directly to `main`.

Direct push to `main` is acceptable only when all of the following are true:

- the user explicitly asked for it
- the change is very small
- the change is repo-meta or workflow-meta rather than product implementation
- skipping the PR does not hide meaningful review risk

Examples of acceptable direct-to-`main` exceptions:

- a tiny `AGENTS.md` rule update
- a very small repo convention note

Examples that should still use a PR:

- code changes
- architectural changes
- anything that advances a real product slice

## Local Sync After Merge

After a PR is merged:

1. switch back to `main`
2. fetch `origin`
3. fast-forward local `main`

Do not continue new ticket work on stale local `main`.

Branch deletion is part of the same closeout, but its deletion checks and
helpers are owned by [Archive And Cleanup](./archive_cleanup.md).

## Tool-Assisted And Manual Boundaries

Automatic or tool-assisted enough:

- local branch creation
- commit and push flow
- PR creation, PR status inspection, and PR merges through authenticated `gh`

Still manual or only partially automated:

- final judgment about whether a slice is ready for review
- direct PR description edits if the available tooling cannot patch the body later

Make the PR description correct at creation time; do not rely on later cleanup
if it can be avoided.
