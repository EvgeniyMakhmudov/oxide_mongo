use std::collections::HashMap;

use super::Language;

mod english;
mod french;
mod russian;
mod spanish;

use english::english_docs;
use french::french_docs;
use russian::russian_docs;
use spanish::spanish_docs;

pub struct DocSection {
    pub title: &'static str,
    pub markdown: &'static str,
}

const DOC_SECTION_ORDER: &[&str] =
    &["general", "quick-start", "supported-commands", "change-stream", "hotkeys"];

pub fn doc_section_order() -> &'static [&'static str] {
    DOC_SECTION_ORDER
}

pub fn doc_section(slug: &str) -> Option<&'static DocSection> {
    doc_section_for_language(slug, super::current_language())
        .or_else(|| doc_section_for_language(slug, Language::English))
}

fn doc_section_for_language(slug: &str, language: Language) -> Option<&'static DocSection> {
    let map: &'static HashMap<&'static str, DocSection> = match language {
        Language::English => english_docs(),
        Language::Russian => russian_docs(),
        Language::Spanish => spanish_docs(),
        Language::French => french_docs(),
    };
    map.get(slug)
}
