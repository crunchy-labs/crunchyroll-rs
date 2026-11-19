//! Type alias for date & time types. Needed as both [jiff](https://docs.rs/jiff) and [chrono](https://docs.rs/chrono) are supported time crates.

cfg_if::cfg_if! {
    if #[cfg(feature = "chrono")] {
        pub type DateTime = chrono::DateTime<chrono::Utc>;
        pub type Duration = chrono::Duration;

        pub(crate) fn now() -> DateTime {
            chrono::Utc::now()
        }

        pub(crate) fn epoch() -> DateTime {
            chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::UNIX_EPOCH)
        }

        pub(crate) fn add_seconds(date: DateTime, seconds: i64) -> DateTime {
            date.checked_add_signed(chrono::Duration::try_seconds(seconds).unwrap())
                .unwrap()
        }

        pub(crate) fn timestamp_millis(date: DateTime) -> i64 {
            date.timestamp_millis()
        }

        pub(crate) fn duration_from_millis(millis: i64) -> Duration {
            chrono::Duration::try_milliseconds(millis).unwrap()
        }

        pub(crate) fn duration_to_millis(duration: &Duration) -> i64 {
            duration.num_milliseconds()
        }
    } else {
        pub type DateTime = jiff::Timestamp;
        pub type Duration = jiff::SignedDuration;

        pub(crate) fn now() -> DateTime {
            jiff::Timestamp::now()
        }

        pub(crate) fn epoch() -> DateTime {
            jiff::Timestamp::UNIX_EPOCH
        }

        pub(crate) fn add_seconds(date: DateTime, seconds: i64) -> DateTime {
            jiff::Timestamp::from_second(date.as_second().saturating_add(seconds)).unwrap()
        }

        pub(crate) fn timestamp_millis(date: DateTime) -> i64 {
            date.as_millisecond()
        }

        pub(crate) fn duration_from_millis(millis: i64) -> Duration {
            jiff::SignedDuration::from_millis(millis)
        }

        pub(crate) fn duration_to_millis(duration: &Duration) -> i64 {
            i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
        }
    }
}
