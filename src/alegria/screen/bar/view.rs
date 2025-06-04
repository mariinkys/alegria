use crate::{
    alegria::{
        core::models::temporal_ticket::TemporalTicket,
        screen::{
            Bar,
            bar::{CurrentPosition, Message, State, SubScreen},
        },
    },
    fl,
};
use iced::{
    Alignment, Length, Pixels, Renderer, Theme,
    advanced::graphics::core::Element,
    time::Instant,
    widget::{Space, button, container, row, text},
};

const GLOBAL_SPACING: f32 = 6.;
const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
const TITLE_TEXT_SIZE: f32 = 25.0;

impl Bar {
    pub fn view<'a>(&self, _now: Instant) -> iced::Element<'a, Message> {
        let content: Element<'a, Message, Theme, Renderer> = match &self.state {
            State::Loading => text("Loading...").into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::Bar {
                    temporal_tickets,
                    product_categories,
                    product_category_products,
                    pagination,
                    current_position,
                } => bar_header(temporal_tickets, current_position),
                SubScreen::Pay => todo!(),
            },
        };

        container(content).center(Length::Fill).into()
    }
}

/// Returns the view of the header row of the bar screen
fn bar_header<'a>(
    temporal_tickets: &[TemporalTicket],
    current_position: &CurrentPosition,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);
    let button_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(button_height);

    let mut header_row = row![
        back_button,
        text(fl!("bar"))
            .size(TITLE_TEXT_SIZE)
            .align_y(Alignment::Center),
        Space::new(Length::Fill, Length::Shrink)
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .spacing(spacing);

    let current_ticket = temporal_tickets.iter().find(|x| {
        x.ticket_location
            == super::match_table_location_with_number(&current_position.table_location)
            && x.table_id == current_position.table_index
    });

    if let Some(c_ticket) = current_ticket {
        if !c_ticket.products.is_empty() && c_ticket.simple_invoice_id.is_some() {
            header_row = header_row.push(
                button(
                    text(fl!("unlock"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                //.on_press(Message::UnlockTicket(c_ticket.clone()))
                .style(button::danger)
                .height(button_height),
            );
        }
        if !c_ticket.products.is_empty() {
            header_row = header_row.push(
                button(
                    text(fl!("print"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                // .on_press(Message::PrintModalAction(
                //     PrintTicketModalActions::ShowModal,
                // ))
                .height(button_height),
            );

            header_row = header_row.push(
                button(
                    text(fl!("pay"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .style(button::success)
                //.on_press(Message::OpenPayScreen)
                .height(button_height),
            );
        }
    }

    header_row.into()
}
