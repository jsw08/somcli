use chrono::{DateTime, Datelike, Utc};
use icalendar::{Calendar, CalendarComponent::Event, CalendarDateTime, Component, DatePerhapsTime};

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

fn calendar_date_time_to_utc(time: CalendarDateTime) -> DateTime<Utc> {
    match time {
        CalendarDateTime::Floating(time) => time.and_utc(),
        CalendarDateTime::Utc(time) => time,
        CalendarDateTime::WithTimezone { date_time, tzid: _ } => date_time.and_utc(),
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
    pub fn from_string(ical_data: String) -> Result<Lessons, String> {
        let calendar: Calendar = ical_data.parse()?;

        Ok(parse_calendar(&calendar))
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
