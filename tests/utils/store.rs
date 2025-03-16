#![allow(dead_code, unused_must_use, clippy::complexity)]

use anyhow::bail;
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::{env, fs};

pub struct Store<T> {
    get_fn: fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>>,
    value: OnceLock<anyhow::Result<T>>,
}

impl<T> Store<T> {
    pub const fn new(
        function: fn() -> Pin<Box<dyn Future<Output = anyhow::Result<T>>>>,
    ) -> Store<T> {
        Store {
            get_fn: function,
            value: OnceLock::new(),
        }
    }

    pub async fn get(&self) -> anyhow::Result<&T> {
        if self.value.get().is_none() {
            let function = self.get_fn.clone();
            let value = function().await;
            self.value.set(value);
        }

        let value = self.value.get().unwrap();
        match value {
            Ok(t) => Ok(t),
            Err(err) => bail!(err.to_string()),
        }
    }

    pub async fn get_mut(&mut self) -> anyhow::Result<&mut T> {
        if self.value.get().is_none() {
            let function = self.get_fn.clone();
            let value = function().await;
            self.value.set(value);
        }

        let value = self.value.get_mut().unwrap();
        match value {
            Ok(t) => Ok(t),
            Err(err) => bail!(err.to_string()),
        }
    }
}

pub fn get_store(key: String) -> Result<String, std::io::Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{key}"));
    fs::read_to_string(path)
}

pub fn set_store(key: String, value: String) -> Result<(), std::io::Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{key}"));
    fs::write(path, value)?;
    Ok(())
}

pub fn has_store(key: String) -> bool {
    env::temp_dir()
        .join(format!(".crunchyroll-rs.{key}"))
        .exists()
}
