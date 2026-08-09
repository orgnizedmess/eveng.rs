use serde::de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor};
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

struct MapOrEmptySeq<K, V>(PhantomData<(K, V)>);

impl<'de, K, V> Visitor<'de> for MapOrEmptySeq<K, V>
where
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = HashMap<K, V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map or an empty array")
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
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_type(de::Unexpected::Seq, &self));
        }
        Ok(HashMap::new())
    }
}

/// For struct fields: `#[serde(deserialize_with = "map_or_empty_seq")]`.
pub(crate) fn map_or_empty_seq<'de, D, K, V>(
    deserializer: D,
) -> std::result::Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    deserializer.deserialize_any(MapOrEmptySeq(PhantomData))
}

/// Same thing, one level deeper: the outer map's values are themselves maps
/// that can come back as `[]`.
pub(crate) fn nested_map_or_empty_seq<'de, D, K, K2, V>(
    deserializer: D,
) -> std::result::Result<HashMap<K, HashMap<K2, V>>, D::Error>
where
    D: Deserializer<'de>,
    K: Eq + Hash + Deserialize<'de>,
    K2: Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    let outer: HashMap<K, WireMap<K2, V>> = map_or_empty_seq(deserializer)?;
    Ok(outer.into_iter().map(|(k, v)| (k, v.0)).collect())
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
        map_or_empty_seq(deserializer).map(WireMap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Item {
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct Holder {
        #[serde(deserialize_with = "map_or_empty_seq")]
        ethernet: HashMap<i32, String>,
        #[serde(deserialize_with = "nested_map_or_empty_seq")]
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
    fn empty_array_becomes_an_empty_map() {
        let wire: WireMap<String, Item> = serde_json::from_str("[]").unwrap();
        assert!(wire.0.is_empty());
    }

    #[test]
    fn string_keys_convert_to_integers() {
        let wire: WireMap<i32, Item> = serde_json::from_str(r#"{"1":{"name":"a"}}"#).unwrap();
        assert_eq!(wire.0[&1].name, "a");
    }

    #[test]
    fn non_empty_array_is_an_error() {
        let err = serde_json::from_str::<WireMap<i32, Item>>(r#"[{"name":"a"}]"#).unwrap_err();
        assert!(
            err.to_string().contains("expected a map or an empty array"),
            "{err}"
        );
    }

    #[test]
    fn nested_empty_arrays_are_handled() {
        let h: Holder =
            serde_json::from_str(r#"{"ethernet":[],"serial":{"1":[],"2":{"3":"x"}}}"#).unwrap();
        assert!(h.ethernet.is_empty());
        assert!(h.serial[&1].is_empty());
        assert_eq!(h.serial[&2][&3], "x");
    }

    #[tokio::test]
    async fn empty_lab_yields_no_nodes() -> Result<()> {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/auth/login"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"code": 200, "message": "User logged in (90013).", "status": "success"}"#,
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/labs/Test.unl/nodes"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"code":200,"status":"success","message":"Successfully listed nodes (60026).","data":[]}"#,
            ))
            .mount(&server)
            .await;

        let client = crate::Client::builder(server.uri(), "Test.unl")?
            .login("admin", "eve")
            .await?;
        assert!(client.nodes().await?.is_empty());
        Ok(())
    }

    #[test]
    fn wire_map_works_inside_response() {
        let r: crate::Response<WireMap<i32, Item>> =
            serde_json::from_str(r#"{"code":200,"status":"success","message":"ok","data":[]}"#)
                .unwrap();
        assert!(r.data.unwrap().0.is_empty());
    }
}
