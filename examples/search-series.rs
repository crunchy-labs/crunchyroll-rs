use anyhow::Result;
use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::common::StreamExt;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(email, password)
        .await?;

    let mut query_result = crunchyroll.query("darling");
    while let Some(s) = query_result.series.next().await {
        let series = s?;

        println!(
            "Queried series {} which has {} seasons",
            series.title, series.season_count
        );
        let seasons = series.seasons().await?;
        for season in seasons {
            println!(
                "Found season {} with audio locale(s) {}",
                season.season_number,
                season
                    .audio_locales
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }

    Ok(())
}
