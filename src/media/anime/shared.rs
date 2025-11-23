use crate::{Request, enum_values};
use chrono::{DateTime, Utc};
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Skippable event like intro or credits.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SkipEventsEvent {
    /// Start of the event in seconds.
    pub start: f32,
    /// End of the event in seconds.
    pub end: f32,

    #[cfg(feature = "__test_strict")]
    approver_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    distribution_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    title: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    series_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    new: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    r#type: crate::StrictValue,
}

/// Information about skippable events like an intro or credits.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[serde(remote = "Self")]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SkipEvents {
    #[serde(default)]
    pub recap: Option<SkipEventsEvent>,
    #[serde(default)]
    pub intro: Option<SkipEventsEvent>,
    #[serde(default)]
    pub credits: Option<SkipEventsEvent>,
    #[serde(default)]
    pub preview: Option<SkipEventsEvent>,

    #[cfg(feature = "__test_strict")]
    media_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    last_updated: crate::StrictValue,
}

impl<'de> Deserialize<'de> for SkipEvents {
    fn deserialize<D>(deserializer: D) -> crate::error::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut as_map = serde_json::Map::deserialize(deserializer)?;

        let objects_to_check = ["recap", "intro", "credits", "preview"];
        for object in objects_to_check {
            let Some(obj) = as_map.get(object) else {
                continue;
            };
            if obj.as_object().is_some_and(|o| o.is_empty())
                // crunchyroll sometimes has a skip events, but it's lacking start or end times.
                // this is just abstracted away since an event without a start or end doesn't make
                // sense to be wrapped in e.g. an Option
                || obj.get("start").unwrap_or(&Value::Null).is_null()
                || obj.get("end").unwrap_or(&Value::Null).is_null()
                // it might also be the case that the end of an event is lower than its start. this
                // logic error is also abstracted away
                || obj.get("start").unwrap().as_f64().unwrap() > obj.get("end").unwrap().as_f64().unwrap()
            {
                as_map.remove(object);
            }
        }

        SkipEvents::deserialize(
            serde_json::to_value(as_map)
                .map_err(|e| Error::custom(e.to_string()))?
                .into_deserializer(),
        )
        .map_err(|e| Error::custom(e.to_string()))
    }
}

/// Media related to the media which queried this struct.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct RelatedMedia<T: Request + DeserializeOwned> {
    pub fully_watched: bool,

    pub playhead: u32,

    #[serde(alias = "panel")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_panel")]
    pub media: T,

    /// Only populated if called with [`Episode::next`] or [`Movie::next`].
    pub shortcut: Option<bool>,
}

/// Information about the playhead of an [`Episode`] or [`Movie`].
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlayheadInformation {
    pub playhead: u32,

    pub content_id: String,

    pub fully_watched: bool,

    /// Date when the last playhead update was
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub last_modified: DateTime<Utc>,
}

enum_values! {
    /// Starts a rating can have. Crunchyroll does not use simple numbers which would be much easier
    /// to work with but own names for every star.
    pub enum RatingStar {
        OneStar = "1s"
        TwoStars = "2s"
        ThreeStars = "3s"
        FourStars = "4s"
        FiveStars = "5s"
    }
}

/// Details about a star rating of [`crate::media::Rating`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct RatingStarDetails {
    /// The amount of user ratings.
    pub displayed: String,
    /// If [`crate::media::RatingStarDetails::displayed`] is > 1000 it gets converted from a normal integer to a
    /// float. E.g. 1700 becomes 1.7. [`crate::media::RatingStarDetails::unit`] is then `K` (= representing
    /// a thousand). If its < 1000, [`crate::media::RatingStarDetails::unit`] is just an empty string.
    pub unit: String,

    /// How many percent of user voted this star. Only populated if this struct is obtained via
    /// [`crate::media::Rating`].
    pub percentage: Option<u8>,
}

/// Overview about rating statistics for a series or movie listing.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Rating {
    #[serde(alias = "1s")]
    pub one_star: RatingStarDetails,
    #[serde(alias = "2s")]
    pub two_stars: RatingStarDetails,
    #[serde(alias = "3s")]
    pub three_stars: RatingStarDetails,
    #[serde(alias = "4s")]
    pub four_stars: RatingStarDetails,
    #[serde(alias = "5s")]
    pub five_stars: RatingStarDetails,

    pub total: u32,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_try_from_string")]
    pub average: f64,

    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub rating: Option<RatingStar>,
}

/// Information about an ad break. Ad breaks are only present with non-premium accounts.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdBreak {
    pub offset_ms: u32,
    /// Type of the add. As far as I can see, can be 'preroll' and 'midroll'
    #[serde(rename = "type")]
    pub ad_type: String,
}
