# RIOTBOX-1540 W-30 output pair integrity

Date: 2026-10-01
Implementation baseline: `59edfc3c1c6e1f7f9c2ad991ef1fc0f61963cc97`.
Integrated audit-only predecessor: `a53853593462bdf1660bfcbd1095f41da6cfaaad`.
Decision: RBX-409
Classification: maintenance/regression, existing offline QA output corruption.

The renderer now rejects physically coupled WAV/metrics outputs before either
can replace the other. Both its explicit-source and explicit synthetic QA modes
keep their valid rendering. This does not change product audio or claim musical,
source-general, human or Windows Audio qualification.

## Cause and red to green

RIOTBOX-1539's repeated actual CLI probe returns zero while a hardlinked
preview.wav/preview.metrics.md pair changes WAV magic from RIFF `52494646`
to Markdown `2320572d`. The source hash remains intact; source preservation
alone never promised mutual output independence. The WAV writer truncates its
entry, then the metrics writer truncates the same physical file. A separate
output counterfactual returns success with a valid WAV.

The first actual public CLI regression goes red on replaced output in explicit
no-source diagnostic mode. After the preflight it passes in both modes and all
prior source/comparison protection cases remain green. The refined original
loop against the exact fresh candidate now returns exit 1 with unchanged input
and both prior output bytes; separate output still returns zero with RIFF:
`bash /tmp/riotbox-qa-output-alias-probe.6WrEPt/repro.sh alias /tmp/riotbox-1507-review.4spOix/target/debug/w30_preview_render`.
Each invocation allocates an isolated owned temporary directory and uses only
the previously generated exact synthetic control; no source discovery.

The issue moved Todo -> In Progress before an independent main-based branch.
The already-reviewed audit merges disjointly without discarding implementation;
no stacked feature branch, backfilled issue, subagent or unrelated change.

## One admission owner and a small output pair gate

The ordinary W-30 output_safety owner is sixteen lines. It receives the actual
WAV and existing metrics naming result after duration/capacity/source preparation
and the existing optional source-preservation check. Both output entries are
admitted before offline rendering and either write, even in no-source QA mode.
They are writer-derived distinct names; when both already exist, the same
shared physical identity implementation checks their mutual distinctness.

Necessary refactoring extracts the existing symlink_metadata/metadata admission
from reject_source_aliases into existing_output_is_regular in the same ordinary
binary-only owner, now 45 lines. False means only a genuinely absent entry;
nonregular/dangling/unknown paths reject, including a late bad metrics entry
with no WAV yet. The source handle still opens before iteration, both compared
handles stay alive, and the same error kinds/literals/metadata sequence remain.
No second filesystem backend, framework, public library API, target/dependency,
resource limit or product-state authority is introduced.

When both outputs exist, identity inspection requires readable/identifiable
regular files. Distinct equal-content files and absent outputs remain valid.
No-source diagnostic mode now rejects unsafe destinations before partial output;
its synthetic rendering is unchanged and never a missing-source product fallback.

## Technical proof and normal compatibility

Six new actual CLI cases, all in both diagnostic modes, cover:

- Hardlinked WAV/metrics preserve both previous outputs and input.
- Unix output symlinks in both directions and aliased output directory.
- Either nonregular destination with previous or absent peer, no early publication.
- Either Unix dangling destination, no target creation or other replacement.
- Equal-content independent output files render decodable WAV plus metrics.
- Both outputs absent, including initially missing parent, remain supported.

The regression owner is 209 lines. Assertions cross the real CLI seam, inspect
actual output bytes and decode successful WAVs, never call private preflight or
mock its identity backend. No existing test is removed, ignored or rewritten;
all 26 prior renderer/comparison safety test bodies remain byte-identical.

All 133 focused test names/statuses match and pass in Debug and Release:
95 retained binary units, six capacity/duration/verification cases, 26 prior
source/comparison safety cases and six new output-pair cases.
Fresh standalone build precedes sequential normal CLI captures, completed
before each hash comparison. All 141 records match baselines: Feral 31, W-30
32, Before/After 31 and comparison 47. All 85 input/output SHA-256 values match
respectively 39, nine, 29 and eight. This also proves unchanged valid consumers
of the shared admission extraction, not only the W-30 happy path.
Only exact already-generated controls from those four parity directories are
opened. No thresholds are tuned against results.

## Solo review and limits

Sequential code-review/Rust/design/spec/evidence lenses inspect complete changed
functions, helper call order and missing/error semantics, both writers and
existing naming convention, all shared consumers, physical alias cases,
regression independence, CLI/schema behavior, module size and workflow/docs.
The small shared admission extraction avoids duplicate safety logic while the
W-30-specific pair rule stays local. Short self-review finds no remaining
actionable finding in this diff; no independent panel or subagents.

Stable filesystem namespace only: no hostile concurrent replacement lock,
atomic whole-pack publication, power-loss or Windows Audio/filesystem/device
execution guarantee. No inferred fix for other renderers' mutual output aliases.
No Core/Session/replay/App/runtime/DSP/algorithm/threshold/schema or frozen
Stage-A change, product fallback, musical/source/diversity/hardness/human/release
claim, real Development/Holdout/commercial audio, DAW or playback.

## Validation and remaining gates

- First red/green: `/tmp/riotbox-1540-output-safety-{red,green}.log`.
- Final six cases: `/tmp/riotbox-1540-output-safety-final.log`.
- Fresh binaries: `/tmp/riotbox-1540-build-reviewed.log`.
- Focused Debug/Release: `/tmp/riotbox-1540-focused-{debug,release}.log`.
- Full source-free CI: `/tmp/riotbox-1540-ci-final.log`, actual exit zero.
- Final warning/error scan, fmt/diff/include gates and targeted RBX-409 readback
  pass. Both unchanged comparison manifests retain valid existing paths.
- Exact-head native PR review/merge, archive and cleanup remain required.
