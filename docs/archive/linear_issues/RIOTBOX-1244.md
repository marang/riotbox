# `RIOTBOX-1244` P023: Strengthen destructive gesture contrast across weak routed cases

- Ticket: `RIOTBOX-1244`
- Title: `P023: Strengthen destructive gesture contrast across weak routed cases`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1244/p023-strengthen-destructive-gesture-contrast-across-weak-routed-cases`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1244-p023-destructive-gesture-contrast`
- Linear branch: `feature/riotbox-1244-p023-strengthen-destructive-gesture-contrast-across-weak`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1219 (https://github.com/marang/riotbox/pull/1219)`
- Merge commit: `4d9dbbf64e33c05a3aa5d1f276c8f03996e5d42e`
- Deleted from Linear: `2026-06-13`
- Verification: `py_compile; destructive-variation-professional-smoke; professional-output-suite-smoke; audio-qa-ci; just ci; GitHub rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1245`

## Why This Ticket Existed

Destructive gestures need audible stage contrast; weak restores must not pass as professional output.

## What Shipped

- Raised the destructive restore-to-pressure gate from 1.12 to 1.18 and added a bounded dense-break restore lift so post-stutter restore lands with clearer impact while remaining diagnostic/unverified.

## Notes

- Focused destructive metrics after the change: dropout_to_stutter_rms_ratio=0.007797300905703769, stutter_to_hook_transient_ratio=2.3696428561031904, restore_to_pressure_rms_ratio=1.2157272397723593 against threshold 1.18; artifacts remain human_verdict=unverified and quality_proof=false.
