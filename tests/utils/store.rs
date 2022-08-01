use std::{env, fs};
use std::io::Error;

pub fn get_store(key: String) -> Result<String, Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{}", key));
    fs::read_to_string(path)
}

pub fn set_store(key: String, value: String) -> Result<(), Error> {
    let path = env::temp_dir().join(format!(".crunchyroll-rs.{}", key));
    fs::write(path, value)?;
    Ok(())
}

pub fn has_store(key: String) -> bool {
    env::temp_dir().join(format!(".crunchyroll-rs.{}", key)).exists()
}
