use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use crate::common::{Available, Crunchy, FromId, Image};
use crate::{Crunchyroll, Locale};
use crate::error::Result;

#[derive(Clone, Serialize)]
#[allow(dead_code)]
pub enum MediaType {
    Series, Movie
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let media_type = match self {
            MediaType::Series => "series",
            MediaType::Movie => "movie_listing"
        };
        write!(f, "{}", media_type)
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct MovieListingImages {
    pub poster_tall: Vec<Vec<Image>>,
    pub poster_wide: Vec<Vec<Image>>
}

/// This struct represents a movie collection.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct MovieListing<'a> {
    #[serde(skip)]
    #[serde(default = "Crunchyroll::default_for_struct")]
    crunchy: Option<&'a Crunchyroll>,

    pub id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub seo_title: String,
    pub description: String,
    pub seo_description: String,
    pub extended_description: String,

    pub movie_release_year: u32,
    pub content_provider: String,

    pub keywords: Vec<String>,
    pub season_tags: Vec<String>,

    pub images: MovieListingImages,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub subtitle_locales: Vec<Locale>,

    pub hd_flag: bool,
    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub free_available_date: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub premium_available_date: DateTime<Utc>,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue
}

impl<'a> Crunchy<'a> for MovieListing<'a> {
    fn get_crunchyroll(&self) -> &'a Crunchyroll {
        self.crunchy.unwrap()
    }
}

impl<'a> Available<'a> for MovieListing<'a> {
    fn available(&self) -> bool {
        !self.is_premium_only || self.get_crunchyroll().config.premium
    }
}

#[async_trait::async_trait]
impl<'a> FromId<'a> for MovieListing<'a> {
    async fn from_id(crunchy: &'a Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/movie_listings/{}", crunchy.config.bucket, id);
        let builder = crunchy.client
            .get(endpoint)
            .query(&crunchy.media_query());

        let mut movie_listing: MovieListing = crunchy.request(builder)
            .await?;
        movie_listing.crunchy = Some(crunchy);
        Ok(movie_listing)
    }
}

type SeriesImages = MovieListingImages;

/// This struct represents a crunchyroll series.
#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Series<'a> {
    #[serde(skip)]
    #[serde(default = "Crunchyroll::default_for_struct")]
    crunchy: Option<&'a Crunchyroll>,

    pub id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub seo_title: String,
    pub description: String,
    pub seo_description: String,
    pub extended_description: String,

    pub series_launch_year: u32,
    pub content_provider: String,

    pub episode_count: u32,
    pub season_count: u32,
    pub media_count: u32,

    pub keywords: Vec<String>,
    pub season_tags: Vec<String>,

    pub images: SeriesImages,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    pub audio_locales: Vec<Locale>,
    pub subtitle_locales: Vec<Locale>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue
}

impl<'a> Crunchy<'a> for Series<'a> {
    fn get_crunchyroll(&self) -> &'a Crunchyroll {
        self.crunchy.unwrap()
    }
}

impl<'a> Available<'a> for Series<'a> {
    fn available(&self) -> bool {
        self.channel_id.is_empty() || self.get_crunchyroll().config.premium
    }
}

#[async_trait::async_trait]
impl<'a> FromId<'a> for Series<'a> {
    async fn from_id(crunchy: &'a Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/series/{}", crunchy.config.bucket, id);
        let builder = crunchy.client
            .get(endpoint)
            .query(&crunchy.media_query());

        let mut series: Series = crunchy.request(builder)
            .await?;
        series.crunchy = Some(crunchy);
        Ok(series)
    }
}
