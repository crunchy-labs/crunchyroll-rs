#![cfg(feature = "parse")]

use regex::Regex;

/// Types of Crunchyroll urls, pointing to series, episodes or movies.
#[derive(Clone, Debug)]
pub enum UrlType {
    /// The parsed url points to a beta series. Use [`Crunchyroll::series_from_id`] with the value
    /// of this field to get a usable struct out of it.
    BetaSeries(String),
    /// The parsed url points to a beta movie listing. Use [`Crunchyroll::movie_listing_from_id`]
    /// with the value of this field to get a usable struct out of it. This kind of url might not
    /// exist in Crunchyroll beta at all but to be api compatible it's included anyway.
    BetaMovieListing(String),
    /// The parsed url points to a beta episode or movie. You can either try
    /// [`Crunchyroll::episode_from_id`] and [`Crunchyroll::movie_from_id`] to guess if it's a
    /// episode or movie (in 99.9% of the time it will be a episode, because (at the time of writing)
    /// Crunchyroll has only 3 movies which are listed as movies. All other movies are listed as
    /// episodes. Makes sense I know) or use [`crate::Media::from_id`]. The value of this field is
    /// the id you have to use in all shown methods.
    BetaEpisodeOrMovie(String),

    /// The parsed url points to a classic series or movie listing. Because classic urls are
    /// structured poorly they cannot be parsed very accurate. You have to search for the series /
    /// movie listing which is the value of this field and hope that the Crunchyroll api returns the
    /// correct series / movie listing.
    ///
    /// _Please just use Crunchyroll beta urls_.
    ClassicSeriesOrMovieListing(String),
    /// The parsed url points to a classic episode. Because classic urls are structured poorly they
    /// cannot be parsed very accurate. You have to search for the
    /// [`UrlType::ClassicEpisodeOrMovie::series_name`] series and then look up the episode which has
    /// [`UrlType::ClassicEpisodeOrMovie::episode_name`] in one of the episode name fields. You can
    /// also (in addition) check if [`UrlType::ClassicEpisodeOrMovie::number`] which represents the
    /// episode number matches with the episode number you got from the looked up episodes. But
    /// [`UrlType::ClassicEpisodeOrMovie::number`] is not so accurate as it seems, for example
    /// episode number 24.9 gets converted to 249.
    ///
    /// _Please just use Crunchyroll beta urls_.
    ClassicEpisode {
        series_name: String,
        episode_name: String,
        number: String,
    },
    /// Just like [`UrlType::ClassicEpisode`] but without episode number and movie instead of episode.
    ///
    /// _Please just use Crunchyroll beta urls_.
    ClassicMovie {
        movie_listing_name: String,
        movie_name: String,
    },
}

/// Extract information out of Crunchyroll urls which are pointing to episodes / movies /
/// series.
///
/// Note: It is recommended to use only Crunchyroll beta urls (`beta.crunchyroll.com`) for this
/// function. Classic urls cannot be parsed properly to guarantee that the results this function
/// delivers really is what the url points to.
pub fn parse_url<S: AsRef<str>>(url: S) -> Option<UrlType> {
    // the regex calls are pretty ugly but for performance reasons it's the best to define them
    // only if needed. once the std lazy api is stabilized they can be moved to the root of this
    // file to make it look cleaner. an external crate to call the regexes lazy would also be an
    // option but it would a little overload if it's only used here

    #[allow(clippy::manual_map)]
    if let Some(capture) = Regex::new(r"^https?://beta\.crunchyroll\.com/([a-zA-Z]{2}/)?(?P<type>series|movie_listing)/(?P<id>.+)/.*$")
        .unwrap()
        .captures(url.as_ref())
    {
        let id = capture.name("id").unwrap().as_str().to_string();
        match capture.name("type").unwrap().as_str() {
            "series" => Some(UrlType::BetaSeries(id)),
            "movie_listing" => Some(UrlType::BetaMovieListing(id)),
            _ => None // should never happen
        }
    } else if let Some(capture) = Regex::new(r"^https?://beta\.crunchyroll\.com/([a-zA-Z]{2}/)?watch/(?P<id>.+)/.*$")
        .unwrap()
        .captures(url.as_ref())
    {
        Some(UrlType::BetaEpisodeOrMovie(capture.name("id").unwrap().as_str().to_string()))
    } else if let Some(aaargh_please_just_use_beta_urls) = Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}/)?(?P<series_or_movie_name>[^/]+)(/videos)?/?$")
        .unwrap()
        .captures(url.as_ref())
    {
        Some(UrlType::ClassicSeriesOrMovieListing(aaargh_please_just_use_beta_urls.name("series_or_movie_name").unwrap().as_str().to_string()))
    } else if let Some(why_do_i_have_to_still_support_this) = Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}/)?(?P<series_name>[^/]+)/episode-(?P<number>[0-9]+)-(?P<episode_name>.+)-.*$")
        .unwrap()
        .captures(url.as_ref())
    {
        Some(UrlType::ClassicEpisode {
            series_name: why_do_i_have_to_still_support_this.name("series_name").unwrap().as_str().to_string(),
            episode_name: why_do_i_have_to_still_support_this.name("episode_name").unwrap().as_str().to_string(),
            number: why_do_i_have_to_still_support_this.name("number").unwrap().as_str().to_string(),
        })
    } else if let Some(plsss) = Regex::new(r"^https?://(www\.)?crunchyroll\.com/([a-zA-Z]{2}/)?(?P<movie_listing_name>[^/]+)/(?P<movie_name>.+)-.*$")
        .unwrap()
        .captures(url.as_ref())
    {
        Some(UrlType::ClassicMovie {
            movie_listing_name: plsss.name("movie_listing_name").unwrap().as_str().to_string(),
            movie_name: plsss.name("movie_name").unwrap().as_str().to_string(),
        })
    } else {
        None
    }
}
