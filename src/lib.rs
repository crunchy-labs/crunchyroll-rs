//! # crunchyroll-rs
//!
//! An easy-to-use, batteries-included library for the undocumented
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
//!     // support for username login was dropped by Crunchyroll on December 6th, 2023
//!     .login_with_credentials("email", "password")
//!     .await?;
//! ```
//!
//! ## Request media
//!
//! You can request media like series, episodes, movies, ... with their corresponding function in
//! the [`Crunchyroll`] struct. Use `Crunchyroll::*_from_id` to get them while `*` is the media type.
//!
//! ```
//! // get the series with the id 'GY8VEQ95Y'
//! let series: Series = crunchy.media_from_id("GY8VEQ95Y").await?;
//!
//! // get the episode with the id 'GRDKJZ81Y'
//! let episode: Episode = crunchy.media_from_id("GY8VEQ95Y").await?;
//!
//! ```
//!
//! If you want to get the children of a "container" media like a series or season, these types
//! implements the appropriate functions to archive this.
//!
//! ```
//! let seasons = series
//!     // get the seasons of this series
//!     .seasons()
//!     .await?;
//! ```
//!
//! ## Streaming
//!
//! _All streams are DRM protected. The library does not contain logic to decrypt it, so if you want
//! to do this, you have to implement it yourself._
//!
//! This crate allows you to get the actual video streams behind episodes and movies.
//!
//! ```
//! let stream = episode
//!     .stream()
//!     .await?;
//! ```
//!
//! Crunchyroll uses the [DASH] video streaming format to distribute their streams. The logic to
//! work with these formats is already implemented into this crate.
//!
//! ```
//! let (mut video_streams, mut audio_streams) = stream
//!     .stream_data(None)
//!     .await?
//!     .unwrap();
//!
//!  // sort the streams to get the stream with the best resolution / bitrate at first
//! video_streams.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth).reverse());
//! audio_streams.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth).reverse());
//!
//! let sink = &mut std::io::sink();
//!
//! // get the segments / video chunks of the first stream (which is the best after it got sorted
//! // above)
//! let video_segments = video_streams[0].segments();
//! let audio_segments = audio_streams[0].segments();
//! // iterate through every segment and write it to the provided writer (which is a sink in this
//! // case; it drops its input immediately). writer can be anything which implements `std::io::Write`
//! // like a file, a pipe, ...
//! for video_segment in video_segments {
//!     sink.write_all(&video_segment.data().await?)?;
//! }
//! for audio_segment in audio_segments {
//!     sink.write_all(&audio_segment.data().await?)?;
//! }
//! ```
//!
//! # Bugs
//! Crunchyroll is awful in keep their api clean. Thus, some things are broken, will break for no
//! reason or aren't well implemented (if at all). The methods added with the
//! `experimental-stabilizations` feature (`CrunchyrollBuilder::stabilization_*`) can be used to
//! prevent some issues. Note that there is no guarantee that these functions will work or that they
//! will not break anything.
//!
//! ### Cloudflare
//! Crunchyroll uses the cloudflare bot protection to detect if requests are made by a human.
//! Obviously this crate makes automated requests and thus, Cloudflare sometimes blocks requests.
//! The crate catches these errors with the [`error::Error::Block`] enum field. The block
//! occurs depending on different factors like your location. If such a block occurs you can try to
//! create a custom [`reqwest::Client`] which has the needed configuration to bypass this check,
//! like other user agents or tls backends (note that [`reqwest`] currently only supports
//! [`native-tls`](https://docs.rs/native-tls/latest/native_tls/) besides [`rustls`] as tls backend,
//! which is confirmed to work with openssl on Linux only, on Windows the blocks are even more
//! aggressive). The configurations may vary on the factors addressed so there is no 100% right way
//! to do it.
//!
//! # Features
//!
//! - **parse** *(enabled by default)*: Enables url parsing.
//! - **tower**: Enables the usage of a [tower](https://docs.rs/tower) compatible middleware.
//! - **experimental-stabilizations**: Provides some functions to maybe fix broken api results. See
//!   [Bugs](#bugs) for more information.
//!
//! # Implementation
//! To ensure at least all existing parts of the library are working as expected, a special feature
//! only for testing is implemented. When running tests with the `__test_strict` feature, it ensures
//! that no fields were added or removed from an api response, otherwise the associated test will
//! fail.
//!
//! [DASH]: https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP

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
pub mod profile;
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
