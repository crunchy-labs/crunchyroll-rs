use crate::common::Image;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(try_from = "Map<String, Value>")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ThumbnailImages {
    pub thumbnail: Vec<Image>,
}

impl TryFrom<Map<String, Value>> for ThumbnailImages {
    type Error = serde_json::Error;

    fn try_from(value: Map<String, Value>) -> Result<Self, Self::Error> {
        if let Some(thumbnail) = value.get("thumbnail") {
            if let Ok(thumb) = serde_json::from_value::<Vec<Vec<Image>>>(thumbnail.clone()) {
                Ok(ThumbnailImages {
                    thumbnail: thumb.into_iter().flatten().collect::<Vec<Image>>(),
                })
            } else {
                Ok(ThumbnailImages {
                    thumbnail: serde_json::from_value(thumbnail.clone())?,
                })
            }
        } else {
            Ok(ThumbnailImages { thumbnail: vec![] })
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(try_from = "Map<String, Value>")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PosterImages {
    pub poster_tall: Vec<Image>,
    pub poster_wide: Vec<Image>,
}

impl TryFrom<Map<String, Value>> for PosterImages {
    type Error = serde_json::Error;

    fn try_from(value: Map<String, Value>) -> Result<Self, Self::Error> {
        let tall = if let Some(tall) = value.get("poster_tall") {
            if let Ok(img) = serde_json::from_value::<Vec<Vec<Image>>>(tall.clone()) {
                img.into_iter().flatten().collect::<Vec<Image>>()
            } else {
                serde_json::from_value(tall.clone())?
            }
        } else {
            vec![]
        };
        let wide = if let Some(wide) = value.get("poster_wide") {
            if let Ok(img) = serde_json::from_value::<Vec<Vec<Image>>>(wide.clone()) {
                img.into_iter().flatten().collect::<Vec<Image>>()
            } else {
                serde_json::from_value(wide.clone())?
            }
        } else {
            vec![]
        };

        Ok(Self {
            poster_tall: tall,
            poster_wide: wide,
        })
    }
}
