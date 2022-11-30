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

**Please use this library via git in your Rust project. We're relying on a [development branch](https://github.com/sagebind/isahc/tree/tls-api-refactor) of [isahc](https://crates.io/crates/isahc) which is why it currently can't be published to [crates.io](https://crates.io/).**

## Documentation

~~The documentation is available at [docs.rs](https://docs.rs/crunchyroll-rs/).~~

Documentation of the latest commit can be found [here](https://crunchy-labs.github.io/crunchyroll-rs/crunchyroll_rs/).

## Example

You need this crate and [tokio](https://github.com/tokio-rs/tokio) as dependency in your Cargo.toml in order to start working:
```toml
[dependencies]
crunchyroll-rs = { git = "https://github.com/crunchy-labs/crunchyroll-rs" }
tokio = { version = "1.22", features = ["full"] }
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

## Development

#### Windows
To get this library working on Windows a manual build of [OpenSSL](https://www.openssl.org/) is required which involves some extra steps.
In order to work this library _must_ use TLSv1.3 (to bypass the Crunchyroll Bot Check) but cannot the default Windows SSL/TLS library does not support TLSv1.3.
[rustls](https://github.com/rustls/rustls) was also considered as a replacement for OpenSSL but unfortunately the Bot Check cannot be bypassed even though TLSv1.3 is supported by it.
See crunchy-labs/crunchy-cli#74 for more information about the initial OpenSSL issue.

This installs openssl via [vcpkg](https://vcpkg.io) and makes it available for Rust (in Powershell):
```shell 
$ git clone https://github.com/Microsoft/vcpkg.git
$ cd vcpkg
$ .\bootstrap-vcpkg.bat
$ .\vcpkg.exe integrate install
$ .\vcpkg.exe install openssl:x64-windows-static-md
$ $env:CFLAGS="-I$pwd\packages\openssl_x64-windows-static-md\include"
```
The last line sets the path to the openssl headers.
This is only temporary for your shell session, it is recommended to set this environment variable in your code editor or somewhere where it can be globally accessed.

The same steps _must_ also be done if you try to use this library in a project and want to export it for Windows.

#### Linux
You need the openssl development package to compile this crate successfully.
They will probably be distributed with your distros package manager.

Arch
```shell
$ pacman -S openssl
```

Debian
```shell
$ apt install libssl-dev
```

Fedora
```shell
$ dnf install openssl-devel
```

Alpine
```shell
$ apk add openssl-dev
```

#### Api Coverage
Crunchyroll regularly updates their api but does not provide any documentation for it.
Because we do not monitor the api constantly, so we cannot immediately say when a new endpoint is added or something has changed on already existing and implemented endpoints (which is semi-covered by the `__test-strict` feature, at least).
If you find an endpoint which is not implemented or has changes feel free to open a new [issue](https://github.com/crunchy-labs/crunchyroll-rs/issues) and tell us, or fork the library and implement it yourself.

## License

This project is licensed under either of the following licenses, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
