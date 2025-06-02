use anyhow::Result;
use crunchyroll_rs::{Crunchyroll, Episode};
use std::env;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(email, password)
        .await?;

    let episode: Episode = crunchyroll.media_from_id("GRDKJZ81Y").await?;
    let stream = episode.stream().await?;
    let mut stream_data = stream.stream_data(None).await?.unwrap();
    // sort after resolutions; best to worst
    stream_data.video.sort_by(|a, b| {
        a.resolution()
            .unwrap()
            .width
            .cmp(&b.resolution().unwrap().width)
            .reverse()
    });

    // get video segments of the stream with the best available resolution
    let segments = stream_data.video[0].segments();

    let sink = &mut std::io::sink();
    for (i, segment) in segments.iter().enumerate() {
        println!("Downloading segment {} of {}", i + 1, segments.len());
        sink.write_all(&segment.data().await?)?;
    }

    Ok(())
}
