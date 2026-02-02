use iced::{
    Color, Point, Radians, Theme, Vector,
    widget::canvas::{self, Frame, LineDash, Path, Stroke},
};

use crate::ui::body::map::types::Drawable;

#[derive(Clone, Debug)]
pub struct Door {
    position: Point,
    rotate: bool,
}

impl Drawable for Door {
    fn draw(&self, frame: &mut Frame, theme: &Theme) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(self.position.x, self.position.y));

            if self.rotate {
                frame.rotate(0.5 * Radians::PI);
            }

            let door_width = 100.0;
            let dot_radius = 3.0;
            let stroke_width = 2.0;
            let door_color = Color::from_rgb(0.0, 1.0, 0.0);

            // 1. Draw the snap-point dots (Frame/Jambs)
            // We create a path for a circle and fill it
            let left_dot = Path::circle(Point::new(-door_width / 2.0, 0.0), dot_radius);
            let right_dot = Path::circle(Point::new(door_width / 2.0, 0.0), dot_radius);

            frame.fill(&left_dot, door_color);
            frame.fill(&right_dot, door_color);

            // 2. Draw the Door Leaf
            frame.stroke(
                &Path::line(
                    Point::new(-door_width / 2.0, 0.0),
                    Point::new(-door_width / 2.0, -door_width),
                ),
                Stroke::default()
                    .with_width(stroke_width)
                    .with_color(door_color),
            );

            // 3. Draw the Swing Arc
            let arc = Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center: Point::new(-door_width / 2.0, 0.0),
                    radius: door_width,
                    start_angle: -0.5 * Radians::PI,
                    end_angle: 0.0 * Radians::PI,
                });
            });

            let segments = [5.0, 5.0];
            let dashed_stroke = Stroke {
                width: stroke_width / 2.0,
                line_dash: LineDash {
                    segments: &segments,
                    offset: 0,
                },
                ..Stroke::default()
            }
            .with_color(door_color);

            frame.stroke(&arc, dashed_stroke);
        });
    }
}

impl Door {
    pub fn get_test() -> Vec<Door> {
        vec![
            Door {
                position: Point::new(10.0, 10.0),
                rotate: false,
            },
            Door {
                position: Point::new(200.0, 200.0),
                rotate: true,
            },
        ]
    }
}
