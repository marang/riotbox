#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


RECIPE15_PROFILES = (
    (
        "Beat03",
        Path("artifacts/audio_qa/local-beat03-feral-grid-auto-proof/feral-grid-demo/manifest.json"),
    ),
    (
        "Beat08",
        Path("artifacts/audio_qa/local-beat08-feral-grid-auto-proof/feral-grid-demo/manifest.json"),
    ),
    (
        "DH_BeatC",
        Path("artifacts/audio_qa/local-dh-beatc-feral-grid-auto-proof/feral-grid-demo/manifest.json"),
    ),
    (
        "Beat20",
        Path("artifacts/audio_qa/local-beat20-feral-grid-auto-fallback-proof/feral-grid-demo/manifest.json"),
    ),
)

GENERATED_OBSERVER_AUDIO_SUMMARIES = (
    (
        "cautious/manual",
        Path("artifacts/audio_qa/local/generated-feral-grid-observer-audio/cautious-manual-confirm.json"),
    ),
    (
        "user override",
        Path("artifacts/audio_qa/local/generated-feral-grid-observer-audio/user-override.json"),
    ),
    (
        "risky override",
        Path("artifacts/audio_qa/local/generated-feral-grid-observer-audio/risky-user-override.json"),
    ),
    (
        "fallback",
        Path("artifacts/audio_qa/local/generated-feral-grid-observer-audio/fallback.json"),
    ),
    (
        "locked grid",
        Path("artifacts/audio_qa/local/generated-feral-grid-observer-audio/locked-grid.json"),
    ),
)

MIN_HIT_RATIO = 0.5


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        default="artifacts/audio_qa/local/p012_all_lane_source_grid_output_proof_summary.md",
        help="Markdown summary output path",
    )
    args = parser.parse_args()

    output = Path(args.output)
    manifests = [(name, load_manifest(path)) for name, path in RECIPE15_PROFILES]
    observer_audio_summaries = [
        (name, load_json_summary(path)) for name, path in GENERATED_OBSERVER_AUDIO_SUMMARIES
    ]
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_summary(manifests, observer_audio_summaries), encoding="utf-8")
    print(f"wrote {output}")
    return 0


def load_manifest(path: Path) -> dict[str, Any]:
    return load_json_object(path, "Recipe 15 manifest")


def load_json_summary(path: Path) -> dict[str, Any]:
    return load_json_object(path, "generated Feral-grid observer/audio summary")


def load_json_object(path: Path, label: str) -> dict[str, Any]:
    if not path.is_file():
        raise SystemExit(f"missing {label} for P012 proof summary: {path}")
    with path.open(encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise SystemExit(f"{label} is not an object: {path}")
    return data


def render_summary(
    manifests: list[tuple[str, dict[str, Any]]],
    observer_audio_summaries: list[tuple[str, dict[str, Any]]],
) -> str:
    lines = [
        "# P012 All-Lane Source-Grid Output Proof Summary",
        "",
        "Status: `pass`",
        "",
        "This summary is generated after `just p012-all-lane-source-grid-output-proof` passes.",
        "It does not replace the underlying observer/audio validators or listening manifests; it is a compact readout of the proof outcomes.",
        "",
        "## Gate Components",
        "",
        "- Observer/audio generated Feral-grid correlation: `pass`",
        "- Recipe 2 observer/audio gate: `pass`",
        "- Recipe 15 real-source auto/fallback proof: `pass`",
        "",
        "## Generated Feral-Grid Observer/Audio Paths",
        "",
        "| Path | Cue | Action | Grid source | Decision | Observer grid use | Manifest grid use | Grid compat | Downbeat compat | Downbeat ambiguity | Alignment | Output issues |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: |",
    ]

    for name, summary in observer_audio_summaries:
        output_path = object_field(summary, "output_path")
        source_timing = object_field(output_path, "source_timing")
        source_timing_alignment = object_field(output_path, "source_timing_alignment")
        lines.append(
            "| {name} | `{cue}` | `{action}` | `{grid_source}` | `{decision}` | `{observer_grid_use}` | `{manifest_grid_use}` | `{grid_compat}` | `{downbeat_compat}` | `{downbeat_ambiguity}` | `{alignment}` | {issues} |".format(
                name=name,
                cue=string_field(source_timing, "cue"),
                action=string_field(source_timing, "actionability"),
                grid_source=string_field(output_path, "grid_bpm_source"),
                decision=string_field(output_path, "grid_bpm_decision_reason"),
                observer_grid_use=string_field(source_timing_alignment, "observer_grid_use"),
                manifest_grid_use=string_field(source_timing_alignment, "manifest_grid_use"),
                grid_compat=string_field(source_timing_alignment, "grid_use_compatibility"),
                downbeat_compat=string_field(
                    source_timing_alignment, "downbeat_offset_compatibility"
                ),
                downbeat_ambiguity=string_field(
                    source_timing_alignment, "downbeat_ambiguity_compatibility"
                ),
                alignment=string_field(source_timing_alignment, "status"),
                issues=list_len(output_path, "issues"),
            )
        )

    lines.extend(
        [
            "",
            "## Recipe 15 Source-Timing Outcomes",
            "",
            "| Source | Cue | Action | Readiness | Manual confirm | Grid source | Decision | Grid use | Phrase count | Phrase bars | BPM | Downbeat | Downbeat score | Downbeat margin | Alt phases | TR-909 | MC-202 | W-30 | Mix |",
            "| --- | --- | --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
        ]
    )

    for name, manifest in manifests:
        source_timing = object_field(manifest, "source_timing")
        metrics = object_field(manifest, "metrics")
        lines.append(
            "| {name} | `{cue}` | `{action}` | `{readiness}` | {manual_confirm} | `{grid_source}` | `{decision}` | `{grid_use}` | {phrase_count} | {phrase_bars} | {bpm} | `{downbeat}` | {downbeat_score} | {downbeat_margin} | {alt_phases} | {tr909} | {mc202} | {w30} | {mix} |".format(
                name=name,
                cue=string_field(source_timing, "cue"),
                action=string_field(source_timing, "actionability"),
                readiness=string_field(source_timing, "readiness"),
                manual_confirm=format_bool(bool_field(source_timing, "requires_manual_confirm")),
                grid_source=string_field(manifest, "grid_bpm_source"),
                decision=string_field(manifest, "grid_bpm_decision_reason"),
                grid_use=string_field(source_timing, "grid_use"),
                phrase_count=int_field(source_timing, "primary_phrase_count"),
                phrase_bars=int_field(source_timing, "primary_phrase_bar_count"),
                bpm=format_optional_float(source_timing.get("primary_bpm")),
                downbeat=string_field(source_timing, "downbeat_status"),
                downbeat_score=format_optional_float(source_timing.get("primary_downbeat_score")),
                downbeat_margin=format_optional_float(
                    source_timing.get("primary_downbeat_margin")
                ),
                alt_phases=int_field(source_timing, "alternate_downbeat_phase_count"),
                tr909=format_metric_hit_ratio(metrics, "tr909_source_grid_alignment"),
                mc202=format_metric_hit_ratio(metrics, "mc202_source_grid_alignment"),
                w30=format_metric_hit_ratio(metrics, "w30_source_grid_alignment"),
                mix=format_metric_hit_ratio(metrics, "source_grid_output_drift"),
            )
        )

    lines.extend(
        [
            "",
            "## Interpretation",
            "",
            "- Generated Feral-grid observer/audio rows show whether control-path and output-path timing evidence agreed for cautious/manual-confirm, user-override, fallback, and locked-grid paths.",
            "- Generated `Cue` and `Action` columns expose the musician-facing consequence for each generated path without opening the JSON summaries.",
            "- Generated `Downbeat ambiguity` shows whether the observer and manifest agree about bar-phase ambiguity, not just selected downbeat offset.",
            "- `source_timing` rows used the current Source Timing BPM while still carrying visible manual-confirm policy where required.",
            "- `static_default` rows prove the conservative fallback boundary; Beat20 currently has useful BPM evidence but ambiguous downbeat evidence.",
            "- `Cue` and `Action` are the compact musician-facing consequence from each manifest's Source Timing evidence.",
            "- `Phrase count` and `Phrase bars` expose the bounded phrase-grid evidence behind short-loop/manual-confirm and locked-grid classifications.",
            "- `Downbeat score`, `Downbeat margin`, and `Alt phases` expose the bounded bar-phase confidence behind the selected downbeat offset.",
            "- Lane columns are hit ratios from the underlying manifests; values at or above `0.5` pass the current bounded P012 smoke threshold.",
            "",
        ]
    )
    return "\n".join(lines)


def object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    value = parent.get(field)
    if not isinstance(value, dict):
        raise SystemExit(f"{field} must be an object")
    return value


def string_field(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"{field} must be a non-empty string")
    return value


def list_len(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, list):
        raise SystemExit(f"{field} must be a list")
    return len(value)


def bool_field(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise SystemExit(f"{field} must be a boolean")
    return value


def int_field(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise SystemExit(f"{field} must be an integer")
    return value


def format_bool(value: bool) -> str:
    return "`yes`" if value else "`no`"


def format_optional_float(value: Any) -> str:
    if isinstance(value, (int, float)):
        return f"{float(value):.3f}"
    return "n/a"


def format_metric_hit_ratio(metrics: dict[str, Any], metric_name: str) -> str:
    metric = object_field(metrics, metric_name)
    value = metric.get("hit_ratio")
    if not isinstance(value, (int, float)):
        raise SystemExit(f"{metric_name}.hit_ratio must be numeric")
    hit_ratio = float(value)
    if hit_ratio < MIN_HIT_RATIO:
        raise SystemExit(
            f"{metric_name}.hit_ratio {hit_ratio:.3f} is below P012 threshold {MIN_HIT_RATIO:.3f}"
        )
    return f"{hit_ratio:.3f}"


if __name__ == "__main__":
    raise SystemExit(main())
