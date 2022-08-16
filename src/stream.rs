use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer, Serialize};
use crate::{Crunchyroll, Executor, FromId, Locale};
use crate::common::Request;
use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};

/// Represents a video stream
#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "T: Clone + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Stream<T: Clone + DeserializeOwned> {
    #[serde(skip)]
    executor: Arc<Executor>,

    /// Audio locale of the stream.
    pub audio_locale: Locale,
    /// All subtitles
    pub subtitles: HashMap<Locale, StreamSubtitle>,

    /// All stream variants.
    /// One stream has multiple variants how it can be delivered. At the time of writing,
    /// all variants are either [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) streams.
    ///
    /// The data is stored in a map where the key represents the data's hardsub locale (-> subtitles
    /// are "burned" into the video) and the value all stream variants.
    /// If you want no hardsub at all, use the `Locale::Custom("".into())` map entry.
    #[serde(rename = "streams")]
    #[serde(deserialize_with = "deserialize_raw")]
    #[cfg_attr(not(feature = "__test_strict"), default(HashMap::new()))]
    pub variants: HashMap<Locale, T>,

    #[cfg(feature = "__test_strict")]
    media_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    captions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "QoS")]
    qos: crate::StrictValue
}

fn deserialize_raw<'de, D: Deserializer<'de>, T: DeserializeOwned>(deserializer: D) -> Result<T, D::Error> {
    let as_map: HashMap<String, HashMap<Locale, StreamVariant>> = HashMap::deserialize(deserializer)?;

    let mut raw: HashMap<Locale, HashMap<String, StreamVariant>> = HashMap::new();
    for (key, value) in as_map {
        for (mut locale, data) in value {
            if locale == Locale::Custom(":".to_string()) {
                locale = Locale::Custom("".to_string());
            }
            if let Some(entry) = raw.get_mut(&locale) {
                entry.insert(key.clone(), data);
            } else {
                raw.insert(locale, HashMap::from([(key.clone(), data)]));
            }
        }
    }

    let as_value = serde_json::to_value(raw).map_err(|e| Error::custom(e.to_string()))?;
    serde_json::from_value(as_value).map_err(|e| Error::custom(e.to_string()))
}

impl<T: Clone + DeserializeOwned> Request for Stream<T> {
    fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        for value in self.subtitles.values_mut() {
            value.executor = executor.clone();
        }
    }
}

#[async_trait::async_trait]
impl<T: Clone + DeserializeOwned> FromId for Stream<T> {
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Stream<T>> {
        let executor = crunchy.executor.clone();

        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams", executor.config.bucket, id);
        let builder = executor.client
            .get(endpoint)
            .query(&executor.media_query());

        executor.request(builder).await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct StreamSubtitle {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub locale: Locale,
    pub url: String,
    pub format: String
}

impl StreamSubtitle {
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let resp = self.executor.client
            .get(self.url)
            .send()
            .await
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext { message: e.to_string() }
            ))?;
        let body = resp.bytes()
            .await
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext { message: e.to_string() }
            ))?;
        w.write_all(body.as_ref())
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext { message: e.to_string() }
            ))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct StreamVariant {
    /// Language of this variant.
    pub hardsub_locale: Locale,
    /// Url to the actual stream.
    /// Usually a [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) stream.
    pub url: String,

    #[cfg(feature = "__test_strict")]
    vcodec: crate::StrictValue
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct VideoVariants {
    pub adaptive_dash: Option<StreamVariant>,
    pub adaptive_hls: Option<StreamVariant>,
    pub download_dash: Option<StreamVariant>,
    pub download_hls: Option<StreamVariant>,
    pub drm_adaptive_dash: Option<StreamVariant>,
    pub drm_adaptive_hls: Option<StreamVariant>,
    pub drm_download_dash: Option<StreamVariant>,
    pub drm_download_hls: Option<StreamVariant>,
    pub drm_multitrack_adaptive_hls_v2: Option<StreamVariant>,
    pub multitrack_adaptive_hls_v2: Option<StreamVariant>,
    pub vo_adaptive_dash: Option<StreamVariant>,
    pub vo_adaptive_hls: Option<StreamVariant>,
    pub vo_drm_adaptive_dash: Option<StreamVariant>,
    pub vo_drm_adaptive_hls: Option<StreamVariant>,

    #[cfg(feature = "__test_strict")]
    urls: crate::StrictValue
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct PlaybackVariants {
    pub adaptive_dash: Option<StreamVariant>,
    pub adaptive_hls: Option<StreamVariant>,
    pub download_hls: Option<StreamVariant>,
    pub drm_adaptive_dash: Option<StreamVariant>,
    pub drm_adaptive_hls: Option<StreamVariant>,
    pub drm_download_hls: Option<StreamVariant>,
    pub trailer_dash: Option<StreamVariant>,
    pub trailer_hls: Option<StreamVariant>,
    pub vo_adaptive_dash: Option<StreamVariant>,
    pub vo_adaptive_hls: Option<StreamVariant>,
    pub vo_drm_adaptive_dash: Option<StreamVariant>,
    pub vo_drm_adaptive_hls: Option<StreamVariant>
}

#[cfg(feature = "streaming")]
mod streaming {
    use std::borrow::BorrowMut;
    use std::io::Write;
    use std::sync::Arc;
    use std::time::Duration;
    use aes::cipher::{BlockDecryptMut, KeyIvInit};
    use serde::de::DeserializeOwned;
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
    use crate::{Executor, Locale, PlaybackVariants, VideoVariants, Stream};
    use crate::stream::StreamVariant;

    pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    #[doc(hidden)]
    pub trait PreferredStream {
        fn preferred_stream(&self) -> StreamVariant;
    }

    impl PreferredStream for VideoVariants {
        fn preferred_stream(&self) -> StreamVariant {
            self.adaptive_hls.clone().unwrap()
        }
    }

    impl PreferredStream for PlaybackVariants {
        fn preferred_stream(&self) -> StreamVariant {
            self.adaptive_hls.clone().unwrap()
        }
    }

    impl<T: Clone + DeserializeOwned + PreferredStream> Stream<T> {
        pub async fn default_streams(&self) -> Result<Vec<VariantData>> {
            if let Some(raw_streams) = self.variants.get(&Locale::Custom("".into())) {
                VariantData::from_hls_master(self.executor.clone(), raw_streams.preferred_stream().url).await
            } else if let Some(raw_streams) = self.variants.get(&Locale::Custom(":".into())) {
                VariantData::from_hls_master(self.executor.clone(), raw_streams.preferred_stream().url).await
            } else {
                Err(CrunchyrollError::InternalError(
                    CrunchyrollErrorContext{ message: "could not find default stream".into() }
                ))
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Resolution {
        pub width: u64,
        pub height: u64
    }

    impl From<m3u8_rs::Resolution> for Resolution {
        fn from(resolution: m3u8_rs::Resolution) -> Self {
            Self{
                height: resolution.height,
                width: resolution.width
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct VariantData {
        executor: Arc<Executor>,

        pub resolution: Resolution,
        pub bandwidth: u64,
        pub fps: f64,
        pub codecs: String,

        url: String,
        key: Option<Aes128CbcDec>,
        segments: Option<Vec<VariantSegment>>
    }

    impl VariantData {
        pub(crate) async fn from_hls_master(executor: Arc<Executor>, url: String) -> Result<Vec<VariantData>> {
            let resp = executor
                .client
                .get(url)
                .send()
                .await
                .map_err(|e| CrunchyrollError::RequestError(
                    CrunchyrollErrorContext{ message: e.to_string() }
                ))?;
            let raw_master_playlist = resp.text()
                .await
                .map_err(|e| CrunchyrollError::RequestError(
                    CrunchyrollErrorContext{ message: e.to_string() }
                ))?;

            let master_playlist = m3u8_rs::parse_master_playlist_res(raw_master_playlist.as_bytes())
                .map_err(|e| CrunchyrollError::DecodeError(
                    CrunchyrollErrorContext{ message: e.to_string() }
                ))?;

            let mut stream_data: Vec<VariantData> = vec![];

            for variant in master_playlist.variants {
                #[cfg(feature = "__test_strict")]
                stream_data.push(VariantData {
                    executor: executor.clone(),

                    resolution: variant.resolution.unwrap().into(),
                    bandwidth: variant.bandwidth,
                    fps: variant.frame_rate.unwrap(),
                    codecs: variant.codecs.unwrap(),

                    url: variant.uri,
                    key: None,
                    segments: None
                });

                #[cfg(not(feature = "__test_strict"))]
                stream_data.push(VariantData {
                    executor: executor.clone(),

                    resolution: variant.resolution.unwrap_or(m3u8_rs::Resolution{ height: 0, width: 0 }).into(),
                    bandwidth: variant.bandwidth,
                    fps: variant.frame_rate.unwrap_or(0 as f64),
                    codecs: variant.codecs.unwrap_or("".into()),

                    url: variant.uri,
                    key: None,
                    segments: None
                });
            }

            Ok(stream_data)
        }

        /// Return all segments in order the variant stream is made of.
        pub async fn segments(&mut self) -> Result<Vec<VariantSegment>> {
            if let Some(segments) = &self.segments {
                Ok(segments.clone())
            } else {
                let resp = self.executor.client
                    .get(self.url.clone())
                    .send()
                    .await
                    .map_err(|e| CrunchyrollError::RequestError(
                        CrunchyrollErrorContext{ message: e.to_string() }
                    ))?;
                let raw_media_playlist = resp.text()
                    .await
                    .map_err(|e| CrunchyrollError::RequestError(
                        CrunchyrollErrorContext{ message: e.to_string() }
                    ))?;
                let media_playlist = m3u8_rs::parse_media_playlist_res(raw_media_playlist.as_bytes())
                    .map_err(|e| CrunchyrollError::DecodeError(
                        CrunchyrollErrorContext{ message: e.to_string() }
                    ))?;

                let mut segments: Vec<VariantSegment> = vec![];
                for segment in media_playlist.segments {
                    if let Some(key) = segment.key {
                        if let Some(url) = key.uri {
                            let resp = self.executor.client
                                .get(url)
                                .send()
                                .await
                                .map_err(|e| CrunchyrollError::DecodeError(
                                    CrunchyrollErrorContext{ message: e.to_string() }
                                ))?;
                            let raw_key = resp.bytes()
                                .await
                                .map_err(|e| CrunchyrollError::RequestError(
                                    CrunchyrollErrorContext{ message: e.to_string() }
                                ))?;

                            let temp_iv = key.iv.unwrap_or("".to_string());
                            let iv = if !temp_iv.is_empty() {
                                temp_iv.as_bytes()
                            } else {
                                raw_key.as_ref()
                            };

                            self.key = Some(
                                Aes128CbcDec::new(raw_key.as_ref().into(), iv.into())
                            );
                        }
                    }

                    segments.push(
                        VariantSegment {
                            executor: self.executor.clone(),
                            key: self.key.clone(),
                            url: segment.uri,
                            length: Duration::from_secs_f32(segment.duration)
                        }
                    )
                }

                self.segments = Some(segments.clone());
                Ok(segments)
            }
        }
    }

    /// Segment [`VariantStream`] data is made of.
    /// Because Crunchyroll uses segment / chunk based video streaming (usually
    /// [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) or
    /// [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP)) the actual
    /// video stream consists of multiple [`VariantSegment`]s.
    #[derive(Clone, Debug)]
    pub struct VariantSegment {
        executor: Arc<Executor>,

        /// Decryption key to decrypt the segment data (if encrypted).
        pub key: Option<Aes128CbcDec>,
        /// Url to the actual data.
        pub url: String,
        /// Video length of this segment.
        pub length: Duration
    }

    impl VariantSegment {
        pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
            let resp = self.executor.client
                .get(self.url)
                .send()
                .await
                .map_err(|e| CrunchyrollError::RequestError(
                    CrunchyrollErrorContext { message: e.to_string() }
                ))?;
            let segment = resp.bytes()
                .await
                .map_err(|e| CrunchyrollError::RequestError(
                    CrunchyrollErrorContext { message: e.to_string() }
                ))?;

            if let Some(key) = self.key {
                let mut temp_encrypted = segment.to_vec();
                let decrypted = key.decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(temp_encrypted.borrow_mut())
                    .map_err(|e| CrunchyrollError::DecodeError(
                        CrunchyrollErrorContext{ message: e.to_string() }
                    ))?;
                w.write(decrypted)
                    .map_err(|e| CrunchyrollError::RequestError(
                        CrunchyrollErrorContext { message: e.to_string() }
                    ))?;
            } else {
                w.write(segment.as_ref())
                    .map_err(|e| CrunchyrollError::RequestError(
                        CrunchyrollErrorContext { message: e.to_string() }
                    ))?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "streaming")]
pub use streaming::*;
