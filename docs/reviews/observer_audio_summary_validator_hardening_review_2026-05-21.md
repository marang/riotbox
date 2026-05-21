# Observer/Audio Summary Validator Hardening Review - 2026-05-21

## Scope

Scheduled narrow review for `RIOTBOX-853` after `RIOTBOX-847` through
`RIOTBOX-852` hardened observer/audio summary validation.

Reviewed:

- `scripts/validate_observer_audio_summary_json.py`
- `scripts/validate_observer_audio_*_fixtures.py`
- `Justfile` recipe `observer-audio-summary-validator-fixtures`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`

This review did not change runtime or audio behavior.

## Findings

### Major: manifest-side Source Timing `grid_use` can be omitted

Location:

- `scripts/validate_observer_audio_summary_json.py:299`
- `scripts/validate_observer_audio_summary_json.py:315`
- `scripts/validate_observer_audio_summary_json.py:825`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:102`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:106`

Category: scope / contract drift

When `output_path.source_timing` is non-null, the contract says the object should
include `grid_use` as `null` or a known Source Timing grid-use classification.
The validator currently calls `require_optional_one_of`, which silently accepts a
missing key and then skips `require_source_timing_grid_use_match`.

That means an externally edited or malformed summary can omit the compact
manifest-side grid-use verdict and still pass the stable JSON validator. This
weakens the P012 trust surface because `locked_grid`, `short_loop_manual_confirm`,
`manual_confirm_only`, `fallback_grid`, and `unavailable` are the machine-readable
labels that downstream gates use to distinguish safe grid use from confirmation
or fallback states.

Reproduction:

```bash
python3 - <<'PY'
import copy, json, subprocess, tempfile
from pathlib import Path

repo = Path(".")
fixture = repo / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json"
data = json.loads(fixture.read_text())
data["output_path"]["source_timing"].pop("grid_use")

with tempfile.TemporaryDirectory() as tmp:
    path = Path(tmp) / "missing_grid_use.json"
    path.write_text(json.dumps(data))
    result = subprocess.run(
        ["python3", "scripts/validate_observer_audio_summary_json.py", str(path)],
        cwd=repo,
        check=False,
    )
    print(result.returncode)
PY
```

Observed: return code `0`.

Suggestion:

Make `source_timing.grid_use` a required nullable field whenever
`source_timing` is present, preserve the existing enum and derivation check when
non-null, and add a negative validator fixture for a missing key.

### Major: W-30 loop-closure metric key can be omitted

Location:

- `scripts/validate_observer_audio_summary_json.py:153`
- `scripts/validate_observer_audio_summary_json.py:271`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:100`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:187`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:207`

Category: scope / contract drift

The metrics contract says `metrics` contains every currently required output
metric field, and documents `w30_source_loop_closure` as `null` or an evidence
object. The validator returns successfully when the key is absent. This is looser
than the other source-grid metrics, where missing required nullable fields are
rejected.

For strict Feral-grid output this can hide missing W-30 loop-closure proof behind
a valid summary shape. The Rust correlator may still flag missing evidence in its
own path, but the standalone stable JSON validator is not a self-contained gate
for edited or externally supplied summaries.

Reproduction:

```bash
python3 - <<'PY'
import copy, json, subprocess, tempfile
from pathlib import Path

repo = Path(".")
fixture = repo / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json"
data = json.loads(fixture.read_text())
data["output_path"]["metrics"].pop("w30_source_loop_closure")

with tempfile.TemporaryDirectory() as tmp:
    path = Path(tmp) / "missing_w30_source_loop_closure.json"
    path.write_text(json.dumps(data))
    result = subprocess.run(
        ["python3", "scripts/validate_observer_audio_summary_json.py", str(path)],
        cwd=repo,
        check=False,
    )
    print(result.returncode)
PY
```

Observed: return code `0`.

Suggestion:

Require the `w30_source_loop_closure` key as an object or `null`, keep the
existing object-shape checks, update older valid fixtures to carry `null`, and
add a negative missing-key fixture.

### Minor: lane recipe case `result` remains stringly

Location:

- `scripts/validate_observer_audio_summary_json.py:555`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:260`
- `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:297`

Category: scope / contract drift

The contract describes lane recipe case `result` as a pack verdict such as
`pass` or `fail`, and strict generated MC-202 proof requires passing case
results. The stable validator currently accepts any non-empty string.

This is lower risk than the missing `grid_use` and W-30 loop-closure key issues
because the validator already checks the MC-202 evidence fields directly, but it
still allows typoed case verdicts to pass as stable summary JSON.

Suggestion:

Promote lane recipe case `result` to a small enum once the required nullable-field
gaps are closed.

## Non-Findings

- The recent MC-202 phrase-grid and source-phrase-slot validators check evidence
  consistency rather than only field presence.
- The fixture driver count is growing, but each driver is still small and
  behavior-focused. A shared Python fixture helper would reduce duplication, but
  it is not more urgent than closing the missing required-field gaps.
- The review scope did not inspect Rust-side observer/audio correlation internals
  or generated audio behavior.

## Chosen Next Slice

Next implementation slice:

`RIOTBOX-854 - Require Source Timing grid_use in observer/audio summary validation`

Goal:

- make `output_path.source_timing.grid_use` present as a known value or `null`
- keep the existing derivation check for non-null grid-use evidence
- add validator fixture coverage for the missing-key failure
- keep the change to summary validation only; no runtime audio behavior changes

This is the smallest useful next slice because `grid_use` is the compact P012
trust label that musicians and QA use to tell grid-locked timing from
manual-confirm or fallback timing.

## Verification Commands

```bash
wc -l scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_*_fixtures.py docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md
rg -n "observer-audio-summary-validator-fixtures|validate_observer_audio_.*fixtures" Justfile scripts docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md
```
