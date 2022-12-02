use std::io::Read;

fn main() -> std::io::Result<()> {
    #[cfg(any(all(windows, target_env = "msvc"), feature = "static-certs"))]
    {
        let cacert = reqwest::blocking::get("https://curl.se/ca/cacert.pem")
            .unwrap()
            .bytes()
            .unwrap()
            .to_vec();

        std::fs::write(
            std::path::Path::new(
                &std::env::var("OUT_DIR").map_err(|_| std::io::ErrorKind::NotFound)?,
            )
            .join("cacert.pem"),
            cacert,
        )?
    }
    Ok(())
}
