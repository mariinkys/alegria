use crate::alegria::screen::{
    Bar,
    bar::{Message, State, SubScreen},
};
use iced::{
    Element, Length,
    time::Instant,
    widget::{container, text},
};

impl Bar {
    pub fn view(&self, _now: Instant) -> Element<Message> {
        let content = match &self.state {
            State::Loading => text("Loading..."),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::Bar {
                    temporal_tickets,
                    product_categories,
                    product_category_products,
                    pagination,
                } => text("Data loaded correctly"),
                SubScreen::Pay => todo!(),
            },
        };

        container(content).center(Length::Fill).into()
    }
}
