use std::{env, fs};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use once_cell::sync::OnceCell;

pub struct Store<T> {
    get_fn: fn() -> Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>>>>,
    value: OnceCell<T>
}

impl<T> Store<T> {
    pub const fn new(function: fn() -> Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>>>>) -> Store<T> {
        Store {
            get_fn: function,
            value: OnceCell::new()
        }
    }

    pub async fn get(&self) -> Result<&T, Box<dyn Error>> {
        if let Some(value) = self.value.get() {
            Ok(value)
        } else {
            let function = self.get_fn.clone();
            let value = function().await?;
            self.value.set(value);
            Ok(self.value.get().unwrap())
        }
    }
}

pub fn get_store(key: String) -> Result<String, std::io::Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{}", key));
    fs::read_to_string(path)
}

pub fn set_store(key: String, value: String) -> Result<(), std::io::Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{}", key));
    fs::write(path, value)?;
    Ok(())
}

pub fn has_store(key: String) -> bool {
    env::temp_dir().join(format!(".crunchyroll-rs.{}", key)).exists()
}
