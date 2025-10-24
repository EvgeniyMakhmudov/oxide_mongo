use iced::alignment::Vertical;
use iced::widget::checkbox::Checkbox;
use iced::widget::pick_list::PickList;
use iced::widget::{self, Button, Column, Container, Row, Space, button, text_input};
use iced::{Color, Element, Length, Shadow, Theme, border};

use crate::Message;
use crate::fonts;
use crate::i18n::{ALL_LANGUAGES, Language, tr, tr_format};
use crate::settings::{ALL_THEMES, AppSettings, ThemeChoice};
use crate::ui::fonts_dropdown::{self, FontDropdown};

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

#[derive(Debug, Clone)]
pub struct SettingsWindowState {
    pub active_tab: SettingsTab,
    pub expand_first_result: bool,
    pub query_timeout_secs: String,
    pub sort_fields_alphabetically: bool,
    pub sort_index_names_alphabetically: bool,
    pub language: Language,
    pub font_options: Vec<fonts_dropdown::FontOption>,
    pub primary_font_open: bool,
    pub primary_font_id: String,
    pub primary_font_size: String,
    pub result_font_open: bool,
    pub result_font_id: String,
    pub result_font_size: String,
    pub theme_choice: ThemeChoice,
    pub validation_error: Option<String>,
}

impl Default for SettingsWindowState {
    fn default() -> Self {
        Self::from_app_settings(&AppSettings::default())
    }
}

impl SettingsWindowState {
    pub fn from_app_settings(settings: &AppSettings) -> Self {
        let font_options: Vec<fonts_dropdown::FontOption> = fonts::available_fonts()
            .iter()
            .map(|opt| fonts_dropdown::FontOption::new(opt.id.clone(), opt.name.clone(), opt.font))
            .collect();

        let primary_font_id = ensure_font_id(&font_options, &settings.primary_font);
        let result_font_id = ensure_font_id(&font_options, &settings.result_font);
        Self {
            active_tab: SettingsTab::Behavior,
            expand_first_result: settings.expand_first_result,
            query_timeout_secs: settings.query_timeout_secs.to_string(),
            sort_fields_alphabetically: settings.sort_fields_alphabetically,
            sort_index_names_alphabetically: settings.sort_index_names_alphabetically,
            language: settings.language,
            font_options,
            primary_font_open: false,
            primary_font_id,
            primary_font_size: settings.primary_font_size.to_string(),
            result_font_open: false,
            result_font_id,
            result_font_size: settings.result_font_size.to_string(),
            theme_choice: settings.theme_choice,
            validation_error: None,
        }
    }

    pub fn to_app_settings(&self) -> Result<AppSettings, String> {
        let timeout =
            parse_integer::<u64>(&self.query_timeout_secs, tr("Query timeout (seconds)"))?;
        let primary_size = parse_integer::<u16>(&self.primary_font_size, tr("Primary Font"))?;
        let result_size = parse_integer::<u16>(&self.result_font_size, tr("Query Result Font"))?;

        if primary_size == 0 || result_size == 0 {
            return Err(tr("Font size must be greater than zero").to_owned());
        }

        Ok(AppSettings {
            expand_first_result: self.expand_first_result,
            query_timeout_secs: timeout,
            sort_fields_alphabetically: self.sort_fields_alphabetically,
            sort_index_names_alphabetically: self.sort_index_names_alphabetically,
            language: self.language,
            primary_font: self.primary_font_id.clone(),
            primary_font_size: primary_size,
            result_font: self.result_font_id.clone(),
            result_font_size: result_size,
            theme_choice: self.theme_choice,
        })
    }
}

fn ensure_font_id(font_options: &[fonts_dropdown::FontOption], value: &str) -> String {
    if let Some(option) = font_options.iter().find(|option| option.id == value) {
        return option.id.clone();
    }

    let lowered = value.trim().to_lowercase();

    if let Some(option) = font_options.iter().find(|option| option.name.to_lowercase() == lowered) {
        return option.id.clone();
    }

    fonts::default_font_id().to_string()
}

fn parse_integer<T>(value: &str, label: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let trimmed = value.trim();
    trimmed
        .parse::<T>()
        .map_err(|_| tr_format("Invalid numeric value for \"{}\".", &[label]).to_string())
}

pub fn settings_view(state: &SettingsWindowState) -> Element<Message> {
    let tab_row = tab_buttons(state.active_tab);

    let tab_content: Element<_> = match state.active_tab {
        SettingsTab::Behavior => behavior_tab(state),
        SettingsTab::Appearance => appearance_tab(state),
    };

    let content = Column::new()
        .spacing(20)
        .push(fonts::primary_text(tr("Settings"), Some(10.0)).color(Color::from_rgb8(0x17, 0x1a, 0x20)))
        .push(tab_row)
        .push(tab_content);

    let mut content = if let Some(error) = &state.validation_error {
        content.push(fonts::primary_text(error.clone(), Some(-1.0)).color(Color::from_rgb8(0xd9, 0x53, 0x4f)))
    } else {
        content
    };

    content = content.push(bottom_actions());

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
    .push(fonts::primary_text(tr("Query timeout (seconds)"), None))
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

fn font_picker_row<'a>(
    state: &'a SettingsWindowState,
    label: &'static str,
    font_id: &'a str,
    font_size: &'a str,
    is_open: bool,
    on_toggle: Message,
    on_font_change: fn(String) -> Message,
    on_size_change: fn(String) -> Message,
) -> Element<'a, Message> {
    let selected = if font_id.is_empty() { None } else { Some(font_id) };

    let dropdown = FontDropdown::new(
        tr(label),
        &state.font_options,
        selected,
        is_open,
        on_toggle,
        on_font_change,
    )
    .width(Length::FillPortion(5))
    .max_height(240.0);

    Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr(label), None).width(Length::FillPortion(3)))
        .push(dropdown)
        .push(
            text_input(tr("Font Size"), font_size)
                .on_input(on_size_change)
                .padding([6, 10])
                .width(Length::Fixed(120.0)),
        )
        .into()
}

fn appearance_tab(state: &SettingsWindowState) -> Element<Message> {
    let language_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr("Language"), None).width(Length::FillPortion(3)))
        .push(
            PickList::new(ALL_LANGUAGES, Some(state.language), Message::SettingsLanguageChanged)
                .width(Length::FillPortion(4)),
        )
        .push(Space::with_width(Length::FillPortion(2)))
        .push(Space::with_width(Length::Fixed(120.0)));

    let primary_row = font_picker_row(
        state,
        "Primary Font",
        &state.primary_font_id,
        &state.primary_font_size,
        state.primary_font_open,
        Message::SettingsPrimaryFontDropdownToggled,
        Message::SettingsPrimaryFontChanged,
        Message::SettingsPrimaryFontSizeChanged,
    );

    let result_row = font_picker_row(
        state,
        "Query Result Font",
        &state.result_font_id,
        &state.result_font_size,
        state.result_font_open,
        Message::SettingsResultFontDropdownToggled,
        Message::SettingsResultFontChanged,
        Message::SettingsResultFontSizeChanged,
    );

    let theme_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr("Theme"), None).width(Length::FillPortion(3)))
        .push(
            PickList::new(ALL_THEMES, Some(state.theme_choice), Message::SettingsThemeChanged)
                .width(Length::FillPortion(4)),
        )
        .push(Space::with_width(Length::FillPortion(2)))
        .push(Space::with_width(Length::Fixed(120.0)));

    Column::new()
        .spacing(16)
        .push(language_row)
        .push(primary_row)
        .push(result_row)
        .push(theme_row)
        .into()
}

fn bottom_actions() -> Element<'static, Message> {
    let apply = Button::new(fonts::primary_text(tr("Apply"), None))
        .padding([6, 16])
        .on_press(Message::SettingsApply);
    let cancel = Button::new(fonts::primary_text(tr("Cancel"), None))
        .padding([6, 16])
        .on_press(Message::SettingsCancel);
    let save = Button::new(fonts::primary_text(tr("Save"), None))
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
    let mut behavior = Button::new(fonts::primary_text(tr(SettingsTab::Behavior.label()), None))
    .padding([6, 16])
    .style(move |_, _| tab_button_style(active == SettingsTab::Behavior));
    if active != SettingsTab::Behavior {
        behavior = behavior.on_press(Message::SettingsTabChanged(SettingsTab::Behavior));
    }

    let mut appearance = Button::new(fonts::primary_text(tr(SettingsTab::Appearance.label()), None))
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
