use loom_core::map::{Map, MapElement, Point, Rotation, seat::Seat};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use uuid::Uuid;

use crate::error::AppError;

const IMAGE_W: u32 = 1200;
const IMAGE_H: u32 = 800;
const PADDING: f32 = 60.0;
const STROKE_WALL: f32 = 6.0;
const STROKE_SEAT: f32 = 2.0;

pub fn render_map_png(map: &Map, highlight_seat_id: Option<Uuid>) -> Result<Vec<u8>, AppError> {
    let points = collect_points(map);
    if points.is_empty() {
        return Err(AppError::FailedPrecondition(
            "map has no renderable elements".to_string(),
        ));
    }

    let (min_x, min_y, max_x, max_y) = bbox(&points);
    let world_w = (max_x - min_x).max(1.0);
    let world_h = (max_y - min_y).max(1.0);
    let avail_w = IMAGE_W as f32 - 2.0 * PADDING;
    let avail_h = IMAGE_H as f32 - 2.0 * PADDING;
    let scale = (avail_w / world_w).min(avail_h / world_h);
    let off_x = PADDING + (avail_w - world_w * scale) / 2.0 - min_x * scale;
    let off_y = PADDING + (avail_h - world_h * scale) / 2.0 - min_y * scale;
    let to_screen = |x: f32, y: f32| (x * scale + off_x, y * scale + off_y);

    let mut pixmap = Pixmap::new(IMAGE_W, IMAGE_H)
        .ok_or_else(|| AppError::Internal("failed to allocate pixmap".to_string()))?;
    pixmap.fill(Color::WHITE);

    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.anti_alias = true;

    for el in &map.elements {
        match el {
            MapElement::Wall(w) => {
                let (sx, sy) = to_screen(w.start.x, w.start.y);
                let (ex, ey) = to_screen(w.end.x, w.end.y);
                let mut pb = PathBuilder::new();
                pb.move_to(sx, sy);
                pb.line_to(ex, ey);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint,
                        &Stroke {
                            width: STROKE_WALL,
                            ..Default::default()
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
            MapElement::Seat(s) => {
                let corners = seat_table_corners(s.position, s.rotation);
                let mut pb = PathBuilder::new();
                let (x0, y0) = to_screen(corners[0].0, corners[0].1);
                pb.move_to(x0, y0);
                for c in &corners[1..] {
                    let (x, y) = to_screen(c.0, c.1);
                    pb.line_to(x, y);
                }
                pb.close();
                if let Some(path) = pb.finish() {
                    if Some(s.id) == highlight_seat_id {
                        pixmap.fill_path(
                            &path,
                            &paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    } else {
                        pixmap.stroke_path(
                            &path,
                            &paint,
                            &Stroke {
                                width: STROKE_SEAT,
                                ..Default::default()
                            },
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            MapElement::Door(_) => {}
        }
    }

    pixmap
        .encode_png()
        .map_err(|e| AppError::Internal(format!("encode png: {e}")))
}

fn collect_points(map: &Map) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    for el in &map.elements {
        match el {
            MapElement::Wall(w) => {
                points.push((w.start.x, w.start.y));
                points.push((w.end.x, w.end.y));
            }
            MapElement::Seat(s) => {
                points.extend(seat_table_corners(s.position, s.rotation));
            }
            MapElement::Door(_) => {}
        }
    }
    points
}

fn bbox(points: &[(f32, f32)]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (x, y) in points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    (min_x, min_y, max_x, max_y)
}

fn seat_table_corners(position: Point, rotation: Rotation) -> [(f32, f32); 4] {
    let total_w = Seat::TABLE_W;
    let total_h = Seat::TABLE_H + Seat::CHAIR_PROTRUSION;
    let cx = position.x + total_w / 2.0;
    let cy = position.y + total_h / 2.0;
    let vshift = Seat::CHAIR_PROTRUSION / 2.0;
    let half_w = Seat::TABLE_W / 2.0;
    let top = -Seat::TABLE_H / 2.0 + vshift;
    let bot = Seat::TABLE_H / 2.0 + vshift;
    let local = [(-half_w, top), (half_w, top), (half_w, bot), (-half_w, bot)];
    let angle = rotation as u16 as f32;
    let rad = angle.to_radians();
    let (s, c) = rad.sin_cos();
    local.map(|(x, y)| (cx + x * c - y * s, cy + x * s + y * c))
}
