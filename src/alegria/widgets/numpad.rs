// SPDX-License-Identifier: GPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Widget;
use iced::advanced::widget::tree::Tree;
use iced::mouse::{self, Cursor};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Border, event};
use iced::{Color, Element, Length, Rectangle, Size};

/// A custom Numpad widget.
///
/// The widget lays out 5 rows:
/// - Rows 0..3: the first 4 rows are arranged in 3 columns:
///     - Row 0: [7, 8, 9]
///     - Row 1: [4, 5, 6]
///     - Row 2: [1, 2, 3]
///     - Row 3: [comma, 0, back]
/// - Row 4: A full-width “delete” button.
///
/// Callback closures are invoked when a button is clicked.
pub struct Numpad<Message: 'static> {
    on_number_clicked: Box<dyn Fn(u8) -> Message>,
    on_comma_clicked: Box<dyn Fn() -> Message>,
    on_back_clicked: Box<dyn Fn() -> Message>,
    on_delete_clicked: Box<dyn Fn() -> Message>,
    button_size: f32,
    spacing: f32,
}

impl<Message> Numpad<Message> {
    /// Create a new Numpad with callbacks.
    pub fn new(
        on_number_clicked: impl Fn(u8) -> Message + 'static,
        on_comma_clicked: impl Fn() -> Message + 'static,
        on_back_clicked: impl Fn() -> Message + 'static,
        on_delete_clicked: impl Fn() -> Message + 'static,
    ) -> Self {
        Self {
            on_number_clicked: Box::new(on_number_clicked),
            on_comma_clicked: Box::new(on_comma_clicked),
            on_back_clicked: Box::new(on_back_clicked),
            on_delete_clicked: Box::new(on_delete_clicked),
            button_size: 50.0,
            spacing: 5.0,
        }
    }
}

impl<Message: 'static, Theme, Renderer> Widget<Message, Theme, Renderer> for Numpad<Message>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        // widget shrinks to fit its computed layout.
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        use iced::advanced::Text;
        use iced::alignment::{Horizontal, Vertical};
        use iced::{Font, Point};

        let bounds = layout.bounds();
        let text_size = 16.0;
        let font = Font::default();

        // Draw keys for rows 0 to 3
        for row in 0..4 {
            for col in 0..3 {
                let x = bounds.x + col as f32 * (self.button_size + self.spacing);
                let y = bounds.y + row as f32 * (self.button_size + self.spacing);
                let rect = Rectangle {
                    x,
                    y,
                    width: self.button_size,
                    height: self.button_size,
                };

                // Draw the button background
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: rect,
                        border: Border {
                            color: Color::BLACK,
                            width: 1.0,
                            radius: 3.0.into(),
                        },
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb(0.9, 0.9, 0.9),
                );

                // Determine the label
                let label = match (row, col) {
                    (0, 0) => "7",
                    (0, 1) => "8",
                    (0, 2) => "9",
                    (1, 0) => "4",
                    (1, 1) => "5",
                    (1, 2) => "6",
                    (2, 0) => "1",
                    (2, 1) => "2",
                    (2, 2) => "3",
                    (3, 0) => ",",
                    (3, 1) => "0",
                    (3, 2) => "←",
                    _ => "",
                };

                // Create text configuration
                let text: Text<String> = Text {
                    content: String::from(label),
                    bounds: rect.size(),
                    size: text_size.into(),
                    line_height: LineHeight::default(),
                    font,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    shaping: Shaping::Basic,
                    wrapping: Wrapping::Word,
                };

                // Calculate centered position
                let text_position =
                    Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

                // Draw the text
                renderer.fill_text(text, text_position, Color::BLACK, bounds);
            }
        }

        // Draw delete button
        let y = bounds.y + 4.0 * (self.button_size + self.spacing);
        let rect = Rectangle {
            x: bounds.x,
            y,
            width: bounds.width,
            height: self.button_size,
        };

        // Draw delete button background
        renderer.fill_quad(
            renderer::Quad {
                bounds: rect,
                border: Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..renderer::Quad::default()
            },
            Color::from_rgb(0.9, 0.9, 0.9),
        );

        // Delete button text
        let text = Text {
            content: String::from("Delete"),
            bounds: rect.size(),
            size: text_size.into(),
            line_height: LineHeight::default(),
            font,
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::Word,
        };

        let text_position = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

        renderer.fill_text(text, text_position, Color::BLACK, bounds);
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        // For rows 0-3 (first 4 rows), each row has a button height plus spacing
        let width = 3.0 * self.button_size + 2.0 * self.spacing;
        let height_first_part = 4.0 * self.button_size + 3.0 * self.spacing;
        let height_delete = self.button_size;
        let total_height = height_first_part + self.spacing + height_delete;
        layout::Node::new(Size::new(width, total_height))
    }

    fn on_event(
        &mut self,
        _state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
            event
        {
            let bounds = layout.bounds();
            if let Some(cursor_pos) = cursor.position() {
                if bounds.contains(cursor_pos) {
                    let local_x = cursor_pos.x - bounds.x;
                    let local_y = cursor_pos.y - bounds.y;
                    let height_first_part = 4.0 * self.button_size + 3.0 * self.spacing;

                    if local_y < height_first_part {
                        // Determine row and column in the first 4 rows.
                        let row = (local_y / (self.button_size + self.spacing)).floor() as usize;
                        let col = (local_x / (self.button_size + self.spacing)).floor() as usize;
                        let message = match row {
                            0 => (self.on_number_clicked)(7 + col as u8), // Row 0: 7, 8, 9
                            1 => (self.on_number_clicked)(4 + col as u8), // Row 1: 4, 5, 6
                            2 => (self.on_number_clicked)(1 + col as u8), // Row 2: 1, 2, 3
                            3 => match col {
                                0 => (self.on_comma_clicked)(),   // Comma
                                1 => (self.on_number_clicked)(0), // 0
                                2 => (self.on_back_clicked)(),    // Back
                                _ => return event::Status::Ignored,
                            },
                            _ => return event::Status::Ignored,
                        };
                        shell.publish(message);
                        return event::Status::Captured;
                    } else {
                        // Check if click is in the delete button row.
                        let delete_row_top = height_first_part + self.spacing;
                        if local_y >= delete_row_top && local_y <= delete_row_top + self.button_size
                        {
                            let message = (self.on_delete_clicked)();
                            shell.publish(message);
                            return event::Status::Captured;
                        }
                    }
                }
            }
        }
        event::Status::Ignored
    }

    // OLD CODE MAY BE USEFUL WHEN UPDATING TO ICED 0.14
    // fn update(
    //     &mut self,
    //     _state: &mut iced::advanced::widget::Tree,
    //     event: &iced::Event,
    //     layout: Layout<'_>,
    //     cursor: iced::advanced::mouse::Cursor,
    //     _renderer: &Renderer,
    //     _clipboard: &mut dyn iced::advanced::Clipboard,
    //     shell: &mut iced::advanced::Shell<'_, Message>,
    //     _viewport: &Rectangle,
    // ) {
    //     if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
    //         event
    //     {
    //         let bounds = layout.bounds();
    //         if bounds.contains(cursor.position().unwrap_or_default()) {
    //             let local_x = cursor.position().unwrap_or_default().x - bounds.x;
    //             let local_y = cursor.position().unwrap_or_default().y - bounds.y;
    //             let height_first_part = 4.0 * self.button_size + 3.0 * self.spacing;

    //             if local_y < height_first_part {
    //                 // Determine row and column in the first 4 rows.
    //                 let row = (local_y / (self.button_size + self.spacing)).floor() as usize;
    //                 let col = (local_x / (self.button_size + self.spacing)).floor() as usize;
    //                 let message = match row {
    //                     0 => (self.on_number_clicked)(7 + col as u8), // Row 0: 7, 8, 9
    //                     1 => (self.on_number_clicked)(4 + col as u8), // Row 1: 4, 5, 6
    //                     2 => (self.on_number_clicked)(1 + col as u8), // Row 2: 1, 2, 3
    //                     3 => match col {
    //                         0 => (self.on_comma_clicked)(),   // Comma
    //                         1 => (self.on_number_clicked)(0), // 0
    //                         2 => (self.on_back_clicked)(),    // Back
    //                         _ => return,
    //                     },
    //                     _ => return,
    //                 };
    //                 shell.publish(message);
    //             } else {
    //                 // Check if click is in the delete button row.
    //                 let delete_row_top = height_first_part + self.spacing;
    //                 if local_y >= delete_row_top && local_y <= delete_row_top + self.button_size {
    //                     let message = (self.on_delete_clicked)();
    //                     shell.publish(message);
    //                 }
    //             }
    //         }
    //     }
    // }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if layout
            .bounds()
            .contains(cursor_position.position().unwrap_or_default())
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<Message: 'static, Theme, Renderer> From<Numpad<Message>>
    for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn from(numpad: Numpad<Message>) -> Self {
        Element::new(numpad)
    }
}
