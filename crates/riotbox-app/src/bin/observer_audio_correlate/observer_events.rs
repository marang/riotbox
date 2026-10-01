use serde_json::Value;

pub(super) fn string_field(event: &Value, field: &str) -> String {
    event[field].as_str().unwrap_or("unknown").to_string()
}

pub(super) fn format_first_commit(event: &Value) -> Option<String> {
    let commit = event["committed"].as_array()?.first()?;
    Some(format!(
        "action {} at {} beat {} bar {} phrase {} sequence {}",
        commit["action_id"].as_u64().unwrap_or_default(),
        commit["boundary"].as_str().unwrap_or("unknown"),
        commit["beat_index"].as_u64().unwrap_or_default(),
        commit["bar_index"].as_u64().unwrap_or_default(),
        commit["phrase_index"].as_u64().unwrap_or_default(),
        commit["commit_sequence"].as_u64().unwrap_or_default()
    ))
}

pub(super) fn collect_commit_summary(events: &[Value]) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut boundaries = Vec::new();

    for commit in events
        .iter()
        .filter(|event| event["event"] == "transport_commit")
        .filter_map(|event| event["committed"].as_array())
        .flatten()
    {
        count += 1;
        let boundary = commit["boundary"].as_str().unwrap_or("unknown").to_string();
        if !boundaries.contains(&boundary) {
            boundaries.push(boundary);
        }
    }

    (count, boundaries)
}
