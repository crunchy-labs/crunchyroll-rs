use chrono::Duration;
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub(crate) fn deserialize_millis_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::milliseconds(i64::deserialize(deserializer)?))
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
