<div align="center">
  <h1>crunchyroll-rs</h1>
  <p>
    <strong>A <a href="https://www.rust-lang.org/">Rust</a> library for the undocumented <a href="https://www.crunchyroll.com/">Crunchyroll</a> api</strong>
  </p>
</div>

<p align="center">
  <a href="#license">
    <img src="https://img.shields.io/crates/l/crunchyroll-rs" alt="License">
  </a>
  <a href="https://discord.gg/PXGPGpQxgk">
    <img src="https://img.shields.io/discord/994882878125121596?logo=discord&logoColor=ffffff" alt="Discord">
  </a>
  <a href="https://github.com/crunchy-labs/crunchyroll-rs/actions">
    <img src="https://github.com/crunchy-labs/crunchyroll-rs/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
</p>


> We are in no way affiliated with, maintained, authorized, sponsored, or officially associated with Crunchyroll LLC or any of its subsidiaries or affiliates.
> The official Crunchyroll website can be found at https://crunchyroll.com/.

## Features

- Full [Tokio](https://tokio.rs/) compatibility

## Get Started

External documentation (on [docs.rs](https://docs.rs/)) is currently not available since we have no release yet.

Because this library has no stable release yet and is under heavy development it must be added as a git dependency:
```toml
[dependencies]
crunchyroll-rs = { git = "https://github.com/crunchy-labs/crunchyroll-rs" }
tokio = { version = "1.20", features = ["full"] }
```

## License

This project is licensed under either of the following licenses, at your option:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
