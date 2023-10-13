use crate::media::Artist;
use crate::{Concert, MusicVideo, Result};

macro_rules! impl_manual_media_serialize {
    ($($media:ident)*) => {
        $(
            impl serde::Serialize for $media {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    $media::serialize(self, serializer)
                }
            }
        )*
    }
}

impl_manual_media_serialize! {
    Concert MusicVideo
}

macro_rules! impl_media_music {
    ($($media_music:ident)*) => {
        $(
            impl $media_music {
                /// Streams for this music video / concert. This endpoint triggers a rate limiting
                /// if requested too much over a short time period (the rate limiting may occur as an
                /// error, Crunchyroll doesn't give a hint that a ratelimit is hit). Unlike
                /// [`crate::Episode`] and [`crate::Movie`] there is no older stream endpoint
                /// available to get the streams from.
                pub async fn stream(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_url(self.executor.clone(), "https://www.crunchyroll.com/content/v2/music", &self.stream_id).await
                }

                /// Check if the music video / concert can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.details.premium || !self.is_premium_only
                }
            }
        )*
    }
}

impl_media_music! {
    Concert MusicVideo
}

macro_rules! music_eq {
    ($($media:ident)*) => {
        $(
            impl PartialEq<Self> for $media {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }
        )*
    }
}

music_eq! {
    MusicVideo Concert Artist
}
