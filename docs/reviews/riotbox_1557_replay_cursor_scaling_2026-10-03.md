# RIOTBOX-1557 — snapshot/undo cursor scaling

Date: 2026-10-03. Base: `fecaf3a7747a14807d67707389a1b49853c0195b`.
Classification: maintenance/regression, protecting P023 Session recall/recovery.
Contract: RBX-426. This is not musical progress or a realtime latency claim.

## Scope and compatibility

RIOTBOX-1491 already improved complete restore-history validation and explicitly
excluded snapshot-cursor planning. This slice measures the public target and
snapshot-comparison planners before replacing their repeated undone-target,
marker and commit-record searches. Existing history validation, sorting and
suffix construction stay intact.

Private Core `UndoCursorSafety` owns one transient forward projection. Every
`Undone` action enters pending positions in vector order; only a prior target of
a committed, explicitly accepted typed Undo marker with a commit record is
removed. The prefix answer retains the earliest unresolved action ID. Untyped
legacy markers do not resolve safety. No prefix/record-index allocation for
plain histories; no safety construction for log-end planning without eligible
nonzero anchors. Cursor zero is always empty.

Snapshot comparison still checks its cursor before history validation. Target
planning still validates history, then target bounds, then every snapshot's
bounds before stricter safety. Highest safe cursor/latest list-position ties,
first-error IDs, historical rejection, the log-end exception and complete
origin/suffix results remain unchanged. No public interface, Core/Session schema,
action, persistence, runtime/DSP, source-admission, dependency or TUI change.
README remains unchanged. Index memory is linear and transient; no fallible or
aggregate-allocation guarantee is introduced.

## Reproducible measurements

Run `just replay-cursor-benchmark`. One ignored release test emits 30 rows:
plain and typed-undo histories, 256/1024/4096 actions (same commit count),
0/16/64/128 snapshots for target planning and one snapshot per comparison.
Typed histories consist of N/4 targets, N/4 reverse-LIFO markers and N/2
committed tail actions. Distinct safe late snapshots deliberately require the
old scan to resolve every target rather than exit early. Fixture sanity is a
separate nonignored test. The no-snapshot rows exercise the common full-tail
path, not a manufactured snapshot requirement.

Each row is a median of five public builds. Fixture creation, result validation
and returned-result destruction are outside timing; all internal validation,
index construction, sorting, suffix allocation and temporary-index destruction
are inside it; the returned plan and its suffix are destroyed after timing.
No files/audio are opened. No wall-clock assertion is in CI.

Target-plan observed medians (milliseconds):

| Actions | Snapshots | Plain before | Plain indexed | Undo before | Undo indexed |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 256 | 0 | 0.107959 | 0.097231 | 0.174196 | 0.154233 |
| 256 | 16 | 0.137422 | 0.128777 | 0.498664 | 0.194326 |
| 256 | 64 | 0.125792 | 0.133891 | 0.913418 | 0.193957 |
| 256 | 128 | 0.136287 | 0.101128 | 2.383265 | 0.207069 |
| 1024 | 0 | 0.498141 | 0.582485 | 0.799951 | 0.766341 |
| 1024 | 16 | 0.547610 | 0.822996 | 5.787637 | 0.687192 |
| 1024 | 64 | 0.858071 | 0.444233 | 18.746780 | 0.681973 |
| 1024 | 128 | 0.723057 | 0.456711 | 39.667911 | 0.633532 |
| 4096 | 0 | 5.403437 | 1.924532 | 4.018950 | 2.151551 |
| 4096 | 16 | 2.931069 | 2.077985 | 129.092751 | 2.604788 |
| 4096 | 64 | 3.211351 | 3.915053 | 580.163189 | 2.780459 |
| 4096 | 128 | 5.257397 | 3.711598 | 898.846364 | 2.582450 |

Single snapshot-comparison medians (milliseconds):

| Actions | Plain before | Plain indexed | Undo before | Undo indexed |
| ---: | ---: | ---: | ---: | ---: |
| 256 | 0.102029 | 0.114636 | 0.196583 | 0.188603 |
| 1024 | 0.471017 | 0.382652 | 0.824891 | 0.480756 |
| 4096 | 4.007281 | 2.126428 | 8.927458 | 2.539734 |

Environment: Intel i7-8750H, x86_64 Arch Linux, rustc 1.98.1, default release
profile. This was a working desktop with unrelated activity, not an idle or
CPU-isolated laboratory. Root's earlier Debug build completed before benchmark
sample collection; no second Riotbox Cargo job ran during the timed samples.
Plain/no-snapshot results show substantial variability (even baseline 4096/0
exceeds 4096/16), so their small differences are not proven speedups or universal
no-regression guarantees. The large snapshot-heavy undo improvement agrees
with removal of repeated history scans: 4096/128 is approximately 348x faster
in this observation, not a hardware-independent promise.

Safety construction is O((actions+commits) log(actions+commits)) worst-case and
linear memory; snapshot safety queries become constant-time. Other public-plan
work is unchanged. Histories with early unresolved entries, other undo shapes,
very large histories and real restore/hydration costs are not characterized by
these two generated shapes. Logs:
`/tmp/riotbox-1557-replay-cursor-{baseline,indexed}.log`.

## Review and verification

Before optimization: 44 focused tests passed, one benchmark intentionally
ignored. The same suite passes afterward, including three public-result
differential tests against the prior scan oracle. They cover all cursors of
plain/typed/mixed-legacy histories, nested undo, reversed numeric ID order,
rejected and queued actions, reordered records, safe ties and absent anchors,
out-of-range snapshots/targets and ten malformed-history variants (duplicate
IDs/records, missing action/record/timestamp, wrong/forward/untyped target and
rejected/missing marker result). All malformed variants are independently
asserted invalid before comparing exact public error precedence.

Independent Maintainer/Adversarial/Performance/API Rust review: zero actionable
findings. Independent Spec/Evidence/Product/Risk code review: zero findings.
Final evidence audit found one P3 inaccurate suffix-destruction timing claim:
fixed to distinguish timed temporary-index destruction from excluded returned-
plan/suffix destruction. Its measurement-variability caution is also reflected
above. All 60 table values were independently checked against the logs. No
finding deferred, rejected or waived. Root's short self-review finds zero
additional defects and
confirms the index is only derived after validation, with no replay/state fork.
Full source-free local `just ci` passes on the final tree
(`/tmp/riotbox-1557-ci-sealed.log`, exit 0), including strict all-target/all-feature
Clippy. Exact-head native CI remains the merge gate.
The first full CI passed tests/contracts/synthetic smokes but strict Clippy
rejected the benchmark's constant assertion. It is replaced by the equivalent
Debug-only panic guard before measurement; no timed workload or production
algorithm changes. The final gate passes on that correction.
Focused logs: `/tmp/riotbox-1557-cursor-{before,after,final-focused}.log`.

Only generated metadata histories; no real Development/Holdout/commercial audio,
source-directory discovery, host runtime/device, DAW, rendering or playback.
No new human/musical/source-general/hardness/release qualification. Architecture
cadence: fourth substantive successor to RIOTBOX-1551's bounded checkpoint;
the next actual callback evidence slice must reassess the fifth-slice cadence.
This second approved task does not absorb the separate callback task.
