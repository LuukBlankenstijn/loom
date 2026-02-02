use iced::{
    Element, Event, Length, Point, Rectangle, Renderer, Theme, Vector, mouse,
    widget::{
        Canvas,
        canvas::{self, Frame, Geometry, Path, Program, Stroke},
    },
};

#[derive(Clone, Debug)]
pub struct Grid {
    offset: Vector<f32>,
    zoom: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Interaction {
    pub is_panning: bool,
    pub last_cursor_pos: Point,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            offset: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    MapPanned(Vector<f32>),
    MapZoomed { factor: f32, cursor: Point },
}

impl Grid {
    pub fn view(&self) -> Element<'_, Message> {
        Canvas::new(self)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: Message) {
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
        }
    }

    fn draw_grid(&self, frame: &mut Frame, bounds: Rectangle, theme: &Theme) {
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
}

impl Program<Message> for Grid {
    type State = Interaction;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            theme.extended_palette().background.neutral.color,
        );
        frame.translate(self.offset);
        frame.scale(self.zoom);
        self.draw_grid(&mut frame, bounds, theme);

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Corrected return type
        let cursor_position = cursor.position_in(bounds);

        match event {
            // 1. Start Panning
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor_position {
                    state.is_panning = true;
                    state.last_cursor_pos = pos;
                    return Some(canvas::Action::request_redraw().and_capture());
                }
            }

            // 2. Stop Panning
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_panning = false;
                return Some(canvas::Action::request_redraw());
            }

            // 3. Handle Panning Movement
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_panning
                    && let Some(pos) = cursor_position
                {
                    let delta = pos - state.last_cursor_pos;
                    state.last_cursor_pos = pos;

                    return Some(canvas::Action::publish(Message::MapPanned(delta)).and_capture());
                }
            }

            // 4. Handle Zooming
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
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

        None
    }
}
