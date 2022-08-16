use rand::seq::SliceRandom;
use crunchyroll_rs::{FromId, VideoVariants, Stream, VariantData, VariantSegment};
use crate::utils::SESSION;
use crate::utils::Store;

mod utils;

static STREAM: Store<Stream<VideoVariants>> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let stream = Stream::<VideoVariants>::from_id(crunchy, "G4GFQP0WM".to_string())
        .await?;
    Ok(stream)
}));
static STREAM_DATA: Store<VariantData> = Store::new(|| Box::pin(async {
    let stream = STREAM.get().await?;
    let mut default_streams = stream.default_streams().await?;

    default_streams
        .sort_by(|a, b| a.resolution.width.cmp(&b.resolution.height).reverse());

    Ok(default_streams.get(0).unwrap().clone())
}));
static STREAM_SEGMENTS: Store<Vec<VariantSegment>> = Store::new(|| Box::pin(async {
    let stream_data = &mut STREAM_DATA.get().await?.clone();
    let segments = stream_data.segments().await?;

    Ok(segments)
}));

#[tokio::test]
async fn stream_from_id() {
    let stream = STREAM.get().await;

    assert!(stream.is_ok(), "{}", stream.unwrap_err())
}

#[tokio::test]
async fn stream_data() {
    let stream_data = STREAM_DATA.get().await;

    assert!(stream_data.is_ok(), "{}", stream_data.unwrap_err())
}

#[tokio::test]
async fn stream_segments() {
    let segments = STREAM_SEGMENTS.get().await;

    assert!(segments.is_ok(), "{}", segments.unwrap_err());
}

#[tokio::test]
async fn stream_process_segments() {
    let segments = STREAM_SEGMENTS.get().await.unwrap();

    let sink = &mut std::io::sink();

    // stream 10 random segments.
    // if the test passes, it's unlikely that some error will occur when streaming all segments (
    // and if it does, hopefully someone using this in production will report it)
    for _ in 0..10 {
        let finished_segment = segments.choose(&mut rand::thread_rng())
            .unwrap()
            .clone()
            .write_to(sink)
            .await;
        assert!(finished_segment.is_ok(), "{}", finished_segment.unwrap_err());
    }
}
