use iced::{
    Color, Point, Radians, Theme, Vector,
    widget::canvas::{self, Frame, LineDash, Path, Stroke},
};
use uuid::Uuid;

use super::Drawable;

#[derive(Clone, Debug)]
pub struct Door {
    id: Uuid,
    position: Point,
    rotate: bool,
}

impl Door {
    const WIDTH: f32 = 100.0;
    pub fn new(position: Point, rotate: bool) -> Self {
        Self {
            id: Uuid::now_v7(),
            position,
            rotate,
        }
    }
}

impl Drawable for Door {
    fn draw(&self, frame: &mut Frame, _theme: &Theme, selected: bool) {
        frame.with_save(|frame| {
            frame.translate(Vector::new(self.position.x, self.position.y));

            if self.rotate {
                frame.rotate(0.5 * Radians::PI);
            }

            let dot_radius = 3.0;
            let stroke_width = 2.0;
            let door_color = Color::from_rgb(0.0, 1.0, 0.0);
            if selected {
                let selection_color = Color::from_rgba(0.0, 0.5, 1.0, 0.3);
                let halo_width = 10.0;

                // 1. Highlight the leaf line
                frame.stroke(
                    &Path::line(
                        Point::new(-Self::WIDTH / 2.0, 0.0),
                        Point::new(-Self::WIDTH / 2.0, -Self::WIDTH),
                    ),
                    Stroke::default()
                        .with_width(halo_width)
                        .with_color(selection_color),
                );

                // 2. Highlight the swing arc
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

            // 1. Draw the snap-point dots (Frame/Jambs)
            // We create a path for a circle and fill it
            let left_dot = Path::circle(Point::new(-Self::WIDTH / 2.0, 0.0), dot_radius);
            let right_dot = Path::circle(Point::new(Self::WIDTH / 2.0, 0.0), dot_radius);

            frame.fill(&left_dot, door_color);
            frame.fill(&right_dot, door_color);

            // 2. Draw the Door Leaf
            frame.stroke(
                &Path::line(
                    Point::new(-Self::WIDTH / 2.0, 0.0),
                    Point::new(-Self::WIDTH / 2.0, -Self::WIDTH),
                ),
                Stroke::default()
                    .with_width(stroke_width)
                    .with_color(door_color),
            );

            // 3. Draw the Swing Arc
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
        let threshold = 10.0; // Margin of error for clicking

        // 1. Transform the click point into the door's local coordinate system.
        let mut local_p = point - Vector::new(self.position.x, self.position.y);

        if self.rotate {
            // Rotate the point by -90 degrees to align with the door's local axis
            let (sin, cos) = (-0.5 * Radians::PI.0).sin_cos();
            let rx = local_p.x * cos - local_p.y * sin;
            let ry = local_p.x * sin + local_p.y * cos;
            local_p = Point::new(rx, ry);
        }

        // 2. Now check the local point against the door parts
        // The door spans from x: -50 to +50 and y: -100 to 0 in local space.

        // Check Jambs (the dots)
        let left_jamb = Point::new(-Self::WIDTH / 2.0, 0.0);
        let right_jamb = Point::new(Self::WIDTH / 2.0, 0.0);

        if local_p.distance(left_jamb) < threshold || local_p.distance(right_jamb) < threshold {
            return true;
        }

        // Check Door Leaf (the vertical line)
        // Line from (-50, 0) to (-50, -100)
        let leaf_start = Point::new(-Self::WIDTH / 2.0, 0.0);
        let leaf_end = Point::new(-Self::WIDTH / 2.0, -Self::WIDTH);
        if distance_to_segment(local_p, leaf_start, leaf_end) < threshold {
            return true;
        }

        // Check Swing Arc (Is it inside the quarter circle?)
        let dist_to_hinge = local_p.distance(left_jamb);
        let is_within_radius = (dist_to_hinge - Self::WIDTH).abs() < threshold;
        let is_within_angles = local_p.x >= -Self::WIDTH / 2.0 && local_p.y <= 0.0;

        if is_within_radius && is_within_angles {
            return true;
        }

        false
    }

    fn move_by(&mut self, delta: Vector) {
        self.position += delta
    }

    fn duplicate(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            ..self.clone()
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
