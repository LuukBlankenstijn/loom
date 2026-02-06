use iced::{
    Color, Point,
    widget::canvas::{Path, Stroke},
};
use uuid::Uuid;

use crate::MapElement;

use super::Drawable;

#[derive(Clone, Debug)]
pub struct Wall {
    id: Uuid,
    start: Point,
    end: Point,
}

impl Wall {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            id: Uuid::now_v7(),
            start,
            end,
        }
    }
    pub fn get_test() -> Vec<Self> {
        vec![
            Self::new(Point::new(50.0, 0.0), Point::new(200.0, 0.0)),
            Self::new(Point::new(200.0, 0.0), Point::new(200.0, 150.0)),
        ]
    }
}

impl Drawable for Wall {
    fn draw(&self, frame: &mut iced::widget::canvas::Frame, _theme: &iced::Theme, selected: bool) {
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

        if selected {
            let selection_color = Color::from_rgba(0.0, 0.5, 1.0, 0.3);
            let halo_width = 10.0;

            frame.stroke(
                &Path::line(self.start, self.end),
                Stroke::default()
                    .with_width(halo_width)
                    .with_color(selection_color),
            );
        }
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn is_hit(&self, point: Point) -> bool {
        let threshold = 5.0;

        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.end.x;
        let y2 = self.end.y;
        let px = point.x;
        let py = point.y;

        // Square of the length of the segment
        let dx = x2 - x1;
        let dy = y2 - y1;
        let l2 = dx * dx + dy * dy;

        // If the segment is just a point (start == end)
        if l2 == 0.0 {
            return point.distance(self.start) < threshold;
        }

        // Calculate the t parameter of the projection
        // t is the position of the point along the line (0.0 to 1.0)
        let t = ((px - x1) * dx + (py - y1) * dy) / l2;

        // Clamp t to ensure we are looking at the segment, not the infinite line
        let t = t.clamp(0.0, 1.0);

        // Find the closest point on the segment
        let closest_x = x1 + t * dx;
        let closest_y = y1 + t * dy;

        // Check the distance from the point to the closest point on the segment
        let distance_sq = (px - closest_x).powi(2) + (py - closest_y).powi(2);

        distance_sq < threshold.powi(2)
    }

    fn move_by(&mut self, delta: iced::Vector) {
        self.start += delta;
        self.end += delta;
    }

    fn duplicate(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            ..self.clone()
        }
    }
}

impl From<Wall> for MapElement {
    fn from(value: Wall) -> Self {
        MapElement::Wall(value)
    }
}
