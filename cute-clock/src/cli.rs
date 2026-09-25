//! Command-line interface.

use clap::Parser;

use crate::face::HourFormat;

/// A small, cute desktop clock with a blinking cat mascot.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "cute-clock", version, about)]
pub struct Args {
    /// Show 12-hour time with an AM/PM marker instead of 24-hour time.
    #[arg(long = "12h")]
    pub twelve_hour: bool,
}

impl Args {
    /// The hour format selected on the command line.
    pub fn hour_format(&self) -> HourFormat {
        if self.twelve_hour {
            HourFormat::Twelve
        } else {
            HourFormat::TwentyFour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn defaults_to_24_hour() {
        let args = Args::try_parse_from(["cute-clock"]).unwrap();
        assert_eq!(args.hour_format(), HourFormat::TwentyFour);
    }

    #[test]
    fn parses_12h_flag() {
        let args = Args::try_parse_from(["cute-clock", "--12h"]).unwrap();
        assert!(args.twelve_hour);
        assert_eq!(args.hour_format(), HourFormat::Twelve);
    }

    #[test]
    fn rejects_unknown_flag() {
        let err = Args::try_parse_from(["cute-clock", "--bogus"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
        // clap exits with status 2 for usage errors.
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn rejects_positional_argument() {
        assert!(Args::try_parse_from(["cute-clock", "extra"]).is_err());
    }

    #[test]
    fn clap_definition_is_valid() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
