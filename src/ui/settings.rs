use iced::alignment::Vertical;
use iced::widget::checkbox::Checkbox;
use iced::widget::pick_list::PickList;
use iced::widget::{self, Button, Column, Container, Row, Space, Text, button, text_input};
use iced::{Color, Element, Length, Shadow, Theme, border};
use std::fmt;

use crate::Message;
use crate::i18n::tr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Behavior,
    Appearance,
}

impl SettingsTab {
    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Behavior => "Behavior",
            SettingsTab::Appearance => "Appearance",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    System,
    Monospace,
    Serif,
}

impl FontChoice {
    pub fn label(self) -> &'static str {
        match self {
            FontChoice::System => "System Default",
            FontChoice::Monospace => "Monospace",
            FontChoice::Serif => "Serif",
        }
    }
}

impl fmt::Display for FontChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl Default for FontChoice {
    fn default() -> Self {
        FontChoice::System
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::System => "System",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
        }
    }
}

impl fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl Default for ThemeChoice {
    fn default() -> Self {
        ThemeChoice::System
    }
}

#[derive(Debug, Clone)]
pub struct SettingsWindowState {
    pub active_tab: SettingsTab,
    pub expand_first_result: bool,
    pub query_timeout_secs: String,
    pub sort_fields_alphabetically: bool,
    pub sort_index_names_alphabetically: bool,
    pub primary_font: FontChoice,
    pub primary_font_size: String,
    pub result_font: FontChoice,
    pub result_font_size: String,
    pub theme_choice: ThemeChoice,
}

impl Default for SettingsWindowState {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::Behavior,
            expand_first_result: true,
            query_timeout_secs: "600".to_string(),
            sort_fields_alphabetically: false,
            sort_index_names_alphabetically: false,
            primary_font: FontChoice::System,
            primary_font_size: "16".to_string(),
            result_font: FontChoice::Monospace,
            result_font_size: "15".to_string(),
            theme_choice: ThemeChoice::System,
        }
    }
}

pub fn settings_view(state: &SettingsWindowState) -> Element<Message> {
    let tab_row = tab_buttons(state.active_tab);

    let tab_content: Element<_> = match state.active_tab {
        SettingsTab::Behavior => behavior_tab(state),
        SettingsTab::Appearance => appearance_tab(state),
    };

    let content = Column::new()
        .spacing(20)
        .push(Text::new(tr("Settings")).size(24).color(Color::from_rgb8(0x17, 0x1a, 0x20)))
        .push(tab_row)
        .push(tab_content)
        .push(bottom_actions());

    let card = Container::new(content).padding(24).width(Length::Fixed(640.0)).style(pane_style);

    Container::new(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| widget::container::Style {
            background: Some(Color::from_rgba8(0x16, 0x1a, 0x1f, 0.55).into()),
            ..Default::default()
        })
        .into()
}

fn behavior_tab(state: &SettingsWindowState) -> Element<Message> {
    let expand_checkbox = Checkbox::new(tr("Expand first result item"), state.expand_first_result)
        .on_toggle(Message::SettingsToggleExpandFirstResult);

    let timeout_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(Text::new(tr("Query timeout (seconds)")).size(14))
        .push(
            text_input(tr("Seconds"), &state.query_timeout_secs)
                .on_input(Message::SettingsQueryTimeoutChanged)
                .padding([6, 10])
                .width(Length::Fixed(120.0)),
        );

    let sort_fields =
        Checkbox::new(tr("Sort fields alphabetically"), state.sort_fields_alphabetically)
            .on_toggle(Message::SettingsToggleSortFields);

    let sort_indexes =
        Checkbox::new(tr("Sort index names alphabetically"), state.sort_index_names_alphabetically)
            .on_toggle(Message::SettingsToggleSortIndexes);

    Column::new()
        .spacing(16)
        .push(expand_checkbox)
        .push(timeout_row)
        .push(sort_fields)
        .push(sort_indexes)
        .into()
}

fn appearance_row<'a>(
    label: &'static str,
    selected_font: FontChoice,
    font_size: &str,
    font_message: fn(FontChoice) -> Message,
    size_message: fn(String) -> Message,
) -> Element<'a, Message> {
    Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(Text::new(tr(label)).size(14).width(Length::FillPortion(3)))
        .push(
            PickList::new(ALL_FONTS, Some(selected_font), font_message)
                .width(Length::FillPortion(4)),
        )
        .push(
            text_input(tr("Font Size"), font_size)
                .on_input(size_message)
                .padding([6, 10])
                .width(Length::Fixed(120.0)),
        )
        .into()
}

fn appearance_tab(state: &SettingsWindowState) -> Element<Message> {
    let primary_row = appearance_row(
        "Primary Font",
        state.primary_font,
        &state.primary_font_size,
        Message::SettingsPrimaryFontChanged,
        Message::SettingsPrimaryFontSizeChanged,
    );

    let result_row = appearance_row(
        "Query Result Font",
        state.result_font,
        &state.result_font_size,
        Message::SettingsResultFontChanged,
        Message::SettingsResultFontSizeChanged,
    );

    let theme_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(Text::new(tr("Theme")).size(14).width(Length::FillPortion(3)))
        .push(
            PickList::new(ALL_THEMES, Some(state.theme_choice), Message::SettingsThemeChanged)
                .width(Length::FillPortion(4)),
        )
        .push(Space::with_width(Length::FillPortion(3)));

    Column::new().spacing(16).push(primary_row).push(result_row).push(theme_row).into()
}

fn bottom_actions() -> Element<'static, Message> {
    let apply = Button::new(Text::new(tr("Apply")).size(14))
        .padding([6, 16])
        .on_press(Message::SettingsApply);
    let cancel = Button::new(Text::new(tr("Cancel")).size(14))
        .padding([6, 16])
        .on_press(Message::SettingsCancel);
    let save = Button::new(Text::new(tr("Save")).size(14))
        .padding([6, 16])
        .on_press(Message::SettingsSave);

    Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(Space::with_width(Length::Fill))
        .push(apply)
        .push(cancel)
        .push(save)
        .into()
}

fn tab_buttons(active: SettingsTab) -> Row<'static, Message> {
    let mut behavior = Button::new(Text::new(tr(SettingsTab::Behavior.label())).size(14))
        .padding([6, 16])
        .style(move |_, _| tab_button_style(active == SettingsTab::Behavior));
    if active != SettingsTab::Behavior {
        behavior = behavior.on_press(Message::SettingsTabChanged(SettingsTab::Behavior));
    }

    let mut appearance = Button::new(Text::new(tr(SettingsTab::Appearance.label())).size(14))
        .padding([6, 16])
        .style(move |_, _| tab_button_style(active == SettingsTab::Appearance));
    if active != SettingsTab::Appearance {
        appearance = appearance.on_press(Message::SettingsTabChanged(SettingsTab::Appearance));
    }

    Row::new().spacing(8).push(behavior).push(appearance)
}

fn tab_button_style(active: bool) -> button::Style {
    let bg_active = Color::from_rgb8(0xd6, 0xe8, 0xff);
    let bg_inactive = Color::from_rgb8(0xf6, 0xf7, 0xfa);
    let border_color = Color::from_rgb8(0xc2, 0xc8, 0xd3);

    button::Style {
        background: Some((if active { bg_active } else { bg_inactive }).into()),
        text_color: Color::BLACK,
        border: border::rounded(6).width(1).color(border_color),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

fn pane_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();

    widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: border::rounded(6).width(1).color(palette.primary.weak.color),
        ..Default::default()
    }
}

const ALL_FONTS: &[FontChoice] = &[FontChoice::System, FontChoice::Monospace, FontChoice::Serif];
const ALL_THEMES: &[ThemeChoice] = &[ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];
