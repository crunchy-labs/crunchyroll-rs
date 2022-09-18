#[allow(clippy::module_inception)] // naming is difficult
mod media;
mod old_media;
mod stream;
mod streaming;

pub use media::*;
pub use stream::*;
#[cfg(feature = "stream")]
pub use streaming::*;

use crate::enum_values;
enum_values! {
    pub enum MediaType {
        Series = "series"
        Movie = "movie_listing"
    }
}
