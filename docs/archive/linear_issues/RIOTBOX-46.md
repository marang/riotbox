# `RIOTBOX-46` Ticket Archive

- Ticket: `RIOTBOX-46`
- Title: `Add phrase-aware TR-909 variation and release behavior`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-46/add-phrase-aware-tr-909-variation-and-release-behavior`
- Project: `Riotbox MVP Buildout`
- Milestone: `TR-909 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-46-tr909-phrase-variation`
- Linear branch: `feature/riotbox-46-add-phrase-aware-tr-909-variation-and-release-behavior`
- Assignee: `Markus`
- Labels: `None`
- PR: `#40`
- Merge commit: `692e8b4`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/README.md`, `docs/research_decision_log.md`, `docs/screenshots/jam_tr909_phrase_variation_baseline.txt`
- Follow-ups: `RIOTBOX-47`, `RIOTBOX-48`

## Why This Ticket Existed

`RIOTBOX-45` added typed pattern adoption to the audible TR-909 seam, but the lane still behaved too flat across phrase transitions and release. The next bounded MVP slice needed to make the same seam respond to phrase context and release state without creating a second timing model, release engine, or device-control path.

## What Shipped

- added typed `Tr909PhraseVariation` to the existing TR-909 audio render contract
- derived phrase variation in `riotbox-app` from committed transport phrase context and explicit release-pattern cues
- made the callback-side renderer vary subdivision, trigger cadence, pitch, gain, and decay from that phrase-variation layer
- extended app-side and audio-side fixtures with phrase-aware projection and release-tail regression coverage
- updated Jam and Log shell diagnostics plus the baseline artifact at `docs/screenshots/jam_tr909_phrase_variation_baseline.txt`
- recorded the seam decision in `docs/research_decision_log.md`

## Notes

- The slice stayed inside the existing queue, replay, transport, and render seams; it did not add a second sequencer or phrase-specific runtime path.
- The main mid-slice debugging work was in the audio regression layer, where the new release-tail behavior needed the fixture window recalibrated to the actual bounded callback output.
- GitHub's combined-status endpoint again returned an empty status set in this environment, and `gh pr checks` was unavailable because the local CLI was not authenticated, so merge used explicit local green verification plus a mergeable PR.
