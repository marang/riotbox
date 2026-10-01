use super::report_model::ObserverSceneMovementEvidence;
use super::value_fields::non_empty_string;
use serde_json::Value;

pub(super) fn collect_observer_scene_movement(
    events: &[Value],
) -> (Option<ObserverSceneMovementEvidence>, bool) {
    let Some(scene) = events
        .iter()
        .rev()
        .filter_map(|event| event["snapshot"].get("scene"))
        .find(|scene| scene["last_movement"].is_object())
    else {
        return (None, false);
    };

    let movement = &scene["last_movement"];
    let contract = &scene["arrangement_contract"];
    let source_monitor = &scene["source_monitor"];
    let evidence = ObserverSceneMovementEvidence {
        active_scene: optional_observer_scene_string(scene, "active_scene"),
        kind: match non_empty_string(movement, "kind") {
            Some(value) => value,
            None => return (None, true),
        },
        direction: match non_empty_string(movement, "direction") {
            Some(value) => value,
            None => return (None, true),
        },
        tr909_intent: match non_empty_string(movement, "tr909_intent") {
            Some(value) => value,
            None => return (None, true),
        },
        mc202_intent: match non_empty_string(movement, "mc202_intent") {
            Some(value) => value,
            None => return (None, true),
        },
        w30_intent: match non_empty_string(movement, "w30_intent") {
            Some(value) => value,
            None => return (None, true),
        },
        intensity: match movement["intensity"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        from_scene: optional_observer_scene_string(movement, "from_scene"),
        to_scene: match non_empty_string(movement, "to_scene") {
            Some(value) => value,
            None => return (None, true),
        },
        committed_bar_index: match movement["committed_bar_index"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        committed_phrase_index: match movement["committed_phrase_index"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        can_use_source_locked_scene_movement: match contract["can_use_source_locked_scene_movement"]
            .as_bool()
        {
            Some(value) => value,
            None => return (None, true),
        },
        source_anchor_seconds: optional_observer_scene_f64(source_monitor, "source_anchor_seconds"),
        source_anchor_position_beats: optional_observer_scene_f64(
            source_monitor,
            "source_anchor_position_beats",
        ),
    };

    (Some(evidence), false)
}

fn optional_observer_scene_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn optional_observer_scene_f64(value: &Value, field: &str) -> Option<f64> {
    match value.get(field) {
        Some(field) if field.is_null() => None,
        Some(field) => field.as_f64(),
        None => None,
    }
}
