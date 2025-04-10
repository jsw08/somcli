use chrono::{DateTime, Datelike, Utc};
use directories::ProjectDirs;
use icalendar::{Calendar, CalendarComponent::Event, CalendarDateTime, Component, DatePerhapsTime};
use std::{fmt, fs, path, time};

pub struct Lesson {
    pub subject: String,
    pub classroom: String,
    pub teacher: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}
pub struct Lessons {
    pub date: DateTime<Utc>,
    pub lessons: Vec<Lesson>,
}

pub enum UpdateCache {
    New,
    Old,
    False,
}

#[derive(Debug)]
pub enum LessonError {
    ParseError(String),
    CacheError(String),
    CachePermission,
    InvalidURL,
    HttpError
}

fn calendar_date_time_to_utc(time: CalendarDateTime) -> DateTime<Utc> {
    match time {
        CalendarDateTime::Floating(time) => time.and_utc(),
        CalendarDateTime::Utc(time) => time,
        CalendarDateTime::WithTimezone { date_time, tzid: _ } => date_time.and_utc(),
    }
}

fn parse_calendar(cal: &Calendar) -> Lessons {
    let now = Utc::now();

    let lessons: Vec<Lesson> = cal
        .components
        .iter()
        .filter_map(|component| {
            let Event(event) = component else { return None };
            let event_start = match event.get_start() {
                Some(DatePerhapsTime::DateTime(time)) => calendar_date_time_to_utc(time),
                _ => return None,
            };
            let event_end = match event.get_end() {
                Some(DatePerhapsTime::DateTime(time)) => calendar_date_time_to_utc(time),
                _ => return None,
            };

            if event_end.year() != now.year()
                || event_end.month() != now.month()
                || event_end.day() != now.day()
            {
                return None;
            }

            let classroom: String;
            let subject: String;
            let teacher: String;
            {
                let event_name = event.get_summary()?;
                let mut parts = event_name.splitn(3, "-").map(|s| s.trim());

                classroom = parts.next()?.to_string();
                subject = parts.next()?.splitn(2, '.').last()?.to_string();
                teacher = parts.next()?.to_string();

                if parts.next().is_some() {
                    return None;
                }
            }

            return Some(Lesson {
                classroom,
                subject,
                teacher,
                end_time: event_end,
                start_time: event_start,
            });
        })
        .collect();

    Lessons {
        date: Utc::now(),
        lessons,
    }
}

impl Lesson {
    pub fn finished(&self) -> bool {
        let now = Utc::now();
        now > self.end_time
    }
    pub fn started(&self) -> bool {
        let now = Utc::now();
        now > self.start_time
    }

    pub fn active(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now <= self.end_time
    }
}

impl Lessons {
    pub fn from_string(ical_data: &str) -> Result<Lessons, String> {
        let calendar: Calendar = ical_data.parse()?;

        Ok(parse_calendar(&calendar))
    }
    pub fn from_path(path: &path::Path) -> Result<Lessons, String> {
        let content =
            fs::read_to_string(path).map_err(|_| "Unable to read the given file.".to_string())?;

        Lessons::from_string(&content)
    }
    pub async fn from_url(url: String) -> Result<(Lessons, UpdateCache), LessonError> {
        let cache_dir = ProjectDirs::from("tf", "jsw", "somcli").ok_or(
            LessonError::CacheError("Couldn't access cache directory".to_string()),
        )?;
        let cache_dir = cache_dir.cache_dir();
        fs::create_dir_all(cache_dir).map_err(|_| LessonError::CachePermission)?;

        let cache_file_path = match url.split("/").last() {
            Some(data) => cache_dir.join(data.to_owned() + ".ics"),
            None => return Err(LessonError::InvalidURL),
        };

        let exists = cache_file_path
            .try_exists()
            .map_err(|_| LessonError::CachePermission)?;
        let update_cache = if !exists {
            UpdateCache::New
        } else {
            let cache_file_elapsed =
                fs::metadata(&cache_file_path).map_err(|_| LessonError::CachePermission)?;
            let cache_file_elapsed = cache_file_elapsed
                .modified()
                .map_err(|_| LessonError::CachePermission)?;
            let cache_file_elapsed = cache_file_elapsed
                .elapsed()
                .map_err(|_| LessonError::CachePermission)?;

            if cache_file_elapsed > time::Duration::from_secs(15 * 60) {
                UpdateCache::Old
            } else {
                UpdateCache::False
            }
        };

        let cache = || -> Result<Lessons, LessonError>  {
            Lessons::from_path(&cache_file_path).map_err(|str| LessonError::CacheError(str))
        };
        let fetch = async || -> Result<Lessons, LessonError> {
            let res = reqwest::get(url).await.map_err(|_| LessonError::HttpError)?;
            let text = res.text().await.map_err(|_| LessonError::ParseError("Unable to parse newly requested ical data.".to_string()))?;

            fs::write(&cache_file_path, &text).map_err(|_| LessonError::CachePermission)?;

            let cal = Lessons::from_string(&text).map_err(|str| LessonError::ParseError(str))?;
            Ok(cal)
        };

        match update_cache {
            UpdateCache::False => cache(),
            UpdateCache::Old => fetch().await.or(cache()),
            UpdateCache::New => fetch().await,
        }.map(|v| (v, update_cache))
    }
}

impl std::error::Error for LessonError {}
impl std::fmt::Display for LessonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LessonError::ParseError(error) => write!(f, "Error parsing calendar: {error}"),
            LessonError::CacheError(error) => write!(f, "Error with caching: {error}"),
            LessonError::CachePermission => write!(
                f,
                "Error accessing cache directory / file. Check your permissions."
            ),
            LessonError::InvalidURL => write!(
                f,
                "Unable to parse given url. Make sure it's an somtoday ical url."
            ),
            LessonError::HttpError => write!(
                f,
                "Couldn't fetch the calendar data from somtoday's servers. Please check the url and your network connection."
            ),
        }
    }
}


