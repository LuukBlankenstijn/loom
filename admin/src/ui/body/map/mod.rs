mod canvas;
mod grid;
mod types;

use iced::Length::Fill;
use iced::widget::Canvas;
use iced::{Element, Task};

use crate::ui::body::map::grid::Grid;
use crate::ui::body::map::types::MapElement;
use crate::ui::body::map::types::door::Door;
use crate::ui::body::map::types::wall::Wall;

#[derive(Debug)]
pub struct MapApp {
    doors: Vec<Door>,
    walls: Vec<Wall>,
    grid: Grid,
    grid_elements: Vec<MapElement>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddElement(MapElement),
    Canvas(grid::Message),
}

impl Default for MapApp {
    fn default() -> Self {
        let doors = Door::get_test();
        let walls = Wall::get_test();
        let door_enums = Door::get_test().into_iter().map(MapElement::Door);
        let wall_enums = Wall::get_test().into_iter().map(MapElement::Wall);
        Self {
            doors,
            walls,
            grid: Grid::new(),
            grid_elements: door_enums.chain(wall_enums).collect(),
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
                _ => self.grid.update_canvas(msg),
            },
            Message::AddElement(element) => {
                match element {
                    MapElement::Door(door) => self.doors.push(door),
                    MapElement::Wall(wall) => self.walls.push(wall),
                };
                self.grid_elements = self.get_all_elements()
            }
        };
        Task::none()
    }

    fn get_all_elements(&self) -> Vec<MapElement> {
        let mut elements = Vec::with_capacity(self.walls.len() + self.doors.len());

        elements.extend(self.walls.iter().cloned().map(MapElement::Wall));
        elements.extend(self.doors.iter().cloned().map(MapElement::Door));

        elements
    }

    pub fn view(&self) -> Element<'_, Message> {
        let canvas: Element<'_, grid::Message> =
            Canvas::new(canvas::MapCanvas::new(&self.grid, &self.grid_elements))
                .width(Fill)
                .height(Fill)
                .into();

        canvas.map(Message::Canvas)
    }
}
