use crate::error::{Error, ErrorKind, is_request_error};
use crate::{Crunchyroll, Executor, Locale, Request, Result};
use byteorder::{BigEndian, ReadBytesExt};
use dash_mpd::{ContentProtection, MPD};
use http::header;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Read};
use std::iter;
use std::ops::Not;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

/// Platforms that can request a [`Stream`]. Because not all platforms have their own variant, use
/// [`StreamPlatform::Custom`] to define one.
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
    #[default]
    TvAndroid,
    TvRoku,
    TvSamsung,
    TvLg,
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

    /// [`Stream::audio_role`], in my tests it always only had one entry. Empty if concert or
    /// music video.
    #[serde(default)]
    pub roles: Vec<String>,

    pub is_premium_only: bool,
    /// If the audio of this version is the native language of this anime.
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
pub struct StreamDrm {
    /// Name of the drm provider, e.g. 'widevine'
    name: String,
    /// Url to request a drm license
    drm_url: String,
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
    /// Either "main" (original language), "dub" (dubbed) or "" (music video or concert).
    #[serde(default)]
    pub audio_role: String,
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub burned_in_locale: Option<Locale>,

    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_hardsubs")]
    pub hard_subs: HashMap<Locale, String>,

    /// All subtitles.
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_subtitles")]
    pub subtitles: Vec<Subtitle>,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_subtitles")]
    pub captions: Vec<Subtitle>,

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

    pub drm: StreamDrm,

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
            StreamPlatform::TvAndroid => ("tv", "android_tv"),
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
            "https://www.crunchyroll.com/playback/v3/{}/{device}/{platform}/play",
            id.as_ref()
        );

        let mut stream = match crunchyroll.executor.get(endpoint).request::<Stream>().await {
            Ok(stream) => stream,
            Err(e) => {
                return match e.kind() {
                    // try to invalidate the session if the decoding failed. a decoding failure
                    // usually means that the request was successful but returned unexpected data.
                    // thus, an active session is issued to the server, but it isn't usable because
                    // this functions returns an error. further stream requests may be blocked until
                    // crunchyroll invalidates the session server-side if it isn't done manually
                    ErrorKind::Decode { content } => {
                        let Some(content) = content else {
                            return Err(e);
                        };
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
                    ErrorKind::Request { status }
                        if status.is_some_and(|s| {
                            s == 400 && e.to_string().starts_with("error 40016")
                        }) => {
                        Err(e.update_msg(|msg| msg.map(|msg| msg + " - This probably means that a custom platform was set and the provided basic auth token is wrong, or the default basic auth token is outdated (if this is the case, check if the library is up-to-date)")))
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
            return Err(Error::error_from_kind(
                ErrorKind::Input,
                "livestream download isn't supported",
            ));
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
        let endpoint = format!("https://www.crunchyroll.com/playback/v1/token/{id}/{token}",);

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
    pub audio: Vec<AudioMediaStream>,
    pub video: Vec<VideoMediaStream>,
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

        let raw_mpd = executor
            .get(url.as_ref())
            .query(&[
                (
                    "accountid",
                    executor.details.account_id().unwrap_or_default().as_str(),
                ),
                ("playbackGuid", token.as_ref()),
            ])
            .request_raw(true)
            .await?;
        // if the response is json and not xml it should always be an error
        if let Ok(json) = serde_json::from_slice(&raw_mpd) {
            is_request_error(json, url.as_ref(), &StatusCode::FORBIDDEN)?;
        }

        let mut mpd: MPD = dash_mpd::parse(&String::from_utf8_lossy(&raw_mpd)).map_err(|e| {
            Error::error_from_other_error_and_url(
                e,
                ErrorKind::Decode {
                    content: Some(raw_mpd.clone()),
                },
                url.as_ref(),
            )
        })?;

        let err_fn = |msg: &str| {
            Error::error_from_kind_and_url(
                ErrorKind::Decode {
                    content: Some(raw_mpd.clone()),
                },
                url.as_ref(),
                msg,
            )
        };

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

            for representation in adaption.representations {
                let drm_types = adaption
                    .ContentProtection
                    .iter()
                    .chain(representation.ContentProtection.iter())
                    .filter_map(Self::drm_type_from_content_protection)
                    .collect::<Vec<_>>();

                let segment_template = representation
                    .SegmentTemplate
                    .as_ref()
                    .or(adaption.SegmentTemplate.as_ref());
                let segment_base = representation
                    .SegmentBase
                    .as_ref()
                    .or(adaption.SegmentBase.as_ref());

                let segment_info = if let Some(template) = segment_template {
                    let segment_lengths = template
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

                    MediaStreamSegmentInfo::Template {
                        representation_id: representation
                            .id
                            .clone()
                            .ok_or("no representation id found")
                            .map_err(err_fn)?,
                        segment_start: template
                            .startNumber
                            .ok_or("no start number found")
                            .map_err(err_fn)? as u32,
                        segment_lengths,
                        segment_init_url: template
                            .initialization
                            .clone()
                            .ok_or("no init url found")
                            .map_err(err_fn)?,
                        segment_media_url: template
                            .media
                            .clone()
                            .ok_or("no media url found")
                            .map_err(err_fn)?,
                        segment_timescale: template
                            .timescale
                            .ok_or("no timescale found")
                            .map_err(err_fn)? as u32,
                    }
                } else if let Some(base) = segment_base {
                    let initialization = base
                        .Initialization
                        .as_ref()
                        .ok_or("no initialization found")
                        .map_err(err_fn)?;
                    let base_url = representation
                        .BaseURL
                        .first()
                        .ok_or("no base url found")
                        .map_err(err_fn)?
                        .base
                        .clone();

                    let index_range = {
                        let index_range = base
                            .indexRange
                            .as_ref()
                            .ok_or("no index range found")
                            .map_err(err_fn)?;
                        let (index_range_start, index_range_end) = index_range
                            .split_once('-')
                            .ok_or("invalid index range found")
                            .map_err(err_fn)?;

                        (
                            index_range_start
                                .parse::<u64>()
                                .map_err(|_| err_fn("invalid index range start"))?,
                            index_range_end
                                .parse::<u64>()
                                .map_err(|_| err_fn("invalid index range end"))?,
                        )
                    };
                    let init_range = {
                        let init_range = initialization
                            .range
                            .as_ref()
                            .ok_or("no init range found")
                            .map_err(err_fn)?;
                        let (init_range_start, init_range_end) = init_range
                            .split_once('-')
                            .ok_or("invalid init range found")
                            .map_err(err_fn)?;

                        (
                            init_range_start
                                .parse::<u64>()
                                .map_err(|_| err_fn("invalid init range start"))?,
                            init_range_end
                                .parse::<u64>()
                                .map_err(|_| err_fn("invalid init range end"))?,
                        )
                    };

                    MediaStreamSegmentInfo::Base {
                        url: base_url,
                        index_range,
                        init_range,
                    }
                } else {
                    return Err(err_fn("unsupported manifest"));
                };

                let media_stream = MediaStream {
                    executor: executor.clone(),
                    bandwidth: representation
                        .bandwidth
                        .ok_or("no bandwidth found")
                        .map_err(err_fn)?,
                    codecs: representation
                        .codecs
                        .ok_or("no codecs found")
                        .map_err(err_fn)?,
                    drm: drm_types.is_empty().not().then(|| MediaStreamDRM {
                        token: token.as_ref().to_string(),
                        types: drm_types.clone(),
                    }),
                    watch_id: watch_id.as_ref().to_string(),
                    segment_base_url: representation
                        .BaseURL
                        .first()
                        .ok_or("no base url found")
                        .map_err(err_fn)?
                        .base
                        .clone(),
                    segment_info,
                };

                if let Some(sampling_rate) = representation.audioSamplingRate {
                    audio.push(AudioMediaStream {
                        media_stream,
                        sampling_rate: sampling_rate
                            .parse::<u32>()
                            .map_err(|e| err_fn(&e.to_string()))?,
                    })
                } else {
                    let (Some(width), Some(height)) = (representation.width, representation.height)
                    else {
                        return Err(err_fn("invalid resolution"));
                    };
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

                    video.push(VideoMediaStream {
                        media_stream,
                        resolution: Resolution { width, height },
                        fps,
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

    fn drm_type_from_content_protection(
        content_protection: &ContentProtection,
    ) -> Option<MediaStreamDRMType> {
        match content_protection.schemeIdUri.as_str() {
            "urn:uuid:9a04f079-9840-4286-ab92-e65be0885f95" => {
                Some(MediaStreamDRMType::Playready {
                    pro: content_protection
                        .msprpro
                        .clone()
                        .and_then(|pro| pro.content),
                    pssh: content_protection.cenc_pssh.is_empty().not().then(|| {
                        content_protection
                            .cenc_pssh
                            .iter()
                            .cloned()
                            .filter_map(|pssh| pssh.content)
                            .collect()
                    }),
                })
            }
            "urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed" => Some(MediaStreamDRMType::Widevine {
                pssh: content_protection
                    .cenc_pssh
                    .iter()
                    .cloned()
                    .filter_map(|pssh| pssh.content)
                    .collect(),
            }),
            _ => None,
        }
    }
}

macro_rules! media_stream_types {
    ($struct_name:ident => $media_stream_ident:ident) => {
        impl $struct_name {
            pub fn into_media_stream(self) -> MediaStream {
                self.$media_stream_ident
            }
        }

        impl std::ops::Deref for $struct_name {
            type Target = MediaStream;

            fn deref(&self) -> &Self::Target {
                &self.$media_stream_ident
            }
        }

        impl std::ops::DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$media_stream_ident
            }
        }
    };
}

#[derive(Clone, Debug, Serialize, Request)]
#[request(executor(media_stream))]
pub struct AudioMediaStream {
    media_stream: MediaStream,

    pub sampling_rate: u32,
}

media_stream_types!(AudioMediaStream => media_stream);

#[derive(Clone, Debug, Serialize, Request)]
#[request(executor(media_stream))]
pub struct VideoMediaStream {
    media_stream: MediaStream,

    pub resolution: Resolution,
    pub fps: f64,
}

media_stream_types!(VideoMediaStream => media_stream);

#[derive(Clone, Debug, Serialize)]
pub(crate) enum MediaStreamSegmentInfo {
    Template {
        representation_id: String,
        segment_start: u32,
        segment_lengths: Vec<u32>,
        segment_init_url: String,
        segment_media_url: String,
        segment_timescale: u32,
    },
    Base {
        url: String,
        index_range: (u64, u64),
        init_range: (u64, u64),
    },
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct MediaStream {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub bandwidth: u64,
    pub codecs: String,

    /// If [`Some`], the stream data is DRM encrypted and the struct contains all data needed for
    /// you to decrypted it. If [`None`], the stream data is not DRM encrypted.
    pub drm: Option<MediaStreamDRM>,

    pub watch_id: String,

    #[serde(skip_serializing)]
    segment_base_url: String,

    #[serde(skip_serializing)]
    segment_info: MediaStreamSegmentInfo,
}

#[derive(Clone, Debug, Serialize, Request)]
pub enum MediaStreamDRMType {
    /// One of both is always set
    Playready {
        pro: Option<String>,
        pssh: Option<Vec<String>>,
    },
    Widevine {
        pssh: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Request)]
pub struct MediaStreamDRM {
    pub token: String,
    pub types: Vec<MediaStreamDRMType>,
}

static SEGMENT_MEDIA_URL_TEMPLATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$(?P<placeholder>RepresentationID|Number|Time|Bandwidth)(%0(?P<padding>\d)d)?\$")
        .unwrap()
});

impl MediaStream {
    /// Returns all segment this stream is made of.
    pub async fn segments(&self) -> Result<Vec<StreamSegment>> {
        Ok(match &self.segment_info {
            MediaStreamSegmentInfo::Template { .. } => self.template_segments(),
            MediaStreamSegmentInfo::Base { .. } => self.base_segments().await?,
        })
    }

    fn template_segments(&self) -> Vec<StreamSegment> {
        let MediaStreamSegmentInfo::Template {
            representation_id,
            segment_start,
            segment_lengths,
            segment_init_url,
            segment_media_url,
            segment_timescale,
        } = &self.segment_info
        else {
            unreachable!()
        };

        let mut segments = vec![StreamSegment {
            executor: self.executor.clone(),
            url: format!(
                "{}{}",
                self.segment_base_url,
                segment_init_url
                    .replace("$RepresentationID$", representation_id)
                    .replace("$Bandwidth$", &self.bandwidth.to_string())
            ),
            length: Duration::from_secs(0),
            range: None,
        }];

        let captures = SEGMENT_MEDIA_URL_TEMPLATE
            .captures_iter(segment_media_url)
            .collect::<Vec<_>>();
        for (i, _) in segment_lengths.iter().enumerate() {
            let mut media_url = segment_media_url.clone();
            let mut offset = 0;
            for capture in &captures {
                let replace_string = match &capture["placeholder"] {
                    "Number" => format!(
                        "{:0width$}",
                        segment_start + i as u32,
                        width = capture
                            .name("padding")
                            .map_or(0, |p| p.as_str().parse().unwrap())
                    ),
                    "Time" => format!(
                        "{:0width$}",
                        i * *segment_timescale as usize,
                        width = capture
                            .name("padding")
                            .map_or(0, |p| p.as_str().parse().unwrap())
                    ),
                    "RepresentationID" => representation_id.clone(),
                    "Bandwidth" => self.bandwidth.to_string(),
                    _ => unreachable!(),
                };

                let mat = capture.get(0).unwrap();
                let replace_start = (mat.start() as i32 + offset) as usize;
                let replace_end = (mat.end() as i32 + offset) as usize;

                let len_before = media_url.len() as i32;
                media_url.replace_range(replace_start..replace_end, &replace_string);
                let len_after = media_url.len() as i32;

                offset += len_after - len_before;
            }

            segments.push(StreamSegment {
                executor: self.executor.clone(),
                url: format!("{}{}", self.segment_base_url, media_url),
                length: Duration::from_millis(
                    ((segment_lengths[i] as f64 / *segment_timescale as f64) * 1000.) as u64,
                ),
                range: None,
            })
        }

        segments
    }

    async fn base_segments(&self) -> Result<Vec<StreamSegment>> {
        let MediaStreamSegmentInfo::Base {
            url,
            index_range,
            init_range,
        } = &self.segment_info
        else {
            unreachable!()
        };

        let (index_range_start, index_range_end) = index_range;

        let sidx_data = self
            .executor
            .get(url)
            .header(
                header::RANGE,
                format!("bytes={index_range_start}-{index_range_end}"),
            )
            .request_raw(false)
            .await?;

        let sidx_box = SidxBox::parse(&sidx_data).map_err(|_| {
            Error::error_from_kind_and_url(
                ErrorKind::Decode {
                    content: Some(sidx_data),
                },
                url,
                "unable to parse sidx box",
            )
        })?;

        let mut segments = Vec::with_capacity(sidx_box.references.len() + 1);
        segments.push(StreamSegment {
            executor: self.executor.clone(),
            url: url.clone(),
            length: Duration::default(),
            range: Some(*init_range),
        });

        let mut current_byte_offset = *index_range_end + sidx_box.first_offset + 1;
        for reference in &sidx_box.references {
            let duration = Duration::from_millis(
                ((reference.subsegment_duration as f64 / sidx_box.timescale as f64) * 1000.) as u64,
            );

            let range_start = current_byte_offset;
            let range_end = current_byte_offset + reference.referenced_size as u64 - 1;

            segments.push(StreamSegment {
                executor: self.executor.clone(),
                url: url.clone(),
                length: duration,
                range: Some((range_start, range_end)),
            });

            current_byte_offset += reference.referenced_size as u64;
        }

        Ok(segments)
    }
}

/// Video resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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
    /// Byte range of this segment.
    pub range: Option<(u64, u64)>,
}

impl StreamSegment {
    /// Get the raw data for the current segment.
    pub async fn data(&self) -> Result<Vec<u8>> {
        let mut builder = self.executor.get(&self.url);
        if let Some((start, end)) = &self.range {
            builder = builder.header(header::RANGE, format!("bytes={}-{}", start, end));
        }
        builder.request_raw(false).await
    }
}

/* -----
copied (and partially modified) from dash-mpd (https://github.com/emarsden/dash-mpd-rs/blob/d19fab6d57c3feac1aa13efa753c197f7baeab92/src/sidx.rs#L13-L101)
atm the struct is gated behind a flag that adds a bunch of dependencies that aren't needed, so
copying is more lightweight
------- */
// A Segment Index Box provides a compact index of one media stream within the media segment to which
// it applies.
#[derive(Debug, Clone, PartialEq)]
struct SidxBox {
    pub version: u8,
    pub flags: u32, // actually only u24
    pub reference_id: u32,
    pub timescale: u32,
    pub earliest_presentation_time: u64,
    pub first_offset: u64,
    pub reference_count: u16,
    pub references: Vec<SidxReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidxReference {
    pub reference_type: u8,
    pub referenced_size: u32,
    pub subsegment_duration: u32,
    pub starts_with_sap: u8, // (actually a boolean)
    pub sap_type: u8,
    pub sap_delta_time: u32,
}

impl SidxBox {
    fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut rdr = Cursor::new(data);
        let _box_size = rdr.read_u32::<BigEndian>()?;
        let mut box_header = [0u8; 4];
        if rdr.read_exact(&mut box_header).is_err() {
            return Err("reading box header".into());
        }
        if !box_header.eq(b"sidx") {
            return Err("expecting sidx BMFF header".into());
        }
        let version = rdr.read_u8()?;
        let flags = rdr.read_u24::<BigEndian>()?;
        let reference_id = rdr.read_u32::<BigEndian>()?;
        let timescale = rdr.read_u32::<BigEndian>()?;
        let earliest_presentation_time = if version == 0 {
            u64::from(rdr.read_u32::<BigEndian>()?)
        } else {
            rdr.read_u64::<BigEndian>()?
        };
        let first_offset = if version == 0 {
            u64::from(rdr.read_u32::<BigEndian>()?)
        } else {
            rdr.read_u64::<BigEndian>()?
        };
        let _reserved = rdr.read_u16::<BigEndian>()?;
        let reference_count = rdr.read_u16::<BigEndian>()?;
        let mut references = Vec::with_capacity(reference_count as usize);
        for _ in 0..reference_count {
            // chunk is 1 bit for reference_type, and 31 bits for referenced_size.
            let chunk = rdr.read_u32::<BigEndian>()?;
            // Reference_type = 1 means a reference to another sidx (hierarchical sidx)
            let reference_type = ((chunk & 0x8000_0000) >> 31) as u8;
            if reference_type != 0 {
                return Err("Don't know how to handle hierarchical sidx".into());
            }
            let referenced_size = chunk & 0x7FFF_FFFF;
            let subsegment_duration = rdr.read_u32::<BigEndian>()?;
            let fields = rdr.read_u32::<BigEndian>()?;
            let starts_with_sap = if (fields >> 31) == 1 { 1 } else { 0 };
            let sap_type = ((fields >> 28) & 0b0111) as u8;
            let sap_delta_time = fields & !(0b1111 << 28);

            references.push(SidxReference {
                reference_type,
                referenced_size,
                subsegment_duration,
                starts_with_sap,
                sap_type,
                sap_delta_time,
            });
        }
        Ok(SidxBox {
            version,
            flags,
            reference_id,
            timescale,
            earliest_presentation_time,
            first_offset,
            reference_count,
            references,
        })
    }
}
