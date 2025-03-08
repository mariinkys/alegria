// SPDX-License-Identifier: GPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Widget;
use iced::advanced::widget::tree::Tree;
use iced::mouse::{self, Cursor};
use iced::overlay::menu;
use iced::theme::palette::{Background, Pair};
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Border, Theme, event};
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
pub struct Numpad<'a, Message: 'a> {
    on_number_clicked: Option<Box<dyn Fn(u8) -> Message + 'a>>,
    on_comma_clicked: Option<Box<dyn Fn() -> Message + 'a>>,
    on_back_clicked: Option<Box<dyn Fn() -> Message + 'a>>,
    on_delete_clicked: Option<Box<dyn Fn() -> Message + 'a>>,
    class: <Theme as Catalog>::Class<'a>,
    button_size: f32,
    spacing: f32,
}

impl<'a, Message> Numpad<'a, Message> {
    /// Create a new Numpad with callbacks.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            on_number_clicked: None,
            on_comma_clicked: None,
            on_back_clicked: None,
            on_delete_clicked: None,
            class: <Theme as Catalog>::default(),
            button_size: 75.0,
            spacing: 5.0,
        }
    }

    /// Sets the message that should be produced when a [`NumPad`] number
    /// is clicked.
    pub fn on_number_clicked(mut self, on_number_clicked: impl Fn(u8) -> Message + 'a) -> Self {
        self.on_number_clicked = Some(Box::new(on_number_clicked));
        self
    }

    /// Sets the message that should be produced when the [`NumPad`] ','
    /// is clicked.
    pub fn on_comma_clicked(mut self, message: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_comma_clicked = Some(Box::new(move || message.clone()));
        self
    }

    /// Sets the message that should be produced when the [`NumPad`] '<-'
    /// is clicked.
    pub fn on_back_clicked(mut self, message: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_back_clicked = Some(Box::new(move || message.clone()));
        self
    }

    /// Sets the message that should be produced when the [`NumPad`] 'Delete'
    /// is clicked.
    pub fn on_delete_clicked(mut self, message: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_delete_clicked = Some(Box::new(move || message.clone()));
        self
    }

    /// Sets the style of the [`NumPad`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = Box::new(style) as StyleFn<'a, Theme>;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for Numpad<'a, Message>
where
    Message: Clone + 'a,
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
        theme: &Theme,
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
        let text_size = 20.0;
        let font = Font::default();
        let style = Catalog::style(theme, &self.class);

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
                            color: style.border.color,
                            width: 1.0,
                            radius: 3.0.into(),
                        },
                        ..renderer::Quad::default()
                    },
                    style.background.base.color,
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
                renderer.fill_text(text, text_position, style.text_color, bounds);
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
                    color: style.border.color,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..renderer::Quad::default()
            },
            style.background.base.color,
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

        renderer.fill_text(text, text_position, style.text_color, bounds);
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
        _tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            | iced::Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_over(layout.bounds()) {
                    let bounds = layout.bounds();
                    let local_x = cursor_position.x - bounds.x;
                    let local_y = cursor_position.y - bounds.y;
                    let height_first_part = 4.0 * self.button_size + 3.0 * self.spacing;

                    if local_y < height_first_part {
                        // Determine row and column for the first four rows.
                        let row = (local_y / (self.button_size + self.spacing)).floor() as usize;
                        let col = (local_x / (self.button_size + self.spacing)).floor() as usize;
                        let maybe_message = match row {
                            0 => self.on_number_clicked.as_ref().map(|f| f(7 + col as u8)),
                            1 => self.on_number_clicked.as_ref().map(|f| f(4 + col as u8)),
                            2 => self.on_number_clicked.as_ref().map(|f| f(1 + col as u8)),
                            3 => match col {
                                0 => self.on_comma_clicked.as_ref().map(|f| f()),
                                1 => self.on_number_clicked.as_ref().map(|f| f(0)),
                                2 => self.on_back_clicked.as_ref().map(|f| f()),
                                _ => None,
                            },
                            _ => None,
                        };

                        if let Some(message) = maybe_message {
                            shell.publish(message);
                            return event::Status::Captured;
                        }
                    } else {
                        // Check if the click is within the delete button row.
                        let delete_row_top = height_first_part + self.spacing;
                        if local_y >= delete_row_top && local_y <= delete_row_top + self.button_size
                        {
                            if let Some(ref on_delete_clicked) = self.on_delete_clicked {
                                shell.publish(on_delete_clicked());
                                return event::Status::Captured;
                            }
                        }
                    }
                }
                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

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

impl<'a, Message, Renderer> From<Numpad<'a, Message>>
    for Element<'a, Message, iced::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(numpad: Numpad<'a, Message>) -> Self {
        Element::new(numpad)
    }
}

/// The appearance of a numberpad.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The text [`Color`] of the numberpad.
    pub text_color: Color,
    /// The [`Background`] of the numberpad.
    pub background: Background,
    /// The [`Border`] of the numberpad.
    pub border: Border,
}

/// The theme catalog of a [`NumPad`].
pub trait Catalog: menu::Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

/// A styling function for a [`NumPad`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

/// The default style of the field of a [`NumPad`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        text_color: palette.background.base.text,
        background: Background {
            base: Pair::new(palette.background.base.color, palette.background.base.color),
            weak: Pair::new(palette.background.weak.color, palette.background.weak.color),
            strong: Pair::new(
                palette.background.strong.color,
                palette.background.strong.color,
            ),
        },
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
    }
}
