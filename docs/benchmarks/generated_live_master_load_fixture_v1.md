# Generated live-master load fixture V1

RIOTBOX-1577 / RBX-443. Classification: `maintenance/regression`, preparation
only. Normative values: [V1 JSON](generated_live_master_load_fixture_v1.json).
This closes a preparation gap for RIOTBOX-1041: silent/default host observations
and device-free recorder ordering do not establish behavior under a loaded
musical composite. It does not execute or qualify that future host test.

## Entry and ownership

```text
cargo run -p riotbox-app --bin dense_break_live_path_render -- \
  --prepare-generated-live-master-load-v1 --output <absolute fresh directory>
```

Linux only; the parent directory must exist. Exactly those three arguments are
accepted. No arbitrary source, preset, timing, renderer or host options. The
owner is exclusively created with mode 0700; existing directories/files/links
reject without reuse, deletion or overwrite. Failed and partial owners remain
available as evidence and may not be retried in place.

The bundled generator is read once into an exclusive owner-local snapshot and
that snapshot is executed. Record its SHA256, the running Linux executable's
inode SHA256 (`/proc/self/exe`), generated PCM identity and later Core/Session/
Graph/capture identities. This is an attributable generated control, not a
relocatable package or a claim of reproducibility across arbitrary Python/libm,
toolchain or producer versions. Files remain ordinary local files; their stored
identities, not a filesystem immutable flag, define the prepared state.

Normal App/Sidecar ingest and musician-accessible committed actions produce
the prefix. `ready_prefix` borrows App and reuses the historical capture length,
capture, raw audition, promotion, Feral preset and MC-202 instigator sequence.
Core/Session own every durable action, transport, confirmed timing, source
binding and capture lineage. The legacy diagnostic entry retains its complete
preflight, family checks, logging and later journey; the generator's absent
`--only` flag retains all twelve sources. No new ActionCommand, App state,
analyzer, recorder, runtime, DSP or production policy exists here.

## Preparation proof and fail-closed result

Save and ordinary Session-only restore must retain serialized Core state and
hydrate the generated original/capture identities. Each window rechecks the
saved Session file hash and restored Core identity/hydration. Existing App
recording-plan admission supplies the intended start/frame count and canonical
tempo. Output metadata is explicitly generated geometry, never device metadata.
The shared realtime-simulation offline mixer evaluates all six pinned windows.
The final report names all five contributors as a composite: TR-909, MC-202,
W-30 preview, W-30 resample tap and Source Monitor. Naming is not a claim that
every lane is active or independently qualified; source contribution remains
part of this composite. No actual recorder controller/callback is run and no
host-qualified recording receipt is persisted.

Technical acceptance requires admitted/restored identity, finite samples,
checked exact interleaved sample count, nonzero activity, `0 < peak_abs < 1`,
zero clips and successful offline artifact publication/readback. RMS and shared
limiter applied-sample counts are retained, not new musical pass thresholds.
The existing writer receives only an exclusive temporary file; decode and hash
the same readback bytes, verify rate/channels/sample count, then publish with
`persist_noclobber`. Occupied files or symlinks stay untouched. Unsupported
publication fails explicitly; no overwrite/copy fallback. No power-loss,
aggregate memory/disk/time budget or hostile same-user concurrent-writer
guarantee is asserted.

Retain computed metrics even if artifact publication fails. Compute all six
otherwise-admitted windows before their aggregate technical verdict. Structural
identity/admission errors stop immediately; uncomputed values are not invented.
Exclusively write `preparation.json` before returning aggregate failure. A report
write error itself fails; no success summary is emitted without the report.
This report is QA evidence, not a replacement Session or future host permission.

Tests cover contract/CLI drift, stale/missing/mutated identities, source/capture
PCM mutation, occupied/missing/link destinations, invalid/silent/clipped payloads,
retained six-window failure metrics, owner reuse and generator compatibility.
Default Cargo tests run the actual generated ingest/save/restore/mixer journey;
the Python selector controls are part of local and native source-free CI.

## Explicit execution fence

`pattern_origin=fixture`, `quality_proof=false`,
`source_intelligence_qualified=false`, `host_execution_allowed=false` and
`live_controller_executed=false` are mandatory. Scripted generated input is
not real-source diversity, human listening, musical/hardness/latency/host/
endurance or release evidence. No real source/capture/Holdout/commercial audio,
source-directory discovery, old ignored attempts, audio device/service/host
metadata, DAW, playback, broad QA or Windows expansion is authorized.

All prior silent-host phases remain consumed. Actual loaded-host execution
requires a separate prospective versioned execution contract, reviewed clean
revision/native CI/binary bindings and fresh explicit authorization. The
RIOTBOX-1572 ALSA trigger remains unexplained; RIOTBOX-1041 and RIOTBOX-1036
remain open. Changing the fixed input/profile/geometry or claims requires a new
version and decision, not post-result tuning.
