# `RIOTBOX-45` Ticket Archive

- Ticket: `RIOTBOX-45`
- Title: `Add first TR-909 pattern adoption inside the audible render seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-45/add-first-tr-909-pattern-adoption-inside-the-audible-render-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-45-tr909-pattern-adoption`
- Linear branch: `feature/riotbox-45-add-first-tr-909-pattern-adoption-inside-the-audible-render`
- Assignee: `Markus`
- Labels: `None`
- PR: `#39`
- Merge commit: `55bd19f`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/README.md`, `docs/research_decision_log.md`, `docs/screenshots/jam_tr909_pattern_adoption_baseline.txt`
- Follow-ups: `RIOTBOX-46`, `RIOTBOX-47`, `RIOTBOX-48`

## Why This Ticket Existed

`RIOTBOX-44` made the current audible TR-909 seam inspectable, but it still remained profile-only. The next bounded MVP slice needed to make the seam musically deeper by letting committed state adopt a small set of pattern shapes without opening a second TR-909 engine, arranger, or editor path.

## What Shipped

- added typed `Tr909PatternAdoption` state to the audio-facing render contract
- derived pattern adoption in `riotbox-app` from committed takeover/support context instead of from a second control path
- made the callback-side renderer vary subdivision, trigger cadence, gain, pitch, and decay from the adopted pattern shape
- extended app-side and callback-side fixture coverage with expected pattern adoption assertions
- updated the Jam and Log shell summaries plus the baseline artifact at `docs/screenshots/jam_tr909_pattern_adoption_baseline.txt`
- recorded the render-contract decision in `docs/research_decision_log.md`

## Notes

- One review-found bug was fixed before merge: adoption could disappear when no explicit `pattern_ref` was present even though committed takeover/support context still implied a valid pattern shape.
- GitHub's combined-status endpoint again returned an empty status set in this environment, and `gh pr checks` was unavailable because the local CLI was not authenticated, so merge used explicit local green verification plus a mergeable PR.
