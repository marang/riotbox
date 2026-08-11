# Audio QA Status, History, And Future Work

Parent: [Audio QA Workflow Spec](../audio_qa_workflow_spec.md)

Descriptive only; executable contracts decide every claim.

## 13. Current Repo Status

Implemented-profile ownership lives in [automated QA](./automated_qa.md),
[artifact/export proof](./manifests_and_artifacts.md), and [human
review](./listening_review.md). Current deterministic render, correlation,
timing, replay/recovery, and bounded export evidence does not prove automatic
recovery, host-audio soak, broad crash recovery, or endurance. General
full-corpus rendering, host capture, CI-wide recipe correlation, and calibrated
human-rubric automation remain future work.

## 14. Near-Term Build Order

Widen buffer metrics and fixture-backed WAV rendering, keep manifests stable,
move calibrated metrics into CI, and keep listening local-first.

## 15. Success Condition

Catch broken audio, reproduce exact comparisons, hear validated cases, and turn
recurring weakness into the smallest useful gate or product fix.
