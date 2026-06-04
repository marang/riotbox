#!/usr/bin/env python3
"""Prototype Riotbox audio-judge readiness from review packs and labels."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.audio_judge_spike.v1"
AGENT_REVIEW_SCHEMA = "riotbox.agent_musical_review_pack.v1"
LABEL_CORPUS_SCHEMA = "riotbox.human_listening_label_corpus.v1"

MIN_LABELS_FOR_READY_SPIKE = 12
HUMAN_VERDICTS = {"pass", "weak", "fail", "inconclusive"}
REQUIRED_READY_VERDICTS = {"pass", "weak", "fail"}
CLAP_PACKAGES = ("torch", "laion_clap")
MERT_PACKAGES = ("torch", "transformers")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent-review", type=Path, required=True)
    parser.add_argument("--label-corpus", type=Path, required=True)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    args = parser.parse_args()

    try:
        agent_review = read_json_object(args.agent_review)
        label_corpus = read_json_object(args.label_corpus)
        report = build_report(agent_review, args.agent_review, label_corpus, args.label_corpus)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(report, indent=2) + "\n")
        if args.markdown_output:
            args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
            args.markdown_output.write_text(render_markdown(report))
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid audio judge spike: {error}", file=sys.stderr)
        return 1

    print(
        "audio judge spike completed: "
        f"recommendation={report['recommendation']['status']} "
        f"matched_labels={report['calibration']['matched_label_count']}"
    )
    return 0


def build_report(
    agent_review: dict[str, Any],
    agent_review_path: Path,
    label_corpus: dict[str, Any],
    label_corpus_path: Path,
) -> dict[str, Any]:
    validate_agent_review(agent_review, agent_review_path)
    labels = validate_label_corpus(label_corpus, label_corpus_path)
    review_pack_id = infer_review_pack_id(agent_review, agent_review_path)
    metrics_baseline = build_metrics_baseline(agent_review)
    calibration = build_calibration(labels, review_pack_id, metrics_baseline)
    providers = [
        optional_provider(
            "clap_optional",
            "CLAP-style audio/text embedding provider",
            CLAP_PACKAGES,
            "Could compare generated packs to Riotbox prompt/rubric text and labeled examples.",
        ),
        optional_provider(
            "mert_optional",
            "MERT-style music-audio embedding provider",
            MERT_PACKAGES,
            "Could compare source-backed performances by musical similarity and human labels.",
        ),
    ]
    recommendation = build_recommendation(labels, calibration, providers)
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "technical_status": "pass",
        "judge_readiness": recommendation["status"],
        "human_verdict": "unverified",
        "inputs": {
            "agent_review": str(agent_review_path),
            "label_corpus": str(label_corpus_path),
            "review_pack_id": review_pack_id,
            "agent_review_schema": agent_review["schema"],
            "label_corpus_schema": label_corpus["schema"],
        },
        "metrics_baseline": metrics_baseline,
        "providers": providers,
        "calibration": calibration,
        "recommendation": recommendation,
        "boundary": (
            "This spike may reject known weak-output shapes and identify calibration gaps. "
            "It must not promote CLAP, MERT, or deterministic metrics to a product musical "
            "pass source without broader Riotbox labels and human listening evidence."
        ),
    }


def validate_agent_review(agent_review: dict[str, Any], path: Path) -> None:
    require(agent_review.get("schema") == AGENT_REVIEW_SCHEMA, f"{path}: unexpected schema")
    require(agent_review.get("schema_version") == 1, f"{path}: schema_version must be 1")
    for field in (
        "result",
        "agent_verdict",
        "human_verdict",
        "source_recognition",
        "hook_after_two_bars",
        "proof",
    ):
        require(field in agent_review, f"{path}: missing {field}")
    require(isinstance(agent_review["proof"], dict), f"{path}: proof must be object")


def validate_label_corpus(label_corpus: dict[str, Any], path: Path) -> list[dict[str, Any]]:
    require(label_corpus.get("schema") == LABEL_CORPUS_SCHEMA, f"{path}: unexpected schema")
    require(label_corpus.get("schema_version") == 1, f"{path}: schema_version must be 1")
    labels = label_corpus.get("labels")
    require(isinstance(labels, list) and labels, f"{path}: labels must be non-empty array")
    result = []
    for index, label in enumerate(labels):
        require(isinstance(label, dict), f"{path}: label {index} must be object")
        for field in (
            "label_id",
            "human_verdict",
            "source_family",
            "source_id",
            "review_pack_schema",
            "review_pack_id",
            "reason_tags",
        ):
            require(field in label, f"{path}: label {index} missing {field}")
        require(
            label["human_verdict"] in HUMAN_VERDICTS,
            f"{path}: label {index} has unknown human_verdict",
        )
        require(
            label["review_pack_schema"] == AGENT_REVIEW_SCHEMA,
            f"{path}: label {index} review_pack_schema must be {AGENT_REVIEW_SCHEMA}",
        )
        result.append(label)
    return result


def infer_review_pack_id(agent_review: dict[str, Any], path: Path) -> str:
    explicit = agent_review.get("review_pack_id") or agent_review.get("pack_id")
    if isinstance(explicit, str) and explicit.strip():
        return explicit
    return path.parent.name


def build_metrics_baseline(agent_review: dict[str, Any]) -> dict[str, Any]:
    proof = agent_review["proof"]
    dimensions = [
        dimension(
            "hook_presence",
            score_at_least(proof, "w30_to_source_rms_ratio", 0.18),
            "W-30 chop should read as the hook within two bars.",
        ),
        dimension(
            "bass_pressure",
            score_at_least(proof, "pressure_low_band_lift_ratio", 1.12),
            "Pressure section should lift low-band energy beyond the opening hook.",
        ),
        dimension(
            "destructive_contrast",
            score_at_most(proof, "dropout_to_stutter_rms_ratio", 0.18),
            "Dropout should be much quieter than the stutter restore.",
        ),
        dimension(
            "restore_impact",
            score_at_least(proof, "restore_to_hook_transient_ratio", 0.85),
            "Restore should land with break transient impact.",
        ),
        dimension(
            "bar_movement",
            score_at_most(proof, "max_adjacent_bar_correlation", 0.985),
            "Adjacent bars should not collapse into the same loop.",
        ),
        dimension(
            "source_transformation",
            score_source_transformation(proof),
            "Source character should be transformed without becoming source-copy collapse.",
        ),
    ]
    score = round(sum(item["score"] for item in dimensions) / len(dimensions), 6)
    predicted_label = predict_label(score, agent_review)
    return {
        "status": "available",
        "provider": "riotbox_metrics_baseline",
        "score": score,
        "predicted_label": predicted_label,
        "agent_verdict": agent_review["agent_verdict"],
        "human_verdict": agent_review["human_verdict"],
        "dimensions": dimensions,
        "failure_codes": agent_review.get("failure_codes", []),
        "limitations": [
            "Can catch known weak-output modes but cannot hear taste, annoyance, or memorability.",
            "Uses dense-break proof fields only; it is not a general Riotbox musical judge.",
        ],
    }


def build_calibration(
    labels: list[dict[str, Any]],
    review_pack_id: str,
    metrics_baseline: dict[str, Any],
) -> dict[str, Any]:
    verdict_counts = count_by(labels, "human_verdict")
    matched = [label for label in labels if label["review_pack_id"] == review_pack_id]
    examples = []
    confusion = empty_confusion_matrix()
    for label in matched:
        expected = label["human_verdict"]
        predicted = metrics_baseline["predicted_label"]
        confusion.setdefault(expected, {})
        confusion[expected][predicted] = confusion[expected].get(predicted, 0) + 1
        examples.append(
            {
                "label_id": label["label_id"],
                "review_pack_id": label["review_pack_id"],
                "human_verdict": expected,
                "predicted_label": predicted,
                "outcome": "match" if expected == predicted else "mismatch",
                "reason_tags": label["reason_tags"],
                "summary": label.get("summary", ""),
            }
        )

    coverage_gaps = [
        {
            "type": "unscored_label",
            "label_id": label["label_id"],
            "review_pack_id": label["review_pack_id"],
            "human_verdict": label["human_verdict"],
            "reason": (
                "No agent-review metrics were supplied for this labeled pack, so the spike "
                "cannot test this verdict yet."
            ),
        }
        for label in labels
        if label["review_pack_id"] != review_pack_id
    ]
    covered_verdicts = {label["human_verdict"] for label in matched}
    missing_ready_verdicts = sorted(REQUIRED_READY_VERDICTS - covered_verdicts)
    return {
        "label_count": len(labels),
        "verdict_counts": verdict_counts,
        "matched_label_count": len(matched),
        "matched_verdicts": sorted(covered_verdicts),
        "missing_ready_verdicts": missing_ready_verdicts,
        "confusion_matrix": confusion,
        "examples": examples,
        "failure_examples": coverage_gaps,
    }


def build_recommendation(
    labels: list[dict[str, Any]],
    calibration: dict[str, Any],
    providers: list[dict[str, Any]],
) -> dict[str, Any]:
    unavailable = [provider["name"] for provider in providers if provider["status"] != "available"]
    reasons = []
    if len(labels) < MIN_LABELS_FOR_READY_SPIKE:
        reasons.append(
            f"Only {len(labels)} labels exist; at least {MIN_LABELS_FOR_READY_SPIKE} are needed "
            "before a judge spike can say anything useful about generalization."
        )
    if calibration["missing_ready_verdicts"]:
        reasons.append(
            "Matched labels do not cover required verdicts: "
            + ", ".join(calibration["missing_ready_verdicts"])
        )
    if unavailable:
        reasons.append(
            "Optional embedding providers unavailable or not configured: " + ", ".join(unavailable)
        )
    if calibration["matched_label_count"] == 0:
        reasons.append("No human label matched the supplied review pack.")
    status = "not_ready" if reasons else "useful"
    return {
        "status": status,
        "result": status,
        "short_reason": (
            "Metrics baseline is wired, but calibrated musical-pass judgment is not ready."
            if status == "not_ready"
            else "Enough labels and provider coverage exist to continue calibration."
        ),
        "reasons": reasons,
        "next_steps": [
            "Generate and label more dense-break pass/weak/fail packs with real listening verdicts.",
            "Add tonal-hook and sparse/bass-pressure labels before trusting cross-source scores.",
            "Only keep CLAP/MERT-style providers if they separate weak hooks from strong hooks better than metrics alone.",
        ],
    }


def optional_provider(
    name: str,
    description: str,
    packages: tuple[str, ...],
    expected_value: str,
) -> dict[str, Any]:
    package_status = {
        package: "available" if importlib.util.find_spec(package) is not None else "missing"
        for package in packages
    }
    available = all(status == "available" for status in package_status.values())
    return {
        "name": name,
        "description": description,
        "status": "available" if available else "unavailable",
        "dependency_status": package_status,
        "decision": "not_evaluated",
        "expected_value": expected_value,
        "runtime_boundary": "optional offline QA only; never realtime audio or product runtime",
    }


def score_at_least(proof: dict[str, Any], field: str, threshold: float) -> float:
    value = number(proof.get(field))
    if value is None:
        return 0.0
    return clamp(value / threshold)


def score_at_most(proof: dict[str, Any], field: str, threshold: float) -> float:
    value = number(proof.get(field))
    if value is None:
        return 0.0
    if value <= threshold:
        return 1.0
    return clamp(threshold / max(value, 1e-9))


def score_source_transformation(proof: dict[str, Any]) -> float:
    correlation = number(proof.get("source_to_performance_correlation"))
    if correlation is None:
        return 0.0
    if 0.20 <= correlation <= 0.90:
        return 1.0
    if correlation < 0.20:
        return clamp(correlation / 0.20)
    return clamp((0.975 - correlation) / (0.975 - 0.90))


def dimension(name: str, score: float, intent: str) -> dict[str, Any]:
    return {
        "name": name,
        "score": round(score, 6),
        "status": "pass" if score >= 0.95 else "weak" if score >= 0.70 else "fail",
        "intent": intent,
    }


def predict_label(score: float, agent_review: dict[str, Any]) -> str:
    if agent_review["result"] != "pass" or score < 0.60:
        return "fail"
    if score < 0.85 or agent_review["agent_verdict"] == "agent_weak":
        return "weak"
    return "pass"


def empty_confusion_matrix() -> dict[str, dict[str, int]]:
    return {
        expected: {predicted: 0 for predicted in ("pass", "weak", "fail")}
        for expected in ("pass", "weak", "fail", "inconclusive")
    }


def count_by(items: list[dict[str, Any]], field: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for item in items:
        value = str(item[field])
        counts[value] = counts.get(value, 0) + 1
    return dict(sorted(counts.items()))


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Audio Judge Spike",
        "",
        f"- Result: `{report['result']}`",
        f"- Judge readiness: `{report['judge_readiness']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Metrics baseline prediction: `{report['metrics_baseline']['predicted_label']}`",
        f"- Metrics baseline score: `{report['metrics_baseline']['score']}`",
        "",
        "## Recommendation",
        "",
        report["recommendation"]["short_reason"],
        "",
    ]
    for reason in report["recommendation"]["reasons"]:
        lines.append(f"- {reason}")
    lines.extend(["", "## Providers", ""])
    for provider in report["providers"]:
        missing = [
            name
            for name, status in provider["dependency_status"].items()
            if status != "available"
        ]
        suffix = f" missing: {', '.join(missing)}" if missing else ""
        lines.append(f"- `{provider['name']}`: `{provider['status']}`.{suffix}")
    lines.extend(["", "## Calibration Examples", ""])
    if report["calibration"]["examples"]:
        for example in report["calibration"]["examples"]:
            lines.append(
                "- "
                f"`{example['label_id']}` human `{example['human_verdict']}` "
                f"predicted `{example['predicted_label']}` outcome `{example['outcome']}`"
            )
    else:
        lines.append("- No matched human labels for this review pack.")
    lines.extend(["", "## Failure Examples", ""])
    if report["calibration"]["failure_examples"]:
        for example in report["calibration"]["failure_examples"]:
            lines.append(
                "- "
                f"`{example['label_id']}` `{example['human_verdict']}`: "
                f"{example['reason']}"
            )
    else:
        lines.append("- No coverage gaps in supplied labels.")
    lines.extend(["", "## Boundary", "", report["boundary"], ""])
    return "\n".join(lines)


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: JSON root must be object")
    return value


def number(value: Any) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    return float(value)


def clamp(value: float) -> float:
    return max(0.0, min(1.0, float(value)))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
