use std::time::Duration;

use kludgine::app::{Window, WindowBehavior};
use kludgine::figures::units::{Lp, Px};
use kludgine::figures::{Angle, Point, Rect, Size};
use kludgine::shapes::{PathBuilder, Shape};
use kludgine::{Color, PreparedGraphic};

fn main() {
    Test::run();
}

const BLUE_TRIANGLE_SIZE: Px = Px(96);
const RED_SQUARE_SIZE: Lp = Lp::inches(1);

struct Test {
    dips_square: PreparedGraphic<Lp>,
    pixels_triangle: PreparedGraphic<Px>,
    angle: Angle,
}

impl WindowBehavior for Test {
    type Context = ();

    fn initialize(
        _window: Window<'_>,
        graphics: &mut kludgine::Graphics<'_>,
        _context: Self::Context,
    ) -> Self {
        let dips_square = Shape::filled_rect(
            Rect::new(
                Point::new(-RED_SQUARE_SIZE / 2, -RED_SQUARE_SIZE / 2),
                Size::new(RED_SQUARE_SIZE, RED_SQUARE_SIZE),
            ),
            Color::RED,
        )
        .prepare(graphics);
        let height = (BLUE_TRIANGLE_SIZE.pow(2) - (BLUE_TRIANGLE_SIZE / 2).pow(2)).sqrt();
        let pixels_triangle = PathBuilder::new(Point::new(-BLUE_TRIANGLE_SIZE / 2, -height / 2))
            .line_to(Point::new(Px(0), height / 2))
            .line_to(Point::new(BLUE_TRIANGLE_SIZE / 2, -height / 2))
            .close()
            .fill(Color::BLUE)
            .prepare(graphics);
        Self {
            dips_square,
            pixels_triangle,
            angle: Angle::degrees(0),
        }
    }

    fn render<'pass>(
        &'pass mut self,
        mut window: Window<'_>,
        graphics: &mut kludgine::RenderingGraphics<'_, 'pass>,
    ) -> bool {
        window.redraw_in(Duration::from_millis(16));
        self.angle += Angle::degrees(180) * window.elapsed();
        self.dips_square.render(
            Point::new(RED_SQUARE_SIZE / 2, RED_SQUARE_SIZE / 2),
            None,
            Some(self.angle),
            graphics,
        );
        self.pixels_triangle.render(
            Point::new(BLUE_TRIANGLE_SIZE / 2, BLUE_TRIANGLE_SIZE / 2),
            None,
            Some(self.angle),
            graphics,
        );
        true
    }
}
