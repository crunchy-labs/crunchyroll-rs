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
  <a href="https://crates.io/crates/crunchyroll-rs">
    <img src="https://img.shields.io/crates/d/crunchyroll-rs" alt="crates.io downloads">
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

## Documentation

The documentation is available at [docs.rs](https://docs.rs/crunchyroll-rs/).

## Example

You need this crate and [tokio](https://github.com/tokio-rs/tokio) as dependency in your Cargo.toml in order to start working:
```toml
[dependencies]
crunchyroll-rs = "0.19"
tokio = { version = "1", features = ["full"] }
```

The following code prints the data of the episode behind the given url:

```rust
use crunchyroll_rs::{Crunchyroll, MediaCollection};
use crunchyroll_rs::parse::UrlType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log in to Crunchyroll with the etp_rt cookie of your browser session
    let crunchyroll = Crunchyroll::builder()
        .login_with_etp_rt("<etp_rt cookie>", Default::default())
        .await?;

    let url = crunchyroll_rs::parse_url("https://www.crunchyroll.com/watch/GRDQPM1ZY/alone-and-lonesome").expect("url is not valid");
    if let UrlType::EpisodeOrMovie(media_id) = url {
        if let MediaCollection::Episode(episode) = crunchyroll.media_collection_from_id(media_id).await? {
            println!(
                "Url is episode {} ({}) of {} season {}",
                episode.episode_number,
                episode.title,
                episode.series_title,
                episode.season_number
            )
        }
    } else {
        panic!("Url is not a episode")
    }

    Ok(())
}
```

_More examples can be found in the [examples/](examples) directory._

#### Login

- `login_with_etp_rt`: log in with the `etp_rt` cookie that is set when logging in via the browser (used in the example above; **recommended**)
- `login_with_oauth_code`: SSO/OAuth flow via Crunchyroll's login page, see the [sso-login example](examples/sso-login)
- `login_with_refresh_token` / `login_with_refresh_token_profile_id`: re-login with a refresh token obtained from one of the other login methods
- `login_anonymously`: log in without an account

> Direct login with email and password isn't possible anymore, this must now be done by using the SSO flow, which requires a webview.

#### API coverage
Crunchyroll regularly updates their api but does not provide any documentation for it.
Because we do not monitor the api constantly, so we cannot immediately say when a new endpoint is added or something has changed on already existing and implemented endpoints (which is semi-covered by the `__test-strict` feature, at least).
If you find an endpoint which is not implemented or has changes feel free to open a new [issue](https://github.com/crunchy-labs/crunchyroll-rs/issues) and tell us, or fork the library and implement it yourself.

#### API credentials
To issue sessions, credentials like the basic auth token and user agent of a Crunchyroll app are needed. By default, sessions are issued with the basic auth token and user agent of the web chrome client, which are bundled with the library. If you want to issue sessions for another platform, for example android tv, fetch the credentials yourself, either with `auth::platform_credentials::get_platform_credentials` and some sort of caching(!), or implement it completely yourself. You can then pass them via `CrunchyrollBuilder::platform`.

#### Cloudflare
Crunchyroll uses the cloudflare bot protection to detect if requests are made by a human. Obviously this crate makes
automated requests and thus, Cloudflare sometimes blocks requests. The crate catches these errors with the `error::CrunchyrollError::Block` enum field.
The block occurs depending on different factors like your location.
If such a block occurs you can try to create a custom `reqwest::Client` which has the needed configuration to bypass this check, like other user agents or
tls backends (note that [reqwest](https://github.com/seanmonstar/reqwest) currently only supports
[native-tls](https://github.com/sfackler/rust-native-tls) besides [rustls](https://github.com/rustls/rustls) as tls
backend, which is confirmed to work with openssl on Linux only, on Windows the blocks are even more aggressive). The
configurations may vary on the factors addressed so there is no 100% right way to do it.

## Disclaimer
This library is unofficial and not affiliated with Crunchyroll in any way.

**You are entirely responsible for what happens when you use crunchyroll-rs.**

## License

This project is licensed under either of the following licenses, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
