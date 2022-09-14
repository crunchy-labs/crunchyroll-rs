use crate::common::{Available, FromId, Image, Request};
use crate::media::Panel;
use crate::Result;
use crate::{options, BulkResult, Executor, Locale};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

pub trait VideoCollection {
    fn id(&self) -> String;
}

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListingImages {
    pub poster_tall: Vec<Vec<Image>>,
    pub poster_wide: Vec<Vec<Image>>,
}

/// This struct represents a movie collection.
#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request, Available, FromId)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListing {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

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

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
}

impl VideoCollection for MovieListing {
    fn id(&self) -> String {
        self.id.clone()
    }
}

type SeriesImages = MovieListingImages;

/// This struct represents a crunchyroll series.
#[allow(dead_code)]
#[derive(Deserialize, Debug, Default, Request, Available, FromId)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Series {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

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
    extended_maturity_rating: crate::StrictValue,
}

impl VideoCollection for Series {
    fn id(&self) -> String {
        self.id.clone()
    }
}

options! {
    SimilarOptions;
    #[doc = "Limit of results to return."]
    limit(u32, "n") = Some(20)
}

impl Series {
    pub async fn similar(&self, options: SimilarOptions) -> Result<BulkResult<Panel>> {
        let endpoint = "https://beta.crunchyroll.com/content/v1/13050d47-adec-50c9-ae63-5e2ed8f4e251/similar_to";
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[
                ("guid".to_string(), self.id.clone()),
                (
                    "locale".to_string(),
                    self.executor.details.locale.to_string(),
                ),
            ]));
        self.executor.request(builder).await
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Default, Request, Available, FromId)]
#[from_id(multiple(Series))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Season {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    pub series_id: String,
    pub channel_id: String,

    pub title: String,
    pub slug_title: String,
    pub seo_title: String,
    pub description: String,
    pub seo_description: String,

    pub season_number: u32,

    pub is_complete: bool,

    pub keywords: Vec<String>,
    pub season_tags: Vec<String>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    // always empty (currently)
    pub audio_locales: Vec<Locale>,
    // always empty (currently)
    pub subtitle_locales: Vec<Locale>,
    // i have no idea what the difference between `audio_locales` and `audio_locale` should be.
    // they're both empty
    pub audio_locale: String,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    // currently empty (on all of my tests) but its might be filled in the future
    images: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_sequence_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    identifier: crate::StrictValue,
}
