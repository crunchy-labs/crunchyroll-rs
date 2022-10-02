use crate::error::{CrunchyrollError, CrunchyrollErrorContext};
use crate::{Locale, Request, Result};
use chrono::Duration;
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
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
            "result must be empty object / map: '{}'",
            value
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

/// [`Vec`] representation of [`deserialize_maybe_broken_locale`].
pub(crate) fn deserialize_maybe_broken_locale_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<Locale>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|v| deserialize_maybe_broken_locale(v.into_deserializer()))
        .collect()
}

/// Some locales are not delivered in the appropriate ISO format (as they should) but in some crappy
/// version of it. The correct format would be, for example `ja-JP` (for japanese) but some requests
/// send it as `jaJP`.
pub(crate) fn deserialize_maybe_broken_locale<'de, D>(deserializer: D) -> Result<Locale, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    for locale in vec![
        Locale::ar_ME,
        Locale::ar_SA,
        Locale::de_DE,
        Locale::en_US,
        Locale::es_419,
        Locale::es_ES,
        Locale::es_LA,
        Locale::fr_FR,
        Locale::it_IT,
        Locale::ja_JP,
        Locale::pt_BR,
        Locale::ru_RU,
    ] {
        if locale.to_string().replace('-', "") == value {
            return Ok(locale);
        }
    }

    Err(D::Error::custom(format!("not a valid locale: '{}'", value)))
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
        if let Ok(stream_id) = deserialize_stream_id(value.into_deserializer()) {
            return Ok(Some(stream_id));
        }
    }
    Ok(None)
}
