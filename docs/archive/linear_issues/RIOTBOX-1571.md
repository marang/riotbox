# `RIOTBOX-1571` — Handle multiline Pulse module metadata without weakening exact owned-module identity

- Ticket: `RIOTBOX-1571`
- Title: `Handle multiline Pulse module metadata without weakening exact owned-module identity`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1571/handle-multiline-pulse-module-metadata-without-weakening-exact-owned`
- Project: `P017 | Live Performance Readiness`
- Milestone: `P017 source-free V3 tooling repair for the RIOTBOX-1041 observation blocker; no issue milestone returned`
- Status: `Done`
- Created: `2026-10-09`
- Started: `2026-10-09`
- Finished: `2026-10-09`
- Branch: `feature/riotbox-1571-multiline-module-identity`
- Linear branch: `feature/riotbox-1571-handle-multiline-pulse-module-metadata-without-weakening`
- Assignee: `Markus`
- Labels: `Bug`
- PR: [#1674](https://github.com/marang/riotbox/pull/1674)
- Merge commit: `58a4eccddf2ff35c0d2e54a0cd578d2da44b30ff`
- Deleted from Linear: `Pending archive merge and verified deletion`
- Verification: Historical canonical generated RED/GREEN, independent 19-test metadata suite and synthetic-clock deadline regression; all 134 source-free host fixtures with ResourceWarnings fatal; full local just ci; exact-head native CI 37977961393 passed; no host invocation
- Docs touched: `docs/benchmarks/silent_host_observation_v3.json`, `docs/benchmarks/silent_host_observation_v3.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1571_silent_host_metadata_v3_2026-10-09.md`, `docs/specs/audio_core_spec.md`
- Follow-ups: RIOTBOX-1572 later owns a separate prospectively authorized V3 observation; RIOTBOX-1041 remains open; related RIOTBOX-1576 architecture checkpoint

## Why This Ticket Existed

The sole RIOTBOX-1570 V2 call consumed its authorization and failed before sink/CPAL launch because short module output contained normal multiline native arguments. Splitting every line into three fields misframed this real pactl surface. Verified PulseAudio v17 upstream emits argument strings without escaping their newlines, while local JSON lacked the assumed index, so fabricated JSON indices or ignoring malformed lines would not preserve exact unload ownership.

## What Shipped

- V3/RBX-440 prospectively versions the changed metadata contract while retaining V1/V2 protocols, failed/consumed evidence, commands and authorization fences unchanged.

- Pair exact JSON module arguments with real short-text module IDs. Reject ambiguous, stale, duplicate or partial frames; native module bodies remain opaque and cannot manufacture Pulse ownership through embedded lines/tabs/apostrophes.

- Retain canonical uint32 ID, exact module type/name, unique sink_name, stereo arguments and observed-ID-replacement checks. Load admission also requires exact-name Node absence. No positional/substring ownership, blind malformed-line skipping or relaxed unload rule was added.

- One absolute cooperative deadline is checked even inside a long token. The current operator uses a pinned V3 loader; historical V2 generated-command compatibility and the disabled ordinary launch CLI remain intact.

- Owning V3 protocol, Audio Core/dev-environment documentation, RBX-440 and a dated evidence report describe the repair and explicit source-free boundary. No Rust/runtime/DSP, ActionCommand, Core/Session/replay or dependency change occurred.

## Historical Verification And Outcomes

- Canonical generated RED controls reproduced multiline and native-apostrophe failures without committing host configuration. The independent 19-test metadata suite passed with valid unrelated multiline/tab-containing arguments, owned identity, injection/ambiguity and malformed controls.

- A synthetic-clock RED/GREEN exposed missing deadline checks within one long token; the repair and independent recheck completed before PR.

- All 134 source-free host fixtures passed with ResourceWarnings fatal, preserving timeout, cleanup, signal and blocked-CLI protections. Full source-free local just ci passed.

- Two independent cross-integration reviews, evidence-document review and self-review retained no unresolved material/P0-P3 findings. Focused independently executed subsets were 59 + 17 + 18 tests.

- Native CI 37977961393 passed both existing jobs at exact reviewed head d90fabe6024a3ab75dab9d7134532464af5ccad7 before the squash merge at 2026-10-09 19:13:31 UTC. Historical closeout synchronized clean main/origin/main and removed exact local/remote branches.

## Notes

- This fix did not make an actual host/audio-service/source query or repeat an attempt. It repairs the concrete tooling blocker, not runtime, physical-device, endurance, musical or release qualification. RIOTBOX-1041 remained open.

- Paired metadata reads and unload are sampled, not atomic. Benign ambiguous/churning metadata may conservatively reject; exact observed replacement is rejected without promising adversarial inter-command atomicity.

- No audio/source/Holdout/commercial/capture access, source discovery, physical output, DAW, human playback, TUI or Windows expansion was authorized. A future host call required fresh explicit consent plus a separate prospective phase/owner; this repair and its generated tests granted none.

- The original Done ticket was retained as recent history after historical closeout. The original failed V1 and consumed V2 result/identities remain historical and were not opened during this archive.

- Detailed historical evidence: [implementation report](../../reviews/riotbox_1571_silent_host_metadata_v3_2026-10-09.md).

- Archive verification on 2026-10-16 read the full current Linear issue, all returned comments (no further page), its linked implementation PR body/files and merge/check metadata. Linear was Done and the exact PR was MERGED with both existing native checks successful. The tests, source/audio exclusions and host observations above are historical evidence, not new executions; archival work did not run product/audio tests, query host audio or open source/ignored evidence artifacts.
