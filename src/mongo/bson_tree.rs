use std::collections::HashSet;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::Wrapping;
use iced::widget::{self, Button, Column, Container, Row, Scrollable, Space};
use iced::{Color, Element, Length, Shadow, Vector, border};
use iced_aw::ContextMenu;
use mongodb::bson::{Bson, Document};

use crate::fonts;
use crate::i18n::tr;
use crate::mongo::shell;
use crate::settings::{
    AppSettings, ButtonColors, MenuColors, RgbaColor, TableColors, ThemePalette,
};
use crate::{Message, TabId, TableContextAction, ValueEditContext};

#[derive(Debug)]
pub struct BsonTree {
    roots: Vec<BsonNode>,
    expanded: HashSet<usize>,
    context: BsonTreeContext,
    next_node_id: usize,
    sort_fields_alphabetically: bool,
    table_colors: TableColors,
    menu_colors: MenuColors,
    text_color: RgbaColor,
    button_colors: ButtonColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BsonTreeStats {
    pub root_count: usize,
    pub total_nodes: usize,
    pub container_nodes: usize,
    pub leaf_nodes: usize,
    pub max_depth: usize,
    pub expanded_nodes: usize,
    pub visible_rows: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct BsonTreeStatsAccumulator {
    total_nodes: usize,
    container_nodes: usize,
    leaf_nodes: usize,
    max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BsonTreeOptions {
    pub sort_fields_alphabetically: bool,
    pub sort_index_names_alphabetically: bool,
    pub table_colors: TableColors,
    pub menu_colors: MenuColors,
    pub text_color: RgbaColor,
    pub button_colors: ButtonColors,
}

impl BsonTreeOptions {
    pub fn new(
        sort_fields_alphabetically: bool,
        sort_index_names_alphabetically: bool,
        table_colors: TableColors,
        menu_colors: MenuColors,
        text_color: RgbaColor,
        button_colors: ButtonColors,
    ) -> Self {
        Self {
            sort_fields_alphabetically,
            sort_index_names_alphabetically,
            table_colors,
            menu_colors,
            text_color,
            button_colors,
        }
    }
}

impl Default for BsonTreeOptions {
    fn default() -> Self {
        let palette = ThemePalette::light();
        Self::new(
            false,
            false,
            palette.table,
            palette.menu,
            palette.text_primary,
            palette.subtle_buttons.clone(),
        )
    }
}

impl From<&AppSettings> for BsonTreeOptions {
    fn from(settings: &AppSettings) -> Self {
        let palette = settings.active_palette();
        let table_colors = palette.table.clone();
        let menu_colors = palette.menu.clone();
        let text_color = palette.text_primary;
        let button_colors = palette.subtle_buttons.clone();
        Self::new(
            settings.sort_fields_alphabetically,
            settings.sort_index_names_alphabetically,
            table_colors,
            menu_colors,
            text_color,
            button_colors,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BsonTreeContext {
    Default,
    Indexes,
}

struct TableContextMenu;

impl TableContextMenu {
    fn new<'a>(
        underlay: impl Into<Element<'a, Message>>,
        overlay: impl Fn() -> Element<'a, Message> + 'a,
    ) -> Element<'a, Message> {
        ContextMenu::new(underlay, overlay).into()
    }
}

fn style_menu_button<'a>(
    button: Button<'a, Message>,
    colors: &MenuColors,
    border_color: Color,
) -> Button<'a, Message> {
    let colors = colors.clone();
    button.style(move |_, status| colors.button_style(6.0, border_color, status))
}

fn menu_item_container<'a>(
    content: Element<'a, Message>,
    colors: &MenuColors,
    border_color: Color,
) -> Element<'a, Message> {
    let background = colors.background.to_color();
    let luminance = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
    let shadow_color = if luminance > 0.5 {
        Color::from_rgba(0.0, 0.0, 0.0, 0.75)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.30)
    };

    Container::new(content)
        .style(move |_| widget::container::Style {
            background: Some(background.into()),
            border: border::rounded(6.0).width(1).color(border_color),
            shadow: Shadow {
                color: shadow_color,
                offset: Vector::new(0.0, 3.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .into()
}

struct BsonRowEntry<'a> {
    depth: usize,
    node: &'a BsonNode,
    expanded: bool,
    path_enabled: bool,
    value_edit_enabled: bool,
    is_root_document: bool,
    relation_hint: Option<&'a str>,
}

#[derive(Debug, Clone)]
struct BsonNode {
    id: usize,
    display_key: Option<String>,
    path_key: Option<String>,
    kind: BsonKind,
    bson: Bson,
}

#[derive(Debug, Clone)]
enum BsonKind {
    Document { children: Option<Vec<BsonNode>> },
    Array { children: Option<Vec<BsonNode>> },
    Value { display: String, ty: String },
}

#[derive(Default)]
struct IdGenerator {
    next_id: usize,
}

impl IdGenerator {
    fn next(&mut self) -> usize {
        let current = self.next_id;
        self.next_id += 1;
        current
    }
}

impl BsonNode {
    fn from_bson_lazy(
        display_key: Option<String>,
        path_key: Option<String>,
        value: &Bson,
        id: &mut IdGenerator,
    ) -> Self {
        let id_value = id.next();
        match value {
            Bson::Document(_) => Self {
                id: id_value,
                display_key,
                path_key,
                kind: BsonKind::Document { children: None },
                bson: value.clone(),
            },
            Bson::Array(_) => Self {
                id: id_value,
                display_key,
                path_key,
                kind: BsonKind::Array { children: None },
                bson: value.clone(),
            },
            other => {
                let (display, ty) = shell::format_bson_scalar(other);
                Self {
                    id: id_value,
                    display_key,
                    path_key,
                    kind: BsonKind::Value { display, ty },
                    bson: other.clone(),
                }
            }
        }
    }

    fn is_container(&self) -> bool {
        matches!(self.kind, BsonKind::Document { .. } | BsonKind::Array { .. })
    }

    fn has_children(&self) -> bool {
        match &self.bson {
            Bson::Document(doc) => !doc.is_empty(),
            Bson::Array(items) => !items.is_empty(),
            _ => false,
        }
    }

    fn children(&self) -> Option<&[BsonNode]> {
        match &self.kind {
            BsonKind::Document { children } | BsonKind::Array { children } => children.as_deref(),
            _ => None,
        }
    }

    fn children_mut(&mut self) -> Option<&mut [BsonNode]> {
        match &mut self.kind {
            BsonKind::Document { children } | BsonKind::Array { children } => {
                children.as_deref_mut()
            }
            _ => None,
        }
    }

    fn materialize_children(&mut self, next_node_id: &mut usize, sort_fields_alphabetically: bool) {
        let mut id_gen = IdGenerator { next_id: *next_node_id };
        match &mut self.kind {
            BsonKind::Document { children } => {
                if children.is_some() {
                    return;
                }

                let Bson::Document(doc) = &self.bson else {
                    *children = Some(Vec::new());
                    return;
                };

                let mut entries: Vec<_> = doc.iter().collect();
                if sort_fields_alphabetically {
                    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
                }

                let loaded_children = entries
                    .into_iter()
                    .map(|(k, v)| {
                        BsonNode::from_bson_lazy(Some(k.clone()), Some(k.clone()), v, &mut id_gen)
                    })
                    .collect();
                *children = Some(loaded_children);
            }
            BsonKind::Array { children } => {
                if children.is_some() {
                    return;
                }

                let Bson::Array(items) = &self.bson else {
                    *children = Some(Vec::new());
                    return;
                };

                let loaded_children = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        BsonNode::from_bson_lazy(
                            Some(format!("[{index}]")),
                            Some(index.to_string()),
                            item,
                            &mut id_gen,
                        )
                    })
                    .collect();
                *children = Some(loaded_children);
            }
            BsonKind::Value { .. } => {}
        }
        *next_node_id = id_gen.next_id;
    }

    fn display_key(&self) -> String {
        self.display_key.clone().unwrap_or_else(|| String::from(tr("value")))
    }

    fn value_display(&self) -> Option<String> {
        match &self.kind {
            BsonKind::Document { .. } => {
                let count = match &self.bson {
                    Bson::Document(doc) => doc.len(),
                    _ => 0,
                };
                Some(format!("Document ({} fields)", count))
            }
            BsonKind::Array { .. } => {
                let count = match &self.bson {
                    Bson::Array(items) => items.len(),
                    _ => 0,
                };
                Some(format!("Array ({} items)", count))
            }
            BsonKind::Value { display, .. } => Some(display.clone()),
        }
    }

    fn type_label(&self) -> String {
        match &self.kind {
            BsonKind::Document { .. } => String::from(tr("Document")),
            BsonKind::Array { .. } => String::from(tr("Array")),
            BsonKind::Value { ty, .. } => ty.clone(),
        }
    }
}

fn is_editable_value(_value: &Bson) -> bool {
    true
}

fn is_array_index_component(component: &str) -> bool {
    !component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit())
}

fn split_identifier_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_was_lower = false;

    for ch in value.chars() {
        if matches!(ch, '_' | '-' | ' ' | '.') {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            previous_was_lower = false;
            continue;
        }

        if ch.is_ascii_uppercase() && previous_was_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }

        previous_was_lower = ch.is_ascii_lowercase();
        current.push(ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn to_snake_case_lower(value: &str) -> Option<String> {
    let words = split_identifier_words(value);
    if words.is_empty() {
        return None;
    }
    let normalized =
        words.into_iter().map(|word| word.to_ascii_lowercase()).collect::<Vec<_>>().join("_");
    if normalized.is_empty() { None } else { Some(normalized) }
}

fn strip_suffix_case_insensitive<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let lowered = value.to_ascii_lowercase();
    if lowered.ends_with(suffix) {
        let stripped = &value[..value.len().saturating_sub(suffix.len())];
        Some(stripped.trim_end_matches(['_', '-']))
    } else {
        None
    }
}

fn singularize(name: &str) -> String {
    if name.ends_with("ies") && name.len() > 3 {
        return format!("{}y", &name[..name.len() - 3]);
    }

    if name.ends_with("ses")
        || name.ends_with("xes")
        || name.ends_with("zes")
        || name.ends_with("ches")
        || name.ends_with("shes")
    {
        return name[..name.len() - 2].to_string();
    }

    if name.ends_with('s') && !name.ends_with("ss") && name.len() > 1 {
        return name[..name.len() - 1].to_string();
    }

    name.to_string()
}

fn pluralize(name: &str) -> String {
    if name.ends_with('y') && name.len() > 1 {
        let previous = name.as_bytes()[name.len() - 2] as char;
        if !matches!(previous, 'a' | 'e' | 'i' | 'o' | 'u') {
            return format!("{}ies", &name[..name.len() - 1]);
        }
    }

    if name.ends_with('s')
        || name.ends_with('x')
        || name.ends_with('z')
        || name.ends_with("ch")
        || name.ends_with("sh")
    {
        return format!("{name}es");
    }

    format!("{name}s")
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if value.is_empty() {
        return;
    }
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

pub(crate) fn related_collection_name_candidates(field_hint: &str) -> Vec<String> {
    let trimmed = field_hint.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut stems = Vec::new();
    stems.push(trimmed.to_string());

    for suffix in [
        "_ids",
        "ids",
        "_id",
        "id",
        "_references",
        "references",
        "_reference",
        "reference",
        "_refs",
        "refs",
        "_ref",
        "ref",
    ] {
        if let Some(stripped) = strip_suffix_case_insensitive(trimmed, suffix) {
            if !stripped.is_empty() {
                push_unique(&mut stems, stripped.to_string());
            }
        }
    }

    let mut normalized = Vec::new();
    for stem in stems {
        if let Some(snake) = to_snake_case_lower(&stem) {
            push_unique(&mut normalized, snake.clone());
            push_unique(&mut normalized, singularize(&snake));
            push_unique(&mut normalized, pluralize(&snake));
        }

        push_unique(&mut normalized, stem.to_ascii_lowercase());
    }

    normalized
}

fn has_related_collection(
    field_hint: &str,
    related_collections_lowercase: &HashSet<String>,
) -> bool {
    related_collection_name_candidates(field_hint)
        .into_iter()
        .map(|candidate| candidate.to_ascii_lowercase())
        .any(|candidate| related_collections_lowercase.contains(&candidate))
}

pub(crate) fn is_supported_reference_id_type(value: &Bson) -> bool {
    !matches!(value, Bson::Array(_) | Bson::RegularExpression(_) | Bson::Undefined | Bson::Null)
}

impl BsonTree {
    pub fn from_values(values: &[Bson], options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let mut roots = Vec::new();

        if values.is_empty() {
            let info_value = Bson::String(String::from(tr("No documents found")));
            let placeholder = BsonNode::from_bson_lazy(
                Some(String::from(tr("info"))),
                None,
                &info_value,
                &mut id_gen,
            );
            roots.push(placeholder);
        } else {
            for (index, value) in values.iter().enumerate() {
                let base_label = format!("[{}]", index + 1);
                let key = match value {
                    Bson::Document(doc) => doc
                        .get("_id")
                        .map(Self::summarize_id)
                        .map(|id| format!("{} {}", base_label, id))
                        .unwrap_or_else(|| base_label.clone()),
                    _ => base_label.clone(),
                };
                roots.push(BsonNode::from_bson_lazy(Some(key), None, value, &mut id_gen));
            }
        }

        let expanded = HashSet::new();

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Default,
            next_node_id: id_gen.next_id,
            sort_fields_alphabetically: options.sort_fields_alphabetically,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn from_error(message: String) -> Self {
        let value = Bson::String(message);
        Self::from_values(std::slice::from_ref(&value), BsonTreeOptions::default())
    }

    pub fn from_distinct(field: String, values: Vec<Bson>, options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let array_bson = Bson::Array(values);
        let path_key = field.clone();
        let mut node =
            BsonNode::from_bson_lazy(Some(field), Some(path_key), &array_bson, &mut id_gen);
        let mut expanded = HashSet::new();
        if node.has_children() {
            node.materialize_children(&mut id_gen.next_id, options.sort_fields_alphabetically);
            expanded.insert(node.id);
        }

        Self {
            roots: vec![node],
            expanded,
            context: BsonTreeContext::Default,
            next_node_id: id_gen.next_id,
            sort_fields_alphabetically: options.sort_fields_alphabetically,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn from_count(value: Bson, options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let node = BsonNode::from_bson_lazy(
            Some(String::from(tr("count"))),
            Some(String::from(tr("count"))),
            &value,
            &mut id_gen,
        );
        Self {
            roots: vec![node],
            expanded: HashSet::new(),
            context: BsonTreeContext::Default,
            next_node_id: id_gen.next_id,
            sort_fields_alphabetically: options.sort_fields_alphabetically,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn from_document(document: Document, options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let value = Bson::Document(document);
        let mut roots = Vec::new();
        let mut expanded = HashSet::new();

        let key = match &value {
            Bson::Document(doc) => doc
                .get("_id")
                .map(Self::summarize_id)
                .unwrap_or_else(|| String::from(tr("document"))),
            _ => String::from(tr("document")),
        };

        let mut node = BsonNode::from_bson_lazy(Some(key), None, &value, &mut id_gen);
        if node.has_children() {
            node.materialize_children(&mut id_gen.next_id, options.sort_fields_alphabetically);
            expanded.insert(node.id);
        }
        roots.push(node);

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Default,
            next_node_id: id_gen.next_id,
            sort_fields_alphabetically: options.sort_fields_alphabetically,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn from_indexes(values: &[Bson], options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let mut roots = Vec::new();

        let mut items: Vec<_> = values.iter().collect();
        if options.sort_index_names_alphabetically {
            items.sort_by(|left, right| {
                let left_name = match left {
                    Bson::Document(doc) => doc.get_str("name").unwrap_or_default(),
                    _ => "",
                };
                let right_name = match right {
                    Bson::Document(doc) => doc.get_str("name").unwrap_or_default(),
                    _ => "",
                };
                left_name.cmp(right_name)
            });
        }

        for (index, value) in items.into_iter().enumerate() {
            let base_label = format!("[{}]", index + 1);
            match value {
                Bson::Document(doc) => {
                    let name = doc.get("name").and_then(|name| name.as_str());
                    let display = match name {
                        Some(name) if !name.is_empty() => format!("{base_label} {name}"),
                        _ => base_label.clone(),
                    };
                    roots.push(BsonNode::from_bson_lazy(Some(display), None, value, &mut id_gen));
                }
                other => {
                    roots.push(BsonNode::from_bson_lazy(
                        Some(base_label.clone()),
                        None,
                        other,
                        &mut id_gen,
                    ));
                }
            }
        }

        let expanded = HashSet::new();

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Indexes,
            next_node_id: id_gen.next_id,
            sort_fields_alphabetically: options.sort_fields_alphabetically,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn is_indexes_view(&self) -> bool {
        matches!(self.context, BsonTreeContext::Indexes)
    }

    pub fn node_index_name(&self, node_id: usize) -> Option<String> {
        if !self.is_indexes_view() {
            return None;
        }
        let node = Self::find_node(&self.roots, node_id)?;
        if !self.is_root_node(node_id) {
            return None;
        }
        match &node.bson {
            Bson::Document(doc) => {
                doc.get("name").and_then(|name| name.as_str()).map(|name| name.to_string())
            }
            _ => None,
        }
    }

    pub fn node_index_hidden(&self, node_id: usize) -> Option<bool> {
        if !self.is_indexes_view() {
            return None;
        }
        let node = Self::find_node(&self.roots, node_id)?;
        if !self.is_root_node(node_id) {
            return None;
        }
        match &node.bson {
            Bson::Document(doc) => doc.get("hidden").and_then(|value| value.as_bool()),
            _ => None,
        }
    }

    pub fn view(&self, tab_id: TabId, related_collections: &[String]) -> Element<'_, Message> {
        let mut rows = Vec::new();
        self.collect_rows(&mut rows);
        let related_collections_lowercase: HashSet<String> =
            related_collections.iter().map(|name| name.to_ascii_lowercase()).collect();

        let row_color_a = self.table_colors.row_even.to_color();
        let row_color_b = self.table_colors.row_odd.to_color();
        let header_bg = self.table_colors.header_background.to_color();
        let separator_color = self.table_colors.separator.to_color();
        let text_color = self.text_color.to_color();

        let header_row = Row::new()
            .spacing(0)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Shrink)
            .push(
                Container::new(fonts::result_text(tr("Key"), None).color(text_color))
                    .width(Length::FillPortion(4))
                    .padding([6, 8]),
            )
            .push(
                Container::new(Space::new().width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .padding([6, 0])
                    .style(move |_| widget::container::Style {
                        background: Some(separator_color.into()),
                        ..Default::default()
                    }),
            )
            .push(
                Container::new(fonts::result_text(tr("Value"), None).color(text_color))
                    .width(Length::FillPortion(5))
                    .padding([6, 8]),
            )
            .push(
                Container::new(Space::new().width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .padding([6, 0])
                    .style(move |_| widget::container::Style {
                        background: Some(separator_color.into()),
                        ..Default::default()
                    }),
            )
            .push(
                Container::new(fonts::result_text(tr("Type"), None).color(text_color))
                    .width(Length::FillPortion(3))
                    .padding([6, 8]),
            );

        let header = Container::new(header_row).width(Length::Fill).height(Length::Shrink).style(
            move |_| widget::container::Style {
                background: Some(header_bg.into()),
                ..Default::default()
            },
        );

        let mut body = Column::new().spacing(1).width(Length::Fill).height(Length::Shrink);

        for (
            index,
            BsonRowEntry {
                depth,
                node,
                expanded,
                path_enabled,
                value_edit_enabled,
                is_root_document,
                relation_hint,
            },
        ) in rows.into_iter().enumerate()
        {
            let background = if index % 2 == 0 { row_color_a } else { row_color_b };

            let mut key_row = Row::new().spacing(6).align_y(Vertical::Center);
            key_row = key_row.push(Space::new().width(Length::Fixed((depth as f32) * 16.0)));

            if node.is_container() {
                let indicator = if expanded { "▼" } else { "▶" };
                let has_children = node.has_children();

                if has_children {
                    let button_colors = self.button_colors.clone();
                    let toggle = Button::new(fonts::result_text(indicator, None).color(text_color))
                        .padding([0, 4])
                        .style(move |_, status| button_colors.style(4.0, status))
                        .on_press(Message::CollectionTreeToggle { tab_id, node_id: node.id });
                    key_row = key_row.push(toggle);
                } else {
                    let disabled = Container::new(
                        fonts::result_text(indicator, None)
                            .color(Color::from_rgb8(0xb5, 0xbc, 0xc6)),
                    )
                    .padding([0, 4])
                    .width(Length::Fixed(18.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center);
                    key_row = key_row.push(disabled);
                }
            } else {
                key_row = key_row.push(Space::new().width(Length::Fixed(18.0)));
            }

            let key_label = node.display_key();
            key_row = key_row.push(
                fonts::result_text(key_label.clone(), None)
                    .color(text_color)
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill),
            );

            let value_text = node.value_display().unwrap_or_default();
            let type_text = node.type_label();

            let key_cell = Container::new(key_row).width(Length::FillPortion(4)).padding([6, 8]);

            let value_cell = Container::new(
                fonts::result_text(value_text.clone(), None)
                    .color(text_color)
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill),
            )
            .width(Length::FillPortion(5))
            .padding([6, 8]);

            let type_cell = Container::new(
                fonts::result_text(type_text.clone(), None)
                    .color(text_color)
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill),
            )
            .width(Length::FillPortion(3))
            .padding([6, 8]);

            let separator = |color: Color| {
                Container::new(Space::new().width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .style(move |_| widget::container::Style {
                        background: Some(color.into()),
                        ..Default::default()
                    })
            };

            let row_content = Row::new()
                .spacing(0)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .push(key_cell)
                .push(separator(separator_color))
                .push(value_cell)
                .push(separator(separator_color))
                .push(type_cell);

            let menu_node_id = node.id;
            let menu_tab_id = tab_id;
            let index_context = if self.is_indexes_view() && is_root_document {
                let (maybe_name, maybe_hidden, ttl_enabled) = match &node.bson {
                    Bson::Document(doc) => {
                        let maybe_name =
                            doc.get("name").and_then(|name| name.as_str()).map(str::to_string);
                        let maybe_hidden = maybe_name
                            .as_ref()
                            .and_then(|_| doc.get("hidden").and_then(|value| value.as_bool()));
                        let ttl_enabled = doc.contains_key("expireAfterSeconds");
                        (maybe_name, maybe_hidden, ttl_enabled)
                    }
                    _ => (None, None, false),
                };
                maybe_name.map(|name| (name, maybe_hidden, ttl_enabled))
            } else {
                None
            };
            let can_open_related_document = is_supported_reference_id_type(&node.bson)
                && relation_hint
                    .map(|hint| has_related_collection(hint, &related_collections_lowercase))
                    .unwrap_or(false);

            let row_container = Container::new(row_content).width(Length::Fill).style(move |_| {
                widget::container::Style {
                    background: Some(background.into()),
                    ..Default::default()
                }
            });

            let menu_colors = self.menu_colors.clone();
            let menu_border = self.table_colors.separator.to_color();

            let row_with_menu = TableContextMenu::new(row_container, move || {
                let mut menu = Column::new().spacing(6).padding([4, 6]);

                if node.is_container() {
                    let expand_button = style_menu_button(
                        Button::new(fonts::primary_text(tr("Expand Hierarchically"), None))
                            .padding([4, 12])
                            .width(Length::Shrink)
                            .on_press(Message::TableContextMenu {
                                tab_id: menu_tab_id,
                                node_id: menu_node_id,
                                action: TableContextAction::ExpandHierarchy,
                            }),
                        &menu_colors,
                        menu_border,
                    );

                    let collapse_button = style_menu_button(
                        Button::new(fonts::primary_text(tr("Collapse Hierarchically"), None))
                            .padding([4, 12])
                            .width(Length::Shrink)
                            .on_press(Message::TableContextMenu {
                                tab_id: menu_tab_id,
                                node_id: menu_node_id,
                                action: TableContextAction::CollapseHierarchy,
                            }),
                        &menu_colors,
                        menu_border,
                    );

                    menu = menu.push(menu_item_container(
                        expand_button.into(),
                        &menu_colors,
                        menu_border,
                    ));
                    menu = menu.push(menu_item_container(
                        collapse_button.into(),
                        &menu_colors,
                        menu_border,
                    ));
                }

                let expand_all_button = style_menu_button(
                    Button::new(fonts::primary_text(tr("Expand All Hierarchically"), None))
                        .padding([4, 12])
                        .width(Length::Shrink)
                        .on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::ExpandHierarchyAll,
                        }),
                    &menu_colors,
                    menu_border,
                );

                let collapse_all_button = style_menu_button(
                    Button::new(fonts::primary_text(tr("Collapse All Hierarchically"), None))
                        .padding([4, 12])
                        .width(Length::Shrink)
                        .on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::CollapseHierarchyAll,
                        }),
                    &menu_colors,
                    menu_border,
                );

                menu = menu.push(menu_item_container(
                    expand_all_button.into(),
                    &menu_colors,
                    menu_border,
                ));
                menu = menu.push(menu_item_container(
                    collapse_all_button.into(),
                    &menu_colors,
                    menu_border,
                ));

                let copy_json = style_menu_button(
                    Button::new(fonts::primary_text(tr("Copy JSON"), None))
                        .padding([4, 12])
                        .width(Length::Shrink)
                        .on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::CopyJson,
                        }),
                    &menu_colors,
                    menu_border,
                );

                let copy_key = style_menu_button(
                    Button::new(fonts::primary_text(tr("Copy Key"), None))
                        .padding([4, 12])
                        .width(Length::Shrink)
                        .on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::CopyKey,
                        }),
                    &menu_colors,
                    menu_border,
                );

                let copy_value = style_menu_button(
                    Button::new(fonts::primary_text(tr("Copy Value"), None))
                        .padding([4, 12])
                        .width(Length::Shrink)
                        .on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::CopyValue,
                        }),
                    &menu_colors,
                    menu_border,
                );

                let mut copy_path = Button::new(fonts::primary_text(tr("Copy Path"), None))
                    .padding([4, 12])
                    .width(Length::Shrink);

                if path_enabled {
                    copy_path = copy_path.on_press(Message::TableContextMenu {
                        tab_id: menu_tab_id,
                        node_id: menu_node_id,
                        action: TableContextAction::CopyPath,
                    });
                }
                let copy_path = style_menu_button(copy_path, &menu_colors, menu_border);

                menu = menu.push(menu_item_container(copy_json.into(), &menu_colors, menu_border));
                menu = menu.push(menu_item_container(copy_key.into(), &menu_colors, menu_border));
                menu = menu.push(menu_item_container(copy_value.into(), &menu_colors, menu_border));
                menu = menu.push(menu_item_container(copy_path.into(), &menu_colors, menu_border));
                if value_edit_enabled {
                    let edit_value = style_menu_button(
                        Button::new(fonts::primary_text(tr("Edit Value Only..."), None))
                            .padding([4, 12])
                            .width(Length::Shrink)
                            .on_press(Message::TableContextMenu {
                                tab_id: menu_tab_id,
                                node_id: menu_node_id,
                                action: TableContextAction::EditValue,
                            }),
                        &menu_colors,
                        menu_border,
                    );
                    menu = menu.push(menu_item_container(
                        edit_value.into(),
                        &menu_colors,
                        menu_border,
                    ));
                }

                if can_open_related_document {
                    let goto_related = style_menu_button(
                        Button::new(fonts::primary_text(tr("Go to Related Document"), None))
                            .padding([4, 12])
                            .width(Length::Shrink)
                            .on_press(Message::TableContextMenu {
                                tab_id: menu_tab_id,
                                node_id: menu_node_id,
                                action: TableContextAction::GoToRelatedDocument,
                            }),
                        &menu_colors,
                        menu_border,
                    );
                    menu = menu.push(menu_item_container(
                        goto_related.into(),
                        &menu_colors,
                        menu_border,
                    ));
                }

                if let Some((index_name, hidden_state, ttl_enabled)) = index_context.clone() {
                    let mut delete_button =
                        Button::new(fonts::primary_text(tr("Delete Index"), None))
                            .padding([4, 12])
                            .width(Length::Shrink);
                    if index_name != "_id_" {
                        delete_button = delete_button.on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::DeleteIndex,
                        });
                    }
                    let delete_button = style_menu_button(delete_button, &menu_colors, menu_border);
                    menu = menu.push(menu_item_container(
                        delete_button.into(),
                        &menu_colors,
                        menu_border,
                    ));

                    let hidden = hidden_state.unwrap_or(false);

                    let mut hide_button = Button::new(fonts::primary_text(tr("Hide Index"), None))
                        .padding([4, 12])
                        .width(Length::Shrink);
                    if !hidden {
                        hide_button = hide_button.on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::HideIndex,
                        });
                    }
                    let hide_button = style_menu_button(hide_button, &menu_colors, menu_border);
                    menu = menu.push(menu_item_container(
                        hide_button.into(),
                        &menu_colors,
                        menu_border,
                    ));

                    let mut unhide_button =
                        Button::new(fonts::primary_text(tr("Unhide Index"), None))
                            .padding([4, 12])
                            .width(Length::Shrink);
                    if hidden {
                        unhide_button = unhide_button.on_press(Message::TableContextMenu {
                            tab_id: menu_tab_id,
                            node_id: menu_node_id,
                            action: TableContextAction::UnhideIndex,
                        });
                    }
                    let unhide_button = style_menu_button(unhide_button, &menu_colors, menu_border);
                    menu = menu.push(menu_item_container(
                        unhide_button.into(),
                        &menu_colors,
                        menu_border,
                    ));

                    if ttl_enabled {
                        let edit_button =
                            Button::new(fonts::primary_text(tr("Edit Index..."), None))
                                .padding([4, 12])
                                .width(Length::Shrink)
                                .on_press(Message::DocumentEditRequested {
                                    tab_id: menu_tab_id,
                                    node_id: menu_node_id,
                                });
                        let edit_button = style_menu_button(edit_button, &menu_colors, menu_border);
                        menu = menu.push(menu_item_container(
                            edit_button.into(),
                            &menu_colors,
                            menu_border,
                        ));
                    } else {
                        let edit_button =
                            Button::new(fonts::primary_text(tr("Edit Index..."), None))
                                .padding([4, 12])
                                .width(Length::Shrink);
                        let edit_button = style_menu_button(edit_button, &menu_colors, menu_border);
                        menu = menu.push(menu_item_container(
                            edit_button.into(),
                            &menu_colors,
                            menu_border,
                        ));
                    }
                } else if is_root_document {
                    let edit_button =
                        Button::new(fonts::primary_text(tr("Edit Document..."), None))
                            .padding([4, 12])
                            .width(Length::Shrink)
                            .on_press(Message::DocumentEditRequested {
                                tab_id: menu_tab_id,
                                node_id: menu_node_id,
                            });
                    let edit_button = style_menu_button(edit_button, &menu_colors, menu_border);
                    menu = menu.push(menu_item_container(
                        edit_button.into(),
                        &menu_colors,
                        menu_border,
                    ));
                }

                menu.into()
            });

            body = body.push(row_with_menu);
        }

        let body_scroll = Scrollable::new(body)
            .id(format!("bson-tree-body-{tab_id}"))
            .on_scroll(move |viewport| Message::CollectionTableScrolled {
                tab_id,
                offset_y: viewport.relative_offset().y,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Column::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(header)
            .push(body_scroll);
        Container::new(content).width(Length::Fill).into()
    }

    pub fn toggle(&mut self, node_id: usize) {
        if self.expanded.contains(&node_id) {
            self.expanded.remove(&node_id);
        } else if self.is_container(node_id) && self.node_has_children(node_id) {
            self.ensure_children_loaded(node_id);
            self.expanded.insert(node_id);
        }
    }

    pub fn expand_recursive(&mut self, node_id: usize) {
        if !self.is_container(node_id) || !self.node_has_children(node_id) {
            return;
        }
        self.ensure_children_loaded(node_id);
        self.expanded.insert(node_id);
        if let Some(child_ids) = Self::find_node(&self.roots, node_id)
            .and_then(BsonNode::children)
            .map(|children| children.iter().map(|child| child.id).collect::<Vec<_>>())
        {
            for child_id in child_ids {
                self.expand_recursive(child_id);
            }
        }
    }

    pub fn set_table_colors(&mut self, colors: TableColors) {
        self.table_colors = colors;
    }

    pub fn set_menu_colors(&mut self, colors: MenuColors) {
        self.menu_colors = colors;
    }

    pub fn set_text_color(&mut self, color: RgbaColor) {
        self.text_color = color;
    }

    pub fn set_button_colors(&mut self, colors: ButtonColors) {
        self.button_colors = colors;
    }

    pub fn collapse_recursive(&mut self, node_id: usize) {
        if !self.is_container(node_id) {
            return;
        }
        if let Some(child_ids) = Self::find_node(&self.roots, node_id)
            .and_then(BsonNode::children)
            .map(|children| children.iter().map(|child| child.id).collect::<Vec<_>>())
        {
            for child_id in child_ids {
                self.collapse_recursive(child_id);
            }
        }
        self.expanded.remove(&node_id);
    }

    pub fn expand_all(&mut self) {
        let root_ids: Vec<usize> = self.roots.iter().map(|node| node.id).collect();
        for root_id in root_ids {
            self.expand_recursive(root_id);
        }
    }

    pub fn collapse_all(&mut self) {
        let root_ids: Vec<usize> = self.roots.iter().map(|node| node.id).collect();
        for root_id in root_ids {
            self.collapse_recursive(root_id);
        }
    }

    pub fn is_root_node(&self, node_id: usize) -> bool {
        self.roots.iter().any(|node| node.id == node_id)
    }

    pub fn first_root_id(&self) -> Option<usize> {
        self.roots.first().map(|node| node.id)
    }

    #[cfg(test)]
    pub(crate) fn root_id_at(&self, index: usize) -> Option<usize> {
        self.roots.get(index).map(|node| node.id)
    }

    pub fn expand_node(&mut self, node_id: usize) {
        if self.is_container(node_id) && self.node_has_children(node_id) {
            self.ensure_children_loaded(node_id);
            self.expanded.insert(node_id);
        }
    }

    pub fn node_display_key(&self, node_id: usize) -> Option<String> {
        Self::find_node(&self.roots, node_id).map(BsonNode::display_key)
    }

    pub fn node_value_display(&self, node_id: usize) -> Option<String> {
        Self::find_node(&self.roots, node_id).map(|node| node.value_display().unwrap_or_default())
    }

    pub fn node_bson(&self, node_id: usize) -> Option<Bson> {
        Self::find_node(&self.roots, node_id).map(|node| node.bson.clone())
    }

    pub fn node_path(&self, node_id: usize) -> Option<String> {
        let nodes = Self::find_node_path(&self.roots, node_id, &mut Vec::new())?;
        let mut components = Vec::new();
        for node in nodes {
            if let Some(component) = &node.path_key {
                components.push(component.clone());
            }
        }

        if components.is_empty() { None } else { Some(components.join(".")) }
    }

    pub fn node_relation_hint(&self, node_id: usize) -> Option<String> {
        let nodes = Self::find_node_path(&self.roots, node_id, &mut Vec::new())?;
        let mut hint = None;
        for node in nodes {
            let Some(component) = node.path_key.as_deref() else {
                continue;
            };
            if is_array_index_component(component) {
                continue;
            }
            hint = Some(component.to_string());
        }
        hint
    }

    #[cfg(test)]
    pub(crate) fn find_node_id_by_path(&mut self, path: &str) -> Option<usize> {
        let components: Vec<&str> =
            path.split('.').filter(|component| !component.is_empty()).collect();
        if components.is_empty() {
            return None;
        }

        let sort_fields_alphabetically = self.sort_fields_alphabetically;
        for root in &mut self.roots {
            if let Some(found) = Self::find_node_by_components_mut(
                root,
                &components,
                &mut self.next_node_id,
                sort_fields_alphabetically,
            ) {
                return Some(found);
            }
        }

        None
    }

    #[cfg(test)]
    fn find_node_by_components_mut(
        node: &mut BsonNode,
        components: &[&str],
        next_node_id: &mut usize,
        sort_fields_alphabetically: bool,
    ) -> Option<usize> {
        if components.is_empty() {
            return Some(node.id);
        }

        if node.is_container() {
            node.materialize_children(next_node_id, sort_fields_alphabetically);
        }

        let (head, tail) = components.split_first().expect("components is not empty");
        let child = match &mut node.kind {
            BsonKind::Document { children } | BsonKind::Array { children } => {
                let children = children.as_mut()?;
                children.iter_mut().find(|candidate| candidate.path_key.as_deref() == Some(*head))
            }
            _ => None,
        }?;

        Self::find_node_by_components_mut(child, tail, next_node_id, sort_fields_alphabetically)
    }

    pub fn value_edit_context(&self, node_id: usize) -> Option<ValueEditContext> {
        let (components, root_doc, value_node) = self.edit_requirements(node_id)?;
        let mut filter = Document::new();
        filter.insert("_id", root_doc.get("_id")?.clone());

        Some(ValueEditContext {
            path: components.join("."),
            filter,
            current_value: value_node.bson.clone(),
        })
    }

    fn edit_requirements(&self, node_id: usize) -> Option<(Vec<String>, &Document, &BsonNode)> {
        let nodes = Self::find_node_path(&self.roots, node_id, &mut Vec::new())?;
        let target = nodes.last()?;

        if !is_editable_value(&target.bson) {
            return None;
        }

        let mut components = Vec::new();
        for node in nodes.iter().skip(1) {
            if let Some(component) = &node.path_key {
                components.push(component.clone());
            }
        }

        if components.is_empty() {
            return None;
        }

        let root = nodes.first()?;
        let root_document = match &root.bson {
            Bson::Document(doc) => doc,
            _ => return None,
        };

        if !root_document.contains_key("_id") {
            return None;
        }

        Some((components, root_document, target))
    }

    fn collect_rows<'a>(&'a self, rows: &mut Vec<BsonRowEntry<'a>>) {
        for root in &self.roots {
            let root_has_id = matches!(&root.bson, Bson::Document(doc) if doc.contains_key("_id"));
            let root_has_path = root.path_key.is_some();
            self.collect_rows_from_node(rows, root, 0, root_has_id, root_has_path, 0, true, None);
        }
    }

    fn collect_rows_from_node<'a>(
        &'a self,
        rows: &mut Vec<BsonRowEntry<'a>>,
        node: &'a BsonNode,
        depth: usize,
        root_has_id: bool,
        has_path: bool,
        path_len_from_root: usize,
        is_root: bool,
        relation_hint: Option<&'a str>,
    ) {
        let expanded = self.expanded.contains(&node.id);
        let value_edit_enabled =
            root_has_id && path_len_from_root > 0 && is_editable_value(&node.bson);
        let is_root_document = is_root && matches!(node.kind, BsonKind::Document { .. });
        let current_relation_hint = node
            .path_key
            .as_deref()
            .filter(|component| !is_array_index_component(component))
            .or(relation_hint);

        rows.push(BsonRowEntry {
            depth,
            node,
            expanded,
            path_enabled: has_path,
            value_edit_enabled,
            is_root_document,
            relation_hint: current_relation_hint,
        });

        if expanded {
            if let Some(children) = node.children() {
                for child in children {
                    let child_has_path = has_path || child.path_key.is_some();
                    let child_path_len_from_root =
                        path_len_from_root + usize::from(child.path_key.is_some());
                    self.collect_rows_from_node(
                        rows,
                        child,
                        depth + 1,
                        root_has_id,
                        child_has_path,
                        child_path_len_from_root,
                        false,
                        current_relation_hint,
                    );
                }
            }
        }
    }

    pub fn diagnostics_stats(&self) -> BsonTreeStats {
        let mut acc = BsonTreeStatsAccumulator::default();
        for root in &self.roots {
            Self::collect_bson_stats(&root.bson, 0, &mut acc);
        }

        let mut visible_rows = Vec::new();
        self.collect_rows(&mut visible_rows);

        BsonTreeStats {
            root_count: self.roots.len(),
            total_nodes: acc.total_nodes,
            container_nodes: acc.container_nodes,
            leaf_nodes: acc.leaf_nodes,
            max_depth: acc.max_depth,
            expanded_nodes: self.expanded.len(),
            visible_rows: visible_rows.len(),
        }
    }

    fn collect_bson_stats(bson: &Bson, depth: usize, acc: &mut BsonTreeStatsAccumulator) {
        acc.total_nodes += 1;
        if depth > acc.max_depth {
            acc.max_depth = depth;
        }

        match bson {
            Bson::Document(doc) => {
                acc.container_nodes += 1;
                for value in doc.values() {
                    Self::collect_bson_stats(value, depth + 1, acc);
                }
            }
            Bson::Array(items) => {
                acc.container_nodes += 1;
                for value in items {
                    Self::collect_bson_stats(value, depth + 1, acc);
                }
            }
            _ => {
                acc.leaf_nodes += 1;
            }
        }
    }

    fn summarize_id(value: &Bson) -> String {
        match value {
            Bson::Document(_) | Bson::Array(_) => format!("{value:?}"),
            _ => shell::format_bson_scalar(value).0,
        }
    }

    fn is_container(&self, node_id: usize) -> bool {
        Self::find_node(&self.roots, node_id).map(BsonNode::is_container).unwrap_or(false)
    }

    fn node_has_children(&self, node_id: usize) -> bool {
        Self::find_node(&self.roots, node_id).map(BsonNode::has_children).unwrap_or(false)
    }

    fn ensure_children_loaded(&mut self, node_id: usize) {
        let sort_fields_alphabetically = self.sort_fields_alphabetically;
        let next_node_id = &mut self.next_node_id;
        if let Some(node) = Self::find_node_mut(&mut self.roots, node_id) {
            node.materialize_children(next_node_id, sort_fields_alphabetically);
        }
    }

    fn find_node<'a>(nodes: &'a [BsonNode], node_id: usize) -> Option<&'a BsonNode> {
        for node in nodes {
            if node.id == node_id {
                return Some(node);
            }

            if let Some(children) = node.children() {
                if let Some(found) = Self::find_node(children, node_id) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn find_node_mut(nodes: &mut [BsonNode], node_id: usize) -> Option<&mut BsonNode> {
        for node in nodes {
            if node.id == node_id {
                return Some(node);
            }

            if let Some(children) = node.children_mut() {
                if let Some(found) = Self::find_node_mut(children, node_id) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn find_node_path<'a>(
        nodes: &'a [BsonNode],
        node_id: usize,
        stack: &mut Vec<&'a BsonNode>,
    ) -> Option<Vec<&'a BsonNode>> {
        for node in nodes {
            stack.push(node);

            if node.id == node_id {
                return Some(stack.clone());
            }

            if let Some(children) = node.children() {
                if let Some(result) = Self::find_node_path(children, node_id, stack) {
                    return Some(result);
                }
            }

            stack.pop();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::tr;
    use mongodb::bson::{doc, oid::ObjectId};

    fn default_options() -> BsonTreeOptions {
        BsonTreeOptions::new(
            false,
            false,
            TableColors::default(),
            MenuColors::default(),
            RgbaColor::default(),
            ButtonColors::default(),
        )
    }

    fn single_document_tree(doc: Document) -> BsonTree {
        BsonTree::from_values(&[Bson::Document(doc)], default_options())
    }

    fn find_child<'a>(node: &'a BsonNode, key: &str) -> &'a BsonNode {
        match &node.kind {
            BsonKind::Document { children } | BsonKind::Array { children } => children
                .as_ref()
                .unwrap_or_else(|| panic!("children for '{}' are not materialized", key))
                .iter()
                .find(|child| child.display_key.as_deref() == Some(key))
                .unwrap_or_else(|| panic!("child '{}' not found", key)),
            _ => panic!("node has no children"),
        }
    }

    #[test]
    fn placeholder_created_when_no_values() {
        let tree = BsonTree::from_values(&[], default_options());
        assert_eq!(tree.roots.len(), 1);
        let root = &tree.roots[0];

        assert_eq!(root.display_key(), tr("info"));
        assert!(matches!(root.kind, BsonKind::Value { .. }));
        assert_eq!(root.bson, Bson::String(tr("No documents found").to_string()));
    }

    #[test]
    fn document_fields_sorted_when_option_enabled() {
        let document = doc! { "c": 3, "a": 1, "b": 2 };
        let options = BsonTreeOptions::new(
            true,
            false,
            TableColors::default(),
            MenuColors::default(),
            RgbaColor::default(),
            ButtonColors::default(),
        );
        let tree = BsonTree::from_values(&[Bson::Document(document.clone())], options);
        let mut tree = tree;
        let root_id = tree.roots[0].id;
        tree.expand_node(root_id);
        let root = &tree.roots[0];

        let child_keys: Vec<String> = match &root.kind {
            BsonKind::Document { children } => {
                let children = children.as_ref().expect("materialized root children");
                children.iter().map(|node| node.display_key.clone().unwrap()).collect()
            }
            _ => panic!("expected document root"),
        };

        assert_eq!(child_keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn lazy_container_reports_children_before_materialization() {
        let id = ObjectId::new();
        let document = doc! {
            "_id": id,
            "profile": { "age": 30 },
            "empty_doc": {},
            "tags": ["a"],
            "empty_arr": []
        };
        let mut tree = single_document_tree(document);
        let root_id = tree.roots[0].id;

        assert!(tree.roots[0].has_children());
        tree.expand_node(root_id);

        let root = &tree.roots[0];
        let profile = find_child(root, "profile");
        let empty_doc = find_child(root, "empty_doc");
        let tags = find_child(root, "tags");
        let empty_arr = find_child(root, "empty_arr");

        assert!(profile.has_children());
        assert!(!empty_doc.has_children());
        assert!(tags.has_children());
        assert!(!empty_arr.has_children());
    }

    #[test]
    fn distinct_tree_expands_root() {
        let tree = BsonTree::from_distinct(
            String::from("tags"),
            vec![Bson::String(String::from("alpha"))],
            default_options(),
        );

        assert_eq!(tree.roots.len(), 1);
        let root_id = tree.roots[0].id;
        assert!(tree.expanded.contains(&root_id));
        match &tree.roots[0].kind {
            BsonKind::Array { children } => {
                let children = children.as_ref().expect("distinct children should be loaded");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected array root"),
        }
        assert!(!tree.is_indexes_view());
    }

    #[test]
    fn distinct_tree_with_empty_array_is_not_expanded() {
        let tree = BsonTree::from_distinct(String::from("tags"), Vec::new(), default_options());

        assert_eq!(tree.roots.len(), 1);
        let root_id = tree.roots[0].id;
        assert!(!tree.expanded.contains(&root_id));
        match &tree.roots[0].kind {
            BsonKind::Array { children } => {
                assert!(children.is_none());
            }
            _ => panic!("expected array root"),
        }
    }

    #[test]
    fn indexes_tree_exposes_metadata_helpers() {
        let index_doc = doc! { "name": "email_1", "hidden": true };
        let tree = BsonTree::from_indexes(&[Bson::Document(index_doc.clone())], default_options());

        assert!(tree.is_indexes_view());
        let root = &tree.roots[0];
        let root_id = root.id;
        assert_eq!(tree.node_index_name(root_id).as_deref(), Some("email_1"));
        assert_eq!(tree.node_index_hidden(root_id), Some(true));
    }

    #[test]
    fn empty_document_is_not_expanded() {
        let tree = BsonTree::from_document(doc! {}, default_options());

        assert_eq!(tree.roots.len(), 1);
        let root_id = tree.roots[0].id;
        assert!(!tree.expanded.contains(&root_id));
        match &tree.roots[0].kind {
            BsonKind::Document { children } => {
                assert!(children.is_none());
            }
            _ => panic!("expected document root"),
        }
    }

    #[test]
    fn toggle_expands_and_collapses_node() {
        let id = ObjectId::new();
        let tree_doc = doc! { "_id": id, "profile": { "age": 30 } };
        let mut tree = single_document_tree(tree_doc);
        let root_id = tree.roots[0].id;
        tree.expand_node(root_id);

        let root = &tree.roots[0];
        let profile_node = find_child(root, "profile");
        let profile_id = profile_node.id;

        assert!(!tree.expanded.contains(&profile_id));
        tree.toggle(profile_id);
        assert!(tree.expanded.contains(&profile_id));
        tree.toggle(profile_id);
        assert!(!tree.expanded.contains(&profile_id));
    }

    #[test]
    fn expand_node_does_not_expand_empty_container() {
        let id = ObjectId::new();
        let tree_doc = doc! { "_id": id, "empty": {} };
        let mut tree = single_document_tree(tree_doc);
        let root_id = tree.roots[0].id;
        tree.expand_node(root_id);
        let empty_id = {
            let root = &tree.roots[0];
            find_child(root, "empty").id
        };

        tree.expand_node(empty_id);
        assert!(!tree.expanded.contains(&empty_id));
    }

    #[test]
    fn collapse_recursive_removes_descendants() {
        let id = ObjectId::new();
        let tree_doc = doc! { "_id": id, "profile": { "address": { "city": "Paris" } } };
        let mut tree = single_document_tree(tree_doc);
        let root_id = tree.roots[0].id;
        tree.expand_node(root_id);

        let profile_id = {
            let root = &tree.roots[0];
            find_child(root, "profile").id
        };
        tree.expand_node(profile_id);
        let address_id = {
            let root = &tree.roots[0];
            let profile_node = find_child(root, "profile");
            find_child(profile_node, "address").id
        };

        tree.expand_recursive(profile_id);
        assert!(tree.expanded.contains(&profile_id));
        assert!(tree.expanded.contains(&address_id));

        tree.collapse_recursive(profile_id);
        assert!(!tree.expanded.contains(&profile_id));
        assert!(!tree.expanded.contains(&address_id));
    }

    #[test]
    fn node_path_handles_array_indices() {
        let id = ObjectId::new();
        let tree_doc = doc! {
            "_id": id,
            "items": [ { "name": "first" } ]
        };
        let mut tree = single_document_tree(tree_doc);
        let root_id = tree.roots[0].id;
        tree.expand_recursive(root_id);
        let root = &tree.roots[0];
        let items_node = find_child(root, "items");
        let first_entry = match &items_node.kind {
            BsonKind::Array { children } => {
                let children = children.as_ref().expect("materialized array children");
                &children[0]
            }
            _ => panic!("expected array"),
        };
        let name_node = find_child(first_entry, "name");

        assert_eq!(tree.node_path(name_node.id).as_deref(), Some("items.0.name"));
    }

    #[test]
    fn value_edit_context_requires_root_id() {
        let doc_without_id = doc! { "profile": { "age": 30 } };
        let mut tree = single_document_tree(doc_without_id);
        let root_id = tree.roots[0].id;
        tree.expand_recursive(root_id);
        let root = &tree.roots[0];
        let profile_node = find_child(root, "profile");
        let age_node = find_child(profile_node, "age");

        assert!(tree.value_edit_context(age_node.id).is_none());
    }

    #[test]
    fn from_count_labeled_correctly() {
        let tree = BsonTree::from_count(Bson::Int64(10), default_options());
        assert_eq!(tree.roots.len(), 1);
        let root = &tree.roots[0];
        assert_eq!(root.display_key(), tr("count"));
        assert_eq!(root.path_key.as_deref(), Some(tr("count")));
        assert_eq!(tree.node_value_display(root.id), Some("10".to_string()));
    }

    #[test]
    fn value_edit_context_builds_filter_and_path() {
        let id = ObjectId::new();
        let document = doc! {
            "_id": id,
            "profile": { "age": 30, "name": "Alice" },
            "active": true
        };
        let mut tree =
            BsonTree::from_values(&[Bson::Document(document.clone())], default_options());
        let root_id = tree.roots[0].id;
        tree.expand_recursive(root_id);
        let root = &tree.roots[0];

        let profile_node = match &root.kind {
            BsonKind::Document { children } => children
                .as_ref()
                .expect("materialized root children")
                .iter()
                .find(|node| node.display_key.as_deref() == Some("profile"))
                .expect("profile field"),
            _ => panic!("expected document root"),
        };

        let age_node = match &profile_node.kind {
            BsonKind::Document { children } => children
                .as_ref()
                .expect("materialized profile children")
                .iter()
                .find(|node| node.display_key.as_deref() == Some("age"))
                .expect("age field"),
            _ => panic!("expected profile document"),
        };

        let context = tree.value_edit_context(age_node.id).expect("context");
        assert_eq!(context.path, "profile.age");
        assert_eq!(context.filter, doc! { "_id": Bson::ObjectId(id) });
        assert_eq!(context.current_value, Bson::Int32(30));
        assert_eq!(tree.node_path(age_node.id).as_deref(), Some("profile.age"));
    }

    #[test]
    fn related_collection_candidates_cover_common_reference_forms() {
        let user_id_candidates = related_collection_name_candidates("userId");
        assert!(user_id_candidates.contains(&String::from("user")));
        assert!(user_id_candidates.contains(&String::from("users")));

        let user_ids_candidates = related_collection_name_candidates("user_ids");
        assert!(user_ids_candidates.contains(&String::from("user")));
        assert!(user_ids_candidates.contains(&String::from("users")));

        let user_ref_candidates = related_collection_name_candidates("user_ref");
        assert!(user_ref_candidates.contains(&String::from("user")));
        assert!(user_ref_candidates.contains(&String::from("users")));
    }

    #[test]
    fn node_relation_hint_uses_last_non_index_component() {
        let id = ObjectId::new();
        let related = ObjectId::new();
        let document = doc! {
            "_id": id,
            "user_ids": [related],
            "meta": { "authorId": related }
        };
        let mut tree = single_document_tree(document);
        let root_id = tree.roots[0].id;
        tree.expand_recursive(root_id);

        let user_array_item = tree.find_node_id_by_path("user_ids.0").expect("array item");
        let author_id = tree.find_node_id_by_path("meta.authorId").expect("author id");

        assert_eq!(tree.node_relation_hint(user_array_item).as_deref(), Some("user_ids"));
        assert_eq!(tree.node_relation_hint(author_id).as_deref(), Some("authorId"));
    }

    #[test]
    fn supported_reference_id_types_match_mongodb_rules_and_app_policy() {
        assert!(is_supported_reference_id_type(&Bson::Int32(42)));
        assert!(is_supported_reference_id_type(&Bson::String(String::from("42"))));
        assert!(is_supported_reference_id_type(&Bson::ObjectId(ObjectId::new())));
        assert!(is_supported_reference_id_type(&Bson::Document(doc! { "tenant": 1, "code": "A" })));

        assert!(!is_supported_reference_id_type(&Bson::Null));
        assert!(!is_supported_reference_id_type(&Bson::Array(vec![Bson::Int32(1)])));
        assert!(!is_supported_reference_id_type(&Bson::Undefined));
        assert!(!is_supported_reference_id_type(&Bson::RegularExpression(mongodb::bson::Regex {
            pattern: String::from("^abc"),
            options: String::new(),
        },)));
    }
}
