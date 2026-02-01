use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub mod docs;

mod french;
mod german;
mod russian;
mod spanish;
use french::french_map;
use german::german_map;
use russian::russian_map;
use spanish::spanish_map;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Russian,
    Spanish,
    French,
    German,
}

static CURRENT_LANGUAGE: OnceLock<RwLock<Language>> = OnceLock::new();

pub const ALL_LANGUAGES: &[Language] =
    &[Language::English, Language::Russian, Language::Spanish, Language::French, Language::German];

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Русский",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

fn language_lock() -> &'static RwLock<Language> {
    CURRENT_LANGUAGE.get_or_init(|| RwLock::new(Language::Russian))
}

pub fn init_language(language: Language) {
    if CURRENT_LANGUAGE.set(RwLock::new(language)).is_err() {
        set_language(language);
    }
}

pub fn set_language(language: Language) {
    let mut guard = language_lock().write().expect("language write lock poisoned");
    *guard = language;
}

fn current_language() -> Language {
    *language_lock().read().expect("language read lock poisoned")
}

fn english_fallback_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        russian_map().iter().map(|(english, russian)| (*russian, *english)).collect()
    })
}

pub fn tr(text: &'static str) -> &'static str {
    let english = english_fallback_map().get(text).copied().unwrap_or(text);
    match current_language() {
        Language::English => english,
        Language::Russian => russian_map().get(english).copied().unwrap_or(english),
        Language::Spanish => spanish_map().get(english).copied().unwrap_or(english),
        Language::French => french_map().get(english).copied().unwrap_or(english),
        Language::German => german_map().get(english).copied().unwrap_or(english),
    }
}

pub fn tr_format(template: &'static str, replacements: &[&str]) -> String {
    let mut result = tr(template).to_owned();
    for value in replacements {
        result = result.replacen("{}", value, 1);
    }
    result
}
