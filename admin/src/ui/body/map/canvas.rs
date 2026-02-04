use std::collections::{HashMap, HashSet};

use iced::{
    Event, Rectangle, Renderer, Theme, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Program, Stroke},
};
use uuid::Uuid;

use super::grid::{Grid, Interaction, Message};
use super::types::{Drawable, MapElement};

pub struct MapCanvas<'a> {
    grid: &'a Grid,
    elements: &'a HashMap<Uuid, MapElement>,
    selected: &'a HashSet<Uuid>,
}

impl<'a> MapCanvas<'a> {
    pub fn new(
        grid: &'a Grid,
        elements: &'a HashMap<Uuid, MapElement>,
        selected: &'a HashSet<Uuid>,
    ) -> Self {
        Self {
            grid,
            elements,
            selected,
        }
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

        frame.translate(self.grid.offset);
        frame.scale(self.grid.zoom);

        self.grid.draw_grid(&mut frame, bounds, theme);

        for element in self.elements.values() {
            let selected = self.selected.contains(&element.get_id());
            element.draw(&mut frame, theme, selected);
        }

        if let Some(start) = state.draw_start {
            let end = self
                .grid
                .snap_to_grid(self.grid.screen_to_world(state.last_cursor_pos));
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
