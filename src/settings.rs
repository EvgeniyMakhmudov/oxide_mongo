use crate::fonts;
use crate::i18n::Language;
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
    if GLOBAL_SETTINGS
        .set(RwLock::new(settings.clone()))
        .is_err()
    {
        replace(settings);
    }
}

fn write_guard() -> RwLockWriteGuard<'static, AppSettings> {
    let lock = GLOBAL_SETTINGS
        .get()
        .expect("settings accessed before initialization");
    lock.write().expect("settings write lock poisoned")
}

pub fn replace(new_settings: AppSettings) {
    let mut guard = write_guard();
    *guard = new_settings;
}

pub const ALL_THEMES: &[ThemeChoice] = &[ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];

impl AppSettings {
    pub fn normalize_fonts(&mut self) {
        self.primary_font = normalize_font_id(&self.primary_font);
        self.result_font = normalize_font_id(&self.result_font);
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

    if let Some(option) = fonts::available_fonts()
        .iter()
        .find(|option| option.name.to_lowercase() == lowered)
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
