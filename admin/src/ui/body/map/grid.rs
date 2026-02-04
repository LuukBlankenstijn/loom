use iced::{
    Event, Point, Rectangle, Theme, Vector,
    keyboard::{self, Key, Modifiers, key::Named},
    mouse,
    widget::canvas::{self, Frame, Path, Stroke},
};

#[derive(Clone, Debug)]
pub struct Grid {
    pub offset: Vector<f32>,
    pub zoom: f32,
}

#[derive(Clone, Debug)]
pub enum Message {
    MapPanned(Vector<f32>),
    MapZoomed { factor: f32, cursor: Point },
    DrawFinish(Point, Point),
    RequestSelect(Point),
    ClearSelection,
    DeleteSelection,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vector::default(),
        }
    }

    pub fn update_canvas(&mut self, message: Message) {
        match message {
            Message::MapPanned(delta) => {
                self.offset.x += delta.x;
                self.offset.y += delta.y;
            }
            Message::MapZoomed { factor, cursor } => {
                let old_zoom = self.zoom;
                let new_zoom = (old_zoom * factor).clamp(0.3, 2.0);
                self.zoom = new_zoom;

                // pan the map to zoom to the cursor
                let actual_factor = new_zoom / old_zoom;
                self.offset.x = cursor.x - (cursor.x - self.offset.x) * actual_factor;
                self.offset.y = cursor.y - (cursor.y - self.offset.y) * actual_factor;
            }
            _ => {}
        }
    }

    pub fn draw_grid(&self, frame: &mut Frame, bounds: Rectangle, theme: &Theme) {
        let palette = theme.extended_palette();

        let grid_size = 100.0;
        let grid_color = palette.secondary.weak.text.scale_alpha(0.3);

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
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = cursor.position_in(bounds);

        match event {
            Event::Mouse(move_event) => {
                match move_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(pos) = cursor_position {
                            // panning and drawing logic
                            if state.modifiers.control() {
                                state.is_panning = true;
                            } else {
                                state.draw_start =
                                    Some(self.snap_to_grid(self.screen_to_world(pos)))
                            }
                            state.last_cursor_pos = pos;

                            // selection logic
                            if state.modifiers.shift() {
                                let world_pos = self.screen_to_world(pos);
                                return Some(canvas::Action::publish(Message::RequestSelect(
                                    world_pos,
                                )));
                            }
                            return Some(canvas::Action::request_redraw().and_capture());
                        }
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
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
                                return Some(canvas::Action::publish(Message::DrawFinish(
                                    start, end,
                                )));
                            }
                        }
                        state.draw_start = None;
                        state.is_panning = false;
                        return Some(canvas::Action::request_redraw());
                    }
                    mouse::Event::CursorMoved { .. } => {
                        if let Some(pos) = cursor_position {
                            if state.is_panning {
                                let delta = pos - state.last_cursor_pos;
                                state.last_cursor_pos = pos;
                                return Some(
                                    canvas::Action::publish(Message::MapPanned(delta))
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
                                canvas::Action::publish(Message::MapZoomed {
                                    factor,
                                    cursor: pos,
                                })
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
            }) => match key {
                Key::Named(Named::Delete) => {
                    return Some(canvas::Action::publish(Message::DeleteSelection));
                }
                Key::Named(Named::Escape) => {
                    return Some(canvas::Action::publish(Message::ClearSelection));
                }
                _ => {}
            },
            _ => {}
        }

        None
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Interaction {
    pub is_panning: bool,
    pub last_cursor_pos: Point,
    pub modifiers: Modifiers,

    pub draw_start: Option<Point>,
}
