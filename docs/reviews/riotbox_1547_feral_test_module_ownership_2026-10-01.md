# RIOTBOX-1547 final Feral test module ownership

Date: 2026-10-01
Implementation baseline: `4e18704e4be5398dbba777dfa4ba12ffedc21866`.
Decision: RBX-415
Classification: mechanical regression ownership, not an audible mechanism.

The final root-test and fixture includes are replaced by ordinary cfg(test)
modules. This closes the textual-include inventory and removes a P023 QA
regression-review obstacle; it does not grant a musical or product verdict.
RIOTBOX-1546 normally merged and its whole accepted candidate tree equals the
baseline. No feature stack or reset is used.

## Ownership and preserved evidence

The twelve existing tests retain their exact tests:: names. Remove the redundant
included module wrapper and wildcard root capture, not test identities or
assertions. Explicit imports identify actual Args/Grid/config/metrics/mix/
TR-909/MC-202/pack owners. The four synthetic PCM/control helpers stay together
under one sibling fixture owner and are pub(super), compiled only for tests.
Their fixture construction never becomes a production missing-source fallback.

All twelve complete test/attribute bodies match after applying the same rustfmt
to the original unwrapped module and candidate. One expression-closure brace
pair is introduced by rustfmt; the legacy include previously skipped its
formatting. Both baselines normalize to the exact same tokens, including every
assertion and literal. Four full fixture bodies match apart from private
visibility/formatting. No test/fixture/ignored state is added, removed or weakened.
The old included fixture file is replaced by complete definitions in the new
owner and remains recoverable through Git history.

Exactly 26 compiler-proven unused root cfg(test) import items are removed after
the first compile diagnostic. Three real test compatibility import groups and
production manifest aliases remain. Root is 208 lines, tests 576 and synthetic
fixtures 78. No numbered or size-only test shards, wildcard replacement,
framework or new public library API. Zero includes is not a thin-facade claim.
The retained zero-byte allowlist passes the existing exact include guard; no
guard implementation or bypass changes. Inventory/policy preserve the remaining
root compatibility assessment as separate scope.

Root main, all production renderer/manifest/report/validation/publication/
safety algorithms, untouched regression/integration consumers, library runtime/
source audio, Core/App/Session/Sidecar/Cargo and frozen Stage-A contracts are
unchanged. No new ActionCommand or product state is introduced.

## Fresh proof

Pre-edit baseline and final Debug/Release each pass the exact same 143 executed
names/statuses, no ignored or lost test. Final bin/test check is warning-free
without suppression. A completed fresh five-binary build precedes four complete
CLI captures and hashes: all 141 parsed records and 85 artifact hashes match
retained accepted original baselines (Feral 31/39, W-30 32/9, Before/After 31/29,
comparator 47/8). No earlier binary or unfinished process substitutes for proof.

The original Feral collision loop fails closed while preserving its generated
input and previous outputs. Both comparator manifests validate with existing
artifacts required. Formatting, diff, zero-include and targeted RBX-415 readback
gates pass. Full source-free just ci exits zero; final check, focused, build and
CI logs contain no compiler/Clippy warning or error. Remaining root compatibility
bridges are separately tracked by RIOTBOX-1548, not silently considered closed.

Baseline: `/tmp/riotbox-1547-baseline-debug.log`.
Final check: `/tmp/riotbox-1547-final-check.log`.
Focused proof: `/tmp/riotbox-1547-focused-{debug,release}.log`.
Fresh binaries: `/tmp/riotbox-1547-build.log`.
Full source-free CI: `/tmp/riotbox-1547-ci-final.log`, actual exit zero.

## Review and boundaries

Sequential solo code-review/Rust/design/spec/evidence review inspects complete
test/fixture definitions and attributes, actual import/type origins, unchanged
namespace paths, cfg(test) exposure, helper visibility, compiler-proven pruning,
formatting normalization and every preserved assertion/literal. Main and the
untouched production/safety/replay/runtime contracts remain intact; fresh
identity and byte evidence agree. Short self-review has no remaining actionable
finding; no independent panel is claimed.

Native exact-head Rust/Windows jobs, fresh PR reviews, merge/main synchronization
and archive/cleanup are separate remaining obligations, not local proof claims.
Only generated controls/retained fixtures: no real Development/Holdout/commercial
audio, source-directory discovery, DAW/device/human playback or subagents. No
DSP/algorithm/threshold/schema/Core/Session/replay/runtime/frozen Stage-A change,
fallback or source/musical/hardness/human/release verdict. Stable-namespace guard
limits remain; no hostile-race, atomic-pack, power-loss or Windows audio/
filesystem/device guarantee is added. RIOTBOX-1509 still needs a fresh minimized
Windows failure, not closure from these green native transport runs.
