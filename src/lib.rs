mod crunchyroll;
mod error;
mod internal;

#[cfg(all(test, feature = "__test_strict"))]
use internal::strict::StrictValue;

pub use crunchyroll::Crunchyroll;
pub use crunchyroll::Locale;
