mod canvas;
mod grid;
mod messsage;
pub mod types;

use grid::Grid;
pub use messsage::Message;
use ordermap::OrderMap;

use std::collections::HashSet;

use iced::Length::Fill;
use iced::widget::Canvas;
use iced::{Element, Task, Vector};
use uuid::Uuid;

use crate::messsage::GridMessage;
use crate::types::Drawable;

#[derive(Default, Debug, Clone)]
pub enum MapMode {
    #[default]
    View,
    Edit,
}

#[derive(Debug)]
pub struct Map<T> {
    grid: Grid,
    start_elements: OrderMap<Uuid, T>,
    elements: OrderMap<Uuid, T>,
    selected: HashSet<Uuid>,
}

impl<T: Drawable> Map<T> {
    pub fn new(elements: Vec<T>) -> Self {
        let elements: OrderMap<Uuid, T> = elements.into_iter().map(|e| (e.get_id(), e)).collect();
        Self {
            start_elements: elements.clone(),
            elements,
            grid: Grid::new(),
            selected: Default::default(),
        }
    }

    pub fn get_changes(&self) -> (Vec<Uuid>, Vec<T>) {
        let deleted: Vec<Uuid> = self
            .start_elements
            .keys()
            .filter(|key| !self.elements.contains_key(*key))
            .cloned()
            .collect();

        let new_or_changed: Vec<T> = self
            .elements
            .iter()
            .filter(|(key, current_val)| match self.start_elements.get(*key) {
                Some(start_val) => *start_val != **current_val,
                None => true,
            })
            .map(|(_, value)| value.clone())
            .collect();

        (deleted, new_or_changed)
    }

    pub fn update_elements(&mut self, elements: Vec<T>) {
        self.elements = elements.into_iter().map(|e| (e.get_id(), e)).collect();
        self.start_elements = self.elements.clone();
    }

    pub fn update(&mut self, message: Message<T>) -> Task<Message<T>> {
        match message {
            Message::Grid(msg) => match msg {
                GridMessage::DrawFinish(start, end) => {
                    return Task::done(Message::DrawFinish(start, end));
                }
                GridMessage::RequestSelect(point) => {
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
            Message::Insert(element) => {
                self.elements.insert(element.get_id(), element);
            }
            Message::ClearSelection => self.selected.clear(),
            Message::DeleteSelection => {
                for id in &self.selected {
                    self.elements.remove(id);
                }
                self.selected.clear();
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
            Message::RotateSelection => {
                for id in &self.selected {
                    if let Some(element) = self.elements.get_mut(id) {
                        element.rotate(None);
                    }
                }
            }
            Message::AddElement(element_generator) => {
                let pos = -self.grid.offset + Vector::new(200.0, 200.0);
                let element = element_generator(self.grid.snap_to_grid((pos.x, pos.y).into()));
                self.elements.insert(element.get_id(), element);
            }
            Message::DrawFinish(_, _) => {} // consumer may handle this to insert an element
        };
        Task::none()
    }

    pub fn view(&self, map_mode: MapMode) -> Element<'_, Message<T>> {
        let canvas: Element<'_, Message<T>> = Canvas::new(canvas::MapCanvas::new(
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
