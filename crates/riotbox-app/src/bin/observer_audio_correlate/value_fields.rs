use serde_json::Value;

pub(super) fn u64_field(object: &serde_json::Map<String, Value>, field: &str) -> Result<u64, ()> {
    object.get(field).and_then(Value::as_u64).ok_or(())
}

pub(super) fn f64_field(object: &serde_json::Map<String, Value>, field: &str) -> Result<f64, ()> {
    object.get(field).and_then(Value::as_f64).ok_or(())
}

pub(super) fn non_negative_f64_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<f64, ()> {
    let value = f64_field(object, field)?;
    if value < 0.0 {
        return Err(());
    }
    Ok(value)
}

pub(super) fn non_empty_string(value: &Value, field: &str) -> Option<String> {
    value[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn string_list(value: &Value, field: &str) -> Option<Vec<String>> {
    value[field]
        .as_array()?
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .collect()
}
