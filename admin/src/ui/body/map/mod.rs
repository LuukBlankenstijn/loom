mod grid;

use iced::Element;
use iced::Length::Fill;
use iced::widget::container;

use crate::ui::body::map::grid::Grid;
pub use crate::ui::body::map::grid::Message;

#[derive(Debug, Default)]
pub struct MapApp {
    grid: Grid,
}

impl MapApp {
    pub fn update(&mut self, message: Message) {
        self.grid.update(message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        // We use a container to make sure the Canvas fills the whole window
        container(self.grid.view()).width(Fill).height(Fill).into()
    }
}
