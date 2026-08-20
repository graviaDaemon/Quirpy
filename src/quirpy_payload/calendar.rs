use crate::quirpy_payload::{PayloadError, escape::escape_ical};
use jiff::civil::{Date, DateTime};

#[derive(Debug, Clone, PartialEq)]
pub struct CalendarFields {
    pub title: String,
    pub location: String,
    pub description: String,
    pub start: DateTime,
    pub end: DateTime,
    pub all_day: bool,
}

impl Default for CalendarFields {
    fn default() -> Self {
        let today = jiff::Zoned::now().date();
        Self {
            title: String::new(),
            location: String::new(),
            description: String::new(),
            start: today.at(9, 0, 0, 0),
            end: today.at(10, 0, 0, 0),
            all_day: false,
        }
    }
}

fn format_date(date: Date) -> String {
    format!("{:04}{:02}{:02}", date.year(), date.month(), date.day())
}

fn format_date_time(value: DateTime) -> String {
    format!(
        "{}T{:02}{:02}{:02}",
        format_date(value.date()),
        value.hour(),
        value.minute(),
        value.second()
    )
}

pub fn build(fields: &CalendarFields) -> Result<String, PayloadError> {
    let title = fields.title.trim();
    if title.is_empty() {
        return Err(PayloadError::MissingField("Event title"));
    }

    let (start, end) = if fields.all_day {
        if fields.end.date() < fields.start.date() {
            return Err(PayloadError::Invalid {
                field: "End date",
                reason: "must not be before the start date".to_owned(),
            });
        }
        let day_after_end = fields
            .end
            .date()
            .tomorrow()
            .map_err(|_| PayloadError::Invalid {
                field: "End date",
                reason: "is out of range".to_owned(),
            })?;
        (
            format!("DTSTART;VALUE=DATE:{}", format_date(fields.start.date())),
            format!("DTEND;VALUE=DATE:{}", format_date(day_after_end)),
        )
    } else {
        if fields.end < fields.start {
            return Err(PayloadError::Invalid {
                field: "End",
                reason: "must not be before the start".to_owned(),
            });
        }
        (
            format!("DTSTART:{}", format_date_time(fields.start)),
            format!("DTEND:{}", format_date_time(fields.end)),
        )
    };

    let mut lines = vec![
        "BEGIN:VCALENDAR".to_owned(),
        "VERSION:2.0".to_owned(),
        "PRODID:-//Quirpy//EN".to_owned(),
        "BEGIN:VEVENT".to_owned(),
        format!("SUMMARY:{}", escape_ical(title)),
        start,
        end,
    ];

    if !fields.location.trim().is_empty() {
        lines.push(format!("LOCATION:{}", escape_ical(fields.location.trim())));
    }
    if !fields.description.trim().is_empty() {
        lines.push(format!(
            "DESCRIPTION:{}",
            escape_ical(fields.description.trim())
        ));
    }

    lines.push("END:VEVENT".to_owned());
    lines.push("END:VCALENDAR".to_owned());
    Ok(lines.join("\r\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn populated() -> CalendarFields {
        CalendarFields {
            title: "Launch party".to_owned(),
            location: "Rotterdam".to_owned(),
            description: "Bring cake".to_owned(),
            start: date(2026, 8, 20).at(14, 0, 0, 0),
            end: date(2026, 8, 20).at(15, 0, 0, 0),
            all_day: false,
        }
    }

    #[test]
    fn timed_event_uses_local_floating_time() {
        assert_eq!(
            build(&populated()).unwrap(),
            [
                "BEGIN:VCALENDAR",
                "VERSION:2.0",
                "PRODID:-//Quirpy//EN",
                "BEGIN:VEVENT",
                "SUMMARY:Launch party",
                "DTSTART:20260820T140000",
                "DTEND:20260820T150000",
                "LOCATION:Rotterdam",
                "DESCRIPTION:Bring cake",
                "END:VEVENT",
                "END:VCALENDAR",
            ]
            .join("\r\n")
        );
    }

    #[test]
    fn all_day_event_omits_optional_lines_and_ends_exclusive() {
        let fields = CalendarFields {
            title: "Holiday".to_owned(),
            location: String::new(),
            description: String::new(),
            all_day: true,
            ..populated()
        };

        assert_eq!(
            build(&fields).unwrap(),
            [
                "BEGIN:VCALENDAR",
                "VERSION:2.0",
                "PRODID:-//Quirpy//EN",
                "BEGIN:VEVENT",
                "SUMMARY:Holiday",
                "DTSTART;VALUE=DATE:20260820",
                "DTEND;VALUE=DATE:20260821",
                "END:VEVENT",
                "END:VCALENDAR",
            ]
            .join("\r\n")
        );
    }

    #[test]
    fn missing_title_is_an_error() {
        let fields = CalendarFields {
            title: String::new(),
            ..populated()
        };
        assert_eq!(build(&fields), Err(PayloadError::MissingField("Event title")));
    }

    #[test]
    fn end_before_start_is_an_error() {
        let fields = CalendarFields {
            end: date(2026, 8, 20).at(13, 0, 0, 0),
            ..populated()
        };
        assert!(matches!(build(&fields), Err(PayloadError::Invalid { .. })));
    }
}
