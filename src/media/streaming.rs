#![cfg(feature = "stream")]

use crate::crunchyroll::USER_AGENT;
use crate::error::CrunchyrollError;
use crate::media::{PlaybackStream, VideoStream};
use crate::{Executor, Locale, Request, Result};
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use http::header;
use isahc::AsyncReadResponseExt;
use std::borrow::BorrowMut;
use std::fmt::Formatter;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

macro_rules! impl_streaming {
    ($($stream:ident)*) => {
        $(
            impl $stream {
                /// Returns data which can be used to get the literal stream data and process it
                /// further (e.g. write them to a file which than can be played).
                /// The locale argument specifies which hardsub (subtitles which are "burned" into
                /// the video) the returned data should have. You can get a list of supported locales
                /// by calling [`VideoStream::streaming_hardsub_locales`] /
                /// [`PlaybackStream::streaming_hardsub_locales`].
                /// Note that this is only the implementation of this crate to stream data. You can
                /// still manually use the variants in [`VideoStream::variants`] /
                /// [`PlaybackStream::variants`] and implement the streaming on your own.
                pub async fn streaming_data(&self, hardsub: Option<Locale>) -> Result<Vec<VariantData>> {
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

    url: String,
}

impl VariantData {
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
            #[cfg(feature = "__test_strict")]
            stream_data.push(VariantData {
                executor: executor.clone(),

                resolution: variant.resolution.unwrap().into(),
                bandwidth: variant.bandwidth,
                fps: variant.frame_rate.unwrap(),
                codecs: variant.codecs.unwrap(),

                url: variant.uri,
            });

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
                fps: variant.frame_rate.unwrap_or(0 as f64),
                codecs: variant.codecs.unwrap_or_else(|| "".into()),

                url: variant.uri,
            });
        }

        Ok(stream_data)
    }

    /// Return all segments in order the variant stream is made of.
    #[allow(dead_code)]
    pub async fn segments(&self) -> Result<Vec<VariantSegment>> {
        let mut resp = self.executor.client.get_async(self.url.clone()).await?;
        let raw_media_playlist = resp.bytes().await?;
        let media_playlist = m3u8_rs::parse_media_playlist_res(raw_media_playlist.as_slice())
            .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;

        let mut segments: Vec<VariantSegment> = vec![];
        let mut key: Option<Aes128CbcDec> = None;

        for segment in media_playlist.segments {
            if let Some(k) = segment.key {
                if let Some(url) = k.uri {
                    let req = isahc::Request::get(url)
                        .header(header::USER_AGENT, USER_AGENT)
                        .body(())
                        .unwrap();
                    let mut resp = isahc::send_async(req).await?;
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
                executor: self.executor.clone(),
                key: key.clone(),
                url: segment.uri,
                length: Duration::from_secs_f32(segment.duration),
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
    executor: Arc<Executor>,

    /// Decryption key to decrypt the segment data (if encrypted).
    pub key: Option<Aes128CbcDec>,
    /// Url to the actual data.
    pub url: String,
    /// Video length of this segment.
    pub length: Duration,
}

impl VariantSegment {
    /// Decrypt a raw segment and return the decrypted raw bytes back. Useful if you want to
    /// implement the full segment download yourself and [`VariantSegment::write_to`] has too many
    /// limitation for your use case (e.g. a if you want to get the download speed of each segment).
    pub fn decrypt(segment_bytes: &mut [u8], key: Option<Aes128CbcDec>) -> Result<&[u8]> {
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
        // The default isahc client the crate uses cannot be used here because it's configured to
        // only accept TLSv1.3 connections (to bypass the cloudflare bot check) but the servers where
        // the stream segments are stored only accept TLS up to v1.2 (but have no cloudflare bot check)
        let req = isahc::Request::get(self.url)
            .header(header::USER_AGENT, USER_AGENT)
            .body(())
            .unwrap();
        let mut resp = isahc::send_async(req).await?;
        let segment = resp.bytes().await?;

        w.write(VariantSegment::decrypt(
            segment.to_vec().borrow_mut(),
            self.key.clone(),
        )?)
        .map_err(|e| CrunchyrollError::Input(e.to_string().into()))?;

        Ok(())
    }
}
