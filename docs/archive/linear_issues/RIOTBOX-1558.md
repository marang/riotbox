# `RIOTBOX-1558` — Instrument production callback heap activity including cold snapshots and capture lifecycle

- Ticket: `RIOTBOX-1558`
- Title: `Instrument production callback heap activity including cold snapshots and capture lifecycle`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1558/instrument-production-callback-heap-activity-including-cold-snapshots`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-03`
- Branch: `feature/riotbox-1558-callback-heap-audit`
- Linear branch: `feature/riotbox-1558-instrument-production-callback-heap-activity-including-cold`
- Assignee: `Markus`
- Labels: `benchmark`, `Audio`, `Improvement`
- PR: [#1661](https://github.com/marang/riotbox/pull/1661)
- Merge commit: `942a8a4412028ce626faae6d8d93c7831af2e7df`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical: source-free `just ci`, release Audio 327 passed/1 existing ignored, targeted actual-callback Debug 5 functions including 7 fresh child cases, canonical Loom 2 models; native CI 37083894800 passed. Current archival check: Linear Done and merged PR metadata only; no heap/model/device rerun.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1558_callback_heap_2026-10-03.md`, `docs/reviews/riotbox_1558_current_state_cadence_2026-10-03.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1559 owns the separate pre-existing mixed-byte ingest identity P2 discovered by the current-state cadence audit; it was not folded into this callback repair. Dependency licensing remains separate.

## Why This Ticket Existed

The third approved engineering task required measured heap activity through the exact production CPAL processing path, including cold snapshots and capture lifecycle, rather than a warmed or shadow callback. It reproduced one cold fresh-worker allocation of 128 bytes from ArcSwap TLS debt-node loading, while empty accounting and subsequent warm processing were zero.

## What Shipped

- The CPAL closure and tests use one private preparation/processor seam with the same render, timing, limiter, post-limiter capture and conversion order. Test-only thread-local probes count alloc/alloc_zeroed/realloc/dealloc over complete invocation/reference-drop scope, with positive/unwind controls and no production instrumentation.
- RBX-427 replaces snapshot ArcSwap loading with preallocated borrowed SPSC source/capture readers and control-side reclamation. Latest control state is separate from recycled slots; consumers are acquired uniquely before the worker, with explicit duplicate-preparation errors, not speculative unsafe ownership.
- RBX-428 gives capture admission/quiescence a permanent single atomic closed-bit/count gate with one linearization point. Reference counts do not stand in for callback activity; incomplete finish, nonwaiting active-lease rejection and stale-buffer rejection are preserved.
- The unchanged cold regression now reports zero of all four heap operations, as do documented warm/lifecycle/format/composite cases. No diagnostic warmup remains as a repair and no callback CAS/spin loop or project-owned unsafe reclamation was introduced.
- The fifth-slice current-state checkpoint is retained separately; its pre-existing ingest consistency finding became RIOTBOX-1559.

## Notes

- Fresh-process coverage includes stopped callback, six normal/overflow generated-source capture cases across F32/I16/U16 and an active generated TR-909/MC-202/W-30/resample composite. Tests assert first/warm heap counts, capture/device float sample parity, generated signal and overflow/missing-source silence; composite evidence is not individual-lane musical quality.
- Lifecycle checks exercise cold/steady/changed/interrupted publication, source replacement/retained references, capture inactive/armed/copy/complete/finish/abort/new buffer, scratch overflow, reclamation/unique-reader/preparation errors, stale payload/timing and stop at the unchanged 240-frame fade endpoint. Bounded concurrent stress uses 512 callbacks and 64 control publications/aborts.
- The two-atomic predecessor failed the strict Loom model under Loom 0.7.2's documented SeqCst-as-AcqRel limitation. This is not evidence of a hardware race. The same assertions pass using the successor's single linearization point, without pruning/skips/weaker assertions; canonical admission source is compiled against Loom atomics, not copied into a shadow model.
- Independent Rust/concurrency and source/factory/spec review fixes include endpoint, missing-source, scheduling-gap and release positive-control test issues. Final independent rechecks/self-review retained no P0–P3 findings in their scopes; product DSP and health gates were not changed.
- Reviewed head `107eaf6c53d8d43e623f76ef7c77acebc6e63584` passed native CI. Historical closeout reports clean synchronized main and verified completion of the three-task batch; the actual callback regression, not a whole-worker heap-free guarantee, is the accepted result.
- Preparation/error callbacks/worker teardown and dependency internals/hardware deadlines are outside measured render scope. Retained slots/observer ownership may delay PCM reclamation; no aggregate RAM or whole-worker heap-free claim. triple_buffer 9.0.0 remains MPL-2.0/MSRV 1.86; Loom is dev-only.
- Source-free generated PCM and preallocated output only; no real Development/Holdout/commercial/source/capture asset reads, directory discovery, host/device/runtime/DAW/playback or human/musical/hardness/source-general/endurance/release qualification. No public Action/Core/Session/replay/timing-geometry/README/Windows expansion.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
