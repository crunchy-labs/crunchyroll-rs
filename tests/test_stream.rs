use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::{Media, Stream, VariantData, VariantSegment};
use crunchyroll_rs::Episode;
use rand::seq::SliceRandom;

mod utils;

static STREAM: Store<Stream> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let stream = Episode::from_id(crunchy, "GRDKJZ81Y")
            .await?
            .streams()
            .await?;
        Ok(stream)
    })
});

#[cfg(feature = "hls-stream")]
static STREAM_HLS_DATA: Store<VariantData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM.get().await?;
        let mut hls_streams = stream.hls_streaming_data(None).await?;

        hls_streams.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width));

        Ok(hls_streams[0].clone())
    })
});
#[cfg(feature = "dash-stream")]
static STREAM_DASH_DATA: Store<VariantData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM.get().await?;
        let mut dash_streams = stream.dash_streaming_data(None).await?.0;

        dash_streams.sort_by(|a, b| a.resolution.width.cmp(&b.resolution.width));

        Ok(dash_streams[0].clone())
    })
});

#[cfg(feature = "hls-stream")]
static STREAM_HLS_SEGMENTS: Store<Vec<VariantSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = STREAM_HLS_DATA.get().await?;
        let segments = stream_data.segments().await?;

        Ok(segments)
    })
});
#[cfg(feature = "dash-stream")]
static STREAM_DASH_SEGMENTS: Store<Vec<VariantSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = STREAM_DASH_DATA.get().await?;
        let segments = stream_data.segments().await?;

        Ok(segments)
    })
});

#[tokio::test]
async fn stream_from_id() {
    assert_result!(STREAM.get().await)
}

#[cfg(feature = "hls-stream")]
#[tokio::test]
async fn stream_hls_data() {
    assert_result!(STREAM_HLS_DATA.get().await)
}

#[cfg(feature = "dash-stream")]
#[tokio::test]
async fn stream_dash_data() {
    assert_result!(STREAM_DASH_DATA.get().await)
}

#[cfg(feature = "hls-stream")]
#[tokio::test]
async fn stream_hls_segments() {
    assert_result!(STREAM_HLS_SEGMENTS.get().await)
}

#[cfg(feature = "dash-stream")]
#[tokio::test]
async fn stream_dash_segments() {
    assert_result!(STREAM_DASH_SEGMENTS.get().await)
}

#[cfg(feature = "hls-stream")]
#[tokio::test]
async fn process_hls_segments() {
    let segments = STREAM_HLS_SEGMENTS.get().await.unwrap();

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

#[cfg(feature = "dash-stream")]
#[tokio::test]
async fn process_dash_segments() {
    let segments = STREAM_DASH_SEGMENTS.get().await.unwrap();

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

#[tokio::test]
async fn stream_versions() {
    assert_result!(STREAM.get().await.unwrap().versions().await)
}
