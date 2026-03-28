use std::collections::HashSet;

use iced::{
    Event, Rectangle, Renderer, Theme, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Program, Stroke},
};
use ordermap::OrderMap;
use uuid::Uuid;

use crate::{MapMode, Message};

use super::grid::{Grid, Interaction};
use super::types::Drawable;

pub struct MapCanvas<'a, T>
where
    T: Drawable,
{
    grid: &'a Grid,
    elements: &'a OrderMap<Uuid, T>,
    selected: &'a HashSet<Uuid>,
    mode: MapMode,
}

impl<'a, T> MapCanvas<'a, T>
where
    T: Drawable,
{
    pub fn new(
        grid: &'a Grid,
        elements: &'a OrderMap<Uuid, T>,
        selected: &'a HashSet<Uuid>,
        mode: MapMode,
    ) -> Self {
        Self {
            grid,
            elements,
            selected,
            mode,
        }
    }
}

impl<'a, T> Program<Message<T>> for MapCanvas<'a, T>
where
    T: Drawable,
{
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

        if matches!(self.mode, MapMode::Edit) {
            self.grid.draw_grid(&mut frame, bounds);
        }

        for element in self.elements.values() {
            let selected = self.selected.contains(&element.get_id());
            element.draw(&mut frame, self.grid.zoom, selected);
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
    ) -> Option<canvas::Action<Message<T>>> {
        self.grid.update(state, event, bounds, cursor, &self.mode)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        self.grid
            .mouse_interaction(state, bounds, cursor, &self.mode)
    }
}
