//! Url parsing.

use regex::Regex;
use std::sync::LazyLock;

static SERIES_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}(-[a-zA-Z]{2})?/)?(?P<type>series|movie_listing|artist)/(?P<id>[^/]+).*$").unwrap()
});
static EPISODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}(-[a-zA-Z]{2})?/)?watch/((?P<music_type>musicvideo|concert)/)?(?P<id>[^/]+).*$").unwrap()
});

/// Types of Crunchyroll urls, pointing to media.
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
#[derive(Clone, Debug)]
pub enum UrlType {
    /// The parsed url points to a series. Use [`crate::Series::from_id`] with the value of this
    /// field to get a usable struct out of it.
    Series(String),
    /// The parsed url points to a movie listing. Use [`crate::MovieListing::from_id`] with the
    /// value of this field to get a usable struct out of it. This kind of url might not exist in
    /// Crunchyroll at all but to be api compatible it's included anyway.
    MovieListing(String),
    /// The parsed url points to a music artist. Use [`crate::Artist::from_id`] with the value of
    /// this field to get a usable struct out of it.
    Artist(String),
    /// The parsed url points to an episode or movie. You can either try
    /// [`crate::Episode::from_id`] and [`crate::Movie::from_id`] to guess if it's an episode or
    /// movie (in 99.9% of the time it will be an episode, because (at the time of writing)
    /// Crunchyroll has only 3 movies which are listed as movies. All other movies are listed as
    /// episodes. Makes sense I know) or use [`crate::MediaCollection::from_id`]. The value of this
    /// field is the id you have to use in all shown methods.
    EpisodeOrMovie(String),
    /// The parsed url points to a music video. Use [`crate::MusicVideo::from_id`] with the value of
    /// this field to get a usable struct out of it.
    MusicVideo(String),
    /// The parsed url points to a concert. Use [`crate::Concert::from_id`] with the value of this
    /// field to get a usable struct out of it.
    Concert(String),
}

/// Extract information out of Crunchyroll urls which are pointing to media.
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
pub fn parse_url<S: AsRef<str>>(url: S) -> Option<UrlType> {
    if let Some(capture) = SERIES_REGEX.captures(url.as_ref()) {
        let id = capture["id"].to_string();
        match &capture["type"] {
            "series" => Some(UrlType::Series(id)),
            "movie_listing" => Some(UrlType::MovieListing(id)),
            "artist" => Some(UrlType::Artist(id)),
            _ => unreachable!(),
        }
    } else if let Some(capture) = EPISODE_REGEX.captures(url.as_ref()) {
        match capture.name("music_type").map(|m| m.as_str()) {
            Some("musicvideo") => Some(UrlType::MusicVideo(capture["id"].to_string())),
            Some("concert") => Some(UrlType::Concert(capture["id"].to_string())),
            None => Some(UrlType::EpisodeOrMovie(capture["id"].to_string())),
            _ => unreachable!(),
        }
    } else {
        None
    }
}
