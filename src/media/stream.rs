use crate::error::CrunchyrollError;
use crate::{Executor, Locale, Request, Result};
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;

trait FixStream: DeserializeOwned {
    type Variant: DeserializeOwned;
}

fn deserialize_streams<'de, D: Deserializer<'de>, T: FixStream>(
    deserializer: D,
) -> Result<HashMap<Locale, T>, D::Error> {
    let as_map: HashMap<String, HashMap<Locale, Value>> = HashMap::deserialize(deserializer)?;

    let mut raw: HashMap<Locale, HashMap<String, Value>> = HashMap::new();
    for (key, value) in as_map {
        for (mut locale, data) in value {
            if locale == Locale::Custom(":".to_string()) {
                locale = Locale::Custom("".to_string());
            }

            // check only for errors and not use the `Ok(...)` result in `raw` because `T::Variant`
            // then must implement `serde::Serialize`
            if let Err(e) = T::Variant::deserialize(&data) {
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

/// A video stream.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(subtitles))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct VideoStream {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub media_id: String,
    /// Audio locale of the stream.
    pub audio_locale: Locale,
    /// All subtitles.
    pub subtitles: HashMap<Locale, StreamSubtitle>,
    pub closed_captions: HashMap<Locale, StreamSubtitle>,

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
    pub variants: HashMap<Locale, VideoVariants>,

    #[cfg(feature = "__test_strict")]
    captions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
}

/// Subtitle for streams.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct StreamSubtitle {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub locale: Locale,
    pub url: String,
    pub format: String,
}

impl StreamSubtitle {
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
pub struct VideoVariant {
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
pub struct VideoVariants {
    pub adaptive_dash: Option<VideoVariant>,
    pub adaptive_hls: Option<VideoVariant>,
    pub download_dash: Option<VideoVariant>,
    pub download_hls: Option<VideoVariant>,
    pub drm_adaptive_dash: Option<VideoVariant>,
    pub drm_adaptive_hls: Option<VideoVariant>,
    pub drm_download_dash: Option<VideoVariant>,
    pub drm_download_hls: Option<VideoVariant>,
    pub drm_multitrack_adaptive_hls_v2: Option<VideoVariant>,
    pub multitrack_adaptive_hls_v2: Option<VideoVariant>,
    pub vo_adaptive_dash: Option<VideoVariant>,
    pub vo_adaptive_hls: Option<VideoVariant>,
    pub vo_drm_adaptive_dash: Option<VideoVariant>,
    pub vo_drm_adaptive_hls: Option<VideoVariant>,

    #[cfg(feature = "__test_strict")]
    urls: Option<crate::StrictValue>,
}

impl FixStream for VideoVariants {
    type Variant = VideoVariant;
}
