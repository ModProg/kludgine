use std::time::Duration;

use kludgine::figures::units::Dips;
use kludgine::figures::{Angle, Point, Rect, Size};
use kludgine::shapes::Shape;
use kludgine::text::TextOrigin;
use kludgine::Color;

const RED_SQUARE_SIZE: Dips = Dips::inches(1);

fn main() {
    let mut angle = Angle::degrees(0);
    kludgine::app::run(move |mut renderer, mut window| {
        window.redraw_in(Duration::from_millis(16));
        angle += Angle::degrees(180) * window.elapsed().as_secs_f32();
        renderer.draw_shape(
            &Shape::filled_rect(
                Rect::<Dips>::new(
                    Point::new(-RED_SQUARE_SIZE / 2, -RED_SQUARE_SIZE / 2),
                    Size::new(RED_SQUARE_SIZE, RED_SQUARE_SIZE),
                ),
                Color::RED,
            ),
            Point::<Dips>::new(RED_SQUARE_SIZE / 2, RED_SQUARE_SIZE / 2),
            Some(angle),
            None,
        );
        renderer.draw_text(
            "Hello, World!",
            TextOrigin::Center,
            Point::<Dips>::new(RED_SQUARE_SIZE / 2, RED_SQUARE_SIZE / 2),
            None,
            None,
        );
        true
    })
}
