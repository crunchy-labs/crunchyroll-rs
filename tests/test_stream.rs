#![cfg(feature = "stream")]

use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::{VariantData, VariantSegment, VideoStream};
use rand::seq::SliceRandom;

mod utils;

static STREAM: Store<VideoStream> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let stream = VideoStream::from_id(crunchy, "G4GFQP0WM".into()).await?;
        Ok(stream)
    })
});
static STREAM_DATA: Store<VariantData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM.get().await?;
        let mut default_streams = stream.streaming_data(None).await?;

        default_streams.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width));

        Ok(default_streams.get(0).unwrap().clone())
    })
});
static STREAM_SEGMENTS: Store<Vec<VariantSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = &mut STREAM_DATA.get().await?.clone();
        let segments = stream_data.segments().await?;

        Ok(segments)
    })
});

#[tokio::test]
async fn stream_from_id() {
    assert_result!(STREAM.get().await)
}

#[tokio::test]
async fn stream_data() {
    assert_result!(STREAM_DATA.get().await)
}

#[tokio::test]
async fn stream_segments() {
    assert_result!(STREAM_SEGMENTS.get().await)
}

#[tokio::test]
async fn stream_process_segments() {
    let segments = STREAM_SEGMENTS.get().await.unwrap();

    let sink = &mut std::io::sink();

    // stream 10 random segments.
    // if the test passes, it's unlikely that some error will occur when streaming all segments (
    // and if it does, hopefully someone using this in production will report it)
    for _ in 0..10 {
        assert_result!(
            segments
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone()
                .write_to(sink)
                .await
        );
    }
}
