//! # crunchyroll-rs
//!
//! A easy-to-use, batteries-included library for the undocumented
//! [Crunchyroll](https://www.crunchyroll.com/) beta api, completely written in Rust.
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
//! # Implementation
//! Because Crunchyroll does not have a fixed api versioning and is currently in its beta phase,
//! changes are likely to happen (even though they weren't very radical in the past) so keep an eye
//! on the version of this library to get new updates and potential fixes.
//!
//! To ensure at least all existing parts of the library are working as expected, a special feature
//! only for testing is implemented. When running tests with the `__test_strict` feature, it ensures
//! that no fields were added or removed from an api response, otherwise the associated test will
//! fail.

pub mod account;
pub mod categories;
pub mod common;
pub mod crunchyroll;
pub mod error;
pub mod list;
pub mod media;
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

pub use common::BulkResult;
pub use crunchyroll::{Crunchyroll, Locale};
pub use media::{Episode, Movie, MovieListing, Season, Series};

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;
