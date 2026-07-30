use serde::{Deserialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use serde_json::Value;

pub(crate) fn number_from_string<'de, D>(deserializer: D) -> std::result::Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i32),
    }

    let value = StringOrNumber::deserialize(deserializer)?;

    match value {
        StringOrNumber::Number(n) => Ok(n),
        StringOrNumber::String(s) => s.parse::<i32>().map_err(serde::de::Error::custom),
    }
}

pub(crate) fn empty_vec_as_map<'de, D, V>(deserializer: D) -> std::result::Result<HashMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Object(map) => {
            serde_json::from_value(Value::Object(map))
                .map_err(serde::de::Error::custom)
        }
        Value::Array(items) if items.is_empty() => {
            Ok(HashMap::new())
        }
        _ => Err(serde::de::Error::custom("expected map or empty array")),
    }
}
