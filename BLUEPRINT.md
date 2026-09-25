# Blueprint: docktermj/playground
<!-- blueprint: sha=e74dc2f41f104d989d9d1c86a14f64b3cbcd9e6a date=2026-09-25 -->

## 1. Purpose

`docktermj/playground` is, in its own words, "just a place to try things". It
is a scratch repository that holds small, unrelated experiments side by side.

It holds two experiments today:

- **`cute-clock`**, a small desktop GUI clock for Ubuntu, written in Rust with
  `eframe`/`egui`. It opens a pastel window titled "Cute Clock" that shows the
  local time in a large rounded readout (24-hour `HH:MM:SS` by default, 12-hour
  with an AM/PM marker when run with `--12h`), the date on a line below, and a
  painted cat whose eyes blink once per second. If the local time zone cannot
  be found, it shows UTC with a small "UTC" label. All display-free logic lives
  in a unit-tested library; the binary only draws the window.
- **A certificate workflow**, a GitHub Actions workflow that, on every push,
  uses `openssl` to create a self-signed certificate authority and then a
  server and a client certificate signed by it, with subject alternative names
  taken from two small extension files under `testdata/certificates/`. It
  exercises no code in the repository.

## 2. Repository layout

- `.github/workflows/cute-clock.yaml`: CI for `cute-clock` (fmt, build, test,
  clippy), see section 4.
- `.github/workflows/test.yaml`: CI that generates and signs test certificates
  with `openssl`, see section 4.
- `.gitignore`: ignore patterns, described below this list.
- `LICENSE`: the license text, see section 9.
- `README.md`: two lines, the heading `# playground` and the sentence
  `Just a place to try things`.
- `cute-clock/Cargo.lock`: the crate's lockfile, see section 9.
- `cute-clock/Cargo.toml`: the `cute-clock` crate manifest (library and
  binary), see section 3.
- `cute-clock/src/cli.rs`: command-line parsing with `clap`.
- `cute-clock/src/clock.rs`: the clock abstraction and time-zone lookup.
- `cute-clock/src/face.rs`: time and date formatting and the per-frame face.
- `cute-clock/src/lib.rs`: the library crate root, declaring and re-exporting
  the modules.
- `cute-clock/src/main.rs`: the `cute-clock` binary: the `eframe` window and
  all painting.
- `cute-clock/src/timing.rs`: the blink phase and the repaint schedule.
- `testdata/certificates/client/ext.cnf`: `openssl` extension file for the
  client certificate, see section 6.
- `testdata/certificates/server/ext.cnf`: `openssl` extension file for the
  server certificate, see section 6.

`.gitignore` starts from GitHub's Go template, with its comments. It ignores
`*.exe`, `*.exe~`, `*.dll`, `*.so`, `*.dylib`, `*.test` and `*.out`, keeps
`vendor/` commented out, and ignores `go.work`, `go.work.sum`, `.env` and
`.history`. Under a final `# Rust build output` comment it ignores `target/`.

## 3. Toolchain and build

### Toolchain

The crate pins its edition and minimum Rust version:

<!-- markdownlint-disable MD013 -->

From `cute-clock/Cargo.toml`:

```toml
edition = "2024"
rust-version = "1.92"
```

<!-- markdownlint-enable MD013 -->

CI installs the latest stable Rust with `clippy` and `rustfmt`:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/cute-clock.yaml`:

```yaml
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
```

<!-- markdownlint-enable MD013 -->

There is no `rust-toolchain.toml`. The `eframe` dependency is held at 0.35,
because 0.36 needs Rust 1.95, above the pinned `rust-version`.

### Manifests

<!-- markdownlint-disable MD013 -->

From `cute-clock/Cargo.toml`:

```toml
[package]
name = "cute-clock"
version = "0.1.0"
edition = "2024"
rust-version = "1.92"
description = "A small, cute desktop clock with a blinking cat mascot."
license = "Apache-2.0"
publish = false

[lib]
name = "cute_clock"
path = "src/lib.rs"

[[bin]]
name = "cute-clock"
path = "src/main.rs"

[dependencies]
clap = { version = "4.6.7", features = ["derive"] }
eframe = "0.35.0"
jiff = "0.2.37"
```

<!-- markdownlint-enable MD013 -->

### Build commands

Building needs the system libraries `egui`/`eframe` link against. CI installs
them on Ubuntu with:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/cute-clock.yaml`:

```sh
sudo apt-get update
sudo apt-get install -y \
  libgl1-mesa-dev \
  libwayland-dev \
  libx11-dev \
  libxcursor-dev \
  libxi-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libxrandr-dev
```

<!-- markdownlint-enable MD013 -->

The format check, build, test and lint commands, from CI, run in
`cute-clock/`:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/cute-clock.yaml`:

```sh
cargo fmt --check
cargo build --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

<!-- markdownlint-enable MD013 -->

A release build and a run, from issue #2's acceptance criteria, also in
`cute-clock/`. The release binary is `target/release/cute-clock`:

```sh
cargo build --release
cargo run -- --12h
```

## 4. CI

### 4.1 `.github/workflows/cute-clock.yaml`

The workflow is named `Cute Clock`. It runs only when the crate or the
workflow itself changes:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/cute-clock.yaml`:

```yaml
on:
  push:
    paths:
      - "cute-clock/**"
      - ".github/workflows/cute-clock.yaml"
  pull_request:
    paths:
      - "cute-clock/**"
      - ".github/workflows/cute-clock.yaml"

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
```

<!-- markdownlint-enable MD013 -->

#### 4.1.1 `cute-clock-linux`

Named `Build, test and lint`. It runs on `ubuntu-latest`, with no `needs` and
no matrix, and runs every command in `cute-clock/`. It uses the workflow-level
env name `CARGO_TERM_COLOR` and no secrets. Its steps, in order: check out the
repository (`actions/checkout@v4`), install the system libraries, install the
Rust toolchain (`dtolnay/rust-toolchain@stable` with `clippy, rustfmt`), cache
cargo (`actions/cache@v4`, keyed on the lockfile's hash), then check
formatting, build, test and run clippy:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/cute-clock.yaml`:

```yaml
  cute-clock-linux:
    name: "Build, test and lint"
    runs-on: ubuntu-latest

    defaults:
      run:
        working-directory: cute-clock

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgl1-mesa-dev \
            libwayland-dev \
            libx11-dev \
            libxcursor-dev \
            libxi-dev \
            libxkbcommon-dev \
            libxkbcommon-x11-dev \
            libxrandr-dev

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            cute-clock/target
          key: cute-clock-${{ runner.os }}-${{ hashFiles('cute-clock/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --check

      - name: Build
        run: cargo build --locked

      - name: Test
        run: cargo test --locked

      - name: Clippy
        run: cargo clippy --locked --all-targets -- -D warnings
```

<!-- markdownlint-enable MD013 -->

### 4.2 `.github/workflows/test.yaml`

The workflow is named `Test workflow` and runs on every push:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/test.yaml`:

```yaml
on: [push]

env:
  OUTPUT_PATH: /certificates

permissions:
  contents: read
```

<!-- markdownlint-enable MD013 -->

#### 4.2.1 `go-test-linux`

Named `Test`. Despite its id, it builds and runs no Go code. It runs on
`ubuntu-latest`, with no `needs` and no matrix, uses the workflow-level env
name `OUTPUT_PATH` and no secrets. It checks out the full history
(`actions/checkout@v4` with `fetch-depth: 0`) and installs `tree`. It then
creates world-writable directories under `OUTPUT_PATH` and generates a
self-signed 4096-bit RSA certificate authority valid for 365 days. Next it
makes a server certificate request and signs it with the CA for 360 days,
using `testdata/certificates/server/ext.cnf`, and does the same for a client
certificate with `testdata/certificates/client/ext.cnf`. It prints each
certificate with `openssl x509 -text` and lists the files with `tree`. One
step, which would search for `*.pem` files, is commented out. The job,
verbatim:

<!-- markdownlint-disable MD013 -->

From `.github/workflows/test.yaml`:

```yaml
  go-test-linux:
    name: "Test"
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install dependencies
        run: |
          sudo apt-get install -y \
            tree

      # - name: Find *.pem files
      #   run: |
      #     sudo find / -name "*.pem"

      - name: Create directories
        run: |
          sudo mkdir ${OUTPUT_PATH}
          sudo mkdir ${OUTPUT_PATH}/certificate-authority
          sudo mkdir ${OUTPUT_PATH}/client
          sudo mkdir ${OUTPUT_PATH}/server
          sudo chmod -R 777 ${OUTPUT_PATH}
          ls -la ${OUTPUT_PATH}

      # -----------------------------------------------------------------------------
      # --- Self-Signed Certificate Authority
      # -----------------------------------------------------------------------------

      - name: Generate self-signed Certificate Authority
        run: |
          openssl req \
              -days 365 \
              -keyout ${OUTPUT_PATH}/certificate-authority/private_key.pem \
              -newkey rsa:4096 \
              -noenc \
              -out ${OUTPUT_PATH}/certificate-authority/certificate.pem \
              -subj "/C=US/ST=FL/L=West Palm Beach/O=Dockter/OU=Test CA/CN=dockter.com" \
              -x509

      - name: View self-signed Certificate Authority
        run: |
          openssl x509 \
              -in ${OUTPUT_PATH}/certificate-authority/certificate.pem \
              -noout \
              -text

      # -----------------------------------------------------------------------------
      # --- Server certificate
      # -----------------------------------------------------------------------------

      - name: Generate server certificate
        run: |
          openssl req \
              -keyout ${OUTPUT_PATH}/server/private_key.pem \
              -newkey rsa:4096 \
              -noenc \
              -out ${OUTPUT_PATH}/server/certificate_request.pem \
              -subj "/C=US/ST=FL/L=West Palm Beach/O=Dockter/OU=Test Server/CN=dockter.com" \

      - name: View certificates
        run: |
          tree ${{ github.workspace }}

      - name: Sign server certificate
        run: |
          openssl x509 \
              -CA ${OUTPUT_PATH}/certificate-authority/certificate.pem \
              -CAcreateserial \
              -CAkey ${OUTPUT_PATH}/certificate-authority/private_key.pem \
              -days 360 \
              -extfile ${{ github.workspace }}/testdata/certificates/server/ext.cnf \
              -in ${OUTPUT_PATH}/server/certificate_request.pem \
              -out ${OUTPUT_PATH}/server/certificate.pem \
              -req

      - name: View server certificate
        run: |
          openssl x509 \
              -in ${OUTPUT_PATH}/server/certificate.pem \
              -noout \
              -text

      # -----------------------------------------------------------------------------
      # --- Client certificate
      # -----------------------------------------------------------------------------

      - name: Generate client certificate
        run: |
          openssl req \
              -keyout ${OUTPUT_PATH}/client/private_key.pem \
              -newkey rsa:4096 \
              -noenc \
              -out ${OUTPUT_PATH}/client/certificate_request.pem \
              -subj "/C=US/ST=FL/L=West Palm Beach/O=Dockter/OU=Test Client/CN=dockter.com" \

      - name: Sign client certificate
        run: |
          openssl x509 \
              -CA ${OUTPUT_PATH}/certificate-authority/certificate.pem \
              -CAcreateserial \
              -CAkey ${OUTPUT_PATH}/certificate-authority/private_key.pem \
              -days 360 \
              -extfile ${{ github.workspace }}/testdata/certificates/client/ext.cnf \
              -in ${OUTPUT_PATH}/client/certificate_request.pem \
              -out ${OUTPUT_PATH}/client/certificate.pem \
              -req

      - name: View client certificate
        run: |
          openssl x509 \
              -in ${OUTPUT_PATH}/client/certificate.pem \
              -noout \
              -text

      # -----------------------------------------------------------------------------
      # --- Epilog
      # -----------------------------------------------------------------------------

      - name: View certificates
        run: |
          tree ${OUTPUT_PATH}
```

<!-- markdownlint-enable MD013 -->

## 5. Components

### 5.1 `cute-clock`

One Cargo package with two targets: the library `cute_clock`
(`src/lib.rs`) and the binary `cute-clock` (`src/main.rs`). The split keeps
everything that can be tested without a window system in the library: parsing
the command line, finding the time zone, formatting the time and date, the
blink phase and the repaint schedule. The binary only wires these into an
`eframe` window and paints. Time comes from `jiff`, arguments from `clap`
(derive), and the window from `eframe` (which brings `egui`).

### 5.2 `cute-clock/src/cli.rs`

Parses the command line into `Args` (section 6.1.2) with `clap`'s derive API.
The only option is the boolean flag `--12h` (field `twelve_hour`), plus the
`--help`/`-h` and `--version`/`-V` flags `clap` adds. `Args::hour_format`
maps the flag to `HourFormat::Twelve` when set and `HourFormat::TwentyFour`
otherwise. Any unknown flag or any positional argument is a usage error.

### 5.3 `cute-clock/src/clock.rs`

Provides "now" behind the `Clock` trait, so tests can freeze time.
`SystemClock` returns `jiff::Timestamp::now()`; `FixedClock` always returns
the timestamp it wraps.

`Zone` is the time zone the clock displays, plus whether it is a fallback.
`Zone::detect` asks `jiff` for the system zone (`TimeZone::try_system`, which
reads `TZ` and the system's zone database) and passes the result to
`Zone::from_lookup`. On `Ok(tz)` that gives `tz` with `is_fallback` false. On
any `Err` it gives UTC with `is_fallback` true. `label` is `Some("UTC")` only
for a fallback, so a system zone that really is UTC shows no label.
`Zone::at` converts a timestamp into that zone.

### 5.4 `cute-clock/src/face.rs`

Decides what one frame shows. `HourFormat` defaults to `TwentyFour`.
`format_time` formats a zoned time with `strftime` pattern `%H:%M:%S` for
24-hour (zero-padded, for example `01:02:03`) or `%I:%M:%S %p` for 12-hour
(`02:07:09 PM`; midnight is `12:00:00 AM`, noon `12:00:00 PM`). Fractional
seconds are truncated, never rounded: `23:59:59.999` is `23:59:59`.
`format_date` uses `%A, %-d %B %Y`, giving for example
`Tuesday, 5 March 2024`, with the day of the month not padded.

`ClockFace::at(now, zone, format)` converts `now` into `zone`, then fills in
the formatted time, the date, the blink phase from the zoned time's
sub-second nanoseconds, and the zone's label. The date follows the zone, so
crossing midnight in a zone east of UTC moves the date forward.

### 5.5 `cute-clock/src/lib.rs`

The library root. It declares the four public modules and re-exports their
main items at the crate root (section 6.1.1), so the binary imports
everything from `cute_clock` directly.

### 5.6 `cute-clock/src/main.rs`

The binary. It parses `Args`, then opens a native `eframe` window titled
`Cute Clock` (the application name is also `Cute Clock`). The window is
320×200 logical pixels by default, resizable, with a minimum of 160×100. The
app holds a `SystemClock`, `Zone::detect()` and the chosen hour format.

On every frame it:

- closes the window if Ctrl+Q was pressed, consuming the key press;
- computes `ClockFace::at` for the current instant and paints it into the
  whole available area;
- asks `egui` to repaint after `next_repaint_delay`, computed from the
  current instant's sub-second nanoseconds. The delay is recomputed from the
  real clock every frame, so the display never drifts.

Painting, where `s` is the scale and all positions are relative to a
320×200 layout scaled by `s` and centred in the window:

- The whole window is filled with pastel pink `#FFE4EC`. The scale `s` is
  the smaller of width/320 and height/200.
- **Cat** on the left: centre `(left + 62s, centre_y + 6s)`, radius
  `r = 44s`. The outline is plum `#6B4E71`, width `max(0.05r, 1)`.
  - Ears, for each side (x mirrored): an outer triangle
    `(0.25, -0.75)`, `(0.95, -1.15)`, `(0.85, -0.35)` (times `r`, from the
    centre) in peach `#FFD8A8` with the outline, and an inner triangle
    `(0.4, -0.72)`, `(0.86, -0.98)`, `(0.8, -0.5)` in pink `#FFB3C6`, no
    stroke. Ears are drawn before the head.
  - Head: a circle of radius `r` in peach with the outline.
  - Blush: circles of radius `0.15r` at `(±0.55r, 0.25r)` in `#FFB8B0`.
  - Eyes at `(±0.36r, -0.12r)`. Open: a plum disc of radius `0.13r` with a
    white highlight of radius `0.045r` offset `(0.04r, -0.05r)`. Closed: a
    happy "^" of two plum segments, width `max(0.06r, 1)`, from
    `(-0.14r, 0.03r)` to `(0, -0.06r)` to `(0.14r, 0.03r)` around the eye.
  - Nose centred at `(0, 0.1r)`: a triangle `(-0.08, -0.05)`,
    `(0.08, -0.05)`, `(0, 0.05)` (times `r`, from the nose) in `#FFB3C6`.
  - Mouth, plum, width `max(0.04r, 1)`, relative to the nose: a segment from
    `(0, 0.05r)` to `(0, 0.14r)`, then on each side `(0, 0.14r)` to
    `(±0.07r, 0.2r)` to `(±0.14r, 0.13r)`, making an "ω".
  - Whiskers in soft plum `#9C84A4`, width `max(0.025r, 0.75)`: for each
    side and each `dy` in `-0.06` and `0.08`, a segment from
    `(±0.45r, nose_y + dy·r)` to `(±1.1r, nose_y + 1.8·dy·r)`.
- **Readout**: a pill from `(left + 122s, top + 40s)` to
  `(right - 10s, bottom - 40s)`, filled `#FFF7FA`, corner radius half its
  height `h`. The time font size is fitted without jitter: measure the
  template `88:88:88 PM` (when the time ends in `M`) or `88:88:88` in the
  proportional font at size 40, then use
  `min(40 · (width - 0.5h) / max(measured, 1), 0.55h)`. The time is centred
  at `(centre_x, centre_y - 0.1h)` in plum. The date is centred at
  `(centre_x, centre_y + 0.3h)` in soft plum, at size
  `min(11s, 0.45 · time size)`.
- **Zone label**, only when the face has one: the text in plum, proportional
  size `11s`, on a pill in pastel blue `#C9E4FF` with padding `(6s, 2s)` and
  corner radius half its height. The pill's top-right corner sits at
  `(right - 8s, top + 8s)`.

`egui`'s default proportional font is used; no font is bundled.

### 5.7 `cute-clock/src/timing.rs`

Both functions are pure functions of the sub-second part of the current time
in nanoseconds, so the display stays locked to the system clock. The input is
first clamped to `0..=999_999_999`.

- `blink_phase` is `Closed` for the first `BLINK_DURATION` (150 ms) of every
  second and `Open` for the rest. The cat therefore blinks exactly once per
  second, in step with the seconds digit.
- `next_repaint_delay` is the time to the end of the blink while the eyes are
  shut, and otherwise the time to the next whole second. It is never zero
  and never more than one second.

## 6. Exact contracts

### 6.1 Public API

`Timestamp` and `Zoned` are `jiff::Timestamp` and `jiff::Zoned`; `TimeZone`
is `jiff::tz::TimeZone`; `Duration` is `std::time::Duration`.

#### 6.1.1 `cute_clock`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/lib.rs`:

```rust
pub mod cli;
pub mod clock;
pub mod face;
pub mod timing;

pub use cli::Args;
pub use clock::{Clock, FixedClock, SystemClock, Zone};
pub use face::{ClockFace, HourFormat};
pub use timing::{BlinkPhase, blink_phase, next_repaint_delay};
```

<!-- markdownlint-enable MD013 -->

#### 6.1.2 `cute_clock::cli`

The doc comments are part of the contract: `clap` turns them into the help
text in section 6.2.

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/cli.rs`:

```rust
/// A small, cute desktop clock with a blinking cat mascot.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "cute-clock", version, about)]
pub struct Args {
    /// Show 12-hour time with an AM/PM marker instead of 24-hour time.
    #[arg(long = "12h")]
    pub twelve_hour: bool,
}

impl Args {
    pub fn hour_format(&self) -> HourFormat { ... }
}
```

<!-- markdownlint-enable MD013 -->

#### 6.1.3 `cute_clock::clock`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/clock.rs`:

```rust
pub trait Clock {
    fn now(&self) -> Timestamp;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp { ... }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Timestamp);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp { ... }
}

#[derive(Debug, Clone)]
pub struct Zone {
    tz: TimeZone,
    is_fallback: bool,
}

impl Zone {
    pub fn detect() -> Self { ... }
    pub fn from_lookup<E>(lookup: Result<TimeZone, E>) -> Self { ... }
    pub fn is_fallback(&self) -> bool { ... }
    pub fn label(&self) -> Option<&'static str> { ... }
    pub fn at(&self, ts: Timestamp) -> Zoned { ... }
}
```

<!-- markdownlint-enable MD013 -->

#### 6.1.4 `cute_clock::face`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/face.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HourFormat {
    /// `HH:MM:SS`, e.g. `14:07:09`.
    #[default]
    TwentyFour,
    /// `hh:MM:SS AM|PM`, e.g. `02:07:09 PM`.
    Twelve,
}

pub fn format_time(t: &Zoned, format: HourFormat) -> String { ... }

pub fn format_date(t: &Zoned) -> String { ... }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockFace {
    pub time: String,
    pub date: String,
    pub blink: BlinkPhase,
    pub zone_label: Option<&'static str>,
}

impl ClockFace {
    pub fn at(now: Timestamp, zone: &Zone, format: HourFormat) -> Self { ... }
}
```

<!-- markdownlint-enable MD013 -->

#### 6.1.5 `cute_clock::timing`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/timing.rs`:

```rust
pub const BLINK_DURATION: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlinkPhase {
    Open,
    Closed,
}

pub fn blink_phase(subsec_nanos: i32) -> BlinkPhase { ... }

pub fn next_repaint_delay(subsec_nanos: i32) -> Duration { ... }
```

<!-- markdownlint-enable MD013 -->

### 6.2 CLI

#### 6.2.1 `cute-clock`

The declaration is `Args` in section 6.1.2. `cute-clock --help` (or `-h`)
prints this and exits 0:

```text
A small, cute desktop clock with a blinking cat mascot.

Usage: cute-clock [OPTIONS]

Options:
      --12h      Show 12-hour time with an AM/PM marker instead of 24-hour time
  -h, --help     Print help
  -V, --version  Print version
```

`cute-clock --version` (or `-V`) prints `cute-clock 0.1.0` and exits 0.

Exit codes:

- `0`: the window was closed (close button or Ctrl+Q), or `--help` or
  `--version` was given.
- `2`: a usage error, such as an unknown flag or any positional argument.
  `clap` prints the error and usage to standard error, for example:

  ```text
  error: unexpected argument '--bogus' found

  Usage: cute-clock [OPTIONS]

  For more information, try '--help'.
  ```

- `1`: `eframe` failed, for example when no display is available. `main`
  returns the `eframe::Result` error and Rust prints it as `Error: ...`.

The only keyboard shortcut is Ctrl+Q, which closes the window.

### 6.3 Config and file formats

The clock reads no config file and no environment variable of its own. The
time zone comes from `jiff`'s system lookup (section 5.3).

The certificate workflow (section 4.2) passes these files to `openssl x509`
as `-extfile`. Neither ends with a newline.

<!-- markdownlint-disable MD013 -->

From `testdata/certificates/client/ext.cnf`:

```text
subjectAltName=DNS:*.senzing.com,DNS:*.com,IP:0.0.0.0
```

<!-- markdownlint-enable MD013 -->

<!-- markdownlint-disable MD013 -->

From `testdata/certificates/server/ext.cnf`:

```text
subjectAltName=DNS:*.senzing.com,DNS:*.com,IP:0.0.0.0
```

<!-- markdownlint-enable MD013 -->

### 6.4 Errors and output

The window's title and size, and the application name:

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/main.rs`:

```rust
const BASE_SIZE: Vec2 = vec2(320.0, 200.0);
const MIN_SIZE: Vec2 = vec2(160.0, 100.0);
            .with_title("Cute Clock")
            .with_inner_size(BASE_SIZE)
            .with_min_inner_size(MIN_SIZE)
            .with_resizable(true),
    eframe::run_native("Cute Clock", options, Box::new(|_cc| Ok(Box::new(app))))
```

<!-- markdownlint-enable MD013 -->

The time and date text formats, and the fallback label, whose exact text the
tests pin:

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/face.rs`:

```rust
        HourFormat::TwentyFour => t.strftime("%H:%M:%S").to_string(),
        HourFormat::Twelve => t.strftime("%I:%M:%S %p").to_string(),
    t.strftime("%A, %-d %B %Y").to_string()
```

<!-- markdownlint-enable MD013 -->

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/clock.rs`:

```rust
        self.is_fallback.then_some("UTC")
```

<!-- markdownlint-enable MD013 -->

## 7. Test-pinned internals

### 7.1 `cute-clock/src/cli.rs`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/cli.rs`:

```rust
use clap::Parser;

use crate::face::HourFormat;
```

<!-- markdownlint-enable MD013 -->

- `clap::Parser` (its `try_parse_from`): `defaults_to_24_hour`,
  `parses_12h_flag`, `rejects_unknown_flag`, `rejects_positional_argument`.
- `HourFormat`: `defaults_to_24_hour`, `parses_12h_flag`.

### 7.2 `cute-clock/src/clock.rs`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/clock.rs`:

```rust
use jiff::{Timestamp, Zoned, tz::TimeZone};
```

<!-- markdownlint-enable MD013 -->

- `Timestamp`: the test helper `ts`, `fixed_clock_returns_its_instant`.
- `TimeZone`: `successful_lookup_is_not_fallback`,
  `failed_lookup_falls_back_to_utc_with_label`,
  `explicit_utc_lookup_has_no_label`, `zone_converts_timestamp`.
- `Zoned` is not used by a test, but shares the line.

### 7.3 `cute-clock/src/face.rs`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/face.rs`:

```rust
use jiff::{Timestamp, Zoned};

use crate::clock::Zone;
use crate::timing::{BlinkPhase, blink_phase};
```

<!-- markdownlint-enable MD013 -->

- `Timestamp` and `Zoned`: the test helper `utc`, used by every formatting
  test; `Timestamp` also by `face_uses_zone_and_label`.
- `Zone` and `BlinkPhase`: `face_uses_zone_and_label`.
- `blink_phase` is not used by a test, but shares the line.

### 7.4 `cute-clock/src/timing.rs`

<!-- markdownlint-disable MD013 -->

From `cute-clock/src/timing.rs`:

```rust
use std::time::Duration;

const NANOS_PER_SEC: i32 = 1_000_000_000;
```

<!-- markdownlint-enable MD013 -->

- `Duration`: `delay_from_mid_second_reaches_next_boundary`,
  `delay_just_before_boundary_is_tiny`, `delay_during_blink_reaches_blink_end`,
  `delay_at_blink_end_reaches_next_boundary`,
  `delay_is_never_zero_and_never_over_a_second`,
  `delay_clamps_out_of_range_input`.
- `NANOS_PER_SEC`: `delay_just_before_boundary_is_tiny`,
  `delay_is_never_zero_and_never_over_a_second`,
  `blink_open_for_rest_of_second`.

## 8. Tests

- **Framework and tools:** Rust's built-in test harness (`#[test]`). There are
  no dev-dependencies. The `cli` tests also use `clap::error::ErrorKind` and
  `clap::CommandFactory`, and the `clock` and `face` tests use
  `jiff::tz::TimeZone` and `jiff::tz::offset`.
- **Layout:** every test lives in an in-module `#[cfg(test)] mod tests` block
  with `use super::*`, at the end of its source file. None needs a display.
  Run them with `cargo test --locked` in `cute-clock/`.
- **Count:** `cli.rs` 5, `clock.rs` 5, `face.rs` 9, `timing.rs` 9; 28 in
  total. `lib.rs` and `main.rs` have none.

### 8.1 `cute-clock/src/cli.rs`

- `defaults_to_24_hour`: parsing `["cute-clock"]` gives
  `HourFormat::TwentyFour`.
- `parses_12h_flag`: `["cute-clock", "--12h"]` sets `twelve_hour` and gives
  `HourFormat::Twelve`.
- `rejects_unknown_flag`: `--bogus` fails with `ErrorKind::UnknownArgument`
  and exit code 2.
- `rejects_positional_argument`: `["cute-clock", "extra"]` is an error.
- `clap_definition_is_valid`: `Args::command().debug_assert()` passes.

### 8.2 `cute-clock/src/clock.rs`

The helper `ts()` is `2024-03-05T14:07:09Z`.

- `successful_lookup_is_not_fallback`: `from_lookup(Ok(UTC+1))` is not a
  fallback and has label `None`.
- `failed_lookup_falls_back_to_utc_with_label`: `from_lookup(Err("no zone"))`
  is a fallback with label `Some("UTC")`, and `at(ts()).hour()` is 14.
- `explicit_utc_lookup_has_no_label`: `from_lookup(Ok(TimeZone::UTC))` has
  label `None`.
- `zone_converts_timestamp`: in a fixed UTC+9 zone, `at(ts()).hour()` is 23.
- `fixed_clock_returns_its_instant`: `FixedClock(ts()).now()` is `ts()`.

### 8.3 `cute-clock/src/face.rs`

The helper `utc(s)` parses `s` as a timestamp and puts it in UTC.

- `twenty_four_hour`: `14:07:09Z` formats as `14:07:09`.
- `twenty_four_hour_is_zero_padded`: `01:02:03Z` formats as `01:02:03`.
- `twelve_hour_pm`: `14:07:09Z` formats as `02:07:09 PM`.
- `twelve_hour_am`: `09:30:00Z` formats as `09:30:00 AM`.
- `midnight`: `00:00:00Z` is `00:00:00` (24-hour) and `12:00:00 AM`
  (12-hour).
- `noon`: `12:00:00Z` is `12:00:00` and `12:00:00 PM`.
- `last_second_of_day`: `23:59:59.999Z` is `23:59:59` and `11:59:59 PM`.
- `date`: `2024-03-05T14:07:09Z` is `Tuesday, 5 March 2024`, and
  `2024-12-25T00:00:00Z` is `Wednesday, 25 December 2024`.
- `face_uses_zone_and_label`: at `2024-03-05T23:30:00.050Z` with a fallback
  zone and 24-hour format, the face is `23:30:00`, `Tuesday, 5 March 2024`,
  `BlinkPhase::Closed`, label `Some("UTC")`. In a fixed UTC+9 zone with
  12-hour format it is `08:30:00 AM`, `Wednesday, 6 March 2024`, label
  `None`.

All the formatting tests use 2024-03-05 unless another date is given.

### 8.4 `cute-clock/src/timing.rs`

`MS` is 1,000,000 nanoseconds, a constant of the test module.

- `delay_from_mid_second_reaches_next_boundary`: 250 ms gives 750 ms, 999 ms
  gives 1 ms.
- `delay_just_before_boundary_is_tiny`: `NANOS_PER_SEC - 1` gives 1 ns.
- `delay_during_blink_reaches_blink_end`: 0 gives `BLINK_DURATION`, 100 ms
  gives 50 ms.
- `delay_at_blink_end_reaches_next_boundary`: 150 ms gives 850 ms.
- `delay_is_never_zero_and_never_over_a_second`: for every sub-second value
  from 0 in steps of 7,919,111 ns, the delay is above zero and at most 1 s.
- `delay_clamps_out_of_range_input`: -5 gives `BLINK_DURATION`, and
  `i32::MAX` gives 1 ns.
- `blink_closed_at_start_of_second`: 0 and 149 ms are `Closed`.
- `blink_open_for_rest_of_second`: 150 ms, 500 ms and `NANOS_PER_SEC - 1` are
  `Open`.
- `blinks_exactly_once_per_second`: sampling every ms over two seconds,
  starting from the phase at 999 ms, gives exactly two `Open` to `Closed`
  transitions.

## 9. Not embedded

### Secrets

none

### Generated files

- `cute-clock/Cargo.lock`: `cargo generate-lockfile` (run in `cute-clock/`).
  CI builds with `--locked`, so the lockfile must be committed.

### Vendored code

none

### Binaries and large fixtures

none

### Licenses

- `LICENSE`: `Apache-2.0`

## 10. Decision log

- #2: added `cute-clock`, a pastel `eframe` clock with a blinking cat, as a
  tested `jiff` + `clap` library plus a thin binary, with its own CI workflow
  (a playful Ubuntu desktop clock; the split keeps all logic testable without
  a display)
