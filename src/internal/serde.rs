use crate::common::Image;
use crate::error::{CrunchyrollError, CrunchyrollErrorContext};
use crate::{Request, Result};
use chrono::Duration;
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::str::FromStr;

#[derive(Request)]
pub(crate) struct EmptyJsonProxy;
impl<'de> Deserialize<'de> for EmptyJsonProxy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Some(map) = &value.as_object() {
            if map.is_empty() {
                return Ok(EmptyJsonProxy);
            }
        }
        Err(Error::custom(format!(
            "result must be empty object / map: '{value}'"
        )))
    }
}
impl From<EmptyJsonProxy> for () {
    fn from(_: EmptyJsonProxy) -> Self {}
}

pub(crate) fn query_to_urlencoded<K: serde::Serialize, V: serde::Serialize>(
    query: Vec<(K, V)>,
) -> Result<Vec<(String, String)>> {
    let mut q = vec![];

    for (k, v) in query.into_iter() {
        let key = serde_json::to_value(k)?;
        let value = serde_json::to_value(v)?;

        let key_as_string = match key {
            Value::Bool(bool) => bool.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => string,
            Value::Null => continue,
            _ => {
                return Err(CrunchyrollError::Internal(
                    CrunchyrollErrorContext::new("value is not supported to urlencode")
                        .with_value(key.to_string().as_bytes()),
                ))
            }
        };
        let value_as_string = match value {
            Value::Bool(bool) => bool.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => string,
            Value::Array(arr) => arr
                .into_iter()
                .map(|vv| match vv {
                    Value::Number(number) => Ok(number.to_string()),
                    Value::String(string) => Ok(string),
                    _ => {
                        return Err(CrunchyrollError::Internal(
                            CrunchyrollErrorContext::new("value is not supported to urlencode")
                                .with_value(vv.to_string().as_bytes()),
                        ))
                    }
                })
                .collect::<Result<Vec<String>>>()?
                .join(","),
            Value::Null => continue,
            _ => {
                return Err(CrunchyrollError::Internal(
                    CrunchyrollErrorContext::new("value is not supported to urlencode")
                        .with_value(value.to_string().as_bytes()),
                ))
            }
        };
        q.push((key_as_string, value_as_string));
    }

    Ok(q)
}

pub(crate) fn deserialize_millis_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::milliseconds(i64::deserialize(deserializer)?))
}

pub(crate) fn deserialize_try_from_string<'de, D, T: FromStr>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    T::from_str(value.as_str()).map_err(|_| Error::custom("could not convert string to T"))
}

/// Some response values are `null` for whatever reason even though they shouldn't be.
/// This is a fix to these events. If this occurs more often, a custom `Deserialize` implementation
/// must be written which automatically detects if a value is `null` even it they shouldn't be
/// and replace it with the [`Default`] implementation of the corresponding type.
pub(crate) fn deserialize_maybe_null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + DeserializeOwned,
{
    let value: Option<T> = Deserialize::deserialize(deserializer)?;
    Ok(value.unwrap_or_default())
}

/// Some responses are empty objects but actually must be array.
pub(crate) fn deserialize_maybe_object_to_array<'de, D, T>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Default + DeserializeOwned,
{
    let value: Value = Deserialize::deserialize(deserializer)?;

    if value.is_object() {
        Ok(vec![])
    } else {
        serde_json::from_value(value).map_err(|e| D::Error::custom(e.to_string()))
    }
}

/// Deserializes a empty string (`""`) to `None`.
pub(crate) fn deserialize_empty_pre_string_to_none<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: From<String>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(T::from(value)))
    }
}

pub(crate) fn deserialize_thumbnail_image<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Image>, D::Error> {
    let as_map = serde_json::Map::deserialize(deserializer)?;

    if let Some(thumbnail) = as_map.get("thumbnail") {
        Ok(serde_json::from_value::<Vec<Vec<Image>>>(thumbnail.clone())
            .map_err(|e| Error::custom(e.to_string()))?
            .into_iter()
            .flatten()
            .collect())
    } else {
        Ok(vec![])
    }
}

pub(crate) fn deserialize_streams_link<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let as_string = String::deserialize(deserializer)?;

    Ok(as_string
        .trim_end_matches("/streams")
        .split('/')
        .last()
        .ok_or_else(|| Error::custom("cannot extract stream id"))?
        .to_string())
}
