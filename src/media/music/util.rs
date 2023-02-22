use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};

/// Extract the `availability` object from the provided map and expand its two values into `obj` as
/// `availabilityStarts` and `availabilityEnds` to keep consistency between the structs.
/// [`crate::Episode`] for example has [`crate::Episode::availability_starts`] and
/// [`crate::Episode::availability_ends`] as field instead of a `availability` object.
pub(crate) fn availability_object_to_keys(
    obj: &mut Map<String, Value>,
) -> Result<(), serde_json::Error> {
    #[derive(Deserialize, smart_default::SmartDefault)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct Availability {
        #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
        start_date: DateTime<Utc>,
        #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
        end_date: DateTime<Utc>,
    }

    if let Some(value) = obj.remove("availability") {
        let availability: Availability = serde_json::from_value(value)?;
        obj.insert(
            "availabilityStarts".to_string(),
            serde_json::to_value(availability.start_date)?,
        );
        obj.insert(
            "availabilityEnds".to_string(),
            serde_json::to_value(availability.end_date)?,
        );
    }

    Ok(())
}
