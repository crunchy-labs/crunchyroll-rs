//! # crunchyroll-rs
//!
//! A easy-to-use, batteries-included library for the undocumented
//! [Crunchyroll](https://www.crunchyroll.com/) api, completely written in Rust.
//!
//! You can use a premium account as well as a non-premium account to use this library, but you
//! will be limited to your account tier access privileges (=> you can't access a premium-only
//! series with a free account).
//!
//! The library has some features to ensure a flawless experience in a ⚡🦀 blazingly fast
//! environment.
//! - Full [Tokio](https://tokio.rs/) compatibility.
//! - Solid tests to [ensure api compatability](#implementation).
//!
//! # Getting started
//!
//! Before you can do anything, you have to instantiate a new [`Crunchyroll`] struct at first. This
//! internally creates a new [`crunchyroll::CrunchyrollBuilder`] instance. All functions of this
//! struct are chaining, which means you can build a working Crunchyroll instance in one expression.
//!
//! ```
//! use crunchyroll_rs::{Crunchyroll, Locale};
//!
//! let crunchy = Crunchyroll::builder()
//!     // set the language in which results should be returned
//!     .locale(Locale::en_US)
//!     // login with user credentials (other login options are also available)
//!     .login_with_credentials("username", "password")
//!     .await?;
//! ```
//!
//! ## Request media
//!
//! You can request media like series, episodes, movies, ... with their corresponding function in
//! the [`Crunchyroll`] struct. Use `Crunchyroll::*_from_id` to get them while `*` is the media type.
//!
//! ```
//! let series: Series = crunchy
//!     // get the series with the id 'GY8VEQ95Y'
//!     .media_from_id("GY8VEQ95Y")
//!     .await?;
//!
//! let episode: Episode = crunchy
//!     // get the episode with the id 'GRDKJZ81Y'
//!     .media_from_id("GRDKJZ81Y")
//!     .await?;
//!
//! ```
//!
//! If you want to get the children of a "container" media like a series or season, these types
//! implements the appropriate functions to archive this.
//!
//! ```
//! let seasons = series
//!     // get the seasons of this episode
//!     .seasons()
//!     .await?;
//! ```
//!
//! ## Streaming
//!
//! This crate allows you to get the actual video streams behind episodes and movies. With
//! [`Episode::streams`] and [`Movie::streams`] you get access to the streams. The returning struct
//! [`media::VideoStream`] has all required information to access the streams.
//!
//! ```
//! let streams = episode
//!     .streams()
//!     .await?;
//! ```
//!
//! Crunchyroll uses the [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) and
//! [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) video streaming
//! formats to distribute their streams. The logic to work with this formats is already implemented
//! into this crate.
//!
//! The feature `hls-stream` and / or `dash-stream` must be activated to get streams. `hls-stream`
//! is activated by default and should be used if you want to get the video + audio combined
//! ([`media::VideoStream::hls_streaming_data`]). `dash-stream` should be used if you want to get
//! the audio and video streams separately ([`media::VideoStream::dash_streaming_data`]).
//!
//! ```
//! let streaming_data = streams
//!     .hls_streaming_data(None)
//!     .await?;
//!
//! // sort the streams to get the stream with the best resolution at first
//! streaming_data.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width).reverse());
//!
//! let sink = &mut std::io::sink();
//!
//! // get the segments / video chunks of the first stream (which is the best after it got sorted
//! // above)
//! let segments = streaming_data[0].segments().await?;
//! // iterate through every segment and write it to the provided writer (which is a sink in this
//! // case; it drops its input immediately). writer can be anything which implements `std::io::Write`
//! // like a file, a pipe, ...
//! for segment in segments {
//!     segment.write_to(sink).await?;
//! }
//! ```
//!
//! # Bugs
//! Crunchyroll is awful in keep their api clean. Thus, some things are broken, will break for no
//! reason or aren't well implemented (if at all). The methods added with the
//! `experimental-stabilizations` feature (`CrunchyrollBuilder::stabilization_*`)
//!
//! # Implementation
//! To ensure at least all existing parts of the library are working as expected, a special feature
//! only for testing is implemented. When running tests with the `__test_strict` feature, it ensures
//! that no fields were added or removed from an api response, otherwise the associated test will
//! fail.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod account;
pub mod categories;
pub mod common;
pub mod crunchyroll;
pub mod devices;
pub mod error;
pub mod feed;
pub mod list;
pub mod media;
#[cfg(feature = "parse")]
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
pub mod parse;
pub mod rating;
pub mod search;

// internal
mod internal;
mod macros;

// internal
pub(crate) use common::Request;
pub(crate) use crunchyroll::Executor;
pub(crate) use error::Result;
pub(crate) use internal::serde::EmptyJsonProxy;
pub(crate) use macros::{enum_values, options};

pub use crunchyroll::{Crunchyroll, Locale};
pub use media::{
    Concert, Episode, MediaCollection, Movie, MovieListing, MusicVideo, Season, Series,
};
#[cfg(feature = "parse")]
pub use parse::{parse_url, UrlType};

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;
