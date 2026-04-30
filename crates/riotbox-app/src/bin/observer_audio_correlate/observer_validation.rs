use std::io;

use serde_json::Value;

const USER_SESSION_OBSERVER_SCHEMA: &str = "riotbox.user_session_observer.v1";

pub(super) fn validate_user_session_observer_events(events: &[Value]) -> Result<(), io::Error> {
    let first = events
        .first()
        .ok_or_else(|| invalid("observer stream is empty"))?;
    require_event(first, "observer_started", 1)?;
    require_equal(first, "schema", USER_SESSION_OBSERVER_SCHEMA, 1)?;
    validate_launch(require_object_field(first, "launch", 1)?)?;

    for (index, event) in events.iter().enumerate().skip(1) {
        let line = index + 1;
        match require_string(event, "event", line)?.as_str() {
            "audio_runtime" => {
                require_string(event, "status", line)?;
            }
            "key_outcome" => {
                require_string(event, "key", line)?;
                require_string(event, "outcome", line)?;
            }
            "transport_commit" => validate_committed(event, line)?,
            other => {
                return Err(invalid(format!(
                    "observer event line {line} has unsupported event {other:?}"
                )));
            }
        }
    }

    Ok(())
}

fn validate_launch(launch: &Value) -> Result<(), io::Error> {
    let mode = require_string(launch, "mode", 1)?;
    match mode.as_str() {
        "ingest" => {
            if string_field_is_present(launch, "source_path")
                || string_field_is_present(launch, "source")
            {
                Ok(())
            } else {
                Err(invalid(
                    "observer launch line 1 ingest mode requires source_path or source",
                ))
            }
        }
        "load" => require_string(launch, "session_path", 1).map(|_| ()),
        other => Err(invalid(format!(
            "observer launch line 1 has unsupported mode {other:?}"
        ))),
    }
}

fn validate_committed(event: &Value, line: usize) -> Result<(), io::Error> {
    let committed = event["committed"].as_array().ok_or_else(|| {
        invalid(format!(
            "observer event line {line} committed must be an array"
        ))
    })?;
    for (index, commit) in committed.iter().enumerate() {
        require_int(commit, "action_id", line, index)?;
        require_string(commit, "boundary", line)?;
        require_int(commit, "beat_index", line, index)?;
        require_int(commit, "bar_index", line, index)?;
        require_int(commit, "phrase_index", line, index)?;
        require_int(commit, "commit_sequence", line, index)?;
    }

    Ok(())
}

fn require_event(event: &Value, expected: &str, line: usize) -> Result<(), io::Error> {
    let actual = require_string(event, "event", line)?;
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "observer event line {line} must be {expected:?}, got {actual:?}"
        )))
    }
}

fn require_equal(event: &Value, field: &str, expected: &str, line: usize) -> Result<(), io::Error> {
    let actual = require_string(event, field, line)?;
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "observer event line {line} {field} must be {expected:?}, got {actual:?}"
        )))
    }
}

fn require_object_field<'a>(
    event: &'a Value,
    field: &str,
    line: usize,
) -> Result<&'a Value, io::Error> {
    let value = &event[field];
    if value.is_object() {
        Ok(value)
    } else {
        Err(invalid(format!(
            "observer event line {line} {field} must be an object"
        )))
    }
}

fn require_string(event: &Value, field: &str, line: usize) -> Result<String, io::Error> {
    event[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            invalid(format!(
                "observer event line {line} {field} must be a non-empty string"
            ))
        })
}

fn require_int(event: &Value, field: &str, line: usize, index: usize) -> Result<(), io::Error> {
    if event[field].as_i64().is_some() {
        Ok(())
    } else {
        Err(invalid(format!(
            "observer event line {line} committed[{index}].{field} must be an integer"
        )))
    }
}

fn string_field_is_present(event: &Value, field: &str) -> bool {
    event[field].as_str().is_some_and(|value| !value.is_empty())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
