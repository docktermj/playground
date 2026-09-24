//! Pure, display-free logic for `cute-clock`.
//!
//! Everything in this library can be unit tested without a window system:
//! command-line parsing, time-zone resolution, time/date formatting, the
//! blink phase of the cat's eyes and the repaint schedule. The binary in
//! `main.rs` only wires these pieces into an `eframe` window.

pub mod cli;
pub mod clock;
pub mod face;
pub mod timing;

pub use cli::Args;
pub use clock::{Clock, FixedClock, SystemClock, Zone};
pub use face::{ClockFace, HourFormat};
pub use timing::{BlinkPhase, blink_phase, next_repaint_delay};
