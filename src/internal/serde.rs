use crate::{Locale, Request};
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

pub(crate) fn deserialize_millis_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::milliseconds(i64::deserialize(deserializer)?))
}

/// Some locales are not delivered in the appropriate ISO format (as they should) but in some crappy
/// version of it. The correct format would be, for example `ja-JP` (for japanese) but some requests
/// send it as `jaJP`. This currently only occurs in [`Vec`] results which contains
/// [`crate::Locale`].
pub(crate) fn deserialize_maybe_broken_locale_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<Locale>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut locales = vec![];

    let values: Vec<String> = Vec::deserialize(deserializer)?;
    for value in values {
        match Locale::from(value.clone()) {
            Locale::Custom(_) => (),
            _ => {
                locales.push(Locale::from(value));
                continue;
            }
        };

        for locale in vec![
            Locale::ar_ME,
            Locale::ar_SA,
            Locale::de_DE,
            Locale::es_419,
            Locale::es_ES,
            Locale::fr_FR,
            Locale::it_IT,
            Locale::ja_JP,
            Locale::pt_BR,
            Locale::ru_RU,
        ] {
            if locale.to_string().replace('-', "") == value {
                locales.push(locale);
                continue;
            }
        }
        locales.push(Locale::Custom(value));
    }
    Ok(locales)
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
