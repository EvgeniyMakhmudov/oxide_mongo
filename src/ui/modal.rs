use iced::widget::{Container, container};
use iced::{Color, Element, Length, Shadow, Vector, border};

use crate::Message;
use crate::settings::ThemePalette;

pub fn color_luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

pub fn modal_overlay_color(palette: &ThemePalette) -> Color {
    let base = palette.widget_background_color();
    if color_luminance(base) > 0.5 {
        Color::from_rgba(0.0, 0.0, 0.0, 0.55)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.35)
    }
}

pub fn modal_shadow_color(palette: &ThemePalette) -> Color {
    let base = palette.widget_background_color();
    if color_luminance(base) > 0.5 {
        Color::from_rgba(0.0, 0.0, 0.0, 0.25)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.5)
    }
}

pub fn error_accent_color(palette: &ThemePalette) -> Color {
    let base = palette.widget_background_color();
    if color_luminance(base) > 0.5 {
        Color::from_rgba(0.85, 0.32, 0.33, 1.0)
    } else {
        Color::from_rgba(1.0, 0.54, 0.55, 1.0)
    }
}

pub fn success_accent_color(palette: &ThemePalette) -> Color {
    palette.primary_buttons.active.to_color()
}

pub fn modal_layout<'a>(
    palette: ThemePalette,
    content: Element<'a, Message>,
    width: Length,
    padding: u16,
    radius: f32,
) -> Element<'a, Message> {
    let card_bg = palette.widget_background_color();
    let border_color = palette.widget_border_color();
    let shadow_color = modal_shadow_color(&palette);
    let overlay_color = modal_overlay_color(&palette);
    let text_color = palette.text_primary.to_color();

    let card =
        Container::new(content).padding(padding).width(width).style(move |_| container::Style {
            background: Some(card_bg.into()),
            border: border::rounded(radius).width(1).color(border_color),
            shadow: Shadow {
                color: shadow_color,
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            text_color: Some(text_color),
            ..Default::default()
        });

    Container::new(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_| container::Style {
            background: Some(overlay_color.into()),
            ..Default::default()
        })
        .into()
}
