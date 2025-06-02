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
                /// Streams for this music video / concert.
                /// All streams are drm encrypted, decryption is not handled in this crate, so you
                /// must do this yourself.
                pub async fn stream(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_id(&$crate::Crunchyroll { executor: self.executor.clone() }, &self.id, &self.executor.details.stream_platform).await
                }

                /// Check if the music video / concert can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.premium().await || !self.is_premium_only
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
