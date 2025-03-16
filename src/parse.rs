//! Url parsing.

use regex::Regex;
use std::sync::OnceLock;

static SERIES_REGEX: OnceLock<Regex> = OnceLock::new();
static MUSIC_REGEX: OnceLock<Regex> = OnceLock::new();
static EPISODE_REGEX: OnceLock<Regex> = OnceLock::new();

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
    /// The parsed url points to a episode or movie. You can either try
    /// [`crate::Episode::from_id`] and [`crate::Movie::from_id`] to guess if it's an episode or
    /// movie (in 99.9% of the time it will be an episode, because (at the time of writing)
    /// Crunchyroll has only 3 movies which are listed as movies. All other movies are listed as
    /// episodes. Makes sense I know) or use [`crate::MediaCollection::from_id`]. The value of this
    /// field is the id you have to use in all shown methods.
    EpisodeOrMovie(String),
    /// The parsed url points to a music video. Use [`crate::MusicVideo::from_id`] with the value of
    /// this field to get a usable struct out of it.
    MusicVideo(String),
    /// The parsed url points to a music video. Use [`crate::Concert::from_id`] with the value of
    /// this field to get a usable struct out of it.
    Concert(String),
}

/// Extract information out of Crunchyroll urls which are pointing to media.
#[cfg_attr(docsrs, doc(cfg(feature = "parse")))]
pub fn parse_url<S: AsRef<str>>(url: S) -> Option<UrlType> {
    let series_regex = SERIES_REGEX.get_or_init(|| {
        Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}(-[a-zA-Z]{2})?/)?(?P<type>series|movie_listing)/(?P<id>[^/]+).*$").unwrap()
    });
    let music_regex = MUSIC_REGEX.get_or_init(|| {
        Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}(-[a-zA-Z]{2})?/)?watch/(?P<music_type>musicvideo|concert)/(?P<id>[^/]+).*$").unwrap()
    });
    let episode_regex = EPISODE_REGEX.get_or_init(|| {
        Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}(-[a-zA-Z]{2})?/)?watch/(?P<id>[^/]+).*$").unwrap()
    });

    #[allow(clippy::manual_map)]
    if let Some(capture) = series_regex.captures(url.as_ref()) {
        let id = capture.name("id").unwrap().as_str().to_string();
        match capture.name("type").unwrap().as_str() {
            "series" => Some(UrlType::Series(id)),
            "movie_listing" => Some(UrlType::MovieListing(id)),
            _ => unreachable!(),
        }
    } else if let Some(capture) = music_regex.captures(url.as_ref()) {
        match capture.name("music_type").unwrap().as_str() {
            "musicvideo" => Some(UrlType::MusicVideo(
                capture.name("id").unwrap().as_str().to_string(),
            )),
            "concert" => Some(UrlType::Concert(
                capture.name("id").unwrap().as_str().to_string(),
            )),
            _ => unreachable!(),
        }
    } else if let Some(capture) = episode_regex.captures(url.as_ref()) {
        Some(UrlType::EpisodeOrMovie(
            capture.name("id").unwrap().as_str().to_string(),
        ))
    } else {
        None
    }
}
