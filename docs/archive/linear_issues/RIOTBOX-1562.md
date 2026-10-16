# `RIOTBOX-1562` — Fail closed on malformed Python WAV containers without terminating the analysis peer

- Ticket: `RIOTBOX-1562`
- Title: `Fail closed on malformed Python WAV containers without terminating the analysis peer`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1562/fail-closed-on-malformed-python-wav-containers-without-terminating-the`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 maintenance of the accepted P023 source-ingest path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1562-validated-python-wave`
- Linear branch: `feature/riotbox-1562-fail-closed-on-malformed-python-wav-containers-without`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1665](https://github.com/marang/riotbox/pull/1665)
- Merge commit: `e6b6f67ac698eee81b43a58c1ade9a7e85690b85`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical Python 18-test suite; Rust Sidecar 32 unit + 3 integration tests; App 4 source-ingest identity/persistence tests; full source-free local just ci; exact-head native CI 37106346588 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1562_python_wave_container_2026-10-03.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: RIOTBOX-1563 (decoded-interleaved-sample admission), separately completed

## Why This Ticket Existed

Generated malformed WAV probes showed that empty/short RIFF or truncated fmt could terminate the Python peer with EOFError, and an oversized skipped JUNK chunk could escape as RuntimeError. Other truncated PCM and partial stereo frames could instead yield a graph because short reads or partial frames were silently accepted. A false RIFF extent also let Python and the unchanged Rust decoder interpret different PCM from the same hashed file. This was a separate container/interpretation defect beyond RIOTBOX-1561's encoded-read budget.

## What Shipped

- One pure source_wave admission/decode owner validates a complete, unambiguous RIFF/WAVE container: exact RIFF/chunk extents, unique ordered fmt/data, supported positive PCM geometry, whole frames and exact read counts, all before feature construction.

- Valid integer PCM scaling, channel averaging, metadata chunks, true empty data and Python's established unpadded terminal odd-data writer convention remain supported. The admitted byte buffer is unchanged; there is no second source read or dependency on private wave internals.

- Malformed parser failures become the existing request-bound non-retryable source_unsupported error; ping and valid analysis remain usable on the same peer. Unrelated analysis programming errors stay visible rather than being disguised as malformed input.

- RBX-432 and Source Graph section 6 freeze the bounded layout contract. Rust decoding, encoded admission, identity, feature algorithms and existing App persistence ownership remain unchanged.

## Historical Verification And Outcomes

- Executed baseline RED: truncated fmt terminated the real Python peer with EOFError. After exception isolation, 11 layout variants still reached feature construction and zero-rate input caused an arithmetic failure; the complete admission repair made these cases green.

- The 18 Python tests included 32 positive width/channel/layout controls, eight empty-data controls, malformed-layout matrices, all proper byte prefixes, count/read faults, same-buffer source replacement and unmasked analysis exceptions.

- Rust Sidecar's 32 unit + 3 integration tests passed; four malformed cases each demonstrated error -> ping -> legitimate analysis on the same real peer.

- Four App identity/persistence tests executed the false-RIFF-extent case accepted by the unchanged Rust decoder but rejected by Python. They proved rejection before external/embedded publication, preservation/reload of old bytes and cache, and absence of new targets.

- Independent spec review compared 32 complete graphs with the base; adversarial review ran 23,365 small generated mutations and 192 valid controls. These are bounded generated checks, not universal format/fuzz/security qualification; Rust/App execution attribution remains distinct from reviewers' inspected logs.

- Full source-free local just ci and native CI 37106346588 passed. Both existing native jobs succeeded at reviewed head abdb8492d3a5282313d28f8de6ba4fe4e0b1212f. Independent reviews and final self-review retained no findings.

## Notes

- This slice deliberately did not add decoded-memory/sample-count admission. Container validity does not prove RSS, aggregate-memory, deadline, musical or release quality.

- No real source/Holdout/commercial audio, source-directory discovery, host/device/DAW/TUI access, playback or Windows expansion occurred.

- Historical closeout synchronized main/origin/main, removed the exact implementation branch locally/remotely and preserved ongoing RIOTBOX-1563 work. The completed issue was retained in Done.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1562_python_wave_container_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
