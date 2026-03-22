use iced::{
    Color, Point, Radians, Vector,
    widget::canvas::{self, Frame, LineDash, Path, Stroke},
};
use loom_core::map::door::Door;
use uuid::Uuid;

use crate::types::prelude::{AddIcedVector, IntoIced};

use super::Drawable;

impl Drawable for Door {
    fn draw(&self, frame: &mut Frame, scale: f32, selected: bool) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(self.position.x, self.position.y));

            frame.rotate(self.rotation.to_iced());

            let dot_radius = 3.0;
            let stroke_width = 2.0 * scale;
            let door_color = Color::from_rgb(0.99, 0.73, 0.19);
            if selected {
                let selection_color = Color::from_rgba(0.0, 0.5, 1.0, 0.3);
                let halo_width = 10.0;
                frame.stroke(
                    &Path::line(
                        Point::new(-Self::WIDTH / 2.0, 0.0),
                        Point::new(-Self::WIDTH / 2.0, -Self::WIDTH),
                    ),
                    Stroke::default()
                        .with_width(halo_width)
                        .with_color(selection_color),
                );

                let arc_path = Path::new(|builder| {
                    builder.arc(canvas::path::Arc {
                        center: Point::new(-Self::WIDTH / 2.0, 0.0),
                        radius: Self::WIDTH,
                        start_angle: -0.5 * Radians::PI,
                        end_angle: 0.0 * Radians::PI,
                    });
                });
                frame.stroke(
                    &arc_path,
                    Stroke::default()
                        .with_width(halo_width)
                        .with_color(selection_color),
                );
            }

            let left_dot = Path::circle(Point::new(-Self::WIDTH / 2.0, 0.0), dot_radius);
            let right_dot = Path::circle(Point::new(Self::WIDTH / 2.0, 0.0), dot_radius);

            frame.fill(&left_dot, door_color);
            frame.fill(&right_dot, door_color);

            frame.stroke(
                &Path::line(
                    Point::new(-Self::WIDTH / 2.0, 0.0),
                    Point::new(-Self::WIDTH / 2.0, -Self::WIDTH),
                ),
                Stroke::default()
                    .with_width(stroke_width)
                    .with_color(door_color),
            );

            let arc = Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center: Point::new(-Self::WIDTH / 2.0, 0.0),
                    radius: Self::WIDTH,
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

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn is_hit(&self, point: Point) -> bool {
        let threshold = 10.0;

        // world -> local: undo translation
        let mut p = point - Vector::new(self.position.x, self.position.y);

        // undo rotation (inverse of what draw() applies)
        let (sin, cos) = (-(self.rotation.to_iced().0)).sin_cos();
        p = Point::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos);

        // now run your existing local-space checks unchanged
        let left_jamb = Point::new(-Self::WIDTH / 2.0, 0.0);
        let right_jamb = Point::new(Self::WIDTH / 2.0, 0.0);

        if p.distance(left_jamb) < threshold || p.distance(right_jamb) < threshold {
            return true;
        }

        let leaf_start = Point::new(-Self::WIDTH / 2.0, 0.0);
        let leaf_end = Point::new(-Self::WIDTH / 2.0, -Self::WIDTH);
        if distance_to_segment(p, leaf_start, leaf_end) < threshold {
            return true;
        }

        let dist_to_hinge = p.distance(left_jamb);
        let is_within_radius = (dist_to_hinge - Self::WIDTH).abs() < threshold;
        let is_within_angles = p.x >= -Self::WIDTH / 2.0 && p.y <= 0.0;

        is_within_radius && is_within_angles
    }

    fn move_by(&mut self, delta: Vector) {
        self.position.add_vector(delta);
    }

    fn duplicate(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            ..self.clone()
        }
    }

    fn rotate(&mut self, rotation: Option<super::Rotation>) {
        if let Some(rotation) = rotation {
            self.rotation = rotation
        } else {
            self.rotation = self.rotation.rotate_cw()
        }
    }
}

fn distance_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let v = Vector::new(b.x - a.x, b.y - a.y);
    let w = Vector::new(p.x - a.x, p.y - a.y);
    let c1 = w.x * v.x + w.y * v.y;
    let c2 = v.x * v.x + v.y * v.y;
    if c1 <= 0.0 {
        return p.distance(a);
    }
    if c2 <= c1 {
        return p.distance(b);
    }
    let b_coord = c1 / c2;
    let pb = a + v * b_coord;
    p.distance(pb)
}
