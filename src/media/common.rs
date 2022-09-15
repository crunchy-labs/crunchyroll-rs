use crate::media::{PlaybackStream, VideoStream};
use crate::Result;

use crate::enum_values;
pub(crate) use proc_macro::Playback;

enum_values! {
    #[derive(Debug)]
    pub enum MediaType {
        Series = "series"
        Movie = "movie_listing"
    }
}

/// Provides playback streams for episodes or movies. Playback streams are mostly used to provide
/// trailers for an episode / movie.
#[async_trait::async_trait]
pub trait Playback {
    /// Returns the playback streams.
    async fn playback(&self) -> Result<PlaybackStream>;
}

/// Provides video streams for episodes or movies. This streams are what the end user sees when
/// watching a video on Crunchyroll.
#[async_trait::async_trait]
pub trait Streams {
    /// Returns the streams.
    async fn streams(&self) -> Result<VideoStream>;
}
