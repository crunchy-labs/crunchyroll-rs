use crate::error::{Error, is_request_error};
use crate::{Crunchyroll, Executor, Locale, Request, Result};
use dash_mpd::MPD;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::sync::Arc;
use std::time::Duration;

/// Platforms that can request a [`Stream`]. Because not all platforms have their own variant, use
/// [`Stream::Custom`] to define one.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum StreamPlatform {
    AndroidPhone,
    AndroidTablet,
    ConsolePs4,
    ConsolePs5,
    ConsoleSwitch,
    ConsoleXboxOne,
    IosIpad,
    IosIphone,
    IosVision,
    TvRoku,
    TvSamsung,
    TvLg,
    #[default]
    WebChrome,
    WebEdge,
    WebFirefox,
    WebSafari,
    Custom {
        /// A device, e.g. `tv` or `web`.
        device: String,
        /// A platform, e.g. `roku` or `chrome`.
        platform: String,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct StreamVersion {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,
    #[serde(skip)]
    platform: StreamPlatform,
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
            &self.platform,
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
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub burned_in_locale: Option<Locale>,

    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_hardsubs")]
    pub hard_subs: HashMap<Locale, String>,

    /// All subtitles.
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_subtitles")]
    pub subtitles: HashMap<Locale, Subtitle>,
    pub captions: HashMap<Locale, Subtitle>,

    /// Either "on_demand", a normal video, or "live", a livestream. If it's "live",
    /// [`Stream::stream_data`] will fail, as only on-demand videos are supported (and livestreams
    /// are very rare, the only occurrences were airing Kaiju No. 8 episodes).
    pub playback_type: String,

    pub token: String,
    /// If [`StreamSession::uses_stream_limits`] is `true`, this means that the stream data will be
    /// DRM encrypted, if `false` it isn't.
    pub session: StreamSession,

    /// All versions of this stream (same stream but each entry has a different language).
    pub versions: Vec<StreamVersion>,

    #[serde(skip)]
    id: String,

    #[cfg(feature = "__test_strict")]
    asset_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
}

impl Stream {
    /// Requests a stream from an id.
    pub async fn from_id(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str>,
        stream_platform: &StreamPlatform,
    ) -> Result<Self> {
        let (device, platform) = match &stream_platform {
            StreamPlatform::AndroidPhone => ("android", "phone"),
            StreamPlatform::AndroidTablet => ("android", "tablet"),
            StreamPlatform::ConsolePs4 => ("console", "ps4"),
            StreamPlatform::ConsolePs5 => ("console", "ps5"),
            StreamPlatform::ConsoleSwitch => ("console", "switch"),
            StreamPlatform::ConsoleXboxOne => ("console", "xbox_one"),
            StreamPlatform::IosIpad => ("ios", "ipad"),
            StreamPlatform::IosIphone => ("ios", "iphone"),
            StreamPlatform::IosVision => ("ios", "vision"),
            StreamPlatform::TvRoku => ("tv", "roku"),
            StreamPlatform::TvSamsung => ("tv", "samsung"),
            StreamPlatform::TvLg => ("tv", "lg"),
            StreamPlatform::WebChrome => ("web", "chrome"),
            StreamPlatform::WebEdge => ("web", "edge"),
            StreamPlatform::WebFirefox => ("web", "firefox"),
            StreamPlatform::WebSafari => ("web", "safari"),
            StreamPlatform::Custom { device, platform } => (device.as_str(), platform.as_str()),
        };

        let endpoint = format!(
            "https://www.crunchyroll.com/playback/v2/{}/{device}/{platform}/play",
            id.as_ref()
        );

        let mut stream = match crunchyroll.executor.get(endpoint).request::<Stream>().await {
            Ok(stream) => stream,
            Err(e) => {
                return match &e {
                    // try to invalidate the session if the decoding failed. a decoding failure
                    // usually means that the request was successful but returned unexpected data.
                    // thus, an active session is issued to the server, but it isn't usable because
                    // this functions returns an error. further stream requests may be blocked until
                    // crunchyroll invalidates the session server-side if it isn't done manually
                    Error::Decode { content, .. } => {
                        let Ok(content_map) = serde_json::from_slice::<Map<String, Value>>(content)
                        else {
                            return Err(e);
                        };
                        let Some(uses_stream_limits) = content_map
                            .get("session")
                            .and_then(|s| s.as_object()?.get("usesStreamLimits")?.as_bool())
                        else {
                            return Err(e);
                        };
                        let Some(token) = content_map.get("token").and_then(|t| t.as_str()) else {
                            return Err(e);
                        };

                        if uses_stream_limits {
                            let _ = Self::invalidate_raw(id.as_ref(), token, &crunchyroll.executor)
                                .await;
                        }
                        Err(e)
                    }
                    _ => Err(e),
                };
            }
        };
        stream.__set_executor(crunchyroll.executor.clone()).await;
        stream.id = id.as_ref().to_string();

        for version in &mut stream.versions {
            version.platform = stream_platform.clone();
        }

        Ok(stream)
    }

    /// Requests all available video and audio streams. Returns [`None`] if the requested hardsub
    /// isn't available.
    /// You will run into an error when requesting this function too often without invalidating the
    /// data. Crunchyroll only allows a certain amount of stream data to be requested at the same
    /// time, typically the exact amount depends on the type of (premium) subscription you have. You
    /// can use [`Stream::invalidate`] to invalidate all stream data for this stream.
    pub async fn stream_data(&self, hardsub: Option<Locale>) -> Result<Option<StreamData>> {
        if self.playback_type == "live" {
            return Err(Error::Input {
                message: "Livestream cannot be downloaded".to_string(),
            });
        }

        if let Some(hardsub) = hardsub {
            let Some(url) = self
                .hard_subs
                .iter()
                .find_map(|(locale, url)| (locale == &hardsub).then_some(url))
            else {
                return Ok(None);
            };
            Ok(Some(
                StreamData::from_url(
                    self.executor.clone(),
                    url,
                    &self.token,
                    &self.id,
                    &self.audio_locale,
                )
                .await?,
            ))
        } else {
            Ok(Some(
                StreamData::from_url(
                    self.executor.clone(),
                    &self.url,
                    &self.token,
                    &self.id,
                    &self.audio_locale,
                )
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

        Self::invalidate_raw(&self.id, &self.token, &self.executor).await
    }

    async fn invalidate_raw(id: &str, token: &str, executor: &Arc<Executor>) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/playback/v1/token/{}/{}",
            id, token
        );

        executor.delete(endpoint).request_raw(true).await?;

        Ok(())
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

#[derive(Clone, Debug, Serialize)]
pub struct StreamData {
    pub audio: Vec<MediaStream>,
    pub video: Vec<MediaStream>,
    pub subtitle: Option<Subtitle>,
}

impl StreamData {
    async fn from_url(
        executor: Arc<Executor>,
        url: impl AsRef<str>,
        token: impl AsRef<str>,
        watch_id: impl AsRef<str>,
        audio_locale: &Locale,
    ) -> Result<Self> {
        let mut video = vec![];
        let mut audio = vec![];
        let mut subtitle = None;

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
                if adaption.mimeType.is_none_or(|mime| mime != "text/vtt") {
                    continue;
                }
                subtitle = Some(Subtitle {
                    executor: executor.clone(),
                    locale: audio_locale.clone(),
                    url: adaption
                        .representations
                        .first()
                        .ok_or("no subtitle representation found")
                        .map_err(err_fn)?
                        .BaseURL
                        .first()
                        .ok_or("no subtitle url found")
                        .map_err(err_fn)?
                        .base
                        .clone(),
                    format: "vtt".to_string(),
                });
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
                    iter::repeat_n(s.d as u32, s.r.unwrap_or_default() as usize + 1)
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

                    video.push(MediaStream {
                        executor: executor.clone(),
                        bandwidth: representation
                            .bandwidth
                            .ok_or("no bandwidth found")
                            .map_err(err_fn)?,
                        codecs: representation
                            .codecs
                            .ok_or("no codecs found")
                            .map_err(err_fn)?,
                        info: MediaStreamInfo::Video { resolution, fps },
                        drm: pssh.as_ref().map(|pssh| MediaStreamDRM {
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

                    audio.push(MediaStream {
                        executor: executor.clone(),
                        bandwidth: representation
                            .bandwidth
                            .ok_or("no bandwith found")
                            .map_err(err_fn)?,
                        codecs: representation
                            .codecs
                            .ok_or("no codecs found")
                            .map_err(err_fn)?,
                        info: MediaStreamInfo::Audio { sampling_rate },
                        drm: pssh.as_ref().map(|pssh| MediaStreamDRM {
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

        Ok(Self {
            audio,
            video,
            subtitle,
        })
    }
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct MediaStream {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub bandwidth: u64,
    pub codecs: String,

    pub info: MediaStreamInfo,
    /// If [`Some`], the stream data is DRM encrypted and the struct contains all data needed for
    /// you to decrypted it. If [`None`], the stream data is not DRM encrypted.
    pub drm: Option<MediaStreamDRM>,

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
pub struct MediaStreamDRM {
    pub pssh: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Request)]
pub enum MediaStreamInfo {
    Audio { sampling_rate: u32 },
    Video { resolution: Resolution, fps: f64 },
}

impl MediaStream {
    /// Returns the streams' audio sampling rate. Only [`Some`] if the stream is an audio stream
    /// (check [`MediaStream::info`]).
    pub fn sampling_rate(&self) -> Option<u32> {
        if let MediaStreamInfo::Audio { sampling_rate } = &self.info {
            Some(*sampling_rate)
        } else {
            None
        }
    }

    /// Returns the streams' video resolution. Only [`Some`] if the stream is a video stream (check
    /// [`MediaStream::info`]).
    pub fn resolution(&self) -> Option<Resolution> {
        if let MediaStreamInfo::Video { resolution, .. } = &self.info {
            Some(resolution.clone())
        } else {
            None
        }
    }

    /// Returns the streams' video fps. Only [`Some`] if the stream is a video stream (check
    /// [`MediaStream::info`]).
    pub fn fps(&self) -> Option<f64> {
        if let MediaStreamInfo::Video { fps, .. } = &self.info {
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
