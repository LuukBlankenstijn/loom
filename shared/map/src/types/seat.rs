use crate::types::prelude::AddIcedVector;

use super::Drawable;
use iced::{
    Color, Point, Radians, Size, Vector,
    widget::canvas::{Frame, Path, Stroke},
};
use loom_core::map::seat::Seat;
use uuid::Uuid;

impl Drawable for Seat {
    fn draw(&self, frame: &mut Frame, scale: f32, selected: bool) {
        let (total_w, total_h) = self.get_total_bounds();

        let accent_color = Color::from_rgb(0.29, 0.51, 0.76);
        let laptop_color = Color::from_rgb(0.3, 0.3, 0.3);
        let chair_color = Color::from_rgba(0.5, 0.5, 0.5, 0.8);

        frame.with_save(|frame| {
            frame.translate(Vector::new(
                self.position.x + total_w / 2.0,
                self.position.y + total_h / 2.0,
            ));

            frame.rotate((self.rotation as u16 as f32).to_radians());

            // Center the drawing logic around the pivot
            let vertical_shift = Self::CHAIR_PROTRUSION / 2.0;

            // 1. Selection Halo
            if selected {
                let selection_color = Color::from_rgba(0.0, 0.5, 1.0, 0.2);
                let halo = Path::rectangle(
                    Point::new(
                        -Self::TABLE_W / 2.0,
                        -Self::TABLE_H / 2.0 - Self::CHAIR_PROTRUSION + vertical_shift,
                    ),
                    Size::new(Self::TABLE_W, Self::TABLE_H + Self::CHAIR_PROTRUSION),
                );
                frame.fill(&halo, selection_color);
            }

            let chair_y = -Self::TABLE_H / 2.0 + vertical_shift;
            let x_offsets = [-65.0, 0.0, 65.0];

            for x in x_offsets {
                let chair_path = Path::new(|b| {
                    b.arc(iced::widget::canvas::path::Arc {
                        center: Point::new(x, chair_y - 5.0),
                        radius: Self::CHAIR_ARC_RADIUS,
                        start_angle: 1.0 * Radians::PI,
                        end_angle: 2.0 * Radians::PI,
                    });
                });
                frame.stroke(
                    &chair_path,
                    Stroke::default()
                        .with_color(chair_color)
                        .with_width(2.0 * scale),
                );
            }

            // 3. Draw Table
            let table_path = Path::rectangle(
                Point::new(-Self::TABLE_W / 2.0, -Self::TABLE_H / 2.0 + vertical_shift),
                Size::new(Self::TABLE_W, Self::TABLE_H),
            );
            frame.stroke(
                &table_path,
                Stroke::default()
                    .with_color(accent_color)
                    .with_width(2.5 * scale),
            );

            let laptop_w = 40.0;
            let laptop_h = 25.0;
            let laptop_rect = Path::rectangle(
                Point::new(-laptop_w / 2.0, -laptop_h / 2.0 + vertical_shift),
                Size::new(laptop_w, laptop_h),
            );
            frame.stroke(
                &laptop_rect,
                Stroke::default()
                    .with_color(laptop_color)
                    .with_width(1.5 * scale),
            );
            frame.stroke(
                &Path::line(
                    Point::new(-laptop_w / 2.0, vertical_shift),
                    Point::new(laptop_w / 2.0, vertical_shift),
                ),
                Stroke::default()
                    .with_color(laptop_color)
                    .with_width(1.0 * scale),
            );
        });
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn is_hit(&self, point: Point) -> bool {
        let (w, h) = self.get_total_bounds();
        point.x >= self.position.x
            && point.x <= self.position.x + w
            && point.y >= self.position.y
            && point.y <= self.position.y + h
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
