use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;

pub(crate) fn number_from_string<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber<T> {
        String(String),
        Number(T),
    }

    let value = StringOrNumber::deserialize(deserializer)?;

    match value {
        StringOrNumber::Number(n) => Ok(n),
        StringOrNumber::String(s) => s.parse::<T>().map_err(de::Error::custom),
    }
}

pub(crate) fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
}

struct MapOrSeq<K, V>(PhantomData<(K, V)>);

impl<'de, K, V> Visitor<'de> for MapOrSeq<K, V>
where
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = HashMap<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map or an array")
    }

    fn visit_map<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
        while let Some((k, v)) = access.next_entry()? {
            map.insert(k, v);
        }
        Ok(map)
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut map = HashMap::new();
        let mut index: usize = 0;

        while let Some(value) = seq.next_element::<V>()? {
            let key = K::deserialize(serde_json::Value::from(index)).map_err(de::Error::custom)?;
            map.insert(key, value);
            index += 1;
        }

        Ok(map)
    }
}

/// For struct fields: `#[serde(deserialize_with = "map_or_seq")]`.
pub(crate) fn map_or_seq<'de, D, K, V>(
    deserializer: D,
) -> std::result::Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserializer.deserialize_any(MapOrSeq(PhantomData))
}

/// For generic positions where no attribute can be attached, i.e. the `T` in
/// [`crate::Response<T>`]. Never appears in the public API: the client methods
/// unwrap it before handing the map back.
#[derive(Debug)]
pub(crate) struct WireMap<K, V>(pub HashMap<K, V>);

impl<'de, K, V> Deserialize<'de> for WireMap<K, V>
where
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        map_or_seq(deserializer).map(WireMap)
    }
}

/// Validation for names in EVE-NG. Allows for letters, digits
/// and an additional set of characters depending on the caller.
pub(crate) fn validate_name(name: impl Into<String>, extra: &[char]) -> crate::Result<()> {
    let name = name.into();
    if let Some(c) = name
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || extra.contains(c)))
    {
        return Err(crate::Error::InvalidName(c));
    }
    Ok(())
}

/// Validates the specified path name, which could be a folder or a lab name.
pub(crate) fn validate_pathname(pathname: impl Into<String>) -> crate::Result<()> {
    validate_name(pathname, &['_', '-', ' '])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Result};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Item {
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct Holder {
        #[serde(deserialize_with = "map_or_seq")]
        ethernet: HashMap<i32, String>,
        #[serde(deserialize_with = "map_or_seq")]
        serial: HashMap<i32, HashMap<i32, String>>,
    }

    #[test]
    fn map_stays_a_map() {
        let m: HashMap<String, Item> =
            serde_json::from_str(r#"{"1":{"name":"a"},"2":{"name":"b"}}"#).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m["1"].name, "a");
    }

    #[test]
    fn array_becomes_a_map() {
        let h: Holder =
            serde_json::from_str(r#"{"ethernet":["a", "b", "c"],"serial":[]}"#).unwrap();
        eprintln!("{:#?}", h.ethernet);
        assert_eq!(h.ethernet[&0], "a");
        assert!(h.serial.is_empty());
    }

    #[test]
    fn string_keys_convert_to_integers() {
        let wire: WireMap<i32, Item> = serde_json::from_str(r#"{"1":{"name":"a"}}"#).unwrap();
        assert_eq!(wire.0[&1].name, "a");
    }

    #[test]
    fn wire_map_works_inside_response() {
        let r: crate::client::Response<WireMap<i32, Item>> =
            serde_json::from_str(r#"{"code":200,"status":"success","message":"ok","data":[]}"#)
                .unwrap();
        assert!(r.data.unwrap().0.is_empty());
    }

    #[test]
    fn validate_folder_name() {
        let result = validate_pathname("Test Folder");
        assert!(result.is_ok());

        let result = validate_pathname("New+Folder");
        assert!(matches!(result, Err(Error::InvalidName('+'))));
    }

    #[test]
    fn validate_lab_name() {
        let result = validate_pathname("Test");
        assert!(result.is_ok());

        let result = validate_pathname("Test%");
        assert!(matches!(result, Err(Error::InvalidName('%'))));
    }
}
