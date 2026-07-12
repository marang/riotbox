# `RIOTBOX-1333` P023: Play real source audio through a duration-aware W-30 sampler path

- Ticket: `RIOTBOX-1333`
- Title: `P023: Play real source audio through a duration-aware W-30 sampler path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1333/p023-play-real-source-audio-through-a-duration-aware-w-30-sampler-path`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-07-11`
- Finished: `2026-07-12`
- Branch: `feature/riotbox-1333-p023-play-real-source-audio-through-a-duration-aware-w-30`
- Linear branch: `feature/riotbox-1333-p023-play-real-source-audio-through-a-duration-aware-w-30`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1365 (https://github.com/marang/riotbox/pull/1365)`
- Merge commit: `231007642d49bbdcb5fac328640f320f4c511f75`
- Deleted from Linear: `2026-07-12`
- Verification: `human_verdict pass for normal and destructive v8 candidates; cargo test -p riotbox-audio/app/core replay; isolated exact-path, stage, restore and recovery probes; full just ci pass; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_core_spec.md; docs/phase_definition_of_done.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1335 and RIOTBOX-1400 consume the landed sampler path for the broader live Golden Path.`

## Why This Ticket Existed

The exact live W-30 path still played technically correct but weak straight capture loops; it needed duration-aware source playback and a source-derived playable hook.

## What Shipped

- Duration-aware committed-capture playback, transient-derived chop retriggers, expressive damage variation, exact mixer/replay proof, updated contracts, and separate human pass verdicts for normal and destructive candidates.

## Notes

- The fixed riff order is a bounded sampler policy over source-derived slices, not a claim of general composition intelligence. P023 remains open for the all-lane Golden Path.
