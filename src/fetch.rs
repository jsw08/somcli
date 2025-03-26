use directories::ProjectDirs;
use std::fmt;
use std::fs::{create_dir_all, metadata, read_to_string, write};
use std::time::Duration;

#[derive(Debug)]
pub enum FetchErr {
    NoCacheDir,
    InvalidURL,
    CachePermission,
    HttpError,
    ParseError,
}
impl std::fmt::Display for FetchErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchErr::NoCacheDir => write!(
                f,
                "Unable to get the local cache directory. You might be on an unsupported system."
            ),
            FetchErr::CachePermission => write!(
                f,
                "Unable to read/write the cache file/directory. Please check your permissions."
            ),
            FetchErr::InvalidURL => write!(
                f,
                "Unable to parse the url, are you sure that it's correct? (please note that we only support somtoday's ical urls)."
            ),
            FetchErr::HttpError => write!(
                f,
                "Couldn't fetch the calendar data from somtoday's servers. Please check the url and your network connection."
            ),
            FetchErr::ParseError => write!(f, "Unable to read the fetched calendar data."),
        }
    }
}

impl std::error::Error for FetchErr {}

pub async fn fetch_calendar(url: &str) -> Result<(String, bool), FetchErr> {
    let Some(proj_dirs) = ProjectDirs::from("tf", "jsw", "somcli") else {
        return Err(FetchErr::NoCacheDir);
    };

    let cache_dir = proj_dirs.cache_dir();
    if let Err(_) = create_dir_all(cache_dir) {
        return Err(FetchErr::CachePermission);
    };

    let cache_file_path = match url.split("/").last() {
        Some(data) => cache_dir.join(data.to_owned() + ".ics"),
        None => return Err(FetchErr::InvalidURL),
    };

    enum UpdateCache {
        True,
        False,
        Old,
    }
    let update_cache = if cache_file_path.exists() {
        let cache_file_metadata =
            metadata(&cache_file_path).map_err(|_| FetchErr::CachePermission)?;
        let cache_file_modified = cache_file_metadata
            .modified()
            .map_err(|_| FetchErr::CachePermission)?;
        let cache_file_elapsed = cache_file_modified
            .elapsed()
            .map_err(|_| FetchErr::CachePermission)?;

        if cache_file_elapsed > Duration::from_secs(15 * 60) {
            UpdateCache::Old
        } else {
            UpdateCache::False
        }
    } else {
        UpdateCache::True
    };

    let fetch_and_update = async || -> Result<(String, bool), FetchErr> {
        let res = reqwest::get(url).await.map_err(|_| FetchErr::HttpError)?;

        let text = res.text().await.map_err(|_| FetchErr::ParseError)?;

        write(&cache_file_path, &text).map_err(|_| FetchErr::CachePermission)?;

        Ok((text, false))
    };
    let read_cache = || -> Result<(String, bool), FetchErr> {
        let text = read_to_string(&cache_file_path).map_err(|_| FetchErr::CachePermission)?;

        Ok((text, true))
    };

    match update_cache {
        UpdateCache::Old => match fetch_and_update().await {
            Ok(data) => Ok(data),
            Err(_) => read_cache(),
        },
        UpdateCache::True => fetch_and_update().await,
        UpdateCache::False => read_cache(),
    }
}
