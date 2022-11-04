<div align="center">
  <h1>crunchyroll-rs</h1>
  <p>
    <strong>A <a href="https://www.rust-lang.org/">Rust</a> library for the undocumented <a href="https://www.crunchyroll.com/">Crunchyroll</a> api.</strong>
  </p>
</div>

<p align="center">
  <img src="https://raw.githubusercontent.com/crunchy-labs/resources/main/crunchyroll-rs.svg" width="460">
</p>

<p align="center">
  <a href="https://crates.io/crates/crunchyroll-rs">
    <img src="https://img.shields.io/crates/v/crunchyroll-rs" alt="crates.io">
  </a>
  <a href="https://docs.rs/crunchyroll-rs/">
    <img src="https://img.shields.io/docsrs/crunchyroll-rs" alt="Docs">
  </a>
  <a href="https://github.com/crunchy-labs/crunchyroll-rs/actions/workflows/ci.yml">
    <img src="https://github.com/crunchy-labs/crunchyroll-rs/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="#license">
    <img src="https://img.shields.io/crates/l/crunchyroll-rs" alt="License">
  </a>
  <a href="https://discord.gg/PXGPGpQxgk">
    <img src="https://img.shields.io/discord/994882878125121596?logo=discord&logoColor=ffffff" alt="Discord">
  </a>
</p>


> We are in no way affiliated with, maintained, authorized, sponsored, or officially associated with Crunchyroll LLC or any of its subsidiaries or affiliates.
> The official Crunchyroll website can be found at https://crunchyroll.com/.

**Please use this library via git in your Rust project. We're relying on a [development branch](https://github.com/sagebind/isahc/tree/tls-api-refactor) of [isahc](https://crates.io/crates/isahc) which is why it currently can't be published to crates.io.**

## Documentation

The documentation is available at [docs.rs](https://docs.rs/crunchyroll-rs/).

## Example

You need this crate and [tokio](https://github.com/tokio-rs/tokio) as dependency in your Cargo.toml in order to start working:
```toml
[dependencies]
crunchyroll-rs = "0.1"
tokio = { version = "1.21", features = ["full"] }
```

The following code prints the data of the episode behind the given url:

```rust
use crunchyroll_rs::{Crunchyroll, MediaCollection};
use crunchyroll_rs::parse::UrlType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // log in to crunchyroll with your username and password
    let crunchyroll = Crunchyroll::builder()
        .login_with_credentials("<username>", "<password>")
        .await?;

    let url = Crunchyroll::parse_url("https://www.crunchyroll.com/watch/GRDQPM1ZY/alone-and-lonesome")?;
    if let UrlType::EpisodeOrMovie(media_id) = url {
        match crunchyroll.media_collection_from_id(media_id).await? {
            MediaCollection::Episode(episode) => {
                println!(
                    "Url is episode {} ({}) of season {} from {}",
                    episode.metadata.episode_number,
                    episode.title, 
                    episode.metadata.season_number,
                    episode.metadata.series_title
                )
            }
            _ => ()
        }
    } else {
        panic!("Url is not a episode")
    }

    Ok(())
}
```

_More examples can be found in the [examples/](examples) directory._

## Api Coverage

Crunchyroll regularly updates their api but does not provide any documentation for it.
Because we do not monitor the api constantly, so we cannot immediately say when a new endpoint is added or something has changed on already existing and implemented endpoints (which is semi-covered by the `__test-strict` feature, at least).
If you find an endpoint which is not implemented or has changes feel free to open a new [issue](https://github.com/crunchy-labs/crunchyroll-rs/issues) and tell us, or fork the library and implement it yourself.

## License

This project is licensed under either of the following licenses, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
