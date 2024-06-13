use crate::error::{is_request_error, Error};
use crate::{Crunchyroll, Executor, Locale, Request, Result};
use dash_mpd::MPD;
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::sync::Arc;
use std::time::Duration;

fn deserialize_hardsubs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<Locale, String>, D::Error> {
    #[derive(Deserialize)]
    struct HardSub {
        url: String,
    }

    Ok(HashMap::<String, HardSub>::deserialize(deserializer)?
        .into_iter()
        .map(|(l, hs)| (Locale::from(l), hs.url))
        .collect())
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct StreamVersion {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,
    #[serde(skip)]
    device: String,
    #[serde(skip)]
    platform: String,
    #[serde(skip)]
    optional_media_type: Option<String>,

    #[serde(rename = "guid")]
    pub id: String,
    #[serde(rename = "media_guid")]
    pub media_id: String,
    #[serde(rename = "season_guid")]
    pub season_id: String,

    pub audio_locale: Locale,

    pub is_premium_only: bool,
    pub original: bool,

    #[cfg(feature = "__test_strict")]
    variant: crate::StrictValue,
}

impl StreamVersion {
    /// Requests an actual [`Stream`] from this version.
    /// This method might throw a too many active streams error. In this case, make sure to
    /// have less/no active other [`Stream`]s open (through this crate or as stream in the browser
    /// or app).
    pub async fn stream(&self) -> Result<Stream> {
        Stream::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            &self.id,
            &self.device,
            &self.platform,
            self.optional_media_type.clone(),
        )
        .await
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSession {
    pub renew_seconds: u32,
    pub no_network_retry_interval_seconds: u32,
    pub no_network_timeout_seconds: u32,
    pub maximum_pause_seconds: u32,
    pub end_of_video_unload_seconds: u32,
    pub session_expiration_seconds: u32,
    pub uses_stream_limits: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[request(executor(versions))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Stream {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub url: String,
    pub audio_locale: Locale,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub burned_in_locale: Option<Locale>,

    #[serde(deserialize_with = "deserialize_hardsubs")]
    pub hard_subs: HashMap<Locale, String>,

    /// All subtitles.
    pub subtitles: HashMap<Locale, Subtitle>,
    pub captions: HashMap<Locale, Subtitle>,

    pub token: String,
    /// If [`StreamSession::uses_stream_limits`] is `true`, this means that the stream data will be
    /// DRM encrypted, if `false` it isn't.
    pub session: StreamSession,

    /// All versions of this stream (same stream but each entry has a different language).
    pub versions: Vec<StreamVersion>,

    #[serde(skip)]
    id: String,
    #[serde(skip)]
    optional_media_type: Option<String>,

    #[cfg(feature = "__test_strict")]
    asset_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    playback_type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
}

impl Stream {
    /// Requests a stream from an id with is always DRM encrypted.
    pub async fn from_id_drm(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str>,
        optional_media_type: Option<String>,
    ) -> Result<Self> {
        Self::from_id(crunchyroll, id, "web", "chrome", optional_media_type).await
    }

    /// Requests a stream from an id with is maybe DRM free. Check
    /// [`Stream::session::uses_stream_limits`], if its `true`, the stream is DRM encrypted, if
    /// `false` not.
    pub async fn from_id_maybe_without_drm(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str>,
        optional_media_type: Option<String>,
    ) -> Result<Self> {
        Self::from_id(crunchyroll, id, "console", "switch", optional_media_type).await
    }

    async fn from_id(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str>,
        device: &str,
        platform: &str,
        optional_media_type: Option<String>,
    ) -> Result<Self> {
        let endpoint = format!(
            "https://cr-play-service.prd.crunchyrollsvc.com/v1/{}{}/{device}/{platform}/play",
            optional_media_type
                .as_ref()
                .map(|omt| format!("{omt}/"))
                .unwrap_or_default(),
            id.as_ref()
        );

        let mut stream = crunchyroll
            .executor
            .get(endpoint)
            .request::<Stream>()
            .await?;
        stream.__set_executor(crunchyroll.executor.clone()).await;
        stream.id = id.as_ref().to_string();
        stream.optional_media_type = optional_media_type;

        for version in &mut stream.versions {
            version.device = device.to_string();
            version.platform = platform.to_string();
            version
                .optional_media_type
                .clone_from(&stream.optional_media_type)
        }

        Ok(stream)
    }

    /// Requests all available video and audio streams. Returns [`None`] if the requested hardsub
    /// isn't available. The first [`Vec<StreamData>`] contains only video streams, the second only
    /// audio streams.
    /// You will run into an error when requesting this function too often without invalidating the
    /// data. Crunchyroll only allows a certain amount of stream data to be requested at the same
    /// time, typically the exact amount depends on the type of (premium) subscription you have. You
    /// can use [`Stream::invalidate`] to invalidate all stream data for this stream.
    pub async fn stream_data(
        &self,
        hardsub: Option<Locale>,
    ) -> Result<Option<(Vec<StreamData>, Vec<StreamData>)>> {
        if let Some(hardsub) = hardsub {
            let Some(url) = self
                .hard_subs
                .iter()
                .find_map(|(locale, url)| (locale == &hardsub).then_some(url))
            else {
                return Ok(None);
            };
            Ok(Some(
                StreamData::from_url(self.executor.clone(), url, &self.token, &self.id).await?,
            ))
        } else {
            Ok(Some(
                StreamData::from_url(self.executor.clone(), &self.url, &self.token, &self.id)
                    .await?,
            ))
        }
    }

    /// Invalidates all the stream data which may be obtained from [`Stream::stream_data`]. You will
    /// run into errors if you request multiple [`Stream::stream_data`]s without invalidating them.
    pub async fn invalidate(self) -> Result<()> {
        if !self.session.uses_stream_limits {
            return Ok(());
        }

        let endpoint = format!(
            "https://cr-play-service.prd.crunchyrollsvc.com/v1/token/{}/{}",
            self.id, self.token
        );

        self.executor.delete(endpoint).request_raw(true).await?;

        Ok(())
    }

    /// Show in which audios this [`Stream`] is also available.
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub fn available_versions(&self) -> Vec<Locale> {
        self.versions
            .iter()
            .map(|v| v.audio_locale.clone())
            .collect()
    }

    /// Get the versions of this [`Stream`] which have the specified audio locale(s). Use
    /// [`Stream::available_versions`] to see all supported locale.
    /// This method might throw a too many active streams error. In this case, make sure to
    /// have less/no active other [`Stream`]s open (through this crate or as stream in the browser
    /// or app).
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn version(&self, audio_locales: Vec<Locale>) -> Result<Vec<Stream>> {
        let mut result = vec![];
        for version in &self.versions {
            if audio_locales.contains(&version.audio_locale) {
                result.push(version.stream().await?)
            }
        }
        Ok(result)
    }

    /// Get all available other versions (same [`Stream`] but different audio locale) for this
    /// [`Stream`].
    /// This method might throw a too many active streams error. In this case, either make sure to
    /// have less/no active other [`Stream`]s open (through this crate or as stream in the browser
    /// or app), or try to use [`Stream::version`] to get only a specific version (requesting too
    /// many [`Stream`]s at once will always result in said error).
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn versions(&self) -> Result<Vec<Stream>> {
        let mut result = vec![];
        for version in &self.versions {
            result.push(version.stream().await?)
        }
        Ok(result)
    }
}

/// Subtitle for streams.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Subtitle {
    #[serde(skip)]
    executor: Arc<Executor>,

    #[serde(rename = "language")]
    pub locale: Locale,
    pub url: String,
    /// Subtitle format. `ass` or `vtt` at the time of writing.
    pub format: String,
}

impl Subtitle {
    /// Get the subtitle as bytes.
    pub async fn data(&self) -> Result<Vec<u8>> {
        self.executor.get(&self.url).request_raw(false).await
    }
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct StreamData {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub bandwidth: u64,
    pub codecs: String,

    pub info: StreamDataInfo,
    /// If [`Some`], the stream data is DRM encrypted and the struct contains all data needed for
    /// you to decrypted it. If [`None`], the stream data is not DRM encrypted.
    pub drm: Option<StreamDataDRM>,

    pub watch_id: String,

    #[serde(skip_serializing)]
    representation_id: String,
    #[serde(skip_serializing)]
    segment_start: u32,
    #[serde(skip_serializing)]
    segment_lengths: Vec<u32>,
    #[serde(skip_serializing)]
    segment_base_url: String,
    #[serde(skip_serializing)]
    segment_init_url: String,
    #[serde(skip_serializing)]
    segment_media_url: String,
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct StreamDataDRM {
    pub pssh: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Request)]
pub enum StreamDataInfo {
    Audio { sampling_rate: u32 },
    Video { resolution: Resolution, fps: f64 },
}

impl StreamData {
    async fn from_url(
        executor: Arc<Executor>,
        url: impl AsRef<str>,
        token: impl AsRef<str>,
        watch_id: impl AsRef<str>,
    ) -> Result<(Vec<StreamData>, Vec<StreamData>)> {
        let mut video = vec![];
        let mut audio = vec![];

        let err_fn = |msg: &str| Error::Request {
            message: msg.to_string(),
            status: None,
            url: url.as_ref().to_string(),
        };

        let raw_mpd = executor
            .get(url.as_ref())
            .query(&[
                (
                    "accountid",
                    executor
                        .details
                        .account_id
                        .clone()
                        .unwrap_or_default()
                        .as_str(),
                ),
                ("playbackGuid", token.as_ref()),
            ])
            .request_raw(true)
            .await?;
        // if the response is json and not xml it should always be an error
        if let Ok(json) = serde_json::from_slice(&raw_mpd) {
            is_request_error(json, url.as_ref(), &StatusCode::FORBIDDEN)?;
        }
        let mut mpd: MPD =
            dash_mpd::parse(&String::from_utf8_lossy(&raw_mpd)).map_err(|e| Error::Decode {
                message: e.to_string(),
                content: raw_mpd,
                url: url.as_ref().to_string(),
            })?;
        let period = mpd.periods.remove(0);

        for adaption in period.adaptations {
            // skip subtitles that are embedded in the mpd manifest for now
            if adaption.contentType.is_some_and(|ct| ct == "text") {
                continue;
            }

            let segment_template = adaption
                .SegmentTemplate
                .ok_or("no segment template found")
                .map_err(err_fn)?;
            let segment_lengths = segment_template
                .SegmentTimeline
                .as_ref()
                .ok_or("no segment timeline found")
                .map_err(err_fn)?
                .segments
                .iter()
                .flat_map(|s| {
                    iter::repeat(s.d as u32)
                        .take(s.r.unwrap_or_default() as usize + 1)
                        .collect::<Vec<u32>>()
                })
                .collect::<Vec<u32>>();
            let segment_init_url = segment_template
                .initialization
                .ok_or("no init url found")
                .map_err(err_fn)?;
            let segment_media_url = segment_template
                .media
                .ok_or("no media url found")
                .map_err(err_fn)?;
            let pssh = adaption.ContentProtection.into_iter().find_map(|cp| {
                cp.cenc_pssh
                    .first()
                    .map(|pssh| pssh.clone().content.expect("pssh"))
            });

            if adaption.maxWidth.is_some() || adaption.maxHeight.is_some() {
                for representation in adaption.representations {
                    let (Some(width), Some(height)) = (representation.width, representation.height)
                    else {
                        return Err(err_fn("invalid resolution"));
                    };
                    let resolution = Resolution { width, height };

                    let frame_rate = representation
                        .frameRate
                        .ok_or("no fps found")
                        .map_err(err_fn)?;
                    let fps: f64 = if let Some((l, r)) = frame_rate.split_once('/') {
                        let left = l
                            .parse::<f64>()
                            .map_err(|_| err_fn(&format!("invalid (left) fps: {l}")))?;
                        let right = r
                            .parse::<f64>()
                            .map_err(|_| err_fn(&format!("invalid (right) fps: {l}")))?;
                        left / right
                    } else {
                        frame_rate
                            .parse()
                            .map_err(|_| err_fn(&format!("invalid fps: {frame_rate}")))?
                    };

                    video.push(Self {
                        executor: executor.clone(),
                        bandwidth: representation
                            .bandwidth
                            .ok_or("no bandwidth found")
                            .map_err(err_fn)?,
                        codecs: representation
                            .codecs
                            .ok_or("no codecs found")
                            .map_err(err_fn)?,
                        info: StreamDataInfo::Video { resolution, fps },
                        drm: pssh.as_ref().map(|pssh| StreamDataDRM {
                            pssh: pssh.clone(),
                            token: token.as_ref().to_string(),
                        }),
                        watch_id: watch_id.as_ref().to_string(),
                        representation_id: representation
                            .id
                            .ok_or("no representation id found")
                            .map_err(err_fn)?,
                        segment_start: segment_template
                            .startNumber
                            .ok_or("no start number found")
                            .map_err(err_fn)? as u32,
                        segment_lengths: segment_lengths.clone(),
                        segment_base_url: representation
                            .BaseURL
                            .first()
                            .ok_or("no base url found")
                            .map_err(err_fn)?
                            .base
                            .clone(),
                        segment_init_url: segment_init_url.clone(),
                        segment_media_url: segment_media_url.clone(),
                    })
                }
            } else {
                for representation in adaption.representations {
                    let sampling_rate = representation
                        .audioSamplingRate
                        .ok_or("no audio sampling rate found")
                        .map_err(err_fn)?
                        .parse::<u32>()
                        .map_err(|e| err_fn(&e.to_string()))?;

                    audio.push(Self {
                        executor: executor.clone(),
                        bandwidth: representation
                            .bandwidth
                            .ok_or("no bandwith found")
                            .map_err(err_fn)?,
                        codecs: representation
                            .codecs
                            .ok_or("no codecs found")
                            .map_err(err_fn)?,
                        info: StreamDataInfo::Audio { sampling_rate },
                        drm: pssh.as_ref().map(|pssh| StreamDataDRM {
                            pssh: pssh.clone(),
                            token: token.as_ref().to_string(),
                        }),
                        watch_id: watch_id.as_ref().to_string(),
                        representation_id: representation
                            .id
                            .ok_or("no representation id found")
                            .map_err(err_fn)?,
                        segment_start: segment_template
                            .startNumber
                            .ok_or("no start number found")
                            .map_err(err_fn)? as u32,
                        segment_lengths: segment_lengths.clone(),
                        segment_base_url: representation
                            .BaseURL
                            .first()
                            .ok_or("no base url found")
                            .map_err(err_fn)?
                            .base
                            .clone(),
                        segment_init_url: segment_init_url.clone(),
                        segment_media_url: segment_media_url.clone(),
                    })
                }
            }
        }

        Ok((video, audio))
    }

    /// Returns the streams' audio sampling rate. Only [`Some`] if the stream is an audio stream
    /// (check [`StreamData::info`]).
    pub fn sampling_rate(&self) -> Option<u32> {
        if let StreamDataInfo::Audio { sampling_rate } = &self.info {
            Some(*sampling_rate)
        } else {
            None
        }
    }

    /// Returns the streams' video resolution. Only [`Some`] if the stream is a video stream (check
    /// [`StreamData::info`]).
    pub fn resolution(&self) -> Option<Resolution> {
        if let StreamDataInfo::Video { resolution, .. } = &self.info {
            Some(resolution.clone())
        } else {
            None
        }
    }

    /// Returns the streams' video fps. Only [`Some`] if the stream is a video stream (check
    /// [`StreamData::info`]).
    pub fn fps(&self) -> Option<f64> {
        if let StreamDataInfo::Video { fps, .. } = &self.info {
            Some(*fps)
        } else {
            None
        }
    }

    /// Returns all segment this stream is made of.
    pub fn segments(&self) -> Vec<StreamSegment> {
        let mut segments = vec![StreamSegment {
            executor: self.executor.clone(),
            url: format!(
                "{}{}",
                self.segment_base_url,
                self.segment_init_url
                    .replace("$RepresentationID$", &self.representation_id)
            ),
            length: Duration::from_secs(0),
        }];

        for i in 0..self.segment_lengths.len() {
            segments.push(StreamSegment {
                executor: self.executor.clone(),
                url: format!(
                    "{}{}",
                    self.segment_base_url,
                    self.segment_media_url
                        .replace("$RepresentationID$", &self.representation_id)
                        .replace("$Number$", &(self.segment_start + i as u32).to_string())
                ),
                length: Duration::from_millis(self.segment_lengths[i] as u64),
            })
        }

        segments
    }
}

/// Video resolution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct StreamSegment {
    #[serde(skip)]
    executor: Arc<Executor>,

    /// Url to the actual data.
    pub url: String,
    /// Video length of this segment.
    pub length: Duration,
}

impl StreamSegment {
    /// Get the raw data for the current segment.
    pub async fn data(&self) -> Result<Vec<u8>> {
        self.executor.get(&self.url).request_raw(false).await
    }
}
