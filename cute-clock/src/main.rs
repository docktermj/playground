//! `cute-clock`: the eframe window. All testable logic lives in the library.

use clap::Parser;
use cute_clock::{Args, BlinkPhase, Clock, ClockFace, HourFormat, SystemClock, Zone};
use eframe::egui::{
    self, Align2, Color32, FontId, Key, Modifiers, Painter, Pos2, Rect, Shape, Stroke, Vec2,
    ViewportCommand, pos2, vec2,
};

/// Default window size; the layout is designed at this size and scaled.
const BASE_SIZE: Vec2 = vec2(320.0, 200.0);
const MIN_SIZE: Vec2 = vec2(160.0, 100.0);

const BACKGROUND: Color32 = Color32::from_rgb(0xFF, 0xE4, 0xEC); // pastel pink
const PILL: Color32 = Color32::from_rgb(0xFF, 0xF7, 0xFA);
const INK: Color32 = Color32::from_rgb(0x6B, 0x4E, 0x71); // soft plum
const SOFT_INK: Color32 = Color32::from_rgb(0x9C, 0x84, 0xA4);
const CAT_FUR: Color32 = Color32::from_rgb(0xFF, 0xD8, 0xA8); // peach
const CAT_EAR_INNER: Color32 = Color32::from_rgb(0xFF, 0xB3, 0xC6);
const CAT_BLUSH: Color32 = Color32::from_rgb(0xFF, 0xB8, 0xB0);
const LABEL_BG: Color32 = Color32::from_rgb(0xC9, 0xE4, 0xFF); // pastel blue

fn main() -> eframe::Result {
    let args = Args::parse();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cute Clock")
            .with_inner_size(BASE_SIZE)
            .with_min_inner_size(MIN_SIZE)
            .with_resizable(true),
        ..Default::default()
    };

    let app = CuteClockApp {
        clock: SystemClock,
        zone: Zone::detect(),
        format: args.hour_format(),
    };
    eframe::run_native("Cute Clock", options, Box::new(|_cc| Ok(Box::new(app))))
}

struct CuteClockApp<C: Clock> {
    clock: C,
    zone: Zone,
    format: HourFormat,
}

impl<C: Clock> eframe::App for CuteClockApp<C> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Q)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        let now = self.clock.now();
        let face = ClockFace::at(now, &self.zone, self.format);
        paint(ui.painter(), ui.max_rect(), &face);

        // Wake exactly when the display next changes (end of blink or the
        // next second boundary), recomputed from the real clock every frame
        // so it never drifts.
        let subsec = self.zone.at(now).subsec_nanosecond();
        ctx.request_repaint_after(cute_clock::next_repaint_delay(subsec));
    }
}

/// Paint the whole clock face into `rect`, scaled to fit.
fn paint(painter: &Painter, rect: Rect, face: &ClockFace) {
    painter.rect_filled(rect, 0.0, BACKGROUND);

    let scale = (rect.width() / BASE_SIZE.x).min(rect.height() / BASE_SIZE.y);
    let content = Rect::from_center_size(rect.center(), BASE_SIZE * scale);

    // Cat on the left third, readout on the right.
    let cat_center = pos2(
        content.left() + 62.0 * scale,
        content.center().y + 6.0 * scale,
    );
    paint_cat(painter, cat_center, 44.0 * scale, face.blink);

    let text_rect = Rect::from_min_max(
        pos2(content.left() + 122.0 * scale, content.top() + 40.0 * scale),
        pos2(
            content.right() - 10.0 * scale,
            content.bottom() - 40.0 * scale,
        ),
    );
    paint_readout(painter, text_rect, scale, face);

    if let Some(label) = face.zone_label {
        let font = FontId::proportional(11.0 * scale);
        let galley = painter.layout_no_wrap(label.to_owned(), font, INK);
        let pad = vec2(6.0, 2.0) * scale;
        let anchor = pos2(content.right() - 8.0 * scale, content.top() + 8.0 * scale);
        let bg = Rect::from_min_size(
            anchor - vec2(galley.size().x + 2.0 * pad.x, 0.0),
            galley.size() + 2.0 * pad,
        );
        painter.rect_filled(bg, bg.height() / 2.0, LABEL_BG);
        painter.galley(bg.min + pad, galley, INK);
    }
}

/// The big rounded time readout with the date underneath.
fn paint_readout(painter: &Painter, rect: Rect, scale: f32, face: &ClockFace) {
    painter.rect_filled(rect, rect.height() / 2.0, PILL);

    // Fit the time to the pill width: measure a fixed-width template at a
    // reference size, then scale, so the font size doesn't jitter as digits change.
    let reference = 40.0;
    let template = if face.time.ends_with('M') {
        "88:88:88 PM"
    } else {
        "88:88:88"
    };
    let measured = painter
        .layout_no_wrap(template.to_owned(), FontId::proportional(reference), INK)
        .size()
        .x;
    let max_width = rect.width() - rect.height() * 0.5;
    let size = (reference * max_width / measured.max(1.0)).min(rect.height() * 0.55);

    let time_center = pos2(rect.center().x, rect.center().y - rect.height() * 0.1);
    painter.text(
        time_center,
        Align2::CENTER_CENTER,
        &face.time,
        FontId::proportional(size),
        INK,
    );
    painter.text(
        pos2(rect.center().x, rect.center().y + rect.height() * 0.3),
        Align2::CENTER_CENTER,
        &face.date,
        FontId::proportional((11.0 * scale).min(size * 0.45)),
        SOFT_INK,
    );
}

/// A round peach cat face with ears, blush, whiskers and blinking eyes.
fn paint_cat(painter: &Painter, c: Pos2, r: f32, blink: BlinkPhase) {
    let outline = Stroke::new((r * 0.05).max(1.0), INK);

    // Ears (drawn first so the head overlaps their base).
    for side in [-1.0, 1.0] {
        let ear = |dx: f32, dy: f32| pos2(c.x + side * dx * r, c.y + dy * r);
        painter.add(Shape::convex_polygon(
            vec![ear(0.25, -0.75), ear(0.95, -1.15), ear(0.85, -0.35)],
            CAT_FUR,
            outline,
        ));
        painter.add(Shape::convex_polygon(
            vec![ear(0.4, -0.72), ear(0.86, -0.98), ear(0.8, -0.5)],
            CAT_EAR_INNER,
            Stroke::NONE,
        ));
    }

    // Head.
    painter.circle(c, r, CAT_FUR, outline);

    // Blush.
    for side in [-1.0, 1.0] {
        painter.circle_filled(
            pos2(c.x + side * 0.55 * r, c.y + 0.25 * r),
            0.15 * r,
            CAT_BLUSH,
        );
    }

    // Eyes.
    let eye_y = c.y - 0.12 * r;
    for side in [-1.0, 1.0] {
        let eye = pos2(c.x + side * 0.36 * r, eye_y);
        match blink {
            BlinkPhase::Open => {
                painter.circle_filled(eye, 0.13 * r, INK);
                painter.circle_filled(eye + vec2(0.04, -0.05) * r, 0.045 * r, Color32::WHITE);
            }
            BlinkPhase::Closed => {
                // A happy closed eye: a little "^" shape.
                let w = 0.14 * r;
                let stroke = Stroke::new((r * 0.06).max(1.0), INK);
                painter.line_segment(
                    [eye + vec2(-w, 0.03 * r), eye + vec2(0.0, -0.06 * r)],
                    stroke,
                );
                painter.line_segment(
                    [eye + vec2(0.0, -0.06 * r), eye + vec2(w, 0.03 * r)],
                    stroke,
                );
            }
        }
    }

    // Nose and "w" mouth.
    let nose = pos2(c.x, c.y + 0.1 * r);
    painter.add(Shape::convex_polygon(
        vec![
            nose + vec2(-0.08, -0.05) * r,
            nose + vec2(0.08, -0.05) * r,
            nose + vec2(0.0, 0.05) * r,
        ],
        CAT_EAR_INNER,
        Stroke::NONE,
    ));
    let mouth = Stroke::new((r * 0.04).max(1.0), INK);
    let m = |dx: f32, dy: f32| nose + vec2(dx, dy) * r;
    painter.line_segment([m(0.0, 0.05), m(0.0, 0.14)], mouth);
    // A smiling "ω": each half dips down then curls back up.
    for side in [-1.0, 1.0] {
        painter.line_segment([m(0.0, 0.14), m(side * 0.07, 0.2)], mouth);
        painter.line_segment([m(side * 0.07, 0.2), m(side * 0.14, 0.13)], mouth);
    }

    // Whiskers.
    let whisker = Stroke::new((r * 0.025).max(0.75), SOFT_INK);
    for side in [-1.0, 1.0] {
        for dy in [-0.06, 0.08] {
            painter.line_segment(
                [
                    pos2(c.x + side * 0.45 * r, nose.y + dy * r),
                    pos2(c.x + side * 1.1 * r, nose.y + dy * 1.8 * r),
                ],
                whisker,
            );
        }
    }
}
