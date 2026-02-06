mod canvas;
mod grid;
mod types;

use grid::Grid;
pub use types::{Door, Drawable, MapElement, Wall};

use std::collections::{HashMap, HashSet};

use iced::Length::Fill;
use iced::widget::Canvas;
use iced::{Element, Task, Vector};
use uuid::Uuid;

use crate::grid::SystemMessage;

#[derive(Default, Debug, Clone)]
pub enum MapMode {
    #[default]
    View,
    Edit,
}

#[derive(Debug)]
pub struct Map {
    grid: Grid,
    elements: HashMap<Uuid, MapElement>,
    selected: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddElement(MapElement),
    ToggleSelect(Uuid),
    System(SystemMessage),
    ClearSelection,
    DeleteSelection,
    DuplicateSelection,
    MoveSelection(Vector),
}

impl Map {
    pub fn new(elements: Vec<MapElement>) -> Self {
        let elements = elements.into_iter().map(|e| (e.get_id(), e)).collect();
        Self {
            elements,
            grid: Grid::new(),
            selected: Default::default(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::System(msg) => match msg {
                grid::SystemMessage::DrawFinish(start, end) => {
                    return Task::done(Message::AddElement(MapElement::Wall(Wall::new(
                        start, end,
                    ))));
                }
                SystemMessage::RequestSelect(point) => {
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
            Message::ClearSelection => self.selected.clear(),
            Message::DeleteSelection => {
                for id in &self.selected {
                    self.elements.remove(id);
                }
                self.selected.clear();
            }
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
            Message::MoveSelection(delta) => {
                for id in &self.selected {
                    if let Some(element) = self.elements.get_mut(id) {
                        element.move_by(delta);
                    }
                }
            }
            Message::DuplicateSelection => {
                let ids = self.selected.clone();
                self.selected.clear();
                for id in ids {
                    if let Some(element) = self.elements.get(&id) {
                        let mut new = element.duplicate();
                        new.move_by(Vector::new(10.0, 10.0));
                        self.selected.insert(new.get_id());
                        self.elements.insert(new.get_id(), new);
                    }
                }
            }
        };
        Task::none()
    }

    pub fn view(&self, map_mode: MapMode) -> Element<'_, Message> {
        let canvas: Element<'_, Message> = Canvas::new(canvas::MapCanvas::new(
            &self.grid,
            &self.elements,
            &self.selected,
            map_mode.clone(),
        ))
        .width(Fill)
        .height(Fill)
        .into();

        canvas
    }
}
