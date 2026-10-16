# `RIOTBOX-1555` — Export bounded 8/16-bar live master recordings as versioned DAWproject clips

- Ticket: `RIOTBOX-1555`
- Title: `Export bounded 8/16-bar live master recordings as versioned DAWproject clips`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1555/export-bounded-816-bar-live-master-recordings-as-versioned-dawproject`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1555-extended-live-master-dawproject`
- Linear branch: `feature/riotbox-1555-export-bounded-816-bar-live-master-recordings-as-versioned`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1658](https://github.com/marang/riotbox/pull/1658)
- Merge commit: `996471448e9f763300e1cc44659ceefb7953ac25`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical: source-free `just ci`, DAW App 22, Core 495 including six new, CLI projections 5, recording controls 21; native CI 37071672280 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1555_extended_live_master_dawproject_2026-10-02.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/replay_model_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: At implementation closeout, RIOTBOX-1036/P016 real-host DAW import/output qualification was dependent on the user's DAW setup; local packaging does not close that audible follow-up.

## Why This Ticket Existed

The existing longer live master recorder could not hand ready V3 eight/sixteen-bar receipts to the local DAWproject writer. This P016 contract enabler under RIOTBOX-1036 removes the 32/64-beat file-handoff gap without confusing archive packaging with DAW-host audible qualification.

## What Shipped

- RBX-424 defines DAWproject V3 for fully-ready committed RuntimeMasterBarWindowV3 EightBars/SixteenBars recordings. Core/Session persists typed duration, source receipt and export-version identity, validating metadata-only restore/replay without recapture or archive rewriting.
- Queue selects the latest eligible V2/V3/V4 receipt before readiness and pins source/version/duration; invalid latest evidence fails closed with no older fallback, and later recordings cannot redirect a pending action.
- The existing exact-read WAV/proof admission and shared four-member archive publisher embed byte-identical recorded WAVs at proof tempo with 32/64-beat zero-origin clips and matching 8/16-bar placement. No-clobber, XML/readback validation and owned-artifact save rollback are reused.
- Legacy V2 recording → DAW V1 and canonical V4 recording → DAW V2 retain serialization, geometry and omitted duration keys. Existing CLI/observer/inspect projections expose correct V3 identity without new TUI controls.

## Notes

- Generated acceptance tests cover both durations/fractional tempos, exact bytes/four members/XML, metadata-only restore, source/version/duration pinning, tampering, no-clobber/save rollback and legacy controls.
- Independent Maintainer/Adversarial review found one P2 omitted V3 readiness-dispatch defect. A shared Core live-master identity predicate and strict archive readiness corrected proof/operator/surface consumers; RED→GREEN and independent recheck closed it. Spec/Evidence review and final self-review added no findings.
- Reviewed head `f49c56720dee7e51ffb6b5ca01ce63ee017cff0b` passed native checks before merge. The closeout reports synchronized clean main and no outstanding GitHub findings. Linear metadata dates below use UTC; the historical closeout described completion as 2026-10-03 Europe/Vienna.
- This extends existing ExportDawSession across queue, side effects, Core/Session replay, observer and QA; no new ActionCommand, JamAppState durable state, archive writer, dependencies, DSP/runtime or Windows scope. The README and unrelated work were preserved.
- Only generated source-free inputs were used. No real source/capture/Holdout/commercial access, host launch or playback occurred. File packaging is not real DAW import/output, device endurance, human, source-general, musical-quality or release qualification.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
