mod canvas;
mod grid;
mod types;

use std::collections::{HashMap, HashSet};

use iced::Length::Fill;
use iced::widget::Canvas;
use iced::{Element, Task};
use uuid::Uuid;

use crate::ui::body::map::grid::Grid;
use crate::ui::body::map::types::door::Door;
use crate::ui::body::map::types::wall::Wall;
use crate::ui::body::map::types::{Drawable, MapElement};

#[derive(Debug)]
pub struct MapApp {
    grid: Grid,
    elements: HashMap<Uuid, MapElement>,
    selected: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddElement(MapElement),
    Canvas(grid::Message),
    ToggleSelect(Uuid),
}

impl Default for MapApp {
    fn default() -> Self {
        let doors = Door::get_test();
        let walls = Wall::get_test();

        let elements = doors
            .into_iter()
            .map(|d| (d.get_id(), MapElement::Door(d)))
            .chain(walls.into_iter().map(|w| (w.get_id(), MapElement::Wall(w))))
            .collect();
        Self {
            elements,
            grid: Grid::new(),
            selected: Default::default(),
        }
    }
}

impl MapApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Canvas(msg) => match msg {
                grid::Message::DrawFinish(start, end) => {
                    return Task::done(Message::AddElement(MapElement::Wall(Wall::new(
                        start, end,
                    ))));
                }
                grid::Message::ClearSelection => self.selected.clear(),
                grid::Message::DeleteSelection => {
                    for id in &self.selected {
                        self.elements.remove(id);
                    }
                    self.selected.clear();
                }
                grid::Message::RequestSelect(point) => {
                    let hit_id = self
                        .elements
                        .values()
                        .find(|e| e.is_hit(point))
                        .map(|e| e.get_id());
                    if let Some(id) = hit_id {
                        return Task::done(Message::ToggleSelect(id));
                    }
                }
                _ => self.grid.update_canvas(msg),
            },
            Message::AddElement(element) => {
                match element {
                    MapElement::Door(door) => {
                        self.elements.insert(door.get_id(), MapElement::Door(door))
                    }
                    MapElement::Wall(wall) => {
                        self.elements.insert(wall.get_id(), MapElement::Wall(wall))
                    }
                };
            }
            Message::ToggleSelect(id) => {
                if self.selected.contains(&id) {
                    self.selected.remove(&id);
                } else {
                    let _ = self.selected.insert(id);
                }
            }
        };
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let canvas: Element<'_, grid::Message> = Canvas::new(canvas::MapCanvas::new(
            &self.grid,
            &self.elements,
            &self.selected,
        ))
        .width(Fill)
        .height(Fill)
        .into();

        canvas.map(Message::Canvas)
    }
}
