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
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_summary(manifests), encoding="utf-8")
    print(f"wrote {output}")
    return 0


def load_manifest(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise SystemExit(f"missing Recipe 15 manifest for P012 proof summary: {path}")
    with path.open(encoding="utf-8") as handle:
        manifest = json.load(handle)
    if not isinstance(manifest, dict):
        raise SystemExit(f"manifest is not an object: {path}")
    return manifest


def render_summary(manifests: list[tuple[str, dict[str, Any]]]) -> str:
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
        "## Recipe 15 Source-Timing Outcomes",
        "",
        "| Source | Cue | Action | Readiness | Manual confirm | Grid source | Decision | Grid use | BPM | Downbeat | TR-909 | MC-202 | W-30 | Mix |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | ---: | --- | ---: | ---: | ---: | ---: |",
    ]

    for name, manifest in manifests:
        source_timing = object_field(manifest, "source_timing")
        metrics = object_field(manifest, "metrics")
        lines.append(
            "| {name} | `{cue}` | `{action}` | `{readiness}` | {manual_confirm} | `{grid_source}` | `{decision}` | `{grid_use}` | {bpm} | `{downbeat}` | {tr909} | {mc202} | {w30} | {mix} |".format(
                name=name,
                cue=string_field(source_timing, "cue"),
                action=string_field(source_timing, "actionability"),
                readiness=string_field(source_timing, "readiness"),
                manual_confirm=format_bool(bool_field(source_timing, "requires_manual_confirm")),
                grid_source=string_field(manifest, "grid_bpm_source"),
                decision=string_field(manifest, "grid_bpm_decision_reason"),
                grid_use=string_field(source_timing, "grid_use"),
                bpm=format_optional_float(source_timing.get("primary_bpm")),
                downbeat=string_field(source_timing, "downbeat_status"),
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
            "- `source_timing` rows used the current Source Timing BPM while still carrying visible manual-confirm policy where required.",
            "- `static_default` rows prove the conservative fallback boundary; Beat20 currently has useful BPM evidence but ambiguous downbeat evidence.",
            "- `Cue` and `Action` are the compact musician-facing consequence from each manifest's Source Timing evidence.",
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


def bool_field(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise SystemExit(f"{field} must be a boolean")
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
