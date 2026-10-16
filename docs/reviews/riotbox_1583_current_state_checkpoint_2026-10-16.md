# Current-state checkpoint — RIOTBOX-1583

Date: 2026-10-16. Frozen implementation: merged main
`418a6a1dc342203e9844434b39f59b347ae255bc` (PR #1687), tree
`8692f39bf97d2eecd1c70d31e8db11c2a675fe88`. The coordinator verified that
this equals the reviewed RIOTBOX-1582 candidate tree at
`720ed9fae9bf68bd951e398362336a75492f0ecf`. This is risk-directed sampling
of the four Rust crates, Python Sidecar and loaded-recorder QA boundaries,
not an exhaustive DSP, UI or repository audit. The report changes no code.

## Findings and dispositions

### [P2] Control snapshots can accept fields from different publications

Location: `crates/riotbox-audio/src/runtime.rs:49–91`, especially
`coherent_snapshot_or()`; concrete consumer
`runtime/shared_transport_tr909.rs:427–480`, followed by the production
callback in `runtime/shared_transport_tr909/output_callback.rs`.

Reviewers: Adversarial Implementation Reviewer, Rust concurrency specialist.

The control→callback revision guards use Acquire/Release operations while
payload fields use Relaxed loads/stores. Equal even revision observations do
not establish the assumed ordering of those independent payload atomics.
A Loom model accepts `(running=false, bpm=132, position=8)` between initial
`(false,132,0)` and the sole update `(true,144,8)`. That tuple was never
published. The callback uses this snapshot to advance timing and render lanes;
the defect can therefore mix transport intent, tempo and position.

The model copies the production helper and transport type/function bodies
byte-for-byte, substituting only Loom atomic imports. The coordinator verified
both source ranges with empty diffs, inspected the test, independently compiled
it and reproduced the failure (exit 101). It is **extract-based model evidence**,
not an actual-source module test, hardware incident or explanation of prior ALSA
errors. The model uses a 1,000-branch limit without time, permutation or
preemption pruning; it finds the counterexample rather than exhausting its
branch budget. A first cached-library compile mismatch was corrected before
the successful model build and is not counted as a test result.

Reproducer: `/tmp/riotbox-1583-audio-sidecar-control-loom.08SNAT/control_publication_loom.rs`,
SHA-256 `570e6ed41a0945c91b8896a2cf7d9754507d56dbef0b4f455fbbf5d0411f694c`.
Independent rebuild/run logs:
`/tmp/riotbox-1583-root-control-loom-build.log` and
`/tmp/riotbox-1583-root-recompiled-control-red.log`.

Disposition: **deferred to the immediate repair
[RIOTBOX-1584](https://linear.app/riotbox/issue/RIOTBOX-1584/make-bounded-control-to-callback-snapshots-memory-model-coherent)**,
not waived. Use a memory-model-sound bounded publication owner and an
actual-source coupled writer/reader regression. Audit all helper consumers,
the unchecked final read in `coherent_snapshot()` and the analogous W-30
preview cache. Preserve cached complete-state fallback, single-writer ownership
and the heap/lock/I/O-free callback. An SC revision alone is not proof for a
Relaxed payload. This is separate from RIOTBOX-1582's callback→App observed
timing owner, which uses SC payload and revision operations and passes its
actual-source Loom models.

### [P2] Exhausted ActionIds can turn an accepted Session into an invalid save

Location: `crates/riotbox-core/src/queue.rs:66–69,100–103`;
`crates/riotbox-app/src/jam_app/persistence.rs:71–72` reserves after the
restored maximum, and `jam_app/transport.rs:73` enqueues ordinary toggles.
`jam_app/persistence/history_validation.rs:42–50` rejects duplicate persisted
IDs. Session File spec §8.7 requires uniqueness across committed,
pending-derived and structural Undo actions.

Reviewers: Adversarial Implementation Reviewer, data-integrity specialist.

Allocation and reservation saturate at `u64::MAX` instead of representing
exhaustion. A coherent imported history ending at MAX is accepted by the normal
App loader. One ordinary transport toggle then reuses MAX. App `save()` succeeds,
but subsequent normal reload and replay reject the duplicate. At MAX−1, the
first toggle correctly allocates MAX and still saves/reloads; the second toggle
duplicates it and produces the same failure. This needs three toggles across
two small accepted imports, not an astronomically long live history.

The auditor executed an external executable against the public Core/App APIs,
and the coordinator inspected its source, independently rebuilt it and reran it
in a fresh temporary owner. The seed uses normal queue commit records, not an
inconsistent action/commit relation. Sessions contain no source, graph or capture references;
no audio runtime is constructed. The executable's exit 0 means its assertions
successfully demonstrated the defect, **not** that save/restore integrity passed.
It also confirms that an unobserved timing snapshot preserves the restored
12.5-beat position.

Reproducer: `/tmp/riotbox-1583-core-app-id-repro-ZaClqO/main.rs`;
independent run `/tmp/riotbox-1583-root-id-recompiled.log` records
`DuplicateActionId` from replay and duplicate-action rejection on reload.

Disposition: **retained for a separate checked-exhaustion repair after 1584**,
tracked by this checkpoint until its implementation issue can be created.
Linear's free issue limit blocked that second ticket; this does not waive the
finding. Refuse exhausted allocation before mutation, preserve durable IDs and
the previous valid save, and cover MAX/MAX−1 plus direct allocation, queue,
Undo, restore and persistence callers. No renumbering, clamping, reuse or panic.
This is a pre-existing extreme imported-ID edge, not a normal-history growth
problem or a regression introduced by RIOTBOX-1582.

No additional P0–P3 defect survives the sampled Core/App, Audio/Sidecar or
coordinator QA/contract checks. Passing existing controls below do not disprove
these two previously uncovered counterexamples.

## System model and inspected boundaries

- Manifest dependency direction remains App → Audio/Sidecar/Core, and Audio
  and Sidecar → Core. Core owns Session identities, queue, commit sequence,
  replay, capture/export lineage and transport truth; no second action,
  persistence or replay model was found in this sample.
- App restores Core history, admits observed callback timing, feeds transport
  boundaries, and orchestrates recording/export effects. Ordinary boundary
  drains leave file-side-effect actions pending. Writers validate owned
  artifacts, bind receipts, save the Session and roll back matching owned
  files/state on failure. Replay stages candidate state before accepting it;
  external file effects are not silently regenerated.
- Audio has distinct input-control and output-observation publications. The
  unique production callback consumes prepared readers/scratch and writes
  post-limiter capture samples. Capture admission closes before quiescent
  finalization, while retired ownership keeps reclamation off the callback.
  These ownership controls do not themselves establish snapshot coherence.
- Rust Sidecar bounds framing, deadlines and protocol identity; Python's
  source admission hashes/decodes the same admitted bytes. Source-monitor
  replacement and regular-descriptor admission retain their own resource and
  identity boundaries. No source file was opened to inspect these paths.
- Source-free QA owns generated fixture admission, copied-byte identity,
  Core-authoritative readiness, raw proof/WAV correlation and process
  containment. The process group and streams must be verified stopped before
  artifact postflight; unverified cleanup remains a typed failure. GNU-timeout
  containment reserves the unreaped leader's process-group identity before
  cleanup. Probe versions/consumed authorizations are not product truth or
  permission for another invocation.

## Newly executed evidence

The Core/App auditor's selections pass (26 test selections):

| Focus | Passed |
| --- | ---: |
| Core queue | 8 |
| Target replay | 8 |
| Export receipt replay | 5 |
| Continued commit sequence | 1 |
| Recording save / failed-save rollback | 2 |
| Ordinary transport commit/replay | 2 |

Logs: `/tmp/riotbox-1583-core-app-{queue,replay-target,receipt-replay,sequence,recording-save,recording-rollback,transport-commit}.log`.

The Audio/Sidecar auditor's existing controls pass (82 test selections):

| Focus | Passed |
| --- | ---: |
| Exact production callback, cold heap and generated output | 6 |
| Capture reader/lifecycle | 8 |
| Existing actual-source Loom models (capture admission + observed timing) | 4 |
| Rust Sidecar client | 25 |
| Python source/byte controls | 20 |
| Source-monitor ownership/replacement | 6 |
| Regular-descriptor file admission | 13 |

Logs: `/tmp/riotbox-1583-audio-sidecar-{callback,capture-reader,existing-loom,client,python-source,source-monitor,file-admission}.log`.
The callback test's isolated child invocations are not counted a second time.

The coordinator's 12 generated QA cross-owner controls pass:
`/tmp/riotbox-1583-qa-ownership.log`. They cover readiness versus final success,
clock/identity/geometry rejection, Core-authoritative V1/V2 validation, admitted
run copies, first-failure no-retry, action/receipt/prior-Session correlation,
raw proof bytes, absent-marker/finalization deadlines, typed group/stream
cleanup and rejection of post-readiness fixture changes. These selections may
overlap broader suites; their counts are not unique repository-wide coverage.

The two new negative repros are separate evidence, not passing CI controls.
No production code or test was changed to obtain them. The frozen main's native
CI run `38872949552` passes both configured jobs; RIOTBOX-1582's exact candidate
local/native CI is inherited supporting evidence, not this report's PR gate.
Independent report-only Spec/Evidence and Product Pragmatist review retains no
additional P0–P3 finding; the coordinator's short self-review also retains none.
Report-branch source-free `just ci`, exact-head native PR results and
merge/closeout are recorded in Linear and the project journal as they complete,
not inferred from those earlier runs.

## Cadence, limits and next priority

Previous checkpoint RIOTBOX-1576 sampled `fe92445e…`; its zero-findings verdict
was bounded by its coverage, not a timeless safety guarantee. Since then,
generated preparation 1577, loaded adapter 1578, prospective observation phases
1579/1581, observer correction 1580 and observed-timing repair 1582 now interact.
This active-phase risk checkpoint is pulled forward to inspect that integration;
operational/report-only branches do not inflate a five-feature count.

Repair 1584 first because normal live control publication is affected; then
repair accepted imported-ID exhaustion. Keep those code changes out of this
report-only branch. No broad redesign, new QA framework or immediate additional
host trial is justified. RIOTBOX-1041 host/endurance and RIOTBOX-1036 DAW
qualification remain open; neither passing controls nor these findings resolve
the original ALSA/backend cause.

Limits include full-runtime memory-model coverage, atomic filesystem snapshots,
power-loss/concurrent-writer guarantees, aggregate/RSS/writer memory, Sidecar
process-tree/kernel stalls, atomic host compare-and-unload, unsampled DSP/UI,
physical-device/endurance, DAW import and human taste. These are exclusions,
not additional demonstrated defects.

No real source/capture/Holdout/commercial audio, source-directory discovery,
ignored prior attempt, host/service/device observation or launch, DAW, human
playback or Windows expansion occurred. No source-general, musical, hardness,
host, latency, endurance or release qualification or new trial authorization
follows. Jev's separately authorized two-choice advisory used the earlier
pre-checkpoint status (1,054 input / 156 output tokens): it favored inspecting
current lifecycle gaps before broader loaded observation. That late result is
not current defect evidence, a test result or permission to run host audio.
