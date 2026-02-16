use iced::{
    Color, Event, Point, Rectangle, Vector,
    keyboard::{self, Key, Modifiers, key::Named},
    mouse,
    widget::canvas::{self, Frame, Path, Stroke},
};

use crate::{MapMode, Message, messsage::GridMessage};

#[derive(Clone, Debug)]
pub struct Grid {
    pub offset: Vector<f32>,
    pub zoom: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Interaction {
    pub is_panning: bool,
    pub last_cursor_pos: Point,
    pub modifiers: Modifiers,

    pub draw_start: Option<Point>,

    pub is_moving: bool,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vector::default(),
        }
    }

    pub fn update_canvas(&mut self, message: GridMessage) {
        match message {
            GridMessage::MapPanned(delta) => {
                self.offset.x += delta.x;
                self.offset.y += delta.y;
            }
            GridMessage::MapZoomed { factor, cursor } => {
                let old_zoom = self.zoom;
                let new_zoom = (old_zoom * factor).clamp(0.1, 2.0);
                self.zoom = new_zoom;

                // pan the map to zoom to the cursor
                let actual_factor = new_zoom / old_zoom;
                self.offset.x = cursor.x - (cursor.x - self.offset.x) * actual_factor;
                self.offset.y = cursor.y - (cursor.y - self.offset.y) * actual_factor;
            }
            _ => {}
        }
    }

    pub fn draw_grid(&self, frame: &mut Frame, bounds: Rectangle) {
        let grid_size = 100.0;
        let grid_color = Color::WHITE.scale_alpha(0.5 * self.zoom);

        // Calculate the visible world-space coordinates
        let top_left = Point::new(-self.offset.x / self.zoom, -self.offset.y / self.zoom);
        let bottom_right = Point::new(
            (bounds.width - self.offset.x) / self.zoom,
            (bounds.height - self.offset.y) / self.zoom,
        );

        let stroke = Stroke::default().with_color(grid_color).with_width(0.5);

        // Draw Vertical Lines
        let start_x = (top_left.x / grid_size).ceil() as i32;
        let end_x = (bottom_right.x / grid_size).floor() as i32;

        for x in start_x..=end_x {
            let x_pos = x as f32 * grid_size;
            frame.stroke(
                &Path::line(
                    Point::new(x_pos, top_left.y),
                    Point::new(x_pos, bottom_right.y),
                ),
                stroke,
            );
        }

        // Draw Horizontal Lines
        let start_y = (top_left.y / grid_size).floor() as i32;
        let end_y = (bottom_right.y / grid_size).ceil() as i32;

        for y in start_y..=end_y {
            let y_pos = y as f32 * grid_size;
            frame.stroke(
                &Path::line(
                    Point::new(top_left.x, y_pos),
                    Point::new(bottom_right.x, y_pos),
                ),
                stroke,
            );
        }
    }

    pub fn screen_to_world(&self, pos: Point) -> Point {
        Point::new(
            (pos.x - self.offset.x) / self.zoom,
            (pos.y - self.offset.y) / self.zoom,
        )
    }

    pub fn snap_to_grid(&self, point: Point) -> Point {
        let snap_units = 10.0;
        let x = (point.x / snap_units).round() * snap_units;
        let y = (point.y / snap_units).round() * snap_units;
        Point::new(x, y)
    }

    pub fn update(
        &self,
        state: &mut Interaction,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        mode: &MapMode,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = cursor.position_in(bounds);
        let can_edit = matches!(mode, MapMode::Edit);

        match event {
            Event::Mouse(move_event) => {
                match move_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(pos) = cursor_position {
                            // panning and drawing logic
                            if state.modifiers.shift() && can_edit {
                                state.draw_start =
                                    Some(self.snap_to_grid(self.screen_to_world(pos)))
                            } else if state.modifiers.alt() && can_edit {
                                state.is_moving = true;
                            } else {
                                state.is_panning = true;
                            }
                            state.last_cursor_pos = pos;
                            return Some(canvas::Action::request_redraw().and_capture());
                        }
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
                        if let Some(pos) = cursor_position {
                            // selection logic
                            if can_edit {
                                let world_pos = self.screen_to_world(pos);
                                return Some(canvas::Action::publish(
                                    GridMessage::RequestSelect(world_pos).into(),
                                ));
                            }
                        }

                        state.draw_start = None;
                        return Some(canvas::Action::request_redraw().and_capture());
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        if let Some(start) = state.draw_start
                            && let Some(pos) = cursor_position
                        {
                            let end = self.snap_to_grid(self.screen_to_world(pos));
                            state.draw_start = None;

                            // Only add if the wall has meaningful length
                            if start.distance(end) > 1.0 {
                                return Some(canvas::Action::publish(
                                    GridMessage::DrawFinish(start, end).into(),
                                ));
                            }
                        }
                        state.draw_start = None;
                        state.is_panning = false;
                        state.is_moving = false;
                        return Some(canvas::Action::request_redraw());
                    }
                    mouse::Event::CursorMoved { .. } => {
                        if let Some(pos) = cursor_position {
                            if state.is_panning {
                                let delta = pos - state.last_cursor_pos;
                                state.last_cursor_pos = pos;
                                return Some(
                                    canvas::Action::publish(GridMessage::MapPanned(delta).into())
                                        .and_capture(),
                                );
                            }
                            if state.is_moving {
                                let current_world = self.screen_to_world(pos);
                                let last_world = self.screen_to_world(state.last_cursor_pos);
                                let delta = self.snap_to_grid(current_world)
                                    - self.snap_to_grid(last_world);
                                state.last_cursor_pos = pos;
                                return Some(
                                    canvas::Action::publish(Message::MoveSelection(delta))
                                        .and_capture(),
                                );
                            }

                            state.last_cursor_pos = pos;

                            if state.draw_start.is_some() {
                                return Some(canvas::Action::request_redraw().and_capture());
                            }
                        }
                    }
                    mouse::Event::WheelScrolled { delta } => {
                        if let Some(pos) = cursor_position {
                            let factor = match delta {
                                mouse::ScrollDelta::Lines { y, .. }
                                | mouse::ScrollDelta::Pixels { y, .. } => {
                                    if y > &0.0 {
                                        1.1
                                    } else if y < &0.0 {
                                        0.9
                                    } else {
                                        return None;
                                    }
                                }
                            };

                            return Some(
                                canvas::Action::publish(
                                    GridMessage::MapZoomed {
                                        factor,
                                        cursor: pos,
                                    }
                                    .into(),
                                )
                                .and_capture(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modified_key: _modified_key,
                physical_key: _physical_key,
                location: _location,
                modifiers: _modifiers,
                text: _text,
                repeat: _repeat,
            }) => match key.as_ref() {
                Key::Named(Named::Delete) => {
                    return Some(canvas::Action::publish(Message::DeleteSelection));
                }
                Key::Named(Named::Escape) => {
                    return Some(canvas::Action::publish(Message::ClearSelection));
                }
                Key::Character("c") => {
                    return Some(canvas::Action::publish(Message::DuplicateSelection));
                }
                Key::Character("r") => {
                    return Some(canvas::Action::publish(Message::RotateSelection));
                }
                _ => {}
            },
            _ => {}
        }

        None
    }

    pub fn mouse_interaction(
        &self,
        state: &Interaction,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        mode: &MapMode,
    ) -> mouse::Interaction {
        let can_edit = matches!(mode, MapMode::Edit);
        if can_edit {
            if state.modifiers.control() {
                return mouse::Interaction::Crosshair;
            } else if state.modifiers.shift() {
                return mouse::Interaction::Cell;
            }
        }
        mouse::Interaction::default()
    }
}
