use iced::{
    Color, Point,
    widget::canvas::{Path, Stroke},
};

use crate::ui::body::map::types::Drawable;

#[derive(Clone, Debug)]
pub struct Wall {
    start: Point,
    end: Point,
}

impl Drawable for Wall {
    fn draw(&self, frame: &mut iced::widget::canvas::Frame, theme: &iced::Theme) {
        let dot_radius = 3.0;
        let stroke_width = 2.0;
        let wall_color = Color::from_rgb(1.0, 0.0, 0.0);

        // 1. Draw the snap-point dots (Frame/Jambs)
        // We create a path for a circle and fill it
        let left_dot = Path::circle(self.start, dot_radius);
        let right_dot = Path::circle(self.end, dot_radius);

        frame.fill(&left_dot, wall_color);
        frame.fill(&right_dot, wall_color);

        frame.stroke(
            &Path::line(self.start, self.end),
            Stroke::default()
                .with_width(stroke_width)
                .with_color(wall_color),
        );
    }
}

impl Wall {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
    pub fn get_test() -> Vec<Wall> {
        vec![
            Wall {
                start: Point::new(60.0, 10.0),
                end: Point::new(200.0, 10.0),
            },
            Wall {
                start: Point::new(200.0, 10.0),
                end: Point::new(200.0, 150.0),
            },
        ]
    }
}
