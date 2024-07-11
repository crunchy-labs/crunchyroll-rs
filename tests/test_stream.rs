use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::{Media, Stream, StreamData, StreamSegment};
use crunchyroll_rs::Episode;
use rand::seq::SliceRandom;
use std::io::Write;

mod utils;

static STREAM: Store<Stream> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let stream = Episode::from_id(crunchy, "GRDKJZ81Y")
            .await?
            .stream()
            .await?;
        Ok(stream)
    })
});

static STREAM_DATA: Store<StreamData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM.get().await?;
        Ok(stream.stream_data(None).await?.unwrap().0.remove(0))
    })
});

static STREAM_SEGMENTS: Store<Vec<StreamSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = STREAM_DATA.get().await?;
        Ok(stream_data.segments())
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
async fn process_segments() {
    let segments = STREAM_SEGMENTS.get().await.unwrap();

    let sink = &mut std::io::sink();

    // stream 10 random segments.
    // if the test passes, it's unlikely that some error will occur when streaming all segments (
    // and if it does, hopefully someone using this in production will report it)
    for _ in 0..10 {
        sink.write(
            &segments
                .choose(&mut rand::thread_rng())
                .unwrap()
                .data()
                .await
                .unwrap(),
        )
        .unwrap();
    }
}

// will throw a too many active streams error
/*#[tokio::test]
async fn stream_versions_drm() {
    assert_result!(STREAM_DRM.get().await.unwrap().versions().await)
}*/
