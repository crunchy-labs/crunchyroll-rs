use crate::internal::sealed::Sealed;
use crate::{
    Artist, Concert, Episode, MediaCollection, Movie, MovieListing, MusicVideo, Season, Series,
};

macro_rules! impl_sealed {
    ($($media:ident)*) => {
        $(
            impl Sealed for $media {}
        )*
    };
}

impl_sealed! {
    Series Season Episode MovieListing Movie Artist MusicVideo Concert
}

macro_rules! impl_from_media_collection {
    ($($media:ident)*) => {
        $(
            impl From<$media> for MediaCollection {
                fn from(value: $media) -> Self {
                    MediaCollection::$media(value)
                }
            }
        )*
    }
}

impl_from_media_collection! {
    Series Season Episode MovieListing Movie Artist MusicVideo Concert
}
