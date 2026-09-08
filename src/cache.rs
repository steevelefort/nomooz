use std::path::PathBuf;

pub fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .unwrap_or_else(|_| format!("{}/.cache", std::env::var("HOME").expect("HOME not set")));
    PathBuf::from(base).join(env!("CARGO_PKG_NAME"))
}

pub fn cache_data(filename: &str, data: String) -> Result<(), std::io::Error> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(filename), data)?;
    Ok(())
}

pub fn load_cache(filename: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(cache_dir().join(filename))
}
