use fontdb::Database;
use iced::Font;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::{OnceLock, RwLock};

pub const MONO_FONT_BYTES: &[u8] = include_bytes!("../assests/fonts/DejaVuSansMono.ttf");
pub const MONO_FONT_NAME: &str = "DejaVu Sans Mono";
pub const MONO_FONT_ID: &str = "bundled:dejavu-sans-mono";
pub const MONO_FONT_LABEL: &str = "DejaVu Sans Mono (Bundled)";
pub const MONO_FONT: Font = Font::with_name(MONO_FONT_NAME);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontOption {
    pub id: String,
    pub name: String,
    pub font: Font,
}

impl FontOption {
    pub fn bundled() -> Self {
        Self { id: MONO_FONT_ID.to_string(), name: MONO_FONT_LABEL.to_string(), font: MONO_FONT }
    }

    pub fn new(id: String, name: String) -> Self {
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        Self { font: Font::with_name(leaked), id, name }
    }
}

impl fmt::Display for FontOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

static FONT_OPTIONS: OnceLock<Vec<FontOption>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct ActiveFonts {
    pub primary_font: Font,
    pub primary_size: f32,
    pub result_font: Font,
    pub result_size: f32,
}

impl ActiveFonts {
    fn default() -> Self {
        Self {
            primary_font: MONO_FONT,
            primary_size: 16.0,
            result_font: MONO_FONT,
            result_size: 14.0,
        }
    }
}

static ACTIVE_FONTS: OnceLock<RwLock<ActiveFonts>> = OnceLock::new();

pub fn available_fonts() -> &'static [FontOption] {
    FONT_OPTIONS.get_or_init(load_font_options).as_slice()
}

pub fn default_font_id() -> &'static str {
    MONO_FONT_ID
}

pub fn font_option_by_id(id: &str) -> Option<FontOption> {
    available_fonts().iter().find(|option| option.id == id).cloned()
}

fn fonts_lock() -> &'static RwLock<ActiveFonts> {
    ACTIVE_FONTS.get_or_init(|| RwLock::new(ActiveFonts::default()))
}

pub fn set_active_fonts(primary_id: &str, primary_size: f32, result_id: &str, result_size: f32) {
    let primary_font = font_option_by_id(primary_id).map(|opt| opt.font).unwrap_or(MONO_FONT);
    let result_font = font_option_by_id(result_id).map(|opt| opt.font).unwrap_or(MONO_FONT);

    let mut guard = fonts_lock().write().expect("active fonts lock poisoned");
    *guard = ActiveFonts { primary_font, primary_size, result_font, result_size };
}

pub fn active_fonts() -> ActiveFonts {
    fonts_lock().read().expect("active fonts lock poisoned").clone()
}

pub fn primary_text<'a>(
    content: impl Into<String>,
    size_delta: Option<f32>,
) -> iced::widget::Text<'a> {
    let fonts = active_fonts();
    let size = fonts.primary_size + size_delta.unwrap_or(0.0);
    iced::widget::Text::new(content.into()).font(fonts.primary_font).size(size)
}

pub fn result_text<'a>(
    content: impl Into<String>,
    size_delta: Option<f32>,
) -> iced::widget::Text<'a> {
    let fonts = active_fonts();
    let size = fonts.result_size + size_delta.unwrap_or(0.0);
    iced::widget::Text::new(content.into()).font(fonts.result_font).size(size)
}

#[allow(dead_code)]
pub fn apply_primary_font<'a>(text: iced::widget::Text<'a>) -> iced::widget::Text<'a> {
    let fonts = active_fonts();
    text.font(fonts.primary_font)
}

#[allow(dead_code)]
pub fn apply_result_font<'a>(text: iced::widget::Text<'a>) -> iced::widget::Text<'a> {
    let fonts = active_fonts();
    text.font(fonts.result_font)
}

fn load_font_options() -> Vec<FontOption> {
    let mut options = Vec::new();
    options.push(FontOption::bundled());

    let mut database = Database::new();
    database.load_system_fonts();

    let mut families: BTreeSet<String> = BTreeSet::new();

    for face in database.faces() {
        for (name, _) in &face.families {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.eq_ignore_ascii_case(MONO_FONT_NAME) {
                continue;
            }
            families.insert(trimmed.to_string());
        }
    }

    for name in families {
        let id = format!("system:{name}");
        options.push(FontOption::new(id, name));
    }

    options
}
