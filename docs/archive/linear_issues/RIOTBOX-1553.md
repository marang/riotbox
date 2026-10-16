# `RIOTBOX-1553` — Qualify one eight-bar live master recording on the real host callback

- Ticket: `RIOTBOX-1553`
- Title: `Qualify one eight-bar live master recording on the real host callback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1553/qualify-one-eight-bar-live-master-recording-on-the-real-host-callback`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1553-eight-bar-host-qualification`
- Linear branch: `feature/riotbox-1553-qualify-one-eight-bar-live-master-recording-on-the-real-host`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1656](https://github.com/marang/riotbox/pull/1656)
- Merge commit: `6a647dac9ae9b5a0cdf752abd5f8447cf37cc0d9`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: one exact real-host eight-bar CPAL take, artifact/proof/receipt/restore checks, bounded continuity-only human keep, source-free `just ci`; native CI 37062706317 passed. Current archival check: tracked report, Linear and merged PR metadata only; no artifact/audio/device/host recheck.
- Docs touched: `docs/reviews/riotbox_1553_eight_bar_host_qualification_2026-10-02.md`
- Follow-ups: RIOTBOX-1554 corrects future descriptive alignment summaries without rewriting this take; RIOTBOX-1555 separately adds V3 DAW packaging. At implementation closeout, RIOTBOX-1036 remained open, including real DAW-host import/output dependent on the user's setup; this archive does not reassess that dependency.

## Why This Ticket Existed

After RIOTBOX-1551/1552, P016/RIOTBOX-1036 lacked real-host evidence for the eight-bar V3 recorder. This bounded follow-up reused the already known held tonal composite and existing post-limiter CPAL path, testing one longer recording's continuity rather than transferring the earlier two-bar musical verdict.

## What Shipped

- A tracked qualification report preserves exactly one eight-bar V3 take: 120 BPM / 4/4, 32 beats, 705,600 stereo float32 frames at 44.1 kHz, 16.000 seconds. Action 8 committed `export-receipt-a-0008`; Action, proof, receipt, WAV, observer and metadata-only restored Session agree, with full V3 readiness true.
- The actual take's PID/client/node, retained route snapshots, exact isolated silent virtual sink target and active stereo links bound the real CPAL output path. No fallback/reconnect/move or default-sink/volume change was admitted; runtime/owned stream exit and temporary sink removal were verified.
- The one unchanged candidate received an independent pre-listen assessment, fresh explicit readiness, one bounded sixteen-second HDMI playback and verified owned-stream cessation. The listener recognized the known material and reported no recording problem; `keep` is limited to recording continuity.
- No product code, DSP, recipe, Action, Session schema, threshold or persistence model changed. The existing held composite and recorder/export truth were reused, without a second take or gain/fade/crossfade repair.

## Notes

- Exact authorized inputs came only from the existing RIOTBOX-1482/1492 owner, case `tonal_rusharp_120`: its registered owner-copy and Session-owned cap-01. The original source was never opened. The fresh issue-owned Session alone received explicit `adopted_legacy_v1` capture identity for already documented matching bytes; this established the attempt's admitted baseline, not historical authenticity.
- Read quotas were fully consumed: owner-copy two reads (identity and normal hydration), capture three (identity/copy, explicit copied legacy adoption and normal hydration). Prior Session/Graph bytes remained unchanged; no other capture/source, protected Holdout/commercial audio or source-directory discovery occurred. These historical permissions are not reopened by archival.
- Initial 48-kHz planning was preserved as metadata, then corrected before audio admission to the source-free production probe's actual 44.1-kHz geometry. A missing HDMI default rejected a capture before launch; source-free re-preflight bound the unchanged isolated sink to the then-current analog default without using an extra take.
- Independent operator review fixed metadata-command timeouts, watchdog/process-group cleanup and launch-interruption handling before dependent execution. A later analog-to-HDMI default change rejected the first playback before launch; fresh HDMI readiness preceded the sole actual playback. Historical existing output volume was not modified.
- Actual callback beat endpoints were `4.000000000000004` to `35.99999999999898`, not replaced with idealized endpoints. There were 1,379 callbacks; all recorded timing/overflow/stream/transport/tempo mismatch counters were zero under existing gates; maximum gap telemetry was 62,426 µs, not proof of an underrun-free physical device.
- Exact-output preflight: sample peak −7.590 dBFS, independent f64 RMS −19.479 dBFS, −16.9 LUFS, estimated −5.3 dBTP, zero clipped/nonfinite samples and no silent bars. The operator's initial f64-versus-stored-f32 RMS equality assumption was corrected by reproducing existing arithmetic; neither product metric nor tolerance changed.
- All contributors were named: W-30 cap-01 live recall plus existing TR-909 break_reinforce support; MC-202 and resample tap idle, raw Source Monitor excluded by riotbox_only. This is a composite, not isolated W-30/drum evidence.
- The sixteen-second playback exited normally; 122 route snapshots bound the active stereo HDMI stream, the owned stream disappeared and default routing stayed unchanged. This verifies cessation of this playback, not silence of unrelated desktop applications. No input audio was reread.
- The human keep is not renewed hook/source recognition/music/hardness/demo/release approval. One ordinary play did not audition a repeated end-to-start seam; raw boundary measurements remain descriptive, not seamless-loop qualification. Physical capture-device/acoustic endurance, DAW import and source-general/P023 completion are unclaimed.
- The stored alignment summary incorrectly says two-bar, but all typed/32-beat evidence establishes eight bars. RIOTBOX-1554 owns future wording only; this hash-bound take remains unchanged. Final independent documentation/self-review had zero remaining findings; source-free CI and native checks passed before merge.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.

## Historical Artifact Identity

- WAV: `520461a582e0a12d4da79bfbb6973ab4f3bda084bd7f21ae561658eaea0919ab`.
- Proof: `e43af263f81aef4c4eaf7500594c6ad5c8cdc92ed86f812aaaa43e09dead211c`.
- Source owner-copy: `ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`.
- Selected cap-01: `8460fd78a74fb1290555413ffd8160262ce88077c55fbdf09e417233d8d1f183`.
- Completed access log: `dbe8ee863d0038ac40f0547f73228c3dd3923cf95585e9a7b325c3552bea9db1`.
- Final continuity-only structured review: `0aa9fce3510a51e8325494003aee3d794abdc8db87ac9484395b49dd1d2b82a5`.
- Unchanged pre-feedback assessment: `f57d04607dadc1e813aaf435fe37e05b5146fe18677c3747c94d0b93983a21ff`.
- These identities come from the merged tracked report. No local audio, ignored attempts, source directory, host or device was accessed for this archive.
