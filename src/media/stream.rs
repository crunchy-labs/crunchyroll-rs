use crate::error::{is_request_error, Error};
use crate::{Crunchyroll, Executor, Locale, Request, Result};
use dash_mpd::MPD;
use reqwest::multipart::Form;
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::Write;
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

#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[request(executor(subtitles))]
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

    /// Might be null, for music videos and concerts mostly.
    #[serde(skip_serializing)]
    versions: Option<Vec<StreamVersion>>,

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

impl Stream {
    /// Uses the endpoint which is also used in the Chrome browser to receive streams. This endpoint
    /// always returns DRM encrypted streams.
    pub async fn drm_from_id(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str>,
        optional_media_type: Option<String>,
    ) -> Result<Self> {
        Self::from_id(crunchyroll, id, "web", "chrome", optional_media_type).await
    }

    /// Uses the endpoint which is also used in the Nintendo Switch to receive streams. At the time
    /// of writing, this is the only known endpoint that still delivers streams without DRM, but
    /// this might change at any time (hence the "mabye" in the function name).
    pub async fn maybe_without_drm_from_id(
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
        stream.executor = crunchyroll.executor.clone();
        stream.id = id.as_ref().to_string();
        stream.optional_media_type = optional_media_type;

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

    /// Invalidates all the stream data which may be obtained from [`Stream::stream_data`]. Only
    /// required if the stream has DRM (if [`Stream::session::uses_stream_limits`] is `true`, stream
    /// data is DRM encrypted, if `false` not).
    pub async fn invalidate(self) -> Result<()> {
        if !self.session.uses_stream_limits {
            return Ok(());
        }

        let endpoint = format!(
            "https://cr-play-service.prd.crunchyrollsvc.com/v1/token/{}/{}/delete",
            self.id, self.token
        );

        self.executor
            .post(endpoint)
            .multipart(Form::new().text(
                "jwtToken",
                self.executor.config.read().await.access_token.clone(),
            ))
            .request_raw(false)
            .await?;

        Ok(())
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
            if self.session.uses_stream_limits {
                result.push(
                    Self::drm_from_id(
                        &Crunchyroll {
                            executor: self.executor.clone(),
                        },
                        id,
                        self.optional_media_type.clone(),
                    )
                    .await?,
                )
            } else {
                result.push(
                    Self::maybe_without_drm_from_id(
                        &Crunchyroll {
                            executor: self.executor.clone(),
                        },
                        id,
                        self.optional_media_type.clone(),
                    )
                    .await?,
                )
            }
        }
        Ok(result)
    }

    pub async fn versions(&self) -> Result<Vec<Stream>> {
        let version_ids = self
            .versions
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|v| v.id.clone())
            .collect::<Vec<String>>();

        let mut result = vec![];
        for id in version_ids {
            if self.session.uses_stream_limits {
                result.push(
                    Self::drm_from_id(
                        &Crunchyroll {
                            executor: self.executor.clone(),
                        },
                        id,
                        self.optional_media_type.clone(),
                    )
                    .await?,
                )
            } else {
                result.push(
                    Self::maybe_without_drm_from_id(
                        &Crunchyroll {
                            executor: self.executor.clone(),
                        },
                        id,
                        self.optional_media_type.clone(),
                    )
                    .await?,
                )
            }
        }
        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct StreamVersion {
    #[serde(rename = "guid")]
    id: String,
    #[serde(rename = "media_guid")]
    media_id: String,
    #[serde(rename = "season_guid")]
    season_id: String,

    audio_locale: Locale,

    is_premium_only: bool,
    original: bool,

    variant: String,
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
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let resp = self.executor.get(self.url).request_raw(false).await?;
        w.write_all(resp.as_ref()).map_err(|e| Error::Input {
            message: e.to_string(),
        })?;
        Ok(())
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
                    let Some((l, r)) = frame_rate.split_once('/') else {
                        return Err(err_fn("invalid fps"));
                    };
                    let left = l.parse().unwrap_or(0f64);
                    let right = r.parse().unwrap_or(0f64);
                    let fps = if left != 0f64 && right != 0f64 {
                        left / right
                    } else {
                        return Err(err_fn("null fps"));
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

        for (i, number) in (self.segment_start..self.segment_lengths.len() as u32).enumerate() {
            segments.push(StreamSegment {
                executor: self.executor.clone(),
                url: format!(
                    "{}{}",
                    self.segment_base_url,
                    self.segment_media_url
                        .replace("$RepresentationID$", &self.representation_id)
                        .replace("$Number$", &number.to_string())
                ),
                length: Duration::from_millis(*self.segment_lengths.get(i).unwrap() as u64),
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
