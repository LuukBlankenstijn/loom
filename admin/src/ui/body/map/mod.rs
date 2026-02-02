mod grid;
mod types;

use iced::Element;
use iced::Length::Fill;
use iced::widget::container;

use crate::ui::body::map::grid::Grid;
pub use crate::ui::body::map::grid::Message;
use crate::ui::body::map::types::MapElement;
use crate::ui::body::map::types::door::Door;
use crate::ui::body::map::types::wall::Wall;

#[derive(Debug)]
pub struct MapApp {
    doors: Vec<Door>,
    walls: Vec<Wall>,
    grid: Grid,
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
            grid: Grid::new(door_enums.chain(wall_enums).collect()),
        }
    }
}

impl MapApp {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::DrawFinish(start, end) => {
                let wall = Wall::new(start, end);
                self.walls.push(wall.clone());
                self.grid.add_element(MapElement::Wall(wall));
            }
            _ => self.grid.update(message),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(self.grid.view()).width(Fill).height(Fill).into()
    }
}
