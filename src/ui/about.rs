use iced::font::Weight;
use iced::widget::image::Handle;
use iced::widget::text::Wrapping;
use iced::widget::{Button, Column, Image, Row, Scrollable, Space, button};
use iced::{Color, Element, Font, Length, Shadow, border};

use crate::Message;
use crate::fonts;
use crate::i18n::tr;
use crate::settings::ThemePalette;
use crate::ui::modal::modal_layout;

const ABOUT_HOMEPAGE: &str = "https://github.com/EvgeniyMakhmudov/oxide_mongo";
const ABOUT_AUTHOR: &str = "Evgeniy Makhmudov";
const ABOUT_SINCE: &str = "2025";

#[derive(Clone, Copy)]
struct LicenseEntry {
    name: &'static str,
    license: &'static str,
    url: &'static str,
}

const PRIMARY_LICENSES: &[LicenseEntry] = &[LicenseEntry {
    name: "oxide_mongo",
    license: "MIT",
    url: "https://github.com/EvgeniyMakhmudov/oxide_mongo/blob/master/LICENSE",
}];

const COLOR_SCHEME_LICENSES: &[LicenseEntry] = &[
    LicenseEntry {
        name: "Solarized",
        license: "MIT",
        url: "https://ethanschoonover.com/solarized/",
    },
    LicenseEntry { name: "Nord", license: "MIT", url: "https://www.nordtheme.com" },
    LicenseEntry { name: "Gruvbox", license: "MIT/X11", url: "https://github.com/morhetz/gruvbox" },
    LicenseEntry {
        name: "OneDark",
        license: "MIT",
        url: "https://github.com/atom/atom/tree/master/packages/one-dark-ui",
    },
    LicenseEntry {
        name: "ObeLight",
        license: "MIT",
        url: "https://github.com/atom/atom/tree/master/packages/one-light-ui",
    },
];

const FONT_LICENSES: &[LicenseEntry] = &[LicenseEntry {
    name: "DejaVu Sans Mono",
    license: "Bitstream Vera License",
    url: "https://github.com/dejavu-fonts/dejavu-fonts",
}];

pub fn about_modal_view(palette: ThemePalette, icon_handle: Handle) -> Element<'static, Message> {
    let text_primary = palette.text_primary.to_color();
    let muted = palette.text_muted.to_color();
    let fonts_state = fonts::active_fonts();
    let bold_font = Font { weight: Weight::Bold, ..fonts_state.primary_font };

    let title = fonts::primary_text(tr("About"), Some(6.0)).color(text_primary);
    let title_name =
        fonts::primary_text("oxide_mongo", Some(6.0)).color(text_primary).font(bold_font);
    let title_row = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .push(title)
        .push(Space::with_width(Length::Fixed(6.0)))
        .push(title_name);
    let icon_size = (fonts::active_fonts().primary_size * 3.0).max(48.0);
    let icon =
        Image::new(icon_handle).width(Length::Fixed(icon_size)).height(Length::Fixed(icon_size));
    let header = Row::new()
        .align_y(iced::alignment::Vertical::Center)
        .push(title_row)
        .push(Space::with_width(Length::Fill))
        .push(icon);
    let summary = fonts::primary_text(
        tr("MongoDB GUI client for browsing collections, running queries, and managing data."),
        Some(-1.0),
    )
    .color(text_primary)
    .wrapping(Wrapping::Word)
    .width(Length::Fill);

    let label = |text: &str| fonts::primary_text(text.to_string(), None).color(muted);
    let value = |text: &str| fonts::primary_text(text.to_string(), None).color(text_primary);
    let homepage_link = link_button(&palette, value(ABOUT_HOMEPAGE), ABOUT_HOMEPAGE);

    let homepage_row = Row::new().spacing(8).push(label(tr("Homepage"))).push(homepage_link);
    let since_row =
        Row::new().spacing(8).push(label(tr("Project started"))).push(value(ABOUT_SINCE));
    let author_row = Row::new().spacing(8).push(label(tr("Author"))).push(value(ABOUT_AUTHOR));

    let close_button = Button::new(fonts::primary_text(tr("Close"), None))
        .padding([6, 16])
        .on_press(Message::AboutModalClose)
        .style({
            let palette = palette.clone();
            move |_, status| palette.subtle_button_style(6.0, status)
        });

    let content: Element<Message> = Column::new()
        .spacing(12)
        .push(header)
        .push(summary)
        .push(homepage_row)
        .push(since_row)
        .push(author_row)
        .push(close_button)
        .into();

    modal_layout(palette, content, Length::Fixed(520.0), 24, 12.0)
}

pub fn licenses_modal_view(palette: ThemePalette) -> Element<'static, Message> {
    let text_primary = palette.text_primary.to_color();
    let muted = palette.text_muted.to_color();
    let fonts_state = fonts::active_fonts();
    let bold_font = Font { weight: Weight::Bold, ..fonts_state.primary_font };
    let unknown_label = tr("Unknown");

    let title = fonts::primary_text(tr("Licenses"), Some(6.0)).color(text_primary);
    let label = |text: &str| fonts::primary_text(text.to_string(), Some(-1.0)).color(muted);
    let value = |text: &str| fonts::primary_text(text.to_string(), Some(-1.0)).color(text_primary);

    let entry_view = |entry: &LicenseEntry| {
        let license_label = if entry.license == "Unknown" { unknown_label } else { entry.license };

        Column::new()
            .spacing(4)
            .push(fonts::primary_text(entry.name, Some(2.0)).font(bold_font).color(text_primary))
            .push(Row::new().spacing(8).push(label(tr("License"))).push(value(license_label)))
            .push(Row::new().spacing(8).push(label(tr("Link"))).push(link_button(
                &palette,
                value(entry.url),
                entry.url,
            )))
    };

    let section_view = |title_key: &'static str, entries: &[LicenseEntry]| {
        let mut column = Column::new().spacing(8).push(
            fonts::primary_text(tr(title_key), Some(4.0)).font(bold_font).color(text_primary),
        );

        for entry in entries {
            column = column.push(entry_view(entry));
        }

        column
    };

    let scroll_content = Column::new()
        .spacing(16)
        .push(section_view("Primary licenses", PRIMARY_LICENSES))
        .push(section_view("Color schemes", COLOR_SCHEME_LICENSES))
        .push(section_view("Fonts", FONT_LICENSES));

    let scrollable =
        Scrollable::new(scroll_content).width(Length::Fill).height(Length::Fixed(360.0));

    let close_button = Button::new(fonts::primary_text(tr("Close"), None))
        .padding([6, 16])
        .on_press(Message::LicensesModalClose)
        .style({
            let palette = palette.clone();
            move |_, status| palette.subtle_button_style(6.0, status)
        });

    let content: Element<Message> =
        Column::new().spacing(12).push(title).push(scrollable).push(close_button).into();

    modal_layout(palette, content, Length::Fixed(560.0), 24, 12.0)
}

fn link_button<'a>(
    palette: &ThemePalette,
    label: iced::widget::Text<'a>,
    url: &'static str,
) -> Button<'a, Message> {
    let palette = palette.clone();
    Button::new(label).padding(0).on_press(Message::OpenUrl(url)).style(move |_, status| {
        let text_color = match status {
            button::Status::Active => palette.primary_buttons.active.to_color(),
            button::Status::Hovered => palette.primary_buttons.hover.to_color(),
            button::Status::Pressed => palette.primary_buttons.pressed.to_color(),
            button::Status::Disabled => palette.text_muted.to_color(),
        };

        button::Style {
            background: None,
            text_color,
            border: border::rounded(0.0).width(0).color(Color::from_rgba8(0, 0, 0, 0.0)),
            shadow: Shadow::default(),
            ..Default::default()
        }
    })
}
