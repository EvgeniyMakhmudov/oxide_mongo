use iced::widget::{Button, Column, Container, Row, Scrollable, Space, Text};
use iced::{Color, Element, Font, Length, alignment::Vertical};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontOption {
    pub id: String,
    pub name: String,
    pub font: Font,
}

impl FontOption {
    pub fn new(id: String, name: String, font: Font) -> Self {
        Self { id, name, font }
    }
}

pub struct FontDropdown<'a, Message> {
    label: &'static str,
    options: &'a [FontOption],
    selected_id: Option<&'a str>,
    is_open: bool,
    on_toggle: Message,
    on_select: Box<dyn Fn(String) -> Message + 'a>,
    width: Length,
    max_height: f32,
}

impl<'a, Message: Clone + 'a> FontDropdown<'a, Message> {
    pub fn new(
        label: &'static str,
        options: &'a [FontOption],
        selected_id: Option<&'a str>,
        is_open: bool,
        on_toggle: Message,
        on_select: impl Fn(String) -> Message + 'a,
    ) -> Self {
        Self {
            label,
            options,
            selected_id,
            is_open,
            on_toggle,
            on_select: Box::new(on_select),
            width: Length::Fill,
            max_height: 220.0,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    fn view(self) -> Element<'a, Message> {
        let selected_option =
            self.selected_id.and_then(|id| self.options.iter().find(|opt| opt.id == id));

        let display_text =
            selected_option.map(|opt| opt.name.clone()).unwrap_or_else(|| String::from(self.label));
        let display_font = selected_option.map(|opt| opt.font).unwrap_or(Font::DEFAULT);

        let header_row = Row::new()
            .align_y(Vertical::Center)
            .spacing(8)
            .push(Text::new(display_text).font(display_font))
            .push(Space::new().width(Length::Fill))
            .push(Text::new(if self.is_open { "▲" } else { "▼" }));

        let mut column = Column::new().spacing(6);

        column = column.push(
            Button::new(header_row)
                .width(self.width)
                .padding([6, 12])
                .on_press(self.on_toggle.clone()),
        );

        if self.is_open {
            let mut list = Column::new().spacing(4);

            for option in self.options {
                let message = (self.on_select)(option.id.clone());
                let mut label = Text::new(option.name.clone()).font(option.font);

                if self.selected_id.map(|id| id == option.id.as_str()).unwrap_or(false) {
                    label = label.color(Color::from_rgb8(0x17, 0x1a, 0x20));
                }

                list = list.push(
                    Button::new(label).width(Length::Fill).padding([6, 10]).on_press(message),
                );
            }

            let list = Scrollable::new(list).height(Length::Fixed(self.max_height));

            column = column.push(Container::new(list).width(self.width).padding(4));
        }

        column.width(self.width).into()
    }
}

impl<'a, Message: Clone + 'a> From<FontDropdown<'a, Message>> for Element<'a, Message> {
    fn from(dropdown: FontDropdown<'a, Message>) -> Self {
        dropdown.view()
    }
}
