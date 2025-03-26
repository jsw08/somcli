mod calendar_parser;
mod fetch;

use chrono::{Datelike, Local, Timelike, Utc};
const URL: &str = "https://api.somtoday.nl/rest/v1/icalendar/stream/d1b640b7-b7da-4aee-a10f-e259a49fcdc5/f7556f69-391b-4aca-adf1-a3e62064711c";

#[tokio::main]
async fn main() -> Result<(), u8> {
    let (ics_data, ics_old) = fetch::fetch_calendar(URL).await.map_err(|e| {
        println!("{}", e);
        return 1

    })?;
    let lessons = calendar_parser::Lessons::from_string(ics_data).map_err(|e| {
        println!("{}", e);
        return 1
    })?;

    {
        let offline = if ics_old {'💾'} else {'🌐'};
        let local_date = lessons.date.with_timezone(&Local);
        let formatted_date = format!("{}:{} - {}.{}.{}", local_date.hour(), local_date.minute(), local_date.day(), local_date.month(), local_date.year());
        println!("---- {} {} ----", offline, formatted_date);
    }

    let now = Utc::now();
    for lesson in lessons.lessons {
        let until_text = if lesson.finished() {
            String::from("✅")
        } else {
            let (time_point, prefix) = if lesson.started() {
                (lesson.end_time, "⏳")
            } else {
                (lesson.start_time, "🚀")
            };

            let difference = time_point.signed_duration_since(now);
            let minutes = difference.num_minutes();

            if minutes < 90 {
                format!("{} {}m", prefix, minutes)
            } else {
                format!("{} {}h", prefix, difference.num_hours())
            }
        };

        println!(
            "{} in {} - {}",
            lesson.subject, lesson.classroom, until_text
        );
    }

    return Ok(());
}
