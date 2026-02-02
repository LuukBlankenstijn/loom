use iced::{
    Event, Rectangle, Renderer, Theme, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Program, Stroke},
};

use super::grid::{Grid, Interaction, Message};
use super::types::{Drawable, MapElement};

pub struct MapCanvas<'a> {
    grid: &'a Grid,
    elements: &'a [MapElement],
}

impl<'a> MapCanvas<'a> {
    pub fn new(grid: &'a Grid, elements: &'a [MapElement]) -> Self {
        Self { grid, elements }
    }
}

impl<'a> Program<Message> for MapCanvas<'a> {
    type State = Interaction;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());

        // We use the references inside the briefcase to draw
        frame.translate(self.grid.offset);
        frame.scale(self.grid.zoom);

        self.grid.draw_grid(&mut frame, bounds, theme);

        for element in self.elements {
            element.draw(&mut frame, theme);
        }

        // Ghost wall math using self.grid
        if let Some(start) = state.draw_start {
            let end = self.grid.screen_to_world(state.last_cursor_pos);
            let ghost_color = theme.extended_palette().primary.weak.color;

            frame.stroke(
                &Path::line(start, end),
                Stroke::default().with_width(1.0).with_color(ghost_color),
            );
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        self.grid.update(state, event, bounds, cursor)
    }
}
