use crate::common::V2BulkResult;
use crate::error::CrunchyrollError;
use crate::{Executor, Locale, Request, Result};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;

fn deserialize_streams<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<Locale, Variants>, D::Error> {
    let as_map: HashMap<String, HashMap<Locale, Value>> = HashMap::deserialize(deserializer)?;

    let mut raw: HashMap<Locale, HashMap<String, Value>> = HashMap::new();
    for (key, value) in as_map {
        for (mut locale, data) in value {
            if locale == Locale::Custom(":".to_string()) {
                locale = Locale::Custom("".to_string());
            }

            // check only for errors and not use the `Ok(...)` result in `raw` because `Variant`
            // then must implement `serde::Serialize`
            if let Err(e) = Variant::deserialize(&data) {
                return Err(Error::custom(e.to_string()));
            }

            if let Some(entry) = raw.get_mut(&locale) {
                entry.insert(key.clone(), data.clone());
            } else {
                raw.insert(locale, HashMap::from([(key.clone(), data)]));
            }
        }
    }

    let as_value = serde_json::to_value(raw).map_err(|e| Error::custom(e.to_string()))?;
    serde_json::from_value(as_value).map_err(|e| Error::custom(e.to_string()))
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct StreamVersion {
    #[serde(rename = "guid")]
    pub(crate) id: String,
    #[serde(rename = "media_guid")]
    pub(crate) media_id: String,
    #[serde(rename = "season_guid")]
    pub(crate) season_id: String,

    pub(crate) audio_locale: Locale,

    pub(crate) is_premium_only: bool,
    pub(crate) original: bool,

    pub(crate) variant: String,
}

/// A video stream.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(subtitles))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Stream {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub media_id: String,
    /// Audio locale of the stream.
    pub audio_locale: Locale,
    /// All subtitles.
    pub subtitles: HashMap<Locale, Subtitle>,
    pub closed_captions: HashMap<Locale, Subtitle>,

    /// All stream variants.
    /// One stream has multiple variants how it can be delivered. At the time of writing,
    /// all variants are either [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) streams.
    ///
    /// The data is stored in a map where the key represents the data's hardsub locale (-> subtitles
    /// are "burned" into the video) and the value all stream variants.
    /// If you want no hardsub at all, use the `Locale::Custom("".into())` map entry.
    #[serde(deserialize_with = "deserialize_streams")]
    #[cfg_attr(not(feature = "__test_strict"), default(HashMap::new()))]
    pub variants: HashMap<Locale, Variants>,

    /// Might be null, for music videos and concerts mostly.
    versions: Option<Vec<StreamVersion>>,
    /// When requesting versions from [`Stream::versions`] this url is required as multiple paths
    /// exists which can lead to the [`Stream`] struct.
    #[serde(skip)]
    pub(crate) version_request_url: String,

    #[cfg(feature = "__test_strict")]
    captions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
}

impl Stream {
    pub(crate) async fn from_url<S: AsRef<str>>(
        executor: Arc<Executor>,
        base: S,
        id: S,
    ) -> Result<Stream> {
        let endpoint = format!("{}/{}/streams", base.as_ref(), id.as_ref());
        let mut data = executor
            .get(endpoint)
            .apply_preferred_audio_locale_query()
            .apply_locale_query()
            .request::<V2BulkResult<serde_json::Map<String, Value>>>()
            .await?;

        let mut map = data.meta.clone();
        map.insert("variants".to_string(), data.data.remove(0).into());

        let mut stream: Stream = serde_json::from_value(serde_json::to_value(map)?)?;
        stream.executor = executor;
        stream.version_request_url = base.as_ref().to_string();

        Ok(stream)
    }

    pub fn available_versions(&self) -> Vec<Locale> {
        self.versions
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|v| v.audio_locale.clone())
            .collect()
    }

    pub async fn version(&self, audio_locales: Vec<Locale>) -> Result<Vec<Stream>> {
        let version_ids = self
            .versions
            .clone()
            .unwrap_or_default()
            .iter()
            .filter_map(|v| {
                if audio_locales.contains(&v.audio_locale) {
                    Some(v.media_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        let mut result = vec![];
        for id in version_ids {
            result.push(
                Stream::from_url(self.executor.clone(), &self.version_request_url, &id).await?,
            );
        }
        Ok(result)
    }

    pub async fn versions(&self) -> Result<Vec<Stream>> {
        let version_ids = self
            .versions
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|v| v.media_id.clone())
            .collect::<Vec<String>>();

        let mut result = vec![];
        for id in version_ids {
            result.push(
                Stream::from_url(self.executor.clone(), &self.version_request_url, &id).await?,
            )
        }
        Ok(result)
    }
}

/// Subtitle for streams.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Subtitle {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub locale: Locale,
    pub url: String,
    pub format: String,
}

impl Subtitle {
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let resp = self.executor.get(self.url).request_raw().await?;
        w.write_all(resp.as_ref())
            .map_err(|e| CrunchyrollError::Input(e.to_string().into()))?;
        Ok(())
    }
}

/// A [`VideoStream`] variant.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Variant {
    /// Language of this variant.
    pub hardsub_locale: Locale,
    /// Url to the actual stream.
    /// Usually a [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) stream.
    pub url: String,
}

/// Stream variants for a [`VideoStream`].
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Variants {
    pub adaptive_dash: Option<Variant>,
    pub adaptive_hls: Option<Variant>,
    pub download_dash: Option<Variant>,
    pub download_hls: Option<Variant>,
    pub drm_adaptive_dash: Option<Variant>,
    pub drm_adaptive_hls: Option<Variant>,
    pub drm_download_dash: Option<Variant>,
    pub drm_download_hls: Option<Variant>,
    pub drm_multitrack_adaptive_hls_v2: Option<Variant>,
    pub multitrack_adaptive_hls_v2: Option<Variant>,
    pub vo_adaptive_dash: Option<Variant>,
    pub vo_adaptive_hls: Option<Variant>,
    pub vo_drm_adaptive_dash: Option<Variant>,
    pub vo_drm_adaptive_hls: Option<Variant>,

    #[cfg(feature = "__test_strict")]
    urls: Option<crate::StrictValue>,
}
