//! What the clock face shows at a given instant.

use jiff::{Timestamp, Zoned};

use crate::clock::Zone;
use crate::timing::{BlinkPhase, blink_phase};

/// 12- or 24-hour display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HourFormat {
    /// `HH:MM:SS`, e.g. `14:07:09`.
    #[default]
    TwentyFour,
    /// `hh:MM:SS AM|PM`, e.g. `02:07:09 PM`.
    Twelve,
}

/// Format the time of day.
pub fn format_time(t: &Zoned, format: HourFormat) -> String {
    match format {
        HourFormat::TwentyFour => t.strftime("%H:%M:%S").to_string(),
        HourFormat::Twelve => t.strftime("%I:%M:%S %p").to_string(),
    }
}

/// Format the date line, e.g. `Tuesday, 5 March 2024`.
pub fn format_date(t: &Zoned) -> String {
    t.strftime("%A, %-d %B %Y").to_string()
}

/// Everything the UI needs to draw one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockFace {
    pub time: String,
    pub date: String,
    pub blink: BlinkPhase,
    pub zone_label: Option<&'static str>,
}

impl ClockFace {
    /// Compute the face for instant `now` shown in `zone`.
    pub fn at(now: Timestamp, zone: &Zone, format: HourFormat) -> Self {
        let local = zone.at(now);
        Self {
            time: format_time(&local, format),
            date: format_date(&local),
            blink: blink_phase(local.subsec_nanosecond()),
            zone_label: zone.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::tz::TimeZone;

    fn utc(s: &str) -> Zoned {
        s.parse::<Timestamp>().unwrap().to_zoned(TimeZone::UTC)
    }

    #[test]
    fn twenty_four_hour() {
        let t = utc("2024-03-05T14:07:09Z");
        assert_eq!(format_time(&t, HourFormat::TwentyFour), "14:07:09");
    }

    #[test]
    fn twenty_four_hour_is_zero_padded() {
        let t = utc("2024-03-05T01:02:03Z");
        assert_eq!(format_time(&t, HourFormat::TwentyFour), "01:02:03");
    }

    #[test]
    fn twelve_hour_pm() {
        let t = utc("2024-03-05T14:07:09Z");
        assert_eq!(format_time(&t, HourFormat::Twelve), "02:07:09 PM");
    }

    #[test]
    fn twelve_hour_am() {
        let t = utc("2024-03-05T09:30:00Z");
        assert_eq!(format_time(&t, HourFormat::Twelve), "09:30:00 AM");
    }

    #[test]
    fn midnight() {
        let t = utc("2024-03-05T00:00:00Z");
        assert_eq!(format_time(&t, HourFormat::TwentyFour), "00:00:00");
        assert_eq!(format_time(&t, HourFormat::Twelve), "12:00:00 AM");
    }

    #[test]
    fn noon() {
        let t = utc("2024-03-05T12:00:00Z");
        assert_eq!(format_time(&t, HourFormat::TwentyFour), "12:00:00");
        assert_eq!(format_time(&t, HourFormat::Twelve), "12:00:00 PM");
    }

    #[test]
    fn last_second_of_day() {
        let t = utc("2024-03-05T23:59:59.999Z");
        assert_eq!(format_time(&t, HourFormat::TwentyFour), "23:59:59");
        assert_eq!(format_time(&t, HourFormat::Twelve), "11:59:59 PM");
    }

    #[test]
    fn date() {
        assert_eq!(
            format_date(&utc("2024-03-05T14:07:09Z")),
            "Tuesday, 5 March 2024"
        );
        assert_eq!(
            format_date(&utc("2024-12-25T00:00:00Z")),
            "Wednesday, 25 December 2024"
        );
    }

    #[test]
    fn face_uses_zone_and_label() {
        let ts: Timestamp = "2024-03-05T23:30:00.050Z".parse().unwrap();
        let fallback = Zone::from_lookup(Err::<TimeZone, _>(()));
        let face = ClockFace::at(ts, &fallback, HourFormat::TwentyFour);
        assert_eq!(face.time, "23:30:00");
        assert_eq!(face.date, "Tuesday, 5 March 2024");
        assert_eq!(face.blink, BlinkPhase::Closed);
        assert_eq!(face.zone_label, Some("UTC"));

        // Crossing midnight in a zone east of UTC changes the date too.
        let tokyo = Zone::from_lookup::<()>(Ok(TimeZone::fixed(jiff::tz::offset(9))));
        let face = ClockFace::at(ts, &tokyo, HourFormat::Twelve);
        assert_eq!(face.time, "08:30:00 AM");
        assert_eq!(face.date, "Wednesday, 6 March 2024");
        assert_eq!(face.zone_label, None);
    }
}
