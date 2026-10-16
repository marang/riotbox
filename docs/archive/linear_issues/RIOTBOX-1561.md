# `RIOTBOX-1561` — Bound Python encoded-source admission before hashing and analysis

- Ticket: `RIOTBOX-1561`
- Title: `Bound Python encoded-source admission before hashing and analysis`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1561/bound-python-encoded-source-admission-before-hashing-and-analysis`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 maintenance of the accepted P023 source-ingest path; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1561-bounded-python-source-reads`
- Linear branch: `feature/riotbox-1561-bound-python-encoded-source-admission-before-hashing-and`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1664](https://github.com/marang/riotbox/pull/1664)
- Merge commit: `34c3c79261e41f8df3f2c59bd2a503a2024171bd`
- Deleted from Linear: `2026-10-16`
- Verification: Historical generated Python 12-test suite and tracing-enabled repeat; Rust Sidecar 32 unit + 2 integration tests; full source-free local just ci; exact-head native CI 37105279061 passed
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1561_python_source_admission_2026-10-03.md`, `docs/specs/audio_core_spec.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: RIOTBOX-1562 (malformed-container admission), RIOTBOX-1563 (early decoded-input sample-count admission); both separately completed

## Why This Ticket Existed

Python resolved a source path and performed an unbounded read before hashing and analysis. Rust's existing RBX-425 limit ran only after analysis returned, so it could not protect that earlier encoded allocation or a FIFO-open wait. The separate RIOTBOX-1560 response-framing repair did not address this input-side defect. This P017 maintenance slice protected P023 ingest failure isolation without adding a new source-selection or musical threshold.

## What Shipped

- One Python descriptor-read owner applies the existing V1 268,435,456-byte encoded-WAV budget before hashing/decoding. It admits the opened descriptor as a regular file, rejects declared oversize before payload reads, uses nonblocking Unix open to avoid FIFO waits, and retains ordinary source-symlink support and canonical-path reporting.

- Unbuffered actual reads remain bounded even with understated or growing metadata. Exact-limit EOF succeeds; one-byte excess fails without admitting truncated audio. One bytearray accumulator avoids retaining a separate object for every short read, then freezes the immutable admitted buffer once.

- Hashing and WAV decoding consume that same admitted buffer. The existing request-correlated, non-retryable source_resource_limit envelope reports budget failure, and synchronized peers remain reusable. Raw I/O returning None/would-block now produces source_unreadable rather than treating a partial prefix as EOF.

- RBX-431 and Source Graph section 6 explicitly extend the fixed encoded policy to Python. Existing Rust production reading, analysis algorithms, Session/replay/protocol schemas and capture/export no-follow policy remain unchanged.

## Historical Verification And Outcomes

- Historical RED/GREEN evidence covered sparse declared oversize reaching payload reads without allocating the nominal large payload, and buffered prefetch advancing the descriptor nine bytes instead of five under a four-byte test budget. Unbuffered bounded reading repaired both.

- The 12 Python tests covered exact/excess/empty input, short/interrupted/growing/error reads, descriptor closure, FIFO/directory rejection, regular-file symlinks, replacement/same-buffer identity and synthetic short-read amplification. A 128 KiB one-byte-read control exposed 16,216,705 traced bytes in the chunk-list implementation; the accumulator met the generous 8x test guard. This measured traced allocations, not RSS or a global memory bound.

- Coordinator self-review found short-read amplification and the None/would-block boundary. Independent adversarial re-review found caller-owned tracemalloc interference in the new test; isolated subprocess measurement repaired the test, normal and PYTHONTRACEMALLOC=1 repeats passed, and the original chunk-list implementation still failed its guard. These were generated contract cases, not a reproduced real-filesystem stall.

- Public Rust-client integration exercised oversize source -> typed error -> ping on the same real Python peer. App error ordering before publication was inspected. The report distinguishes this inspection from executed public-client checks.

- Full local source-free just ci passed; it began before the final two-line would-block guard, whose delta was covered by final focused and independent 12-test runs. Native CI 37105279061 passed both existing jobs on reviewed head 5877d116650418f416b6d45ffb16ab3aa9caf91c before merge. Independent reviews and final self-review had no remaining findings.

## Notes

- Encoded admission is not a decoded float-list/frame/feature-array bound, RSS limit, aggregate-concurrency guarantee or analysis deadline. It adds no sound, quality, source-general or release claim.

- No real source/Holdout/commercial audio, source-directory discovery, playback, host/DAW/TUI access or Windows expansion was part of this work.

- Historical closeout synchronized main/origin/main to the merge and removed the exact implementation branch locally/remotely while preserving the unrelated dirty RIOTBOX-1562 worktree. The issue was intentionally retained in Done at that time.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1561_python_source_admission_2026-10-03.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
