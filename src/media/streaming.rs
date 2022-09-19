#![cfg(feature = "stream")]

use crate::error::CrunchyrollError;
use crate::media::{PlaybackStream, VideoStream};
use crate::{Executor, Locale, Request, Result};
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use std::borrow::BorrowMut;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[async_trait::async_trait]
pub trait DefaultStreams {
    async fn default_streams(&self) -> Result<Vec<VariantData>>;
}

#[async_trait::async_trait]
impl DefaultStreams for VideoStream {
    async fn default_streams(&self) -> Result<Vec<VariantData>> {
        if let Some(raw_streams) = self.variants.get(&Locale::Custom("".into())) {
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
                "could not find default stream".into(),
            ))
        }
    }
}

#[async_trait::async_trait]
impl DefaultStreams for PlaybackStream {
    async fn default_streams(&self) -> Result<Vec<VariantData>> {
        if let Some(raw_streams) = self.variants.get(&Locale::Custom("".into())) {
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
                "could not find default stream".into(),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
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
    key: Option<Aes128CbcDec>,
    segments: Option<Vec<VariantSegment>>,
}

impl VariantData {
    pub(crate) async fn from_hls_master(
        executor: Arc<Executor>,
        url: String,
    ) -> Result<Vec<VariantData>> {
        let resp = executor.client.get(url).send().await?;
        let raw_master_playlist = resp.text().await?;

        let master_playlist = m3u8_rs::parse_master_playlist_res(raw_master_playlist.as_bytes())
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
                key: None,
                segments: None,
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
                key: None,
                segments: None,
            });
        }

        Ok(stream_data)
    }

    /// Return all segments in order the variant stream is made of.
    #[allow(dead_code)]
    pub async fn segments(&mut self) -> Result<Vec<VariantSegment>> {
        if let Some(segments) = &self.segments {
            Ok(segments.clone())
        } else {
            let resp = self.executor.client.get(self.url.clone()).send().await?;
            let raw_media_playlist = resp.text().await?;
            let media_playlist = m3u8_rs::parse_media_playlist_res(raw_media_playlist.as_bytes())
                .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;

            let mut segments: Vec<VariantSegment> = vec![];
            for segment in media_playlist.segments {
                if let Some(key) = segment.key {
                    if let Some(url) = key.uri {
                        let resp = self.executor.client.get(url).send().await?;
                        let raw_key = resp.bytes().await?;

                        let temp_iv = key.iv.unwrap_or_else(|| "".to_string());
                        let iv = if !temp_iv.is_empty() {
                            temp_iv.as_bytes()
                        } else {
                            raw_key.as_ref()
                        };

                        self.key = Some(Aes128CbcDec::new(raw_key.as_ref().into(), iv.into()));
                    }
                }

                segments.push(VariantSegment {
                    executor: self.executor.clone(),
                    key: self.key.clone(),
                    url: segment.uri,
                    length: Duration::from_secs_f32(segment.duration),
                })
            }

            self.segments = Some(segments.clone());
            Ok(segments)
        }
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
    #[allow(dead_code)]
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let resp = self.executor.client.get(self.url).send().await?;
        let segment = resp.bytes().await?;

        if let Some(key) = self.key {
            let mut temp_encrypted = segment.to_vec();
            let decrypted = key
                .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(
                    temp_encrypted.borrow_mut(),
                )
                .map_err(|e| CrunchyrollError::Decode(e.to_string().into()))?;
            w.write(decrypted)
                .map_err(|e| CrunchyrollError::Input(e.to_string().into()))?;
        } else {
            w.write(segment.as_ref())
                .map_err(|e| CrunchyrollError::Input(e.to_string().into()))?;
        }
        Ok(())
    }
}
