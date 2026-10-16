# `RIOTBOX-1556` — Bound source WAV reads and decoded PCM allocations without weakening identity admission

- Ticket: `RIOTBOX-1556`
- Title: `Bound source WAV reads and decoded PCM allocations without weakening identity admission`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1556/bound-source-wav-reads-and-decoded-pcm-allocations-without-weakening`
- Project: `P017 | Live Performance Readiness`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1556-bounded-source-memory`
- Linear branch: `feature/riotbox-1556-bound-source-wav-reads-and-decoded-pcm-allocations-without`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1659](https://github.com/marang/riotbox/pull/1659)
- Merge commit: `fecaf3a7747a14807d67707389a1b49853c0195b`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: source-free `just ci`; Audio source 33 Debug + 33 Release, App 39 distinct admission/preflight controls; native CI 37076959159 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/research_decision_log.md`, `docs/reviews/riotbox_1556_rust_source_memory_2026-10-03.md`, `docs/specs/audio_core_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: RIOTBOX-1557 and RIOTBOX-1558 own the separate measured cursor/callback tasks. Python ingest is outside this Rust boundary; later RIOTBOX-1559/1561/1562/1563 address identity and Python admission gaps.

## Why This Ticket Existed

The Rust original-source descriptor reader used unbounded read_to_end and PCM decode collected samples without an explicit payload budget. First in the approved three-task engineering sequence, this P017 maintenance slice protects P023 source loading/Session restore from per-operation growth without changing asset identity or musical behavior.

## What Shipped

- RBX-425 freezes Rust admission V1: 256 MiB encoded WAV bytes per read and 67,108,864 interleaved decoded f32 samples (256 MiB) per PCM16/24 decode, with typed resource-limit/allocation failures.
- One regular opened-descriptor reader serves original sources and capture hydration. It rejects oversized metadata before allocation, enforces actual bytes even if size understates/grows, uses capped fallible growth and a stack-only non-appended one-byte overrun probe, and never returns truncated success or reopens the path.
- Decode checks encoded/output sample counts and arithmetic before fallible exact PCM reservation, preserving the existing PCM16/24 converter and same-buffer hash/decode bytes.
- Original-source regular symlinks remain supported; captures keep no-follow/nonblocking admission. Oversized historical assets become unavailable honestly, preserving identities and saved Session bytes rather than truncation, migration or fallback music.

## Notes

- RED controls showed the old reader/decoder accepting five units under four-unit injected budgets. GREEN tests cover exact/one-over/empty bounds, metadata rejection and understated growth, short/Interrupted/error reads, chunk identity, ordinary PCM16/24 mono/stereo and ignored chunks.
- Sparse production-cap+1 App tests reject before reading/hashing giant fixture contents; tests do not allocate enormous buffers solely to hit limits. A fallible capacity-overflow control returns AllocationFailed without huge allocation.
- Independent Rust review found zero actionable defects. Spec/Evidence review found a P3 missing direct capture no-follow regression; added, passed and independently rechecked. Final self-review had zero additional findings.
- The final reviewed head `1cd74b9bac1efd34e99a0e2b66fa5dd80267045c` passed native CI; historical closeout reports main synchronization and no outstanding PR findings.
- This is each Rust payload only, not caller-owned bytes/vectors, aggregate caches, allocator overhead, writers/renders, CPU/I/O deadlines or whole-process OOM safety. Earlier Python analysis allocations remain outside the contract, so no end-to-end import-memory claim follows.
- Generated controls/sparse temporary fixtures only: no real Development/Holdout/commercial access, source-directory discovery, device/runtime/DAW/playback, DSP/TUI/Windows work or new musical/source-general/release claim. No persistence/schema/public override/dependency or second truth was added.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
