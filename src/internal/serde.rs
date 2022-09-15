use std::str::FromStr;
use crate::Request;
use chrono::Duration;
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Request)]
pub(crate) struct EmptyJsonProxy;
impl<'de> Deserialize<'de> for EmptyJsonProxy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(map) = serde_json::Map::deserialize(deserializer) {
            if map.is_empty() {
                return Ok(EmptyJsonProxy);
            }
        }
        Err(Error::custom("result must be empty object / map"))
    }
}
impl From<EmptyJsonProxy> for () {
    fn from(_: EmptyJsonProxy) -> Self {}
}

pub(crate) fn deserialize_millis_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::milliseconds(i64::deserialize(deserializer)?))
}

pub(crate) fn deserialize_try_from_string<'de, D, T: FromStr>(deserializer: D) -> Result<T, D::Error> where D: Deserializer<'de> {
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

/// Sometimes response values are `"none"` but should actually be `null`. This function implements
/// this functionality.
pub(crate) fn deserialize_maybe_none_to_option<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Deserialize::deserialize(deserializer)?;
    if let Some(maybe_none) = &value {
        if maybe_none == "none" || maybe_none.is_empty() {
            Ok(None)
        } else {
            Ok(value)
        }
    } else {
        Ok(None)
    }
}

/// Deserializes a empty string (`""`) to `None`.
pub(crate) fn deserialize_empty_pre_string_to_none<'de, D, T> (deserializer: D) -> Result<Option<T>, D::Error> where D: Deserializer<'de>, T: From<String> {
    let value: String = Deserialize::deserialize(deserializer)?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(T::from(value)))
    }
}

pub(crate) fn deserialize_stream_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    struct StreamHref {
        href: String,
    }
    #[derive(Deserialize)]
    struct Streams {
        streams: StreamHref,
    }

    let streams: Streams = Streams::deserialize(deserializer)?;

    let mut split_streams = streams
        .streams
        .href
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    split_streams.reverse();
    if let Some(stream_id) = split_streams.get(1) {
        Ok(stream_id.clone())
    } else {
        Err(Error::custom("cannot extract stream id"))
    }
}

pub(crate) fn deserialize_resource<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    struct ResourceHref {
        href: String,
    }
    #[derive(Deserialize)]
    struct Resource {
        resource: ResourceHref,
    }

    let resource: Resource = Resource::deserialize(deserializer)?;
    Ok(resource.resource.href)
}

pub(crate) fn deserialize_stream_id_option<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    if let Some(value) = Option::<Value>::deserialize(deserializer)? {
        Ok(Some(
            deserialize_stream_id(value.into_deserializer())
                .map_err(|e| Error::custom(e.to_string()))?,
        ))
    } else {
        Ok(None)
    }
}
