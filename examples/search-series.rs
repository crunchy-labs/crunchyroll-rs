use anyhow::Result;
use crunchyroll_rs::search::QueryOptions;
use crunchyroll_rs::Crunchyroll;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await?;

    let options = QueryOptions::default()
        // return 2o items max
        .limit(20);
    let result = crunchyroll.query("darling".into(), options).await?;

    let series = result.series.unwrap();
    for s in series.items {
        println!(
            "Queried series {} which has {} seasons",
            s.title, s.metadata.season_count
        );
        let seasons = s.seasons().await?;
        for season in seasons.items {
            println!(
                "Found season {} with audio locale(s) {}",
                season.metadata.season_number,
                season
                    .metadata
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
