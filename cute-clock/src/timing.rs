//! Blink phase and repaint scheduling.
//!
//! Both are pure functions of the sub-second part of the current time, so
//! the display stays locked to the system clock: every repaint recomputes
//! the delay from the real time instead of adding a fixed interval, which
//! would accumulate drift.

use std::time::Duration;

const NANOS_PER_SEC: i32 = 1_000_000_000;

/// How long the cat's eyes stay shut at the start of every second.
pub const BLINK_DURATION: Duration = Duration::from_millis(150);

/// Whether the cat's eyes are open or shut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlinkPhase {
    Open,
    Closed,
}

fn clamp_subsec(subsec_nanos: i32) -> i32 {
    subsec_nanos.clamp(0, NANOS_PER_SEC - 1)
}

fn blink_nanos() -> i32 {
    BLINK_DURATION.subsec_nanos() as i32
}

/// The eyes close for [`BLINK_DURATION`] at the start of each second, so
/// the cat blinks exactly once per second, in step with the seconds digit.
pub fn blink_phase(subsec_nanos: i32) -> BlinkPhase {
    if clamp_subsec(subsec_nanos) < blink_nanos() {
        BlinkPhase::Closed
    } else {
        BlinkPhase::Open
    }
}

/// Time until the display next needs to change.
///
/// That is the end of the current blink if the eyes are shut, otherwise the
/// next whole-second boundary. `subsec_nanos` is the sub-second part of the
/// current time (`0..1_000_000_000`); out-of-range values are clamped.
pub fn next_repaint_delay(subsec_nanos: i32) -> Duration {
    let subsec = clamp_subsec(subsec_nanos);
    let target = if subsec < blink_nanos() {
        blink_nanos()
    } else {
        NANOS_PER_SEC
    };
    Duration::from_nanos((target - subsec) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: i32 = 1_000_000;

    #[test]
    fn delay_from_mid_second_reaches_next_boundary() {
        assert_eq!(next_repaint_delay(250 * MS), Duration::from_millis(750));
        assert_eq!(next_repaint_delay(999 * MS), Duration::from_millis(1));
    }

    #[test]
    fn delay_just_before_boundary_is_tiny() {
        assert_eq!(
            next_repaint_delay(NANOS_PER_SEC - 1),
            Duration::from_nanos(1)
        );
    }

    #[test]
    fn delay_during_blink_reaches_blink_end() {
        assert_eq!(next_repaint_delay(0), BLINK_DURATION);
        assert_eq!(next_repaint_delay(100 * MS), Duration::from_millis(50));
    }

    #[test]
    fn delay_at_blink_end_reaches_next_boundary() {
        assert_eq!(next_repaint_delay(150 * MS), Duration::from_millis(850));
    }

    #[test]
    fn delay_is_never_zero_and_never_over_a_second() {
        for subsec in (0..NANOS_PER_SEC).step_by(7_919_111) {
            let d = next_repaint_delay(subsec);
            assert!(d > Duration::ZERO, "{subsec}");
            assert!(d <= Duration::from_secs(1), "{subsec}");
        }
    }

    #[test]
    fn delay_clamps_out_of_range_input() {
        assert_eq!(next_repaint_delay(-5), BLINK_DURATION);
        assert_eq!(next_repaint_delay(i32::MAX), Duration::from_nanos(1));
    }

    #[test]
    fn blink_closed_at_start_of_second() {
        assert_eq!(blink_phase(0), BlinkPhase::Closed);
        assert_eq!(blink_phase(149 * MS), BlinkPhase::Closed);
    }

    #[test]
    fn blink_open_for_rest_of_second() {
        assert_eq!(blink_phase(150 * MS), BlinkPhase::Open);
        assert_eq!(blink_phase(500 * MS), BlinkPhase::Open);
        assert_eq!(blink_phase(NANOS_PER_SEC - 1), BlinkPhase::Open);
    }

    #[test]
    fn blinks_exactly_once_per_second() {
        // Count Open -> Closed transitions over two seconds sampled every ms.
        let mut transitions = 0;
        let mut prev = blink_phase(999 * MS);
        for ms in 0..2000 {
            let phase = blink_phase((ms % 1000) * MS);
            if prev == BlinkPhase::Open && phase == BlinkPhase::Closed {
                transitions += 1;
            }
            prev = phase;
        }
        assert_eq!(transitions, 2);
    }
}
