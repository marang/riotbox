# `RIOTBOX-1560` — Bound Sidecar response framing and fail closed on oversized replies

- Ticket: `RIOTBOX-1560`
- Title: `Bound Sidecar response framing and fail closed on oversized replies`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1560/bound-sidecar-response-framing-and-fail-closed-on-oversized-replies`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-03`
- Started: `2026-10-03`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1560-bounded-sidecar-responses`
- Linear branch: `feature/riotbox-1560-bound-sidecar-response-framing-and-fail-closed-on-oversized`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1663](https://github.com/marang/riotbox/pull/1663)
- Merge commit: `2df1c772fcd8fb57785ffd094d5cbb245d47f428`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: generated public-client RED→GREEN, Sidecar 32 unit + 1 integration, admission 6 Release, source-free `just ci`; native CI 37103897824 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1560_sidecar_response_admission_2026-10-03.md`, `docs/specs/technology_stack_spec.md`
- Follow-ups: RIOTBOX-1561 owns the earlier Python encoded-file admission gap; RIOTBOX-1562/1563 address other Python container/decoded bounds. This issue remains wire-framing-only.

## Why This Ticket Existed

The synchronous Rust Sidecar transport appended stdout to an unbounded Vec until newline/EOF. Its absolute I/O deadline limited waiting but not memory consumed by a fast faulty peer. Verified on main e80cb816, this P017 maintenance slice removes a concrete P023 ingest failure-isolation gap without changing audible generation or protocol 0.1.

## What Shipped

- RBX-430 and Technology Stack §3.4 freeze response admission V1 at 8 MiB of wire bytes including CR/LF, for control and analysis responses. Valid in-budget UTF-8, newline/CRLF, coalesced frames, legacy final EOF without newline and the existing 1 MiB regression remain supported.
- Reads are restricted to remaining admitted space with one non-appended byte detecting overrun. Requested frame/tail capacity growth is bounded and fallible before append/allocation.
- Typed operation-aware oversized/allocation failures use the existing fatal transport path: invalidate reuse, close pipes, terminate and reap the direct child. Valid legitimate Sidecar errors remain reusable; existing absolute deadline behavior is preserved.
- Public-client generated metadata peers exercise exact-limit and one-over terminated/unterminated/EOF replies, byte-counting UTF-8, analysis context, near-limit coalescing, bounded growth and child cleanup/fail-closed subsequent use.

## Notes

- RED demonstrated that the old public client accepted an oversized valid Pong. GREEN passed 32 unit/one integration test, six admission tests in release and a final focused rerun after explicitly pinning the V1 constant. No global allocator exhaustion was induced.
- Independent Spec/Evidence and Adversarial/Rust/API reviewers retained no P0–P3 findings, rerunning admission/buffer tests and, for Rust, deadline controls; final self-review also found none. A supplemental Jev claim check matched the manual reading but was not code-review authority or a Riotbox model dependency.
- Exact reviewed head `b5df129d8385b83104297b26a3bb9fec319502d5` passed native CI. Historical closeout reports main synchronization without touching active 1561 work and deletion of exact merged local/remote feature refs after merge/ancestry checks, preserving history.
- The limit applies to wire-frame requested storage, not parsed JSON, outgoing requests, Python hashing/analysis, allocator overhead, total-process RSS/OOM, process-tree containment or device/endurance guarantees. Direct child cleanup is not a process-tree claim.
- No new dependency, worker thread, Action, Session/replay model, DSP, Windows feature or parallel transport was introduced. Generated metadata peers/source-free CI only; no source/capture/Holdout/commercial access, source discovery, host/DAW/playback or musical/source-general/release qualification.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
