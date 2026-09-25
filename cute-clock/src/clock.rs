//! Clock and time-zone abstractions, so tests can control "now".

use jiff::{Timestamp, Zoned, tz::TimeZone};

/// A source of the current instant.
pub trait Clock {
    /// The current instant.
    fn now(&self) -> Timestamp;
}

/// The real system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

/// A clock frozen at a fixed instant (for tests).
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Timestamp);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0
    }
}

/// The time zone the clock displays, and whether it is a UTC fallback.
#[derive(Debug, Clone)]
pub struct Zone {
    tz: TimeZone,
    is_fallback: bool,
}

impl Zone {
    /// Detect the system's local time zone, falling back to UTC.
    pub fn detect() -> Self {
        Self::from_lookup(TimeZone::try_system())
    }

    /// Build a zone from the result of a time-zone lookup.
    ///
    /// On failure the zone is UTC and [`Zone::is_fallback`] is `true`.
    /// Factored out of [`Zone::detect`] so the fallback can be tested
    /// without touching the process environment.
    pub fn from_lookup<E>(lookup: Result<TimeZone, E>) -> Self {
        match lookup {
            Ok(tz) => Self {
                tz,
                is_fallback: false,
            },
            Err(_) => Self {
                tz: TimeZone::UTC,
                is_fallback: true,
            },
        }
    }

    /// Whether the local zone could not be found and UTC is used instead.
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }

    /// The small label to show on the clock face, if any.
    pub fn label(&self) -> Option<&'static str> {
        self.is_fallback.then_some("UTC")
    }

    /// The instant `ts` in this zone.
    pub fn at(&self, ts: Timestamp) -> Zoned {
        ts.to_zoned(self.tz.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> Timestamp {
        "2024-03-05T14:07:09Z".parse().unwrap()
    }

    #[test]
    fn successful_lookup_is_not_fallback() {
        let tz = TimeZone::fixed(jiff::tz::offset(1));
        let zone = Zone::from_lookup::<()>(Ok(tz));
        assert!(!zone.is_fallback());
        assert_eq!(zone.label(), None);
    }

    #[test]
    fn failed_lookup_falls_back_to_utc_with_label() {
        let zone = Zone::from_lookup(Err::<TimeZone, _>("no zone"));
        assert!(zone.is_fallback());
        assert_eq!(zone.label(), Some("UTC"));
        assert_eq!(zone.at(ts()).hour(), 14);
    }

    #[test]
    fn explicit_utc_lookup_has_no_label() {
        // Only a *fallback* gets the label, not a real UTC system zone.
        let zone = Zone::from_lookup::<()>(Ok(TimeZone::UTC));
        assert_eq!(zone.label(), None);
    }

    #[test]
    fn zone_converts_timestamp() {
        let tz = TimeZone::fixed(jiff::tz::offset(9));
        let zone = Zone::from_lookup::<()>(Ok(tz));
        assert_eq!(zone.at(ts()).hour(), 23);
    }

    #[test]
    fn fixed_clock_returns_its_instant() {
        assert_eq!(FixedClock(ts()).now(), ts());
    }
}
