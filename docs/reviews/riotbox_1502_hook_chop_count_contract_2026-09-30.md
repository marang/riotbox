# RIOTBOX-1502 — Hook/Chop reverse-count evidence

Classification: diagnostic-contract maintenance. Branch:
`feature/riotbox-1502-hook-chop-evidence`; baseline:
`bc819b0d389622b1c07a1494632cdcceb80ff6af`.
The P023 blocker removed is inconsistent automated diagnostic acceptance:
passing one suite gate did not establish its written two-reverse requirement.
This slice grants no product intelligence, hardness or human musical pass.

## Historical contract resolution

Targeted Git history establishes the sequence, without loading the Decision Log
wholesale or consulting source results:

- `5aab284051a4d1b9946f7b16b84d7e04be9afe2c` introduced the riff scaffold with
  a **one**-reverse spec, dense validation, suite and tonal child gates.
- RIOTBOX-1317, `5812ad665bafd79074dee66d373d6d4bcc11749a`, deliberately raised
  the written requirement and dense rendering/validation to **two**, but did
  not update the suite or tonal child gate.
- RBX-111 later strengthened offset/hit/velocity diversity without relaxing
  reverse count. RBX-379 explicitly recorded the remaining discrepancy rather
  than silently changing it during the numeric audit.

The outcome is incomplete propagation of an intentional contract strengthening,
not permission to lower the written requirement. RBX-383 records the resolution.
No historical hash-bound evidence or human verdict is rewritten. A former
one-reverse pass retains only its weaker historical diagnostic meaning.

## Semantic owner and compatibility

`scripts/hook_chop_diagnostic_contract.py` owns the unchanged two-reverse
renderer policy and shared recorded-count checks. Dense validation, tonal child
classification/validation, and suite dense/matrix/tonal-WAV acceptance use it.
The existing 12-value general numeric owner remains unchanged.

Recorded counts must be nonnegative integer JSON numbers or supported legacy
finite integral floats. Reverse count cannot exceed total riff hits. Missing,
null, boolean, string, fractional, nonfinite or impossible evidence fails closed.
Valid zero/one counts remain recorded values for failing diagnostics, not errors
invented into measurements. Aggregation requires every relevant case; invalid
proof or an empty relevant family emits null evidence, never an invented zero
or a minimum over just the remaining cases. Non-hook-forward families stay
outside these aggregates.

Existing V1 field names remain. New valid producer reports retain their numeric
values; absent/invalid reverse evidence is now explicit null and cannot pass.
This checks recorded internal consistency, not report authenticity or playback.
It adds no alternate Source Graph, Session, action, replay, arrangement or
app-local truth. No Rust/audio callback change or new dependency is introduced.

The renderer's former literal two is now the same constant value from the
shared owner. No source-selection, hit placement, gain, reverse assignment,
sample processing or algorithm changes. A repo-only comparison of the old and
new pattern function over 100 synthetic metadata sets in three source-family
policies returned 300 exactly equal patterns
(`/tmp/riotbox-1502-pattern-equivalence.log`). This is structural regression
evidence, not an audible or human approval.

## Regression and verification

The first metadata run failed on the old implementation
(`/tmp/riotbox-1502-count-red.log`), including the one-reverse loophole on all
three suite surfaces and tonal child validation. The expanded suite has seven
test methods covering:

- zero/one/two, including legacy `2.0`, on every suite surface;
- malformed, missing, nonfinite, fractional and impossible counts;
- exact producer counts/minima, null propagation and empty-family evidence;
- dense/tonal child validation and the tonal classification bypass;
- all-case aggregation and exclusion of unrelated source families.

`just hook-chop-diagnostic-contract-fixtures` passes
(`/tmp/riotbox-1502-count-final.log`); the gate is wired into normal source-free
`just ci` and explicit GitHub Python commands (the runner does not install Just).
`just ci-gate-contract-fixtures` and `git diff --check` pass. Full `just ci`
passed (`/tmp/riotbox-1502-ci.log`): App 770, Audio 279, Core 470, Sidecar 24,
binary/integration tests, Python contracts including this seven-test gate,
source-free synthetic QA, formatting, tracked JSON and strict Clippy. The final
focused run adds malformed total-hit subcases without changing production code.
Submitted-head GitHub CI remains required before integration.

## Branch review and limits

Sequential solo `code-review` lenses cover contract provenance, strict count
domain, inclusive acceptance, all-case aggregation, family scope, unchanged
renderer behavior, test/CI wiring and historical evidence preservation. No
independent reviewer or subagent approval is claimed. No retained P0–P3 finding.
History inspection corrected the provisional attribution: the stronger contract
was introduced by RIOTBOX-1317, not the earlier one-reverse scaffold. Self-review
confirms that validators/classifier share the owner, aggregate case omissions
cannot pass, valid legacy numeric values stay supported, CI stays source-free,
and the former renderer literal is still exactly two.

All work uses repo text, synthetic metadata, mock peers and source-free normal
CI. No real source, active holdout, commercial reference, source directory,
candidate render, human playback or DAW is opened. Frozen Stage-A algorithms,
thresholds, access boundaries and JSON contracts remain untouched. This does
not recalibrate taste or claim two reverse gestures automatically sound good.
