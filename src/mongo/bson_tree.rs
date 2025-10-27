use std::collections::HashSet;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::Wrapping;
use iced::widget::{self, Button, Column, Container, Row, Scrollable, Space};
use iced::{Color, Element, Length};
use iced_aw::ContextMenu;
use mongodb::bson::{Bson, Document};

use crate::fonts;
use crate::i18n::tr;
use crate::mongo::shell;
use crate::settings::{AppSettings, ButtonColors, MenuColors, RgbaColor, TableColors, ThemePalette};
use crate::{Message, TabId, TableContextAction, ValueEditContext};

#[derive(Debug)]
pub struct BsonTree {
    roots: Vec<BsonNode>,
    expanded: HashSet<usize>,
    context: BsonTreeContext,
    table_colors: TableColors,
    menu_colors: MenuColors,
    text_color: RgbaColor,
    button_colors: ButtonColors,
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

struct BsonRowEntry<'a> {
    depth: usize,
    node: &'a BsonNode,
    expanded: bool,
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
    Document(Vec<BsonNode>),
    Array(Vec<BsonNode>),
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
    fn from_bson(
        display_key: Option<String>,
        path_key: Option<String>,
        value: &Bson,
        id: &mut IdGenerator,
        options: &BsonTreeOptions,
    ) -> Self {
        let id_value = id.next();
        match value {
            Bson::Document(map) => {
                let mut entries: Vec<_> = map.iter().collect();
                if options.sort_fields_alphabetically {
                    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
                }
                let children = entries
                    .into_iter()
                    .map(|(k, v)| {
                        BsonNode::from_bson(Some(k.clone()), Some(k.clone()), v, id, options)
                    })
                    .collect();
                Self {
                    id: id_value,
                    display_key,
                    path_key,
                    kind: BsonKind::Document(children),
                    bson: value.clone(),
                }
            }
            Bson::Array(items) => {
                let children = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let display = format!("[{index}]");
                        BsonNode::from_bson(
                            Some(display),
                            Some(index.to_string()),
                            item,
                            id,
                            options,
                        )
                    })
                    .collect();
                Self {
                    id: id_value,
                    display_key,
                    path_key,
                    kind: BsonKind::Array(children),
                    bson: value.clone(),
                }
            }
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
        matches!(self.kind, BsonKind::Document(_) | BsonKind::Array(_))
    }

    fn children(&self) -> Option<&[BsonNode]> {
        match &self.kind {
            BsonKind::Document(children) | BsonKind::Array(children) => Some(children),
            _ => None,
        }
    }

    fn display_key(&self) -> String {
        self.display_key.clone().unwrap_or_else(|| String::from(tr("value")))
    }

    fn value_display(&self) -> Option<String> {
        match &self.kind {
            BsonKind::Document(children) => Some(format!("Document ({} fields)", children.len())),
            BsonKind::Array(children) => Some(format!("Array ({} items)", children.len())),
            BsonKind::Value { display, .. } => Some(display.clone()),
        }
    }

    fn type_label(&self) -> String {
        match &self.kind {
            BsonKind::Document(_) => String::from(tr("Document")),
            BsonKind::Array(_) => String::from(tr("Array")),
            BsonKind::Value { ty, .. } => ty.clone(),
        }
    }
}

fn is_editable_scalar(_value: &Bson) -> bool {
    true
}

impl BsonTree {
    pub fn from_values(values: &[Bson], options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let mut roots = Vec::new();

        if values.is_empty() {
            let info_value = Bson::String(String::from(tr("No documents found")));
            let placeholder = BsonNode::from_bson(
                Some(String::from(tr("info"))),
                None,
                &info_value,
                &mut id_gen,
                &options,
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
                roots.push(BsonNode::from_bson(Some(key), None, value, &mut id_gen, &options));
            }
        }

        let expanded = HashSet::new();

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Default,
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
        let node =
            BsonNode::from_bson(Some(field), Some(path_key), &array_bson, &mut id_gen, &options);
        let mut expanded = HashSet::new();
        expanded.insert(node.id);

        Self {
            roots: vec![node],
            expanded,
            context: BsonTreeContext::Default,
            table_colors: options.table_colors.clone(),
            menu_colors: options.menu_colors.clone(),
            text_color: options.text_color,
            button_colors: options.button_colors.clone(),
        }
    }

    pub fn from_count(value: Bson, options: BsonTreeOptions) -> Self {
        let mut id_gen = IdGenerator::default();
        let node = BsonNode::from_bson(
            Some(String::from(tr("count"))),
            Some(String::from(tr("count"))),
            &value,
            &mut id_gen,
            &options,
        );
        let mut expanded = HashSet::new();
        expanded.insert(node.id);
        Self {
            roots: vec![node],
            expanded,
            context: BsonTreeContext::Default,
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

        let node = BsonNode::from_bson(Some(key), None, &value, &mut id_gen, &options);
        expanded.insert(node.id);
        roots.push(node);

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Default,
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
                    roots.push(BsonNode::from_bson(
                        Some(display),
                        None,
                        value,
                        &mut id_gen,
                        &options,
                    ));
                }
                other => {
                    roots.push(BsonNode::from_bson(
                        Some(base_label.clone()),
                        None,
                        other,
                        &mut id_gen,
                        &options,
                    ));
                }
            }
        }

        let expanded = HashSet::new();

        Self {
            roots,
            expanded,
            context: BsonTreeContext::Indexes,
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

    pub fn view(&self, tab_id: TabId) -> Element<Message> {
        let mut rows = Vec::new();
        self.collect_rows(&mut rows, &self.roots, 0);

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
                Container::new(Space::with_width(Length::Fixed(1.0)))
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
                Container::new(Space::with_width(Length::Fixed(1.0)))
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

        for (index, BsonRowEntry { depth, node, expanded }) in rows.into_iter().enumerate() {
            let background = if index % 2 == 0 { row_color_a } else { row_color_b };

            let mut key_row = Row::new().spacing(6).align_y(Vertical::Center);
            key_row = key_row.push(Space::with_width(Length::Fixed((depth as f32) * 16.0)));

            if node.is_container() {
                let indicator = if expanded { "▼" } else { "▶" };
                let has_children =
                    node.children().map(|children| !children.is_empty()).unwrap_or(false);

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
                key_row = key_row.push(Space::with_width(Length::Fixed(18.0)));
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
                Container::new(Space::with_width(Length::Fixed(1.0)))
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
            let path_enabled = self.node_path(menu_node_id).is_some();
            let is_root_document = depth == 0 && matches!(node.kind, BsonKind::Document(_));
            let value_edit_enabled = self.can_edit_value(menu_node_id);
            let index_context = if self.is_indexes_view() && is_root_document {
                let maybe_name = self.node_index_name(menu_node_id);
                let maybe_hidden =
                    maybe_name.as_ref().and_then(|_| self.node_index_hidden(menu_node_id));
                let ttl_enabled = self
                    .node_bson(menu_node_id)
                    .and_then(|bson| match bson {
                        Bson::Document(doc) => {
                            if doc.contains_key("expireAfterSeconds") {
                                Some(true)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or(false);
                maybe_name.map(|name| (name, maybe_hidden, ttl_enabled))
            } else {
                None
            };

            let row_container = Container::new(row_content).width(Length::Fill).style(move |_| {
                widget::container::Style {
                    background: Some(background.into()),
                    ..Default::default()
                }
            });

            let menu_colors = self.menu_colors.clone();
            let menu_border = self.table_colors.separator.to_color();

            let row_with_menu = TableContextMenu::new(row_container, move || {
                let mut menu = Column::new().spacing(4).padding([4, 6]);

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

                    menu = menu.push(expand_button);
                    menu = menu.push(collapse_button);
                }

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

                menu = menu.push(copy_json);
                menu = menu.push(copy_key);
                menu = menu.push(copy_value);
                menu = menu.push(copy_path);
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
                    menu = menu.push(edit_value);
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
                    menu = menu.push(delete_button);

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
                    menu = menu.push(hide_button);

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
                    menu = menu.push(unhide_button);

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
                        menu = menu.push(edit_button);
                    } else {
                        let edit_button =
                            Button::new(fonts::primary_text(tr("Edit Index..."), None))
                                .padding([4, 12])
                                .width(Length::Shrink);
                        let edit_button = style_menu_button(edit_button, &menu_colors, menu_border);
                        menu = menu.push(edit_button);
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
                    menu = menu.push(edit_button);
                }

                menu.into()
            });

            body = body.push(row_with_menu);
        }

        let body_scroll = Scrollable::new(body).width(Length::Fill).height(Length::Fill);

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
        } else if self.is_container(node_id) {
            self.expanded.insert(node_id);
        }
    }

    pub fn expand_recursive(&mut self, node_id: usize) {
        if !self.is_container(node_id) {
            return;
        }
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

    pub fn is_root_node(&self, node_id: usize) -> bool {
        self.roots.iter().any(|node| node.id == node_id)
    }

    pub fn first_root_id(&self) -> Option<usize> {
        self.roots.first().map(|node| node.id)
    }

    pub fn expand_node(&mut self, node_id: usize) {
        if self.is_container(node_id) {
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

    fn can_edit_value(&self, node_id: usize) -> bool {
        self.edit_requirements(node_id).is_some()
    }

    fn edit_requirements(&self, node_id: usize) -> Option<(Vec<String>, &Document, &BsonNode)> {
        let nodes = Self::find_node_path(&self.roots, node_id, &mut Vec::new())?;
        let target = nodes.last()?;

        if !matches!(target.kind, BsonKind::Value { .. }) {
            return None;
        }

        if !is_editable_scalar(&target.bson) {
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

    fn collect_rows<'a>(
        &'a self,
        rows: &mut Vec<BsonRowEntry<'a>>,
        nodes: &'a [BsonNode],
        depth: usize,
    ) {
        for node in nodes {
            let expanded = self.expanded.contains(&node.id);
            rows.push(BsonRowEntry { depth, node, expanded });
            if expanded {
                if let Some(children) = node.children() {
                    self.collect_rows(rows, children, depth + 1);
                }
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
