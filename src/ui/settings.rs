use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::checkbox::Checkbox;
use iced::widget::pick_list::PickList;
use iced::widget::{
    Button, Column, Container, Row, Scrollable, Space, button, container, text_input,
};
use iced::{Color, Element, Length, Shadow, border};

use crate::Message;
use crate::fonts;
use crate::i18n::{ALL_LANGUAGES, Language, tr, tr_format};
use crate::settings::{
    ALL_LOG_LEVELS, ALL_THEMES, AppSettings, DEFAULT_LOG_FILE_NAME, LogLevel, RgbaColor,
    ThemeChoice, ThemeColors, ThemePalette,
};
use crate::ui::fonts_dropdown::{self, FontDropdown};
use crate::ui::modal::modal_layout;
use iced_aw::ColorPicker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Behavior,
    Appearance,
    ColorTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColorField {
    WidgetBackground,
    WidgetBorder,
    SubtleActive,
    SubtleHover,
    SubtlePressed,
    SubtleText,
    SubtleBorder,
    PrimaryActive,
    PrimaryHover,
    PrimaryPressed,
    PrimaryText,
    PrimaryBorder,
    TableRowEven,
    TableRowOdd,
    TableHeaderBackground,
    TableSeparator,
    MenuBackground,
    MenuHoverBackground,
    MenuText,
}

const WIDGET_FIELDS: &[(ThemeColorField, &'static str)] = &[
    (ThemeColorField::WidgetBackground, "Widget Background"),
    (ThemeColorField::WidgetBorder, "Widget Border"),
];

const SUBTLE_BUTTON_FIELDS: &[(ThemeColorField, &'static str)] = &[
    (ThemeColorField::SubtleActive, "Active"),
    (ThemeColorField::SubtleHover, "Hover"),
    (ThemeColorField::SubtlePressed, "Pressed"),
    (ThemeColorField::SubtleText, "Text"),
    (ThemeColorField::SubtleBorder, "Border"),
];

const PRIMARY_BUTTON_FIELDS: &[(ThemeColorField, &'static str)] = &[
    (ThemeColorField::PrimaryActive, "Active"),
    (ThemeColorField::PrimaryHover, "Hover"),
    (ThemeColorField::PrimaryPressed, "Pressed"),
    (ThemeColorField::PrimaryText, "Text"),
    (ThemeColorField::PrimaryBorder, "Border"),
];

const TABLE_FIELDS: &[(ThemeColorField, &'static str)] = &[
    (ThemeColorField::TableRowEven, "Even Row"),
    (ThemeColorField::TableRowOdd, "Odd Row"),
    (ThemeColorField::TableHeaderBackground, "Header Background"),
    (ThemeColorField::TableSeparator, "Separator"),
];

const MENU_FIELDS: &[(ThemeColorField, &'static str)] = &[
    (ThemeColorField::MenuBackground, "Menu Background"),
    (ThemeColorField::MenuHoverBackground, "Menu Hover Background"),
    (ThemeColorField::MenuText, "Menu Text"),
];

impl SettingsTab {
    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Behavior => "Behavior",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::ColorTheme => "Color Theme",
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
    pub logging_enabled: bool,
    pub logging_level: LogLevel,
    pub logging_path: String,
    pub language: Language,
    pub font_options: Vec<fonts_dropdown::FontOption>,
    pub primary_font_open: bool,
    pub primary_font_id: String,
    pub primary_font_size: String,
    pub result_font_open: bool,
    pub result_font_id: String,
    pub result_font_size: String,
    pub theme_choice: ThemeChoice,
    pub theme_light: ThemePalette,
    pub theme_dark: ThemePalette,
    pub theme_solarized_light: ThemePalette,
    pub theme_solarized_dark: ThemePalette,
    pub theme_nord_light: ThemePalette,
    pub theme_nord_dark: ThemePalette,
    pub theme_gruvbox_light: ThemePalette,
    pub theme_gruvbox_dark: ThemePalette,
    pub theme_one_light: ThemePalette,
    pub theme_one_dark: ThemePalette,
    pub active_color_picker: Option<ThemeColorField>,
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
            logging_enabled: settings.logging_enabled,
            logging_level: settings.logging_level,
            logging_path: settings.logging_path.clone(),
            language: settings.language,
            font_options,
            primary_font_open: false,
            primary_font_id,
            primary_font_size: settings.primary_font_size.to_string(),
            result_font_open: false,
            result_font_id,
            result_font_size: settings.result_font_size.to_string(),
            theme_choice: settings.theme_choice,
            theme_light: settings.theme_colors.light.clone(),
            theme_dark: settings.theme_colors.dark.clone(),
            theme_solarized_light: settings.theme_colors.solarized_light.clone(),
            theme_solarized_dark: settings.theme_colors.solarized_dark.clone(),
            theme_nord_light: settings.theme_colors.nord_light.clone(),
            theme_nord_dark: settings.theme_colors.nord_dark.clone(),
            theme_gruvbox_light: settings.theme_colors.gruvbox_light.clone(),
            theme_gruvbox_dark: settings.theme_colors.gruvbox_dark.clone(),
            theme_one_light: settings.theme_colors.one_light.clone(),
            theme_one_dark: settings.theme_colors.one_dark.clone(),
            active_color_picker: None,
            validation_error: None,
        }
    }

    pub fn to_app_settings(&self) -> Result<AppSettings, String> {
        let timeout =
            parse_integer::<u64>(&self.query_timeout_secs, tr("Query timeout (seconds)"))?;
        let primary_size = parse_integer::<u16>(&self.primary_font_size, tr("Primary Font"))?;
        let result_size = parse_integer::<u16>(&self.result_font_size, tr("Query Result Font"))?;
        let log_path = if self.logging_path.trim().is_empty() {
            DEFAULT_LOG_FILE_NAME.to_string()
        } else {
            self.logging_path.trim().to_string()
        };

        if primary_size == 0 || result_size == 0 {
            return Err(tr("Font size must be greater than zero").to_owned());
        }

        Ok(AppSettings {
            expand_first_result: self.expand_first_result,
            query_timeout_secs: timeout,
            sort_fields_alphabetically: self.sort_fields_alphabetically,
            sort_index_names_alphabetically: self.sort_index_names_alphabetically,
            logging_enabled: self.logging_enabled,
            logging_level: self.logging_level,
            logging_path: log_path,
            language: self.language,
            primary_font: self.primary_font_id.clone(),
            primary_font_size: primary_size,
            result_font: self.result_font_id.clone(),
            result_font_size: result_size,
            theme_choice: self.theme_choice,
            theme_colors: ThemeColors {
                light: self.theme_light.clone(),
                dark: self.theme_dark.clone(),
                solarized_light: self.theme_solarized_light.clone(),
                solarized_dark: self.theme_solarized_dark.clone(),
                nord_light: self.theme_nord_light.clone(),
                nord_dark: self.theme_nord_dark.clone(),
                gruvbox_light: self.theme_gruvbox_light.clone(),
                gruvbox_dark: self.theme_gruvbox_dark.clone(),
                one_light: self.theme_one_light.clone(),
                one_dark: self.theme_one_dark.clone(),
            },
        })
    }

    fn edit_theme_choice(&self) -> ThemeChoice {
        match self.theme_choice {
            ThemeChoice::System => ThemeChoice::Light,
            other => other,
        }
    }

    fn palette_for_edit(&self) -> &ThemePalette {
        match self.edit_theme_choice() {
            ThemeChoice::Light => &self.theme_light,
            ThemeChoice::Dark => &self.theme_dark,
            ThemeChoice::SolarizedLight => &self.theme_solarized_light,
            ThemeChoice::SolarizedDark => &self.theme_solarized_dark,
            ThemeChoice::NordLight => &self.theme_nord_light,
            ThemeChoice::NordDark => &self.theme_nord_dark,
            ThemeChoice::GruvboxLight => &self.theme_gruvbox_light,
            ThemeChoice::GruvboxDark => &self.theme_gruvbox_dark,
            ThemeChoice::OneLight => &self.theme_one_light,
            ThemeChoice::OneDark => &self.theme_one_dark,
            ThemeChoice::System => &self.theme_light,
        }
    }

    fn palette_for_edit_mut(&mut self) -> &mut ThemePalette {
        match self.edit_theme_choice() {
            ThemeChoice::Light => &mut self.theme_light,
            ThemeChoice::Dark => &mut self.theme_dark,
            ThemeChoice::SolarizedLight => &mut self.theme_solarized_light,
            ThemeChoice::SolarizedDark => &mut self.theme_solarized_dark,
            ThemeChoice::NordLight => &mut self.theme_nord_light,
            ThemeChoice::NordDark => &mut self.theme_nord_dark,
            ThemeChoice::GruvboxLight => &mut self.theme_gruvbox_light,
            ThemeChoice::GruvboxDark => &mut self.theme_gruvbox_dark,
            ThemeChoice::OneLight => &mut self.theme_one_light,
            ThemeChoice::OneDark => &mut self.theme_one_dark,
            ThemeChoice::System => &mut self.theme_light,
        }
    }

    fn color_for_field(&self, field: ThemeColorField) -> RgbaColor {
        let palette = self.palette_for_edit();
        match field {
            ThemeColorField::WidgetBackground => palette.widget_background,
            ThemeColorField::WidgetBorder => palette.widget_border,
            ThemeColorField::SubtleActive => palette.subtle_buttons.active,
            ThemeColorField::SubtleHover => palette.subtle_buttons.hover,
            ThemeColorField::SubtlePressed => palette.subtle_buttons.pressed,
            ThemeColorField::SubtleText => palette.subtle_buttons.text,
            ThemeColorField::SubtleBorder => palette.subtle_buttons.border,
            ThemeColorField::PrimaryActive => palette.primary_buttons.active,
            ThemeColorField::PrimaryHover => palette.primary_buttons.hover,
            ThemeColorField::PrimaryPressed => palette.primary_buttons.pressed,
            ThemeColorField::PrimaryText => palette.primary_buttons.text,
            ThemeColorField::PrimaryBorder => palette.primary_buttons.border,
            ThemeColorField::TableRowEven => palette.table.row_even,
            ThemeColorField::TableRowOdd => palette.table.row_odd,
            ThemeColorField::TableHeaderBackground => palette.table.header_background,
            ThemeColorField::TableSeparator => palette.table.separator,
            ThemeColorField::MenuBackground => palette.menu.background,
            ThemeColorField::MenuHoverBackground => palette.menu.hover_background,
            ThemeColorField::MenuText => palette.menu.text,
        }
    }

    pub fn set_color_for_field(&mut self, field: ThemeColorField, color: Color) {
        let palette = self.palette_for_edit_mut();
        let value = RgbaColor::from(color);
        match field {
            ThemeColorField::WidgetBackground => palette.widget_background = value,
            ThemeColorField::WidgetBorder => palette.widget_border = value,
            ThemeColorField::SubtleActive => palette.subtle_buttons.active = value,
            ThemeColorField::SubtleHover => palette.subtle_buttons.hover = value,
            ThemeColorField::SubtlePressed => palette.subtle_buttons.pressed = value,
            ThemeColorField::SubtleText => palette.subtle_buttons.text = value,
            ThemeColorField::SubtleBorder => palette.subtle_buttons.border = value,
            ThemeColorField::PrimaryActive => palette.primary_buttons.active = value,
            ThemeColorField::PrimaryHover => palette.primary_buttons.hover = value,
            ThemeColorField::PrimaryPressed => palette.primary_buttons.pressed = value,
            ThemeColorField::PrimaryText => palette.primary_buttons.text = value,
            ThemeColorField::PrimaryBorder => palette.primary_buttons.border = value,
            ThemeColorField::TableRowEven => palette.table.row_even = value,
            ThemeColorField::TableRowOdd => palette.table.row_odd = value,
            ThemeColorField::TableHeaderBackground => palette.table.header_background = value,
            ThemeColorField::TableSeparator => palette.table.separator = value,
            ThemeColorField::MenuBackground => palette.menu.background = value,
            ThemeColorField::MenuHoverBackground => palette.menu.hover_background = value,
            ThemeColorField::MenuText => palette.menu.text = value,
        }
    }

    pub fn reset_theme_colors(&mut self) {
        match self.edit_theme_choice() {
            ThemeChoice::Light => self.theme_light = ThemePalette::light(),
            ThemeChoice::Dark => self.theme_dark = ThemePalette::dark(),
            ThemeChoice::SolarizedLight => {
                self.theme_solarized_light = ThemePalette::solarized_light()
            }
            ThemeChoice::SolarizedDark => {
                self.theme_solarized_dark = ThemePalette::solarized_dark()
            }
            ThemeChoice::NordLight => self.theme_nord_light = ThemePalette::nord_light(),
            ThemeChoice::NordDark => self.theme_nord_dark = ThemePalette::nord_dark(),
            ThemeChoice::GruvboxLight => self.theme_gruvbox_light = ThemePalette::gruvbox_light(),
            ThemeChoice::GruvboxDark => self.theme_gruvbox_dark = ThemePalette::gruvbox_dark(),
            ThemeChoice::OneLight => self.theme_one_light = ThemePalette::one_light(),
            ThemeChoice::OneDark => self.theme_one_dark = ThemePalette::one_dark(),
            ThemeChoice::System => self.theme_light = ThemePalette::light(),
        }
        self.active_color_picker = None;
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

pub fn settings_view(state: &SettingsWindowState) -> Element<'_, Message> {
    let palette = state.palette_for_edit().clone();
    let text_color = palette.text_primary.to_color();
    let muted_color = palette.text_muted.to_color();

    let header = fonts::primary_text(tr("Settings"), Some(10.0)).color(text_color);

    let tab_row = tab_buttons(state.active_tab);

    let tab_content: Element<_> = match state.active_tab {
        SettingsTab::Behavior => behavior_tab(state, text_color),
        SettingsTab::Appearance => appearance_tab(state, text_color),
        SettingsTab::ColorTheme => color_theme_tab(state, palette.clone(), text_color, muted_color),
    };

    let mut scroll_content = Column::new().spacing(20).push(tab_content);

    if let Some(error) = &state.validation_error {
        scroll_content = scroll_content.push(
            fonts::primary_text(error.clone(), Some(-1.0))
                .color(Color::from_rgb8(0xd9, 0x53, 0x4f)),
        );
    }

    let scrollable =
        Scrollable::new(scroll_content).width(Length::Fill).height(Length::Fixed(360.0));

    let card_content = Column::new()
        .spacing(16)
        .push(header)
        .push(tab_row)
        .push(scrollable)
        .push(bottom_actions(&palette));

    let card_element: Element<Message> = card_content.into();
    modal_layout(palette, card_element, Length::Fixed(640.0), 24, 12.0)
}

fn behavior_tab(state: &SettingsWindowState, text_color: Color) -> Element<'_, Message> {
    let expand_checkbox = Checkbox::new(tr("Expand first result item"), state.expand_first_result)
        .on_toggle(Message::SettingsToggleExpandFirstResult);

    let timeout_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr("Query timeout (seconds)"), None).color(text_color))
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

    let logging_enabled = Checkbox::new(tr("Enable logging"), state.logging_enabled)
        .on_toggle(Message::SettingsToggleLogging);

    let log_level_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr("Log level"), None).color(text_color))
        .push(
            PickList::new(
                ALL_LOG_LEVELS,
                Some(state.logging_level),
                Message::SettingsLogLevelChanged,
            )
            .width(Length::Fixed(180.0)),
        );

    let log_path_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr("Log file path"), None).color(text_color))
        .push(
            text_input(tr("Path"), &state.logging_path)
                .on_input(Message::SettingsLogPathChanged)
                .padding([6, 10])
                .width(Length::Fill),
        );

    Column::new()
        .spacing(16)
        .push(expand_checkbox)
        .push(timeout_row)
        .push(sort_fields)
        .push(sort_indexes)
        .push(logging_enabled)
        .push(log_level_row)
        .push(log_path_row)
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
    text_color: Color,
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
        .push(fonts::primary_text(tr(label), None).color(text_color).width(Length::FillPortion(3)))
        .push(dropdown)
        .push(
            text_input(tr("Font Size"), font_size)
                .on_input(on_size_change)
                .padding([6, 10])
                .width(Length::Fixed(120.0)),
        )
        .into()
}

fn appearance_tab(state: &SettingsWindowState, text_color: Color) -> Element<'_, Message> {
    let language_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(
            fonts::primary_text(tr("Language"), None)
                .color(text_color)
                .width(Length::FillPortion(3)),
        )
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
        text_color,
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
        text_color,
    );

    Column::new().spacing(16).push(language_row).push(primary_row).push(result_row).into()
}

fn color_theme_tab(
    state: &SettingsWindowState,
    palette: ThemePalette,
    text_color: Color,
    _muted_color: Color,
) -> Element<'_, Message> {
    let theme_row = Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(
            fonts::primary_text(tr("Theme"), None).color(text_color).width(Length::FillPortion(3)),
        )
        .push(
            PickList::new(ALL_THEMES, Some(state.theme_choice), Message::SettingsThemeChanged)
                .width(Length::FillPortion(4)),
        )
        .push(Space::with_width(Length::FillPortion(2)))
        .push(Space::with_width(Length::Fixed(120.0)));

    let reset_palette = palette.clone();
    let reset_button =
        Button::new(fonts::primary_text(tr("Default Colors"), None).color(text_color))
            .padding([6, 16])
            .on_press(Message::SettingsThemeColorsReset)
            .style(move |_, status| reset_palette.subtle_button_style(6.0, status));

    let reset_row = Row::new().spacing(12).push(Space::with_width(Length::Fill)).push(reset_button);

    Column::new()
        .spacing(20)
        .push(theme_row)
        .push(color_group(state, &palette, text_color, "Widget Surfaces", WIDGET_FIELDS))
        .push(color_group(state, &palette, text_color, "Subtle Buttons", SUBTLE_BUTTON_FIELDS))
        .push(color_group(state, &palette, text_color, "Primary Buttons", PRIMARY_BUTTON_FIELDS))
        .push(color_group(state, &palette, text_color, "Table Rows", TABLE_FIELDS))
        .push(color_group(state, &palette, text_color, "Menu Items", MENU_FIELDS))
        .push(reset_row)
        .into()
}

fn color_group<'a>(
    state: &'a SettingsWindowState,
    palette: &ThemePalette,
    text_color: Color,
    title_key: &'static str,
    fields: &'a [(ThemeColorField, &'static str)],
) -> Element<'a, Message> {
    let fonts_state = fonts::active_fonts();
    let heading_font = iced::Font { weight: Weight::Bold, ..fonts_state.primary_font };

    let mut column = Column::new().spacing(12);
    column = column
        .push(fonts::primary_text(tr(title_key), Some(2.0)).font(heading_font).color(text_color));

    for (field, label_key) in fields.iter().copied() {
        column = column.push(color_picker_row(state, palette, field, label_key, text_color));
    }

    column.into()
}

fn color_picker_row<'a>(
    state: &'a SettingsWindowState,
    palette: &ThemePalette,
    field: ThemeColorField,
    label: &'static str,
    text_color: Color,
) -> Element<'a, Message> {
    let color_value = state.color_for_field(field);
    let color = color_value.to_color();
    let show_picker = state.active_color_picker == Some(field);
    let hex_text = color_value.to_hex();

    let swatch_color = color;
    let swatch =
        Container::new(Space::new(Length::Fixed(32.0), Length::Fixed(20.0))).style(move |_| {
            container::Style {
                background: Some(swatch_color.into()),
                border: border::rounded(4).width(1).color(Color::from_rgba8(0, 0, 0, 0.2)),
                ..Default::default()
            }
        });

    let hex_label = fonts::primary_text(hex_text, Some(-1.0)).color(text_color);

    let button_content =
        Row::new().spacing(8).align_y(Vertical::Center).push(swatch).push(hex_label);

    let palette_clone = palette.clone();
    let picker_button = Button::new(button_content)
        .padding([4, 12])
        .on_press(Message::SettingsColorPickerOpened(field))
        .style(move |_, status| palette_clone.subtle_button_style(6.0, status));

    let color_picker = ColorPicker::new(
        show_picker,
        color,
        picker_button,
        Message::SettingsColorPickerCanceled,
        move |selected| Message::SettingsColorChanged(field, selected),
    );

    let picker_element: Element<_> = color_picker.into();
    let picker_container = Container::new(picker_element).width(Length::FillPortion(4));

    Row::new()
        .spacing(12)
        .align_y(Vertical::Center)
        .push(fonts::primary_text(tr(label), None).color(text_color).width(Length::FillPortion(4)))
        .push(picker_container)
        .push(Space::with_width(Length::Fill))
        .into()
}

fn bottom_actions(palette: &ThemePalette) -> Element<'static, Message> {
    let apply_palette = palette.clone();
    let cancel_palette = palette.clone();
    let save_palette = palette.clone();

    let apply = Button::new(fonts::primary_text(tr("Apply"), None))
        .padding([6, 16])
        .on_press(Message::SettingsApply)
        .style(move |_, status| apply_palette.primary_button_style(6.0, status));

    let cancel = Button::new(fonts::primary_text(tr("Cancel"), None))
        .padding([6, 16])
        .on_press(Message::SettingsCancel)
        .style(move |_, status| cancel_palette.primary_button_style(6.0, status));

    let save = Button::new(fonts::primary_text(tr("Save"), None))
        .padding([6, 16])
        .on_press(Message::SettingsSave)
        .style(move |_, status| save_palette.primary_button_style(6.0, status));

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

    let mut appearance =
        Button::new(fonts::primary_text(tr(SettingsTab::Appearance.label()), None))
            .padding([6, 16])
            .style(move |_, _| tab_button_style(active == SettingsTab::Appearance));
    if active != SettingsTab::Appearance {
        appearance = appearance.on_press(Message::SettingsTabChanged(SettingsTab::Appearance));
    }

    let mut color_theme =
        Button::new(fonts::primary_text(tr(SettingsTab::ColorTheme.label()), None))
            .padding([6, 16])
            .style(move |_, _| tab_button_style(active == SettingsTab::ColorTheme));
    if active != SettingsTab::ColorTheme {
        color_theme = color_theme.on_press(Message::SettingsTabChanged(SettingsTab::ColorTheme));
    }

    Row::new().spacing(8).push(behavior).push(appearance).push(color_theme)
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
