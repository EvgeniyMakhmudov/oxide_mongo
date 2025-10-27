use crate::fonts;
use crate::i18n::Language;
use iced::widget::{self, button};
use iced::{Color, Shadow, border};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, RwLockWriteGuard};

pub const SETTINGS_FILE_NAME: &str = "settings.toml";

static GLOBAL_SETTINGS: OnceLock<RwLock<AppSettings>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const fn label(self) -> &'static str {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: Language,
    pub expand_first_result: bool,
    pub query_timeout_secs: u64,
    pub sort_fields_alphabetically: bool,
    pub sort_index_names_alphabetically: bool,
    pub primary_font: String,
    pub primary_font_size: u16,
    pub result_font: String,
    pub result_font_size: u16,
    pub theme_choice: ThemeChoice,
    pub theme_colors: ThemeColors,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::English,
            expand_first_result: true,
            query_timeout_secs: 600,
            sort_fields_alphabetically: false,
            sort_index_names_alphabetically: false,
            primary_font: fonts::default_font_id().to_string(),
            primary_font_size: 16,
            result_font: fonts::default_font_id().to_string(),
            result_font_size: 14,
            theme_choice: ThemeChoice::System,
            theme_colors: ThemeColors::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn to_color(self) -> Color {
        Color::from_rgba8(self.r, self.g, self.b, self.a as f32 / 255.0)
    }

    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Default for RgbaColor {
    fn default() -> Self {
        Self::opaque(0, 0, 0)
    }
}

impl From<Color> for RgbaColor {
    fn from(color: Color) -> Self {
        fn to_u8(component: f32) -> u8 {
            (component.clamp(0.0, 1.0) * 255.0).round() as u8
        }

        Self { r: to_u8(color.r), g: to_u8(color.g), b: to_u8(color.b), a: to_u8(color.a) }
    }
}

impl From<RgbaColor> for Color {
    fn from(value: RgbaColor) -> Self {
        value.to_color()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ButtonColors {
    pub active: RgbaColor,
    pub hover: RgbaColor,
    pub pressed: RgbaColor,
    pub text: RgbaColor,
    pub border: RgbaColor,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            active: RgbaColor::opaque(0, 0, 0),
            hover: RgbaColor::opaque(0, 0, 0),
            pressed: RgbaColor::opaque(0, 0, 0),
            text: RgbaColor::opaque(255, 255, 255),
            border: RgbaColor::opaque(0, 0, 0),
        }
    }
}

impl ButtonColors {
    pub fn style(&self, radius: f32, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => self.active,
            button::Status::Hovered => self.hover,
            button::Status::Pressed => self.pressed,
            button::Status::Disabled => self.active,
        };

        button::Style {
            background: Some(background.to_color().into()),
            text_color: self.text.to_color(),
            border: border::rounded(radius).width(1).color(self.border.to_color()),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableColors {
    pub row_even: RgbaColor,
    pub row_odd: RgbaColor,
    pub header_background: RgbaColor,
    pub separator: RgbaColor,
}

impl Default for TableColors {
    fn default() -> Self {
        Self {
            row_even: RgbaColor::opaque(0, 0, 0),
            row_odd: RgbaColor::opaque(0, 0, 0),
            header_background: RgbaColor::opaque(0, 0, 0),
            separator: RgbaColor::opaque(0, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuColors {
    pub background: RgbaColor,
    pub hover_background: RgbaColor,
    pub text: RgbaColor,
}

impl Default for MenuColors {
    fn default() -> Self {
        Self {
            background: RgbaColor::opaque(0, 0, 0),
            hover_background: RgbaColor::opaque(0, 0, 0),
            text: RgbaColor::opaque(255, 255, 255),
        }
    }
}

impl MenuColors {
    pub fn button_style(
        &self,
        radius: f32,
        border_color: Color,
        status: button::Status,
    ) -> button::Style {
        let background = match status {
            button::Status::Active => self.background,
            button::Status::Hovered => self.hover_background,
            button::Status::Pressed => self.hover_background,
            button::Status::Disabled => self.background,
        };

        button::Style {
            background: Some(background.to_color().into()),
            text_color: self.text.to_color(),
            border: border::rounded(radius).width(1).color(border_color),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemePalette {
    pub widget_background: RgbaColor,
    pub widget_border: RgbaColor,
    pub subtle_buttons: ButtonColors,
    pub primary_buttons: ButtonColors,
    pub table: TableColors,
    pub menu: MenuColors,
    pub text_primary: RgbaColor,
    pub text_muted: RgbaColor,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::light()
    }
}

impl ThemePalette {
    pub fn light() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0xef, 0xf1, 0xf5),
            widget_border: RgbaColor::opaque(0xd0, 0xd4, 0xda),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0xf3, 0xf5, 0xfa),
                hover: RgbaColor::opaque(0xe8, 0xec, 0xf5),
                pressed: RgbaColor::opaque(0xdc, 0xe2, 0xef),
                text: RgbaColor::opaque(0x22, 0x28, 0x38),
                border: RgbaColor::opaque(0xc6, 0xcc, 0xd9),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x36, 0x71, 0xc9),
                hover: RgbaColor::opaque(0x2f, 0x63, 0xb0),
                pressed: RgbaColor::opaque(0x26, 0x54, 0x98),
                text: RgbaColor::opaque(0xff, 0xff, 0xff),
                border: RgbaColor::opaque(0x1f, 0x41, 0x73),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0xfe, 0xfe, 0xfe),
                row_odd: RgbaColor::opaque(0xf9, 0xfd, 0xf9),
                header_background: RgbaColor::opaque(0xef, 0xf1, 0xf5),
                separator: RgbaColor::opaque(0xd0, 0xd4, 0xda),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0xff, 0xff, 0xff),
                hover_background: RgbaColor::opaque(0xe6, 0xec, 0xf8),
                text: RgbaColor::opaque(0x17, 0x1a, 0x20),
            },
            text_primary: RgbaColor::opaque(0x17, 0x1a, 0x20),
            text_muted: RgbaColor::opaque(0x55, 0x5f, 0x73),
        }
    }

    pub fn dark() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0x22, 0x27, 0x31),
            widget_border: RgbaColor::opaque(0x39, 0x40, 0x4d),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0x2c, 0x33, 0x40),
                hover: RgbaColor::opaque(0x35, 0x3c, 0x4a),
                pressed: RgbaColor::opaque(0x3f, 0x47, 0x57),
                text: RgbaColor::opaque(0xdf, 0xe4, 0xee),
                border: RgbaColor::opaque(0x47, 0x50, 0x5f),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x4a, 0x8d, 0xff),
                hover: RgbaColor::opaque(0x5c, 0x99, 0xff),
                pressed: RgbaColor::opaque(0x3f, 0x7c, 0xe6),
                text: RgbaColor::opaque(0xff, 0xff, 0xff),
                border: RgbaColor::opaque(0x29, 0x62, 0xd9),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0x2e, 0x34, 0x40),
                row_odd: RgbaColor::opaque(0x27, 0x2d, 0x38),
                header_background: RgbaColor::opaque(0x25, 0x2b, 0x36),
                separator: RgbaColor::opaque(0x44, 0x4c, 0x5a),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0x24, 0x29, 0x34),
                hover_background: RgbaColor::opaque(0x30, 0x37, 0x43),
                text: RgbaColor::opaque(0xe5, 0xea, 0xf3),
            },
            text_primary: RgbaColor::opaque(0xe5, 0xea, 0xf3),
            text_muted: RgbaColor::opaque(0xa9, 0xb1, 0xc1),
        }
    }

    pub fn widget_background_color(&self) -> Color {
        self.widget_background.to_color()
    }

    pub fn widget_border_color(&self) -> Color {
        self.widget_border.to_color()
    }

    pub fn subtle_button_style(&self, radius: f32, status: button::Status) -> button::Style {
        self.subtle_buttons.style(radius, status)
    }

    pub fn primary_button_style(&self, radius: f32, status: button::Status) -> button::Style {
        self.primary_buttons.style(radius, status)
    }

    pub fn menu_button_style(&self, radius: f32, status: button::Status) -> button::Style {
        let background = match status {
            button::Status::Active => self.menu.background,
            button::Status::Hovered => self.menu.hover_background,
            button::Status::Pressed => self.menu.hover_background,
            button::Status::Disabled => self.menu.background,
        };

        button::Style {
            background: Some(background.to_color().into()),
            text_color: self.menu.text.to_color(),
            border: border::rounded(radius).width(1).color(self.widget_border_color()),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }

    pub fn container_style(&self, radius: f32) -> widget::container::Style {
        widget::container::Style {
            background: Some(self.widget_background_color().into()),
            border: border::rounded(radius).width(1).color(self.widget_border_color()),
            text_color: Some(self.text_primary.to_color()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeColors {
    pub light: ThemePalette,
    pub dark: ThemePalette,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self { light: ThemePalette::light(), dark: ThemePalette::dark() }
    }
}

impl ThemeColors {
    pub fn palette(&self, choice: ThemeChoice) -> &ThemePalette {
        match choice {
            ThemeChoice::System | ThemeChoice::Light => &self.light,
            ThemeChoice::Dark => &self.dark,
        }
    }
}

#[derive(Debug)]
pub enum SettingsLoadError {
    Io(io::Error),
    Parse(toml::de::Error),
}

impl fmt::Display for SettingsLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsLoadError::Io(error) => write!(f, "I/O error: {}", error),
            SettingsLoadError::Parse(error) => write!(f, "Parse error: {}", error),
        }
    }
}

impl std::error::Error for SettingsLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SettingsLoadError::Io(error) => Some(error),
            SettingsLoadError::Parse(error) => Some(error),
        }
    }
}

#[derive(Debug)]
pub enum SettingsSaveError {
    Io(io::Error),
    Serialize(toml::ser::Error),
}

impl fmt::Display for SettingsSaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsSaveError::Io(error) => write!(f, "I/O error: {}", error),
            SettingsSaveError::Serialize(error) => write!(f, "Serialize error: {}", error),
        }
    }
}

impl std::error::Error for SettingsSaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SettingsSaveError::Io(error) => Some(error),
            SettingsSaveError::Serialize(error) => Some(error),
        }
    }
}

pub fn settings_path() -> PathBuf {
    PathBuf::from(SETTINGS_FILE_NAME)
}

pub fn load_from_disk() -> Result<AppSettings, SettingsLoadError> {
    let path = settings_path();
    match fs::read_to_string(path) {
        Ok(contents) => toml::from_str::<AppSettings>(&contents)
            .map(|mut settings| {
                settings.normalize_fonts();
                settings
            })
            .map_err(SettingsLoadError::Parse),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut settings = AppSettings::default();
            settings.normalize_fonts();
            Ok(settings)
        }
        Err(error) => Err(SettingsLoadError::Io(error)),
    }
}

pub fn save_to_disk(settings: &AppSettings) -> Result<(), SettingsSaveError> {
    let rendered = toml::to_string_pretty(settings).map_err(SettingsSaveError::Serialize)?;
    fs::write(settings_path(), rendered).map_err(SettingsSaveError::Io)
}

pub fn initialize(settings: AppSettings) {
    if GLOBAL_SETTINGS.set(RwLock::new(settings.clone())).is_err() {
        replace(settings);
    }
}

fn write_guard() -> RwLockWriteGuard<'static, AppSettings> {
    let lock = GLOBAL_SETTINGS.get().expect("settings accessed before initialization");
    lock.write().expect("settings write lock poisoned")
}

pub fn replace(new_settings: AppSettings) {
    let mut guard = write_guard();
    *guard = new_settings;
}

pub const ALL_THEMES: &[ThemeChoice] =
    &[ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];

impl AppSettings {
    pub fn normalize_fonts(&mut self) {
        self.primary_font = normalize_font_id(&self.primary_font);
        self.result_font = normalize_font_id(&self.result_font);
    }

    pub fn active_palette(&self) -> &ThemePalette {
        self.theme_colors.palette(self.theme_choice)
    }
}

fn normalize_font_id(value: &str) -> String {
    if let Some(option) = fonts::font_option_by_id(value) {
        return option.id;
    }

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fonts::default_font_id().to_string();
    }

    let lowered = trimmed.to_lowercase();

    if let Some(option) =
        fonts::available_fonts().iter().find(|option| option.name.to_lowercase() == lowered)
    {
        return option.id.clone();
    }

    match lowered.as_str() {
        "system" | "system default" | "sans" => fonts::default_font_id().to_string(),
        "monospace" => fonts::font_option_by_id(fonts::MONO_FONT_ID)
            .map(|option| option.id)
            .unwrap_or_else(|| fonts::default_font_id().to_string()),
        "serif" => fonts::available_fonts()
            .iter()
            .find(|option| option.name.to_lowercase().contains("serif"))
            .map(|option| option.id.clone())
            .unwrap_or_else(|| fonts::default_font_id().to_string()),
        _ => fonts::default_font_id().to_string(),
    }
}
