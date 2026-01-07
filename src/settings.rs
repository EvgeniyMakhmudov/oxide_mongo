use crate::fonts;
use crate::i18n::Language;
use iced::widget::button;
use iced::{Color, Shadow, border};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, RwLockWriteGuard};

pub const SETTINGS_FILE_NAME: &str = "settings.toml";
pub const DEFAULT_LOG_FILE_NAME: &str = "oxide_mongo.log";

static GLOBAL_SETTINGS: OnceLock<RwLock<AppSettings>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
    SolarizedLight,
    SolarizedDark,
    NordLight,
    NordDark,
    GruvboxLight,
    GruvboxDark,
    OneLight,
    OneDark,
}

impl ThemeChoice {
    pub const fn label(self) -> &'static str {
        match self {
            ThemeChoice::System => "System",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
            ThemeChoice::SolarizedLight => "Solarized Light",
            ThemeChoice::SolarizedDark => "Solarized Dark",
            ThemeChoice::NordLight => "Nord Light",
            ThemeChoice::NordDark => "Nord Dark",
            ThemeChoice::GruvboxLight => "Gruvbox Light",
            ThemeChoice::GruvboxDark => "Gruvbox Dark",
            ThemeChoice::OneLight => "One Light",
            ThemeChoice::OneDark => "One Dark",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub const fn label(self) -> &'static str {
        match self {
            LogLevel::Error => "Error",
            LogLevel::Warn => "Warn",
            LogLevel::Info => "Info",
            LogLevel::Debug => "Debug",
            LogLevel::Trace => "Trace",
        }
    }

    pub const fn to_level_filter(self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
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
    pub logging_enabled: bool,
    pub logging_level: LogLevel,
    pub logging_path: String,
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
            logging_enabled: false,
            logging_level: LogLevel::Info,
            logging_path: DEFAULT_LOG_FILE_NAME.to_string(),
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

    pub fn solarized_light() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0xee, 0xe8, 0xd5),
            widget_border: RgbaColor::opaque(0x93, 0xa1, 0xa1),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0xfd, 0xf6, 0xe3),
                hover: RgbaColor::opaque(0xf4, 0xe9, 0xc1),
                pressed: RgbaColor::opaque(0xe9, 0xdd, 0xaf),
                text: RgbaColor::opaque(0x65, 0x7b, 0x83),
                border: RgbaColor::opaque(0x93, 0xa1, 0xa1),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x26, 0x8b, 0xd2),
                hover: RgbaColor::opaque(0x2a, 0xa1, 0x98),
                pressed: RgbaColor::opaque(0x1f, 0x6b, 0xa8),
                text: RgbaColor::opaque(0xfd, 0xf6, 0xe3),
                border: RgbaColor::opaque(0x1f, 0x4f, 0x6b),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0xfd, 0xf6, 0xe3),
                row_odd: RgbaColor::opaque(0xf5, 0xe9, 0xd0),
                header_background: RgbaColor::opaque(0xee, 0xe8, 0xd5),
                separator: RgbaColor::opaque(0x93, 0xa1, 0xa1),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0xfd, 0xf6, 0xe3),
                hover_background: RgbaColor::opaque(0xee, 0xe8, 0xd5),
                text: RgbaColor::opaque(0x65, 0x7b, 0x83),
            },
            text_primary: RgbaColor::opaque(0x65, 0x7b, 0x83),
            text_muted: RgbaColor::opaque(0x93, 0xa1, 0xa1),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0x07, 0x36, 0x42),
            widget_border: RgbaColor::opaque(0x58, 0x6e, 0x75),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0x07, 0x36, 0x42),
                hover: RgbaColor::opaque(0x0c, 0x4f, 0x5c),
                pressed: RgbaColor::opaque(0x06, 0x31, 0x3b),
                text: RgbaColor::opaque(0x93, 0xa1, 0xa1),
                border: RgbaColor::opaque(0x58, 0x6e, 0x75),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x26, 0x8b, 0xd2),
                hover: RgbaColor::opaque(0x2a, 0xa1, 0x98),
                pressed: RgbaColor::opaque(0x1f, 0x69, 0x96),
                text: RgbaColor::opaque(0xfd, 0xf6, 0xe3),
                border: RgbaColor::opaque(0x1c, 0x4f, 0x6e),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0x00, 0x2b, 0x36),
                row_odd: RgbaColor::opaque(0x07, 0x36, 0x42),
                header_background: RgbaColor::opaque(0x00, 0x3b, 0x4d),
                separator: RgbaColor::opaque(0x58, 0x6e, 0x75),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0x00, 0x2b, 0x36),
                hover_background: RgbaColor::opaque(0x07, 0x36, 0x42),
                text: RgbaColor::opaque(0x93, 0xa1, 0xa1),
            },
            text_primary: RgbaColor::opaque(0x93, 0xa1, 0xa1),
            text_muted: RgbaColor::opaque(0x58, 0x6e, 0x75),
        }
    }

    pub fn nord_light() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0xec, 0xef, 0xf4),
            widget_border: RgbaColor::opaque(0xd8, 0xde, 0xe9),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0xe5, 0xe9, 0xf0),
                hover: RgbaColor::opaque(0xd8, 0xde, 0xe9),
                pressed: RgbaColor::opaque(0xcc, 0xd3, 0xe0),
                text: RgbaColor::opaque(0x4c, 0x56, 0x6a),
                border: RgbaColor::opaque(0xc0, 0xcb, 0xd9),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x5e, 0x81, 0xac),
                hover: RgbaColor::opaque(0x81, 0xa1, 0xc1),
                pressed: RgbaColor::opaque(0x4c, 0x6a, 0x94),
                text: RgbaColor::opaque(0xec, 0xef, 0xf4),
                border: RgbaColor::opaque(0x3b, 0x54, 0x79),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0xf4, 0xf6, 0xfb),
                row_odd: RgbaColor::opaque(0xec, 0xef, 0xf4),
                header_background: RgbaColor::opaque(0xe5, 0xe9, 0xf0),
                separator: RgbaColor::opaque(0xd8, 0xde, 0xe9),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0xf4, 0xf6, 0xfb),
                hover_background: RgbaColor::opaque(0xec, 0xef, 0xf4),
                text: RgbaColor::opaque(0x4c, 0x56, 0x6a),
            },
            text_primary: RgbaColor::opaque(0x4c, 0x56, 0x6a),
            text_muted: RgbaColor::opaque(0x7b, 0x88, 0xa1),
        }
    }

    pub fn nord_dark() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0x2e, 0x34, 0x40),
            widget_border: RgbaColor::opaque(0x3b, 0x42, 0x52),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0x3b, 0x42, 0x52),
                hover: RgbaColor::opaque(0x43, 0x4c, 0x5e),
                pressed: RgbaColor::opaque(0x29, 0x2d, 0x37),
                text: RgbaColor::opaque(0xd8, 0xde, 0xe9),
                border: RgbaColor::opaque(0x4c, 0x56, 0x6a),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x88, 0xc0, 0xd0),
                hover: RgbaColor::opaque(0x81, 0xa1, 0xc1),
                pressed: RgbaColor::opaque(0x5e, 0x81, 0xac),
                text: RgbaColor::opaque(0x2e, 0x34, 0x40),
                border: RgbaColor::opaque(0x3b, 0x54, 0x79),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0x2e, 0x34, 0x40),
                row_odd: RgbaColor::opaque(0x23, 0x29, 0x35),
                header_background: RgbaColor::opaque(0x3b, 0x42, 0x52),
                separator: RgbaColor::opaque(0x4c, 0x56, 0x6a),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0x23, 0x29, 0x35),
                hover_background: RgbaColor::opaque(0x2e, 0x34, 0x40),
                text: RgbaColor::opaque(0xd8, 0xde, 0xe9),
            },
            text_primary: RgbaColor::opaque(0xd8, 0xde, 0xe9),
            text_muted: RgbaColor::opaque(0x7b, 0x88, 0xa1),
        }
    }

    pub fn gruvbox_light() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0xf2, 0xe5, 0xbc),
            widget_border: RgbaColor::opaque(0xd5, 0xc4, 0xa1),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0xfb, 0xf1, 0xc7),
                hover: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
                pressed: RgbaColor::opaque(0xd5, 0xc4, 0xa1),
                text: RgbaColor::opaque(0x3c, 0x38, 0x36),
                border: RgbaColor::opaque(0xd5, 0xc4, 0xa1),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0xb1, 0x62, 0x86),
                hover: RgbaColor::opaque(0xd3, 0x86, 0x9b),
                pressed: RgbaColor::opaque(0x8f, 0x3f, 0x71),
                text: RgbaColor::opaque(0xfb, 0xf1, 0xc7),
                border: RgbaColor::opaque(0x73, 0x23, 0x51),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0xfb, 0xf1, 0xc7),
                row_odd: RgbaColor::opaque(0xf9, 0xf5, 0xd7),
                header_background: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
                separator: RgbaColor::opaque(0xd5, 0xc4, 0xa1),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0xfb, 0xf1, 0xc7),
                hover_background: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
                text: RgbaColor::opaque(0x3c, 0x38, 0x36),
            },
            text_primary: RgbaColor::opaque(0x3c, 0x38, 0x36),
            text_muted: RgbaColor::opaque(0x7c, 0x6f, 0x64),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0x3c, 0x38, 0x36),
            widget_border: RgbaColor::opaque(0x50, 0x49, 0x45),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0x3c, 0x38, 0x36),
                hover: RgbaColor::opaque(0x50, 0x49, 0x45),
                pressed: RgbaColor::opaque(0x28, 0x28, 0x28),
                text: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
                border: RgbaColor::opaque(0x66, 0x5c, 0x54),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0xd7, 0x99, 0x21),
                hover: RgbaColor::opaque(0xfa, 0xbd, 0x2f),
                pressed: RgbaColor::opaque(0xb5, 0x76, 0x14),
                text: RgbaColor::opaque(0x28, 0x28, 0x28),
                border: RgbaColor::opaque(0x8f, 0x61, 0x0f),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0x32, 0x30, 0x2f),
                row_odd: RgbaColor::opaque(0x28, 0x28, 0x28),
                header_background: RgbaColor::opaque(0x50, 0x49, 0x45),
                separator: RgbaColor::opaque(0x66, 0x5c, 0x54),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0x28, 0x28, 0x28),
                hover_background: RgbaColor::opaque(0x3c, 0x38, 0x36),
                text: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
            },
            text_primary: RgbaColor::opaque(0xeb, 0xdb, 0xb2),
            text_muted: RgbaColor::opaque(0xbd, 0xae, 0x93),
        }
    }

    pub fn one_light() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0xf5, 0xf5, 0xf5),
            widget_border: RgbaColor::opaque(0xd0, 0xd0, 0xd0),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0xfd, 0xfd, 0xfd),
                hover: RgbaColor::opaque(0xee, 0xee, 0xee),
                pressed: RgbaColor::opaque(0xdd, 0xdd, 0xdd),
                text: RgbaColor::opaque(0x38, 0x3a, 0x42),
                border: RgbaColor::opaque(0xc5, 0xc5, 0xc5),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x40, 0x7b, 0xd8),
                hover: RgbaColor::opaque(0x2f, 0x66, 0xc5),
                pressed: RgbaColor::opaque(0x25, 0x54, 0xa6),
                text: RgbaColor::opaque(0xfd, 0xfd, 0xfd),
                border: RgbaColor::opaque(0x1c, 0x44, 0x85),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0xfd, 0xfd, 0xfd),
                row_odd: RgbaColor::opaque(0xf0, 0xf0, 0xf0),
                header_background: RgbaColor::opaque(0xee, 0xee, 0xee),
                separator: RgbaColor::opaque(0xd0, 0xd0, 0xd0),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0xfd, 0xfd, 0xfd),
                hover_background: RgbaColor::opaque(0xee, 0xee, 0xee),
                text: RgbaColor::opaque(0x38, 0x3a, 0x42),
            },
            text_primary: RgbaColor::opaque(0x38, 0x3a, 0x42),
            text_muted: RgbaColor::opaque(0x6c, 0x70, 0x7c),
        }
    }

    pub fn one_dark() -> Self {
        Self {
            widget_background: RgbaColor::opaque(0x28, 0x2c, 0x34),
            widget_border: RgbaColor::opaque(0x3e, 0x44, 0x51),
            subtle_buttons: ButtonColors {
                active: RgbaColor::opaque(0x28, 0x2c, 0x34),
                hover: RgbaColor::opaque(0x32, 0x37, 0x45),
                pressed: RgbaColor::opaque(0x20, 0x23, 0x2b),
                text: RgbaColor::opaque(0xe5, 0xe5, 0xe5),
                border: RgbaColor::opaque(0x4c, 0x54, 0x63),
            },
            primary_buttons: ButtonColors {
                active: RgbaColor::opaque(0x61, 0xaf, 0xef),
                hover: RgbaColor::opaque(0x52, 0x9c, 0xd9),
                pressed: RgbaColor::opaque(0x3b, 0x78, 0xa6),
                text: RgbaColor::opaque(0x16, 0x1b, 0x22),
                border: RgbaColor::opaque(0x2b, 0x55, 0x7b),
            },
            table: TableColors {
                row_even: RgbaColor::opaque(0x23, 0x27, 0x2f),
                row_odd: RgbaColor::opaque(0x1b, 0x1f, 0x26),
                header_background: RgbaColor::opaque(0x32, 0x37, 0x45),
                separator: RgbaColor::opaque(0x4c, 0x54, 0x63),
            },
            menu: MenuColors {
                background: RgbaColor::opaque(0x1b, 0x1f, 0x26),
                hover_background: RgbaColor::opaque(0x23, 0x27, 0x2f),
                text: RgbaColor::opaque(0xe5, 0xe5, 0xe5),
            },
            text_primary: RgbaColor::opaque(0xe5, 0xe5, 0xe5),
            text_muted: RgbaColor::opaque(0x93, 0x99, 0xa6),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeColors {
    pub light: ThemePalette,
    pub dark: ThemePalette,
    pub solarized_light: ThemePalette,
    pub solarized_dark: ThemePalette,
    pub nord_light: ThemePalette,
    pub nord_dark: ThemePalette,
    pub gruvbox_light: ThemePalette,
    pub gruvbox_dark: ThemePalette,
    pub one_light: ThemePalette,
    pub one_dark: ThemePalette,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            light: ThemePalette::light(),
            dark: ThemePalette::dark(),
            solarized_light: ThemePalette::solarized_light(),
            solarized_dark: ThemePalette::solarized_dark(),
            nord_light: ThemePalette::nord_light(),
            nord_dark: ThemePalette::nord_dark(),
            gruvbox_light: ThemePalette::gruvbox_light(),
            gruvbox_dark: ThemePalette::gruvbox_dark(),
            one_light: ThemePalette::one_light(),
            one_dark: ThemePalette::one_dark(),
        }
    }
}

impl ThemeColors {
    pub fn palette(&self, choice: ThemeChoice) -> &ThemePalette {
        match choice {
            ThemeChoice::System | ThemeChoice::Light => &self.light,
            ThemeChoice::Dark => &self.dark,
            ThemeChoice::SolarizedLight => &self.solarized_light,
            ThemeChoice::SolarizedDark => &self.solarized_dark,
            ThemeChoice::NordLight => &self.nord_light,
            ThemeChoice::NordDark => &self.nord_dark,
            ThemeChoice::GruvboxLight => &self.gruvbox_light,
            ThemeChoice::GruvboxDark => &self.gruvbox_dark,
            ThemeChoice::OneLight => &self.one_light,
            ThemeChoice::OneDark => &self.one_dark,
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
                settings.normalize_logging();
                settings
            })
            .map_err(SettingsLoadError::Parse),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut settings = AppSettings::default();
            settings.normalize_fonts();
            settings.normalize_logging();
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

pub const ALL_THEMES: &[ThemeChoice] = &[
    ThemeChoice::System,
    ThemeChoice::Light,
    ThemeChoice::Dark,
    ThemeChoice::SolarizedLight,
    ThemeChoice::SolarizedDark,
    ThemeChoice::NordLight,
    ThemeChoice::NordDark,
    ThemeChoice::GruvboxLight,
    ThemeChoice::GruvboxDark,
    ThemeChoice::OneLight,
    ThemeChoice::OneDark,
];

pub const ALL_LOG_LEVELS: &[LogLevel] =
    &[LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug, LogLevel::Trace];

impl AppSettings {
    pub fn normalize_fonts(&mut self) {
        self.primary_font = normalize_font_id(&self.primary_font);
        self.result_font = normalize_font_id(&self.result_font);
    }

    pub fn normalize_logging(&mut self) {
        if self.logging_path.trim().is_empty() {
            self.logging_path = DEFAULT_LOG_FILE_NAME.to_string();
        }
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
