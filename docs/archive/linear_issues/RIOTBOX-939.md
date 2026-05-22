# `RIOTBOX-939` Pin generated source-timing phrase evidence in observer/audio gates

- Ticket: `RIOTBOX-939`
- Title: `Pin generated source-timing phrase evidence in observer/audio gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-939/pin-generated-source-timing-phrase-evidence-in-observeraudio-gates`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-939-pin-generated-source-timing-phrase-evidence-in-observeraudio`
- Linear branch: `feature/riotbox-939-pin-generated-source-timing-phrase-evidence-in-observeraudio`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#932 (https://github.com/marang/riotbox/pull/932)`
- Merge commit: `53841a23458cc6315f2ecb9d3f7be4985e481e5c`
- Deleted from Linear: `2026-05-22`
- Verification: `just observer-audio-correlate-locked-grid-json-fixture; just observer-audio-correlate-generated-feral-grid; just ci; GitHub Actions Rust CI run 26295961347 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-940 continues by rejecting contradictory phrase evidence in the observer/audio summary validator.`

## Why This Ticket Existed

P012 source-timing QA needed generated observer/audio gates to prove phrase evidence survives into output summaries, not only BPM/grid/downbeat evidence.

## What Shipped

- Pinned output_path.source_timing phrase_status, primary_phrase_count, and primary_phrase_bar_count across cautious/manual-confirm, user override, risky override, fallback unavailable, locked-grid, and locked-grid fixture paths.

## Notes

- QA hardening only; no detector or runtime behavior changed.
