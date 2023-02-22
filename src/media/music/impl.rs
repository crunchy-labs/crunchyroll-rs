use crate::media::Artist;
use crate::{Concert, MusicVideo, Result};

macro_rules! impl_media_music {
    ($($media_music:ident)*) => {
        $(
            impl $media_music {
                /// Streams for this music video / concert.
                pub async fn streams(&self) -> Result<$crate::media::VideoStream> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content/v2/music/{}/streams",
                        self.stream_id
                    );
                    let mut data = self.executor.get(endpoint)
                        .apply_preferred_audio_locale_query()
                        .apply_locale_query()
                        .request::<$crate::common::V2BulkResult<serde_json::Map<String, serde_json::Value>>>()
                        .await?;

                    let mut map = data.meta.clone();
                    map.insert("variants".to_string(), data.data.remove(0).into());

                    Ok(serde_json::from_value(serde_json::to_value(map)?)?)
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
