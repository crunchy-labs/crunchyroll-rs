use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::{Media, Stream, StreamData, StreamSegment};
use crunchyroll_rs::Episode;
use rand::seq::SliceRandom;
use std::io::Write;

mod utils;

static STREAM_DRM: Store<Stream> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let stream = Episode::from_id(crunchy, "GRDKJZ81Y")
            .await?
            .stream()
            .await?;
        Ok(stream)
    })
});

static STREAM_DATA_DRM: Store<StreamData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM_DRM.get().await?;
        Ok(stream.stream_data(None).await?.unwrap().0.remove(0))
    })
});

static STREAM_SEGMENTS_DRM: Store<Vec<StreamSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = STREAM_DATA_DRM.get().await?;
        Ok(stream_data.segments())
    })
});

static STREAM_MAYBE_WITHOUT_DRM: Store<Stream> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let stream = Episode::from_id(crunchy, "GRDKJZ81Y")
            .await?
            .stream_maybe_without_drm()
            .await?;
        Ok(stream)
    })
});

static STREAM_DATA_MAYBE_WITHOUT_DRM: Store<StreamData> = Store::new(|| {
    Box::pin(async {
        let stream = STREAM_MAYBE_WITHOUT_DRM.get().await?;
        Ok(stream.stream_data(None).await?.unwrap().0.remove(0))
    })
});

static STREAM_SEGMENTS_MAYBE_WITHOUT_DRM: Store<Vec<StreamSegment>> = Store::new(|| {
    Box::pin(async {
        let stream_data = STREAM_DATA_MAYBE_WITHOUT_DRM.get().await?;
        Ok(stream_data.segments())
    })
});

#[tokio::test]
async fn stream_from_id_drm() {
    assert_result!(STREAM_DRM.get().await)
}

#[tokio::test]
async fn stream_data_drm() {
    assert_result!(STREAM_DATA_DRM.get().await)
}

#[tokio::test]
async fn stream_segments_drm() {
    assert_result!(STREAM_SEGMENTS_DRM.get().await)
}

#[tokio::test]
async fn process_segments_drm() {
    let segments = STREAM_SEGMENTS_DRM.get().await.unwrap();

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

#[tokio::test]
async fn stream_from_id_maybe_without_drm() {
    assert_result!(STREAM_MAYBE_WITHOUT_DRM.get().await)
}

#[tokio::test]
async fn stream_data_maybe_without_drm() {
    assert_result!(STREAM_DATA_MAYBE_WITHOUT_DRM.get().await)
}

#[tokio::test]
async fn stream_segments_maybe_without_drm() {
    assert_result!(STREAM_SEGMENTS_MAYBE_WITHOUT_DRM.get().await)
}

#[tokio::test]
async fn process_segments_maybe_without_drm() {
    let segments = STREAM_SEGMENTS_MAYBE_WITHOUT_DRM.get().await.unwrap();

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
async fn stream_versions_maybe_without_drm() {
    assert_result!(
        STREAM_MAYBE_WITHOUT_DRM
            .get()
            .await
            .unwrap()
            .versions()
            .await
    )
}*/

#[tokio::test]
async fn stream_maybe_without_drm_is_really_drm_free() {
    assert!(STREAM_DATA_MAYBE_WITHOUT_DRM
        .get()
        .await
        .unwrap()
        .drm
        .is_none())
}
