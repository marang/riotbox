# RIOTBOX-1546 Feral stem semantic owners

Date: 2026-10-01
Implementation baseline: `e424febc50280b6e85043a678f0b375b87807012`.
Integrated review checkpoint: `8629de2c8889f689e6fdc61cc6dc2fb3e6f7763b`.
Decision: RBX-414
Classification: mechanical maintenance with explicit responsibility ownership.

The mixed stem include is replaced by ordinary binary-private W-30 trigger,
pack validation, audio/metrics publication and reproduction-command owners.
This removes a P023 QA artifact-review obstacle, not a musical mechanism or
product qualification gate. The broader RIOTBOX-1545 checkpoint is normally
merged before the contract is pinned; its archive-only successor is integrated.

## Ownership and compatibility

Seven complete W-30 trigger/control functions stay together in
w30_trigger_render. Two grid/report gates share pack_validation; two WAV/metrics
writers share artifact_io; command construction and private POSIX quoting
share verification_command. Only actual sibling/test operations are exposed
as pub(super). The renderer directly imports the three operations' owners,
removing all four remaining parent bridges. Manifest directly imports command
construction. Root aliases retained for untouched tests are cfg(test).

The complete validator deliberately stays cohesive rather than being split
by size. No wildcard root capture, numbered shard, new public library API,
framework or duplicate evidence/product model is introduced. PackReport remains
the existing offline value. Quoting constructs a literal string, never executes
a shell; existing adversarial command tests and artifact bytes are preserved.

All thirteen complete function/attribute token inventories match the old owner,
ignoring comments, formatting and binary-private visibility only. The two
existing legacy allow attributes remain unchanged; no warning suppression is
added. Root main, both renderer definitions and fourteen manifest definitions
also match. Existing test bodies/identities, shared safety/output plan, report
fields, library DSP/runtime/Core/App/Session/Sidecar/Cargo and frozen Stage-A
contracts are unchanged. One stale test comment is corrected, not a test body.
The obsolete source file is replaced by its complete definitions in semantic
owners and remains recoverable in Git history.

Owners are 178/255/57/23 lines, root 279 and renderer 261. Two textual includes
remain: root tests and their helper. Policy/inventory keep those visible; this
is not a thin-facade or fully completed root migration claim.

## Fresh proof

Pre-edit baseline and final Debug/Release each pass the same 143 focused
names/statuses, all executed successfully with no ignored case. Final bin/test
check is warning-free; a completed fresh five-binary build precedes actual CLI
captures and hashes. All 141 parsed CLI records match retained original
baselines, and all 85 artifact hashes match the accepted predecessor's identical
original-baseline bytes: Feral 31/39, W-30 32/9, Before/After 31/29 and comparator
47/8. JSON whitespace is not a CLI semantic difference; every parsed record
and every artifact byte is compared.

The original Feral collision loop rejects against the fresh candidate while
preserving generated input and both previous outputs. Both comparison manifests
validate with existing artifacts required. Full source-free just ci exits zero;
final check, focused, build and CI logs contain no warning/error. Formatting,
diff, include and targeted RBX-414 gates pass separately from native PR gates.

Baseline: `/tmp/riotbox-1546-baseline-debug.log`.
Final check: `/tmp/riotbox-1546-final-check.log`.
Focused proof: `/tmp/riotbox-1546-focused-{debug,release}.log`.
Fresh binaries: `/tmp/riotbox-1546-build.log`.
Full source-free CI: `/tmp/riotbox-1546-ci-final.log`, actual exit zero.

## Review and limits

Sequential solo code-review/Rust/design/spec/evidence lenses inspect all complete
owners, direct import origins and actual sibling callers, private visibility,
root compatibility consumers, unchanged algorithms/thresholds/attributes,
source/capacity/error/publication order, shared guards, test identities and
byte proof. No remaining actionable finding is demonstrated; no independent
reviewer panel is claimed. The initial import-pruning compile error is caught
and corrected with the real test-only enum alias, not a changed test or a
suppression. Fresh final checks supersede those diagnostic attempts.

Native exact-head Ubuntu/Windows PR CI, fresh reviews, merge/main synchronization
and archive/cleanup remain separate obligations, never inferred from local
proof. Only generated controls and retained fixtures are used: no real
Development/Holdout/commercial audio, source-directory discovery, DAW/device/
playback or subagents. No DSP/algorithm/threshold/schema/Core/Session/replay/
runtime/frozen Stage-A change, fallback or source/musical/hardness/human/release
verdict. Stable-namespace guard limits are unchanged; no hostile-race,
atomic-pack, power-loss or Windows audio/filesystem/device guarantee is added.
