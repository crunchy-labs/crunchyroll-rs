#![cfg(any(feature = "hls-stream", feature = "dash-stream"))]

use crate::crunchyroll::USER_AGENT;
use crate::error::CrunchyrollError;
use crate::media::{PlaybackStream, VideoStream};
use crate::{Executor, Locale, Request, Result};
use http::header;
use isahc::config::Configurable;
use isahc::tls::TlsConfigBuilder;
use isahc::{AsyncReadResponseExt, HttpClient, HttpClientBuilder};
use std::borrow::BorrowMut;
use std::fmt::Formatter;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "hls-stream")]
pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
#[cfg(not(feature = "hls-stream"))]
pub type Aes128CbcDec = ();

macro_rules! impl_streaming {
    ($($stream:ident)*) => {
        $(
            impl $stream {
                /// Returns streaming data which can be used to get the literal stream data and
                /// process it further (e.g. write them to a file which than can be played), based
                /// of the [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) stream
                /// Crunchyroll provides.
                /// The locale argument specifies which hardsub (subtitles which are "burned" into
                /// the video) the returned data should have. You can get a list of supported locales
                /// by calling [`VideoStream::streaming_hardsub_locales`] /
                /// [`PlaybackStream::streaming_hardsub_locales`].
                /// The result contains video + audio data (combined). If you want to get video and
                /// audio separately, check out [`VideoStream::dash_streaming_data`] /
                /// [`PlaybackStream::dash_streaming_data`].
                /// Note that this is only the implementation of this crate to stream data. You can
                /// still manually use the variants in [`VideoStream::variants`] /
                /// [`PlaybackStream::variants`] and implement the streaming on your own.
                #[cfg(feature = "hls-stream")]
                pub async fn hls_streaming_data(&self, hardsub: Option<Locale>) -> Result<Vec<VariantData>> {
                    if let Some(locale) = hardsub {
                        if let Some(raw_streams) = self.variants.get(&locale) {
                            VariantData::from_hls_master(
                                self.executor.clone(),
                                raw_streams.adaptive_hls.as_ref().unwrap().url.clone()
                            )
                            .await
                        } else {
                            Err(CrunchyrollError::Input(
                                format!("could not find any stream with hardsub locale '{}'", locale).into()
                            ))
                        }
                    } else if let Some(raw_streams) = self.variants.get(&Locale::Custom("".into())) {
                        VariantData::from_hls_master(
                            self.executor.clone(),
                            raw_streams.adaptive_hls.as_ref().unwrap().url.clone(),
                        )
                        .await
                    } else if let Some(raw_streams) = self.variants.get(&Locale::Custom(":".into())) {
                        VariantData::from_hls_master(
                            self.executor.clone(),
                            raw_streams.adaptive_hls.as_ref().unwrap().url.clone(),
                        )
                        .await
                    } else {
                        Err(CrunchyrollError::Internal(
                            "could not find supported stream".into(),
                        ))
                    }
                }

                /// Returns streaming data which can be used to get the literal stream data and
                /// process it further (e.g. write them to a file which than can be played), based
                /// of the
                /// [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP)
                /// stream Crunchyroll provides.
                /// The locale argument specifies which hardsub (subtitles which are "burned" into
                /// the video) the returned data should have. You can get a list of supported locales
                /// by calling [`VideoStream::streaming_hardsub_locales`] /
                /// [`PlaybackStream::streaming_hardsub_locales`].
                /// The result is a tuple; the first [`Vec<VariantData>`] contains only video data,
                /// without any audio; the second [`Vec<VariantData>`] contains only audio data,
                /// without any video. If you want video + audio combined, check out
                /// [`VideoStream::dash_streaming_data`] / [`PlaybackStream::dash_streaming_data`].
                /// Note that this is only the implementation of this crate to stream data. You can
                /// still manually use the variants in [`VideoStream::variants`] /
                /// [`PlaybackStream::variants`] and implement the streaming on your own.
                #[cfg(feature = "dash-stream")]
                pub async fn dash_streaming_data(&self, hardsub: Option<Locale>) -> Result<(Vec<VariantData>, Vec<VariantData>)> {
                    let url = if let Some(locale) = hardsub {
                        if let Some(raw_streams) = self.variants.get(&locale) {
                            raw_streams.adaptive_dash.as_ref().unwrap().url.clone()
                        } else {
                            return Err(CrunchyrollError::Input(
                                format!("could not find any stream with hardsub locale '{}'", locale).into()
                            ))
                        }
                    } else if let Some(raw_streams) = self.variants.get(&Locale::Custom("".into())) {
                        raw_streams.adaptive_dash.as_ref().unwrap().url.clone()
                    } else if let Some(raw_streams) = self.variants.get(&Locale::Custom(":".into())) {
                        raw_streams.adaptive_dash.as_ref().unwrap().url.clone()
                    } else {
                        return Err(CrunchyrollError::Internal(
                            "could not find supported stream".into(),
                        ))
                    };

                    let mut video = vec![];
                    let mut audio = vec![];

                    let raw_mpd = self.executor.get(url)
                        .request_raw()
                        .await?;
                    let period = dash_mpd::parse(&String::from_utf8_lossy(raw_mpd.as_slice()).to_string().as_str())
                        .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?
                        .periods[0]
                        .clone();
                    let adaptions = period.adaptations;

                    for adaption in adaptions {
                        if adaption.maxWidth.is_some() || adaption.maxHeight.is_some() {
                            video.extend(VariantData::from_mpeg_mpd_representations(self.executor.clone(), adaption.SegmentTemplate.expect("dash segment template"), adaption.representations).await?)
                        } else {
                            audio.extend(VariantData::from_mpeg_mpd_representations(self.executor.clone(), adaption.SegmentTemplate.expect("dash segment template"), adaption.representations).await?)
                        }
                    }

                    Ok((video, audio))
                }

                /// Return all supported hardsub locales which can be used as argument in
                /// [`VideoStream::streaming_data`] / [`PlaybackStream::streaming_data`].
                pub fn streaming_hardsub_locales(&self) -> Vec<Locale> {
                    self.variants.iter().filter_map(|(locale, variant)| if variant.adaptive_hls.is_some() {
                        Some(locale.clone())
                    } else {
                        None
                    }).collect()
                }
            }
        )*
    }
}

impl_streaming! {
    VideoStream PlaybackStream
}

#[derive(Clone, Debug)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl From<m3u8_rs::Resolution> for Resolution {
    fn from(resolution: m3u8_rs::Resolution) -> Self {
        Self {
            height: resolution.height,
            width: resolution.width,
        }
    }
}

#[derive(Clone, Debug)]
enum VariantDataUrl {
    #[cfg(feature = "hls-stream")]
    Hls { url: String },
    #[cfg(feature = "dash-stream")]
    MpegDash {
        id: String,
        base: String,
        init: String,
        fragments: String,
        start: u32,
        count: u32,
    },
}

/// Streaming data for a variant.
#[allow(dead_code)]
#[derive(Clone, Debug, Request)]
#[request(executor(segments))]
pub struct VariantData {
    executor: Arc<Executor>,

    pub resolution: Resolution,
    pub bandwidth: u64,
    pub fps: f64,
    pub codecs: String,

    url: VariantDataUrl,
}

impl VariantData {
    #[cfg(feature = "hls-stream")]
    pub(crate) async fn from_hls_master(
        executor: Arc<Executor>,
        url: String,
    ) -> Result<Vec<VariantData>> {
        let mut resp = executor.client.get_async(url).await?;
        let raw_master_playlist = resp.bytes().await?;

        let master_playlist = m3u8_rs::parse_master_playlist_res(raw_master_playlist.as_slice())
            .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;

        let mut stream_data: Vec<VariantData> = vec![];

        for variant in master_playlist.variants {
            #[cfg(not(feature = "__test_strict"))]
            stream_data.push(VariantData {
                executor: executor.clone(),

                resolution: variant
                    .resolution
                    .unwrap_or(m3u8_rs::Resolution {
                        height: 0,
                        width: 0,
                    })
                    .into(),
                bandwidth: variant.bandwidth,
                fps: variant.frame_rate.unwrap_or_default(),
                codecs: variant.codecs.unwrap_or_default(),

                url: VariantDataUrl::Hls { url: variant.uri },
            });

            #[cfg(feature = "__test_strict")]
            stream_data.push(VariantData {
                executor: executor.clone(),

                resolution: variant.resolution.unwrap().into(),
                bandwidth: variant.bandwidth,
                fps: variant.frame_rate.unwrap(),
                codecs: variant.codecs.unwrap(),

                url: VariantDataUrl::Hls { url: variant.uri },
            });
        }

        Ok(stream_data)
    }

    #[cfg(feature = "dash-stream")]
    pub(crate) async fn from_mpeg_mpd_representations(
        executor: Arc<Executor>,
        segment_template: dash_mpd::SegmentTemplate,
        representations: Vec<dash_mpd::Representation>,
    ) -> Result<Vec<VariantData>> {
        let mut stream_data = vec![];

        for representation in representations {
            let string_fps = representation.frameRate.unwrap_or_default();
            let mut fps_split = string_fps.split('/');
            let left = fps_split.next().unwrap_or("0").parse().unwrap_or(0f64);
            let right = fps_split.next().unwrap_or("0").parse().unwrap_or(0f64);
            let fps = if left != 0f64 && right != 0f64 {
                left / right
            } else {
                0f64
            };

            let mut segment_count = 0u32;
            for segment in segment_template
                .SegmentTimeline
                .clone()
                .expect("dash segment timeline")
                .segments
            {
                segment_count += segment.r.unwrap_or(1) as u32
            }

            #[cfg(not(feature = "__test_strict"))]
            stream_data.push(VariantData {
                executor: executor.clone(),
                resolution: Resolution {
                    height: representation.height.unwrap_or_default(),
                    width: representation.width.unwrap_or_default(),
                },
                bandwidth: representation.bandwidth.unwrap_or_default(),
                fps,
                codecs: representation.codecs.unwrap_or_default(),
                url: VariantDataUrl::MpegDash {
                    id: representation.id.expect("dash representation id"),
                    base: representation
                        .BaseURL
                        .get(0)
                        .expect("dash base url")
                        .base
                        .clone(),
                    init: segment_template
                        .initialization
                        .clone()
                        .expect("dash initialization url"),
                    fragments: segment_template.media.clone().expect("dash media url"),
                    start: segment_template.startNumber.expect("dash start number") as u32,
                    count: segment_count,
                },
            });

            #[cfg(feature = "__test_strict")]
            stream_data.push(VariantData {
                executor: executor.clone(),
                resolution: Resolution {
                    // unwrap_or_default is called here because a audio representation has no
                    // resolution
                    height: representation.height.unwrap_or_default(),
                    width: representation.width.unwrap_or_default(),
                },
                bandwidth: representation.bandwidth.unwrap(),
                fps,
                codecs: representation.codecs.unwrap(),
                url: VariantDataUrl::MpegDash {
                    id: representation.id.expect("dash representation id"),
                    base: representation
                        .BaseURL
                        .get(0)
                        .expect("dash base url")
                        .base
                        .clone(),
                    init: segment_template
                        .initialization
                        .clone()
                        .expect("dash initialization url"),
                    fragments: segment_template.media.clone().expect("dash media url"),
                    start: segment_template.startNumber.expect("dash start number") as u32,
                    count: segment_count,
                },
            })
        }

        Ok(stream_data)
    }

    /// Return a [`isahc::HttpClient`] which can be used to download segments (via
    /// [`VariantData::segments`]). The normal [`crate::Crunchyroll::client`] cannot be used because
    /// its configuration is not compatible with the segment download servers.
    pub fn download_client(&self) -> HttpClient {
        #[cfg(not(any(all(windows, target_env = "msvc"), feature = "static-certs")))]
        let tls = TlsConfigBuilder::default().build();
        #[cfg(any(all(windows, target_env = "msvc"), feature = "static-certs"))]
        let tls = TlsConfigBuilder::default()
            .root_cert_store(isahc::tls::RootCertStore::from(
                isahc::tls::Certificate::from_pem(include_bytes!(concat!(
                    env!("OUT_DIR"),
                    "/cacert.pem"
                ))),
            ))
            .build();

        HttpClientBuilder::new()
            .default_header(header::USER_AGENT, USER_AGENT)
            .default_header(header::ACCEPT, "*")
            .tls_config(tls)
            .build()
            .unwrap()
    }

    /// Return all segments in order the variant stream is made of.
    pub async fn segments(&self) -> Result<Vec<VariantSegment>> {
        match &self.url {
            #[cfg(feature = "hls-stream")]
            VariantDataUrl::Hls { .. } => self.hls_segments().await,
            #[cfg(feature = "dash-stream")]
            VariantDataUrl::MpegDash { .. } => self.dash_segments().await,
        }
    }

    #[cfg(feature = "hls-stream")]
    async fn hls_segments(&self) -> Result<Vec<VariantSegment>> {
        use aes::cipher::KeyIvInit;

        #[allow(irrefutable_let_patterns)]
        let VariantDataUrl::Hls { url } = &self.url else {
            return Err(CrunchyrollError::Internal("variant url should be hls".into()))
        };

        let mut resp = self.executor.client.get_async(url).await?;
        let raw_media_playlist = resp.bytes().await?;
        let media_playlist = m3u8_rs::parse_media_playlist_res(raw_media_playlist.as_slice())
            .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;

        let mut segments: Vec<VariantSegment> = vec![];
        let mut key: Option<Aes128CbcDec> = None;

        let download_client = Arc::new(self.download_client());

        for segment in media_playlist.segments {
            if let Some(k) = segment.key {
                if let Some(url) = k.uri {
                    let mut resp = download_client.clone().get_async(url).await?;
                    let raw_key = resp.bytes().await?;

                    let temp_iv = k.iv.unwrap_or_default();
                    let iv = if !temp_iv.is_empty() {
                        temp_iv.as_bytes()
                    } else {
                        raw_key.as_ref()
                    };

                    key = Some(Aes128CbcDec::new(raw_key.as_slice().into(), iv.into()));
                }
            }

            segments.push(VariantSegment {
                download_client: download_client.clone(),
                key: key.clone(),
                url: segment.uri,
                length: Some(Duration::from_secs_f32(segment.duration)),
            })
        }

        Ok(segments)
    }

    // Get the m3u8 url if you want to use ffmpeg to handle all the download process
    // It can be faster to download yourself segment by segment
    #[cfg(feature = "hls-stream")]
    pub fn hls_master_url(&self) -> Result<String> {
        #[allow(irrefutable_let_patterns)]
        let VariantDataUrl::Hls { url } = &self.url else {
            return Err(CrunchyrollError::Internal("variant url should be hls".into()))
        };
        Ok(url.to_string())
    }

    #[cfg(feature = "dash-stream")]
    async fn dash_segments(&self) -> Result<Vec<VariantSegment>> {
        #[allow(irrefutable_let_patterns)]
        let VariantDataUrl::MpegDash { id, base, init, fragments, start, count } = self.url.clone() else {
            return Err(CrunchyrollError::Internal("variant url should be dash".into()))
        };

        let download_client = Arc::new(self.download_client());

        let mut segments = vec![VariantSegment {
            download_client: download_client.clone(),
            key: None,
            url: base.clone() + &init.replace("$RepresentationID$", &id),
            length: None,
        }];

        for i in start..count + start + 1 {
            segments.push(VariantSegment {
                download_client: download_client.clone(),
                key: None,
                url: base.clone()
                    + &fragments
                        .replace("$Number$", &i.to_string())
                        .replace("$RepresentationID$", &id),
                length: None,
            })
        }

        Ok(segments)
    }
}

/// A single segment, representing a part of a video stream.
/// Because Crunchyroll uses segment / chunk based video streaming (usually
/// [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) or
/// [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP)) the actual
/// video stream consists of multiple [`VariantSegment`]s.
#[allow(dead_code)]
#[derive(Clone, Debug, Request)]
pub struct VariantSegment {
    download_client: Arc<HttpClient>,

    /// Decryption key to decrypt the segment data (if encrypted).
    pub key: Option<Aes128CbcDec>,
    /// Url to the actual data.
    pub url: String,
    /// Video length of this segment. Is [`Some`] if this segment was generated from a function
    /// utilizing hls. Is [`None`] if generated from a dash function.
    pub length: Option<Duration>,
}

impl VariantSegment {
    /// Decrypt a raw segment and return the decrypted raw bytes back. Useful if you want to
    /// implement the full segment download yourself and [`VariantSegment::write_to`] has too many
    /// limitation for your use case (e.g. a if you want to get the download speed of each segment).
    pub fn decrypt(segment_bytes: &mut [u8], key: Option<Aes128CbcDec>) -> Result<&[u8]> {
        use aes::cipher::BlockDecryptMut;
        if let Some(key) = key {
            let decrypted = key
                .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(segment_bytes)
                .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;
            Ok(decrypted)
        } else {
            Ok(segment_bytes)
        }
    }

    /// Write this segment to a writer.
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let mut resp = self.download_client.get_async(self.url).await?;
        let segment = resp.bytes().await?;

        w.write(VariantSegment::decrypt(
            segment.to_vec().borrow_mut(),
            self.key.clone(),
        )?)
        .map_err(|e| CrunchyrollError::Input(e.to_string().into()))?;

        Ok(())
    }
}
