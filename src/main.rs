use iced::alignment::{Horizontal, Vertical};
use iced::border;
use iced::widget::pane_grid::ResizeEvent;
use iced::widget::text_editor::{self, Action as TextEditorAction, Content as TextEditorContent};
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Space, Text, button, container, pane_grid,
    text, text_input,
};
use iced::widget::scrollable;
use iced::window;
use iced::{Color, Element, Length, Renderer, Subscription, Task, Theme, application};
use iced_aw::menu::{Item as MenuItemWidget, Menu, MenuBar};
use mongodb::bson::{self, Bson, Document};
use mongodb::sync::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DEFAULT_URI: &str = "mongodb://localhost:27017";
type TabId = u32;
type ClientId = u32;

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);
const DEFAULT_RESULT_LIMIT: i64 = 50;
const DEFAULT_RESULT_SKIP: u64 = 0;
const ICON_NETWORK: &str = "icons/network_115x128.png";
const ICON_DATABASE: &str = "icons/database_105x128.png";
const ICON_COLLECTION: &str = "icons/collection_108x128.png";


fn main() -> iced::Result {
    let icon = window::icon::from_file("icons/oxide_mongo_256x256.png")
        .map_err(|error| iced::Error::WindowCreationFailed(Box::new(error)))?;

    let mut window_settings = window::Settings::default();
    window_settings.icon = Some(icon);
    window_settings.size.width += 240.0;

    application("Oxide Mongo GUI", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(window_settings)
        .run_with(App::init)
}

struct App {
    panes: pane_grid::State<PaneContent>,
    tabs: Vec<TabData>,
    active_tab: Option<TabId>,
    next_tab_id: TabId,
    last_tool: Option<Tool>,
    clients: Vec<OMDBClient>,
    next_client_id: ClientId,
    last_collection_click: Option<CollectionClick>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneContent {
    Sidebar,
    Main,
}

#[derive(Debug)]
struct TabData {
    id: TabId,
    title: String,
    content: TabKind,
}

#[derive(Debug)]
enum TabKind {
    TextEditor { text: String },
    Form { input: String, clicks: u32 },
    Collection(CollectionTab),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Tool1,
    Tool2,
    Tool3,
}

#[derive(Debug, Clone)]
enum Message {
    MenuItemSelected(TopMenu, MenuEntry),
    ToolSelected(Tool),
    TabSelected(TabId),
    TabClosed(TabId),
    TextTabChanged(TabId, String),
    FormTextChanged(TabId, String),
    FormButtonPressed(TabId),
    PaneResized(ResizeEvent),
    ConnectionCompleted { client_id: ClientId, result: Result<ConnectionBootstrap, String> },
    ToggleClient(ClientId),
    ToggleDatabase { client_id: ClientId, db_name: String },
    CollectionsLoaded { client_id: ClientId, db_name: String, result: Result<Vec<String>, String> },
    CollectionClicked { client_id: ClientId, db_name: String, collection: String },
    CollectionEditorAction { tab_id: TabId, action: TextEditorAction },
    CollectionSend(TabId),
    CollectionTreeToggle { tab_id: TabId, node_id: usize },
    CollectionSkipChanged { tab_id: TabId, value: String },
    CollectionLimitChanged { tab_id: TabId, value: String },
    CollectionSkipPrev(TabId),
    CollectionSkipNext(TabId),
    CollectionQueryCompleted {
        tab_id: TabId,
        result: Result<Vec<Bson>, String>,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopMenu {
    File,
    View,
    Options,
    Windows,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuEntry {
    Action(&'static str),
}

#[derive(Debug, Clone)]
enum OMDBConnection {
    Uri(String),
}

#[derive(Debug, Clone)]
struct OMDBClient {
    id: ClientId,
    name: String,
    connection: OMDBConnection,
    status: ConnectionStatus,
    expanded: bool,
    handle: Option<Arc<Client>>,
    databases: Vec<DatabaseNode>,
}

#[derive(Debug, Clone)]
enum ConnectionStatus {
    Connecting,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone)]
struct DatabaseNode {
    name: String,
    expanded: bool,
    state: DatabaseState,
    collections: Vec<CollectionNode>,
}

#[derive(Debug, Clone)]
enum DatabaseState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Debug, Clone)]
struct CollectionNode {
    name: String,
}

#[derive(Debug, Clone)]
struct ConnectionBootstrap {
    handle: Arc<Client>,
    databases: Vec<String>,
}

struct CollectionClick {
    client_id: ClientId,
    db_name: String,
    collection: String,
    at: Instant,
}

#[derive(Debug)]
struct CollectionTab {
    client_id: ClientId,
    client_name: String,
    db_name: String,
    collection: String,
    editor: TextEditorContent,
    panes: pane_grid::State<CollectionPane>,
    bson_tree: BsonTree,
    skip_input: String,
    limit_input: String,
    last_query_duration: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectionPane {
    Request,
    Response,
}

#[derive(Debug)]
struct BsonTree {
    roots: Vec<BsonNode>,
    expanded: HashSet<usize>,
}

struct BsonRowEntry<'a> {
    depth: usize,
    node: &'a BsonNode,
    expanded: bool,
}

#[derive(Debug, Clone)]
struct BsonNode {
    id: usize,
    key: Option<String>,
    kind: BsonKind,
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
    fn from_bson(key: Option<String>, value: &Bson, id: &mut IdGenerator) -> Self {
        let id_value = id.next();
        match value {
            Bson::Document(map) => {
                let children =
                    map.iter().map(|(k, v)| BsonNode::from_bson(Some(k.clone()), v, id)).collect();
                Self { id: id_value, key, kind: BsonKind::Document(children) }
            }
            Bson::Array(items) => {
                let children = items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| BsonNode::from_bson(Some(format!("[{index}]")), item, id))
                    .collect();
                Self { id: id_value, key, kind: BsonKind::Array(children) }
            }
            other => {
                let (display, ty) = format_bson_scalar(other);
                Self { id: id_value, key, kind: BsonKind::Value { display, ty } }
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
        self.key.clone().unwrap_or_else(|| String::from("value"))
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
            BsonKind::Document(_) => String::from("Document"),
            BsonKind::Array(_) => String::from("Array"),
            BsonKind::Value { ty, .. } => ty.clone(),
        }
    }
}

fn format_bson_scalar(value: &Bson) -> (String, String) {
    match value {
        Bson::String(s) => (s.clone(), String::from("String")),
        Bson::Boolean(b) => (b.to_string(), String::from("Boolean")),
        Bson::Int32(i) => (i.to_string(), String::from("Int32")),
        Bson::Int64(i) => (i.to_string(), String::from("Int64")),
        Bson::Double(f) => {
            if f.is_finite() {
                (format!("{f}"), String::from("Double"))
            } else {
                (format!("Double({f})"), String::from("Double"))
            }
        }
        Bson::Decimal128(d) => (format!("Decimal128(\"{}\")", d), String::from("Decimal128")),
        Bson::DateTime(dt) => match dt.try_to_rfc3339_string() {
            Ok(iso) => (iso, String::from("DateTime")),
            Err(_) => (format!("DateTime({})", dt.timestamp_millis()), String::from("DateTime")),
        },
        Bson::ObjectId(oid) => (format!("ObjectId(\"{}\")", oid), String::from("ObjectId")),
        Bson::Binary(bin) => (
            format!("Binary(len={}, subtype={:?})", bin.bytes.len(), bin.subtype),
            String::from("Binary"),
        ),
        Bson::Symbol(sym) => (format!("Symbol({sym:?})"), String::from("Symbol")),
        Bson::RegularExpression(regex) => {
            if regex.options.is_empty() {
                (format!("Regex({:?})", regex.pattern), String::from("Regex"))
            } else {
                (format!("Regex({:?}, {:?})", regex.pattern, regex.options), String::from("Regex"))
            }
        }
        Bson::Timestamp(ts) => (
            format!("Timestamp(time={}, increment={})", ts.time, ts.increment),
            String::from("Timestamp"),
        ),
        Bson::JavaScriptCode(code) => (format!("Code({code:?})"), String::from("JavaScriptCode")),
        Bson::JavaScriptCodeWithScope(code_with_scope) => {
            let scope_len = code_with_scope.scope.len();
            (
                format!("CodeWithScope({:?}, scope_fields={})", code_with_scope.code, scope_len),
                String::from("JavaScriptCodeWithScope"),
            )
        }
        Bson::DbPointer(ptr) => (format!("DbPointer({ptr:?})"), String::from("DbPointer")),
        Bson::Undefined => (String::from("undefined"), String::from("Undefined")),
        Bson::Null => (String::from("null"), String::from("Null")),
        Bson::MinKey => (String::from("MinKey"), String::from("MinKey")),
        Bson::MaxKey => (String::from("MaxKey"), String::from("MaxKey")),
        Bson::Document(_) | Bson::Array(_) => unreachable!("containers handled separately"),
    }
}

impl BsonTree {
    fn from_values(values: &[Bson]) -> Self {
        let mut id_gen = IdGenerator::default();
        let mut roots = Vec::new();

        if values.is_empty() {
            let info_value = Bson::String("Документы не найдены".into());
            let placeholder =
                BsonNode::from_bson(Some(String::from("info")), &info_value, &mut id_gen);
            roots.push(placeholder);
        } else {
            for (index, value) in values.iter().enumerate() {
                let key = match value {
                    Bson::Document(doc) => doc
                        .get("_id")
                        .map(Self::summarize_id)
                        .unwrap_or_else(|| format!("doc[{index}]")),
                    _ => format!("doc[{index}]"),
                };
                roots.push(BsonNode::from_bson(Some(key), value, &mut id_gen));
            }
        }

        let expanded = HashSet::new();

        Self { roots, expanded }
    }

    fn from_error(message: String) -> Self {
        let value = Bson::String(message);
        Self::from_values(std::slice::from_ref(&value))
    }

    fn view(&self, tab_id: TabId) -> Element<Message> {
        let mut rows = Vec::new();
        self.collect_rows(&mut rows, &self.roots, 0);

        let row_color_a = Color::from_rgb8(0xfe, 0xfe, 0xfe);
        let row_color_b = Color::from_rgb8(0xf9, 0xfd, 0xf9);
        let header_bg = Color::from_rgb8(0xef, 0xf1, 0xf5);
        let separator_color = Color::from_rgb8(0xd0, 0xd4, 0xda);

        let header_row = Row::new()
            .spacing(0)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Shrink)
            .push(
                Container::new(Text::new("Key").size(14))
                    .width(Length::FillPortion(4))
                    .padding([6, 8])
            )
            .push(
                Container::new(Space::with_width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .padding([6, 0])
                    .style(move |_| iced::widget::container::Style {
                        background: Some(separator_color.into()),
                        ..Default::default()
                    }),
            )
            .push(
                Container::new(Text::new("Value").size(14))
                    .width(Length::FillPortion(5))
                    .padding([6, 8]),
            )
            .push(
                Container::new(Space::with_width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .padding([6, 0])
                    .style(move |_| iced::widget::container::Style {
                        background: Some(separator_color.into()),
                        ..Default::default()
                    }),
            )
            .push(
                Container::new(Text::new("Type").size(14))
                    .width(Length::FillPortion(3))
                    .padding([6, 8]),
            );

        let header = Container::new(header_row).width(Length::Fill).height(Length::Shrink).style(move |_| {
            iced::widget::container::Style {
                background: Some(header_bg.into()),
                ..Default::default()
            }
        });

        let mut body = Column::new().spacing(1).width(Length::Fill).height(Length::Shrink);

        for (index, BsonRowEntry { depth, node, expanded }) in rows.into_iter().enumerate() {
            let background = if index % 2 == 0 { row_color_a } else { row_color_b };

            let mut key_row = Row::new().spacing(6).align_y(Vertical::Center);
            key_row = key_row.push(Space::with_width(Length::Fixed((depth as f32) * 16.0)));

            if node.is_container() {
                let indicator = if expanded { "▼" } else { "▶" };
                let toggle = Button::new(Text::new(indicator))
                    .padding([0, 4])
                    .on_press(Message::CollectionTreeToggle { tab_id, node_id: node.id });
                key_row = key_row.push(toggle);
            } else {
                key_row = key_row.push(Space::with_width(Length::Fixed(18.0)));
            }

            let key_label = node.display_key();
            key_row = key_row.push(Text::new(key_label.clone()).size(14));

            let value_text = node.value_display().unwrap_or_default();
            let type_text = node.type_label();

            let key_cell = Container::new(key_row).width(Length::FillPortion(4)).padding([6, 8]);

            let value_cell = Container::new(Text::new(value_text.clone()).size(14))
                .width(Length::FillPortion(5))
                .padding([6, 8]);

            let type_cell = Container::new(Text::new(type_text.clone()).size(14))
                .width(Length::FillPortion(3))
                .padding([6, 8]);

            let separator = |color: Color| {
                Container::new(Space::with_width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Shrink)
                    .style(move |_| iced::widget::container::Style {
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

            let row = Container::new(row_content).width(Length::Fill).style(move |_| {
                iced::widget::container::Style {
                    background: Some(background.into()),
                    ..Default::default()
                }
            });

            body = body.push(row);
        }

        let c_body = Container::new(body).width(Length::Fill).height(Length::Shrink).style(move |_| {
            iced::widget::container::Style {
                background: Some(header_bg.into()),
                ..Default::default()
            }
        });


        let scrollable_body = Scrollable::new(c_body)
                .width(Length::Fill);

        Column::new()
            .spacing(2)
            .height(Length::Shrink)
            .push(header)
            .push(scrollable_body)
            .into()

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
            if node.is_container() && expanded {
                if let Some(children) = node.children() {
                    self.collect_rows(rows, children, depth + 1);
                }
            }
        }
    }

    fn toggle(&mut self, node_id: usize) {
        if self.expanded.contains(&node_id) {
            self.expanded.remove(&node_id);
        } else if self.is_container(node_id) {
            self.expanded.insert(node_id);
        }
    }

    fn summarize_id(value: &Bson) -> String {
        match value {
            Bson::Document(_) | Bson::Array(_) => format!("{value:?}"),
            _ => format_bson_scalar(value).0,
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
}

impl CollectionTab {
    fn new(
        client_id: ClientId,
        client_name: String,
        db_name: String,
        collection: String,
        values: Vec<Bson>,
    ) -> Self {
        let (mut panes, top) = pane_grid::State::new(CollectionPane::Request);
        let (_, split) = panes
            .split(pane_grid::Axis::Horizontal, top, CollectionPane::Response)
            .expect("failed to split collection tab panes");
        panes.resize(split, 0.25);

        let bson_tree = BsonTree::from_values(&values);
        let editor_text = format!(
            "db.getCollection('{collection_name}').find({{}})",
            collection_name = collection.as_str()
        );

        Self {
            client_id,
            client_name,
            db_name,
            collection,
            editor: TextEditorContent::with_text(&editor_text),
            panes,
            bson_tree,
            skip_input: DEFAULT_RESULT_SKIP.to_string(),
            limit_input: DEFAULT_RESULT_LIMIT.to_string(),
            last_query_duration: None,
        }
    }

    fn view(&self, tab_id: TabId) -> Element<Message> {
        let skip_tab_id = tab_id;
        let limit_tab_id = tab_id;
        let skip_prev_tab_id = tab_id;
        let skip_next_tab_id = tab_id;

        let duration_text = self
            .last_query_duration
            .map(Self::format_duration)
            .unwrap_or_else(|| String::from("—"));

        let icon_size = 18.0;

        let skip_input = text_input("skip", &self.skip_input)
            .padding([4, 6])
            .align_x(Horizontal::Center)
            .on_input(move |value| Message::CollectionSkipChanged { tab_id: skip_tab_id, value })
            .width(Length::Fixed(52.0));

        let limit_input = text_input("limit", &self.limit_input)
            .padding([4, 6])
            .align_x(Horizontal::Center)
            .on_input(move |value| Message::CollectionLimitChanged { tab_id: limit_tab_id, value })
            .width(Length::Fixed(52.0));

        let skip_prev = Button::new(Text::new("◀").size(16))
            .on_press(Message::CollectionSkipPrev(skip_prev_tab_id))
            .padding([2, 6]);

        let skip_next = Button::new(Text::new("▶").size(16))
            .on_press(Message::CollectionSkipNext(skip_next_tab_id))
            .padding([2, 6]);

        let navigation = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(skip_prev)
            .push(skip_input)
            .push(limit_input)
            .push(skip_next);

        let connection_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(ICON_NETWORK)
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(Text::new(self.client_name.clone()).size(14));

        let database_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(ICON_DATABASE)
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(Text::new(self.db_name.clone()).size(14));

        let collection_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(ICON_COLLECTION)
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(Text::new(self.collection.clone()).size(14));

        let info_labels = Row::new()
            .spacing(12)
            .align_y(Vertical::Center)
            .push(connection_label)
            .push(database_label)
            .push(collection_label)
            .push(Text::new(format!("Время: {}", duration_text)).size(14));

        let info_row = Row::new()
            .spacing(16)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(
                Container::new(info_labels)
                    .width(Length::Fill)
                    .padding([0, 4]),
            )
            .push(navigation);

        let panel_bg = Color::from_rgb8(0xef, 0xf1, 0xf5);
        let panel_border = Color::from_rgb8(0xd0, 0xd4, 0xda);

        let info_panel = Container::new(info_row)
            .width(Length::Fill)
            .padding([8, 12])
            .style(move |_| iced::widget::container::Style {
                background: Some(panel_bg.into()),
                border: border::rounded(6).width(1).color(panel_border),
                ..Default::default()
            });

        let panes = pane_grid::PaneGrid::new(&self.panes, |_, pane, _| match pane {
            CollectionPane::Request => pane_grid::Content::new(self.request_view(tab_id)),
            CollectionPane::Response => pane_grid::Content::new(self.response_view(tab_id)),
        })
        .spacing(8)
        .height(Length::Fill);

        Column::new()
            .spacing(8)
            .push(info_panel)
            .push(panes)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn request_view(&self, tab_id: TabId) -> Element<Message> {
        let editor_height = 4.0 * 24.0;

        let editor = text_editor::TextEditor::new(&self.editor)
            .on_action(move |action| Message::CollectionEditorAction { tab_id, action })
            .height(Length::Fixed(editor_height));

        let send_button = Button::new(Text::new("Send"))
            .on_press(Message::CollectionSend(tab_id))
            .padding([4, 12]);

        let controls_row = Row::new()
            .spacing(0)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(
                Container::new(editor)
                    .width(Length::FillPortion(9))
                    .height(Length::Fixed(editor_height)),
            )
            .push(
                Container::new(send_button)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(editor_height))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            );

        Column::new()
            .spacing(8)
            .push(controls_row)
            .push(Space::with_height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn response_view(&self, tab_id: TabId) -> Element<Message> {
        self.bson_tree.view(tab_id)
    }

    fn toggle_node(&mut self, node_id: usize) {
        self.bson_tree.toggle(node_id);
    }

    fn update_skip(&mut self, value: String) {
        self.skip_input = Self::sanitize_numeric(value);
    }

    fn update_limit(&mut self, value: String) {
        self.limit_input = Self::sanitize_numeric(value);
    }

    fn decrement_skip_by_limit(&mut self) {
        let limit = self.parse_limit_u64();
        if limit == 0 {
            return;
        }

        let skip = self.parse_skip_u64();
        let new_skip = skip.saturating_sub(limit);
        self.skip_input = Self::format_numeric(new_skip);
    }

    fn increment_skip_by_limit(&mut self) {
        let limit = self.parse_limit_u64();
        if limit == 0 {
            return;
        }

        let skip = self.parse_skip_u64();
        let new_skip = skip.saturating_add(limit);
        self.skip_input = Self::format_numeric(new_skip);
    }

    fn skip_value(&self) -> u64 {
        self.parse_skip_u64()
    }

    fn limit_value(&self) -> u64 {
        self.parse_limit_u64()
    }

    fn parse_filter(&self, text: &str) -> Result<Document, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(Document::new());
        }

        let candidate = if let Some(argument) = Self::extract_find_argument(trimmed) {
            argument
        } else {
            trimmed.to_string()
        };

        let cleaned = candidate.trim().trim_end_matches(';').trim();

        if cleaned.is_empty() {
            return Ok(Document::new());
        }

        let value: Value = serde_json::from_str(cleaned)
            .map_err(|error| format!("JSON parse error: {error}"))?;

        if !value.is_object() {
            return Err(String::from("Запрос find должен быть JSON-объектом"));
        }

        bson::to_document(&value).map_err(|error| format!("BSON conversion error: {error}"))
    }

    fn extract_find_argument(text: &str) -> Option<String> {
        const MARKER: &str = ".find(";
        let start = text.find(MARKER)? + MARKER.len();
        let mut depth = 0u32;
        let mut end_index = None;

        for (offset, ch) in text[start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        end_index = Some(start + offset);
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        let end = end_index?;
        Some(text[start..end].to_string())
    }

    fn sanitize_numeric<S: AsRef<str>>(value: S) -> String {
        let filtered: String = value
            .as_ref()
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect();
        let trimmed = filtered.trim_start_matches('0');
        if trimmed.is_empty() { String::from("0") } else { trimmed.to_string() }
    }

    fn parse_skip_u64(&self) -> u64 {
        self.skip_input.parse().unwrap_or(DEFAULT_RESULT_SKIP)
    }

    fn parse_limit_u64(&self) -> u64 {
        self.limit_input.parse().unwrap_or(DEFAULT_RESULT_LIMIT as u64)
    }

    fn format_numeric(value: u64) -> String {
        value.to_string()
    }

    fn format_duration(duration: Duration) -> String {
        if duration < Duration::from_secs(60) {
            format!("{:.3}", duration.as_secs_f64())
        } else {
            let total_seconds = duration.as_secs();
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            let tenths = (duration.subsec_millis() / 100) % 10;
            format!("{}:{:02}.{}", minutes, seconds, tenths)
        }
    }

    fn set_tree_from_bson(&mut self, values: Vec<Bson>) {
        self.bson_tree = BsonTree::from_values(&values);
    }

    fn set_tree_error(&mut self, error: String) {
        self.bson_tree = BsonTree::from_error(error);
    }
}

impl Default for App {
    fn default() -> Self {
        let (mut panes, sidebar) = pane_grid::State::new(PaneContent::Sidebar);
        let (_content_pane, split) = panes
            .split(pane_grid::Axis::Vertical, sidebar, PaneContent::Main)
            .expect("failed to split pane grid");
        panes.resize(split, 0.25);

        Self {
            panes,
            tabs: Vec::new(),
            active_tab: None,
            next_tab_id: 1,
            last_tool: None,
            clients: Vec::new(),
            next_client_id: 1,
            last_collection_click: None,
        }
    }
}

impl App {
    fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MenuItemSelected(menu, entry) => {
                println!("Menu '{menu:?}' entry '{entry:?}' clicked");
                Task::none()
            }
            Message::ToolSelected(tool) => self.handle_tool(tool),
            Message::TabSelected(id) => {
                if self.tabs.iter().any(|tab| tab.id == id) {
                    self.active_tab = Some(id);
                }
                Task::none()
            }
            Message::TabClosed(id) => {
                if let Some(position) = self.tabs.iter().position(|tab| tab.id == id) {
                    self.tabs.remove(position);
                    if self.active_tab == Some(id) {
                        self.active_tab = self
                            .tabs
                            .get(position.saturating_sub(1))
                            .or_else(|| self.tabs.get(position))
                            .map(|tab| tab.id);
                    }
                }
                Task::none()
            }
            Message::TextTabChanged(id, value) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    if let TabKind::TextEditor { text } = &mut tab.content {
                        *text = value;
                    }
                }
                Task::none()
            }
            Message::FormTextChanged(id, value) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    if let TabKind::Form { input, .. } = &mut tab.content {
                        *input = value;
                    }
                }
                Task::none()
            }
            Message::FormButtonPressed(id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    if let TabKind::Form { clicks, .. } = &mut tab.content {
                        *clicks += 1;
                    }
                }
                Task::none()
            }
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
                Task::none()
            }
            Message::ConnectionCompleted { client_id, result } => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    match result {
                        Ok(ConnectionBootstrap { handle, mut databases }) => {
                            databases.sort_unstable();
                            client.status = ConnectionStatus::Ready;
                            client.handle = Some(handle);
                            client.databases =
                                databases.into_iter().map(DatabaseNode::new).collect();
                            client.expanded = true;
                        }
                        Err(error) => {
                            client.status = ConnectionStatus::Failed(error);
                            client.databases.clear();
                        }
                    }
                }
                Task::none()
            }
            Message::ToggleClient(client_id) => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    client.expanded = !client.expanded;
                }
                Task::none()
            }
            Message::ToggleDatabase { client_id, db_name } => {
                let mut request: Option<(Arc<Client>, String)> = None;

                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    if let Some(database) = client.databases.iter_mut().find(|d| d.name == db_name)
                    {
                        database.expanded = !database.expanded;
                        if database.expanded {
                            match &database.state {
                                DatabaseState::Idle | DatabaseState::Error(_) => {
                                    database.state = DatabaseState::Loading;
                                    database.collections.clear();
                                    if let Some(handle) = client.handle.clone() {
                                        request = Some((handle, database.name.clone()));
                                    } else {
                                        database.state = DatabaseState::Error(
                                            "Нет активного соединения".to_owned(),
                                        );
                                    }
                                }
                                DatabaseState::Loading | DatabaseState::Loaded => {}
                            }
                        }
                    }
                }

                if let Some((handle, db_name)) = request {
                    let db_for_task = db_name.clone();
                    let db_for_message = db_name;

                    Task::perform(
                        async move { fetch_collections(handle, db_for_task) },
                        move |result| Message::CollectionsLoaded {
                            client_id,
                            db_name: db_for_message.clone(),
                            result,
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::CollectionsLoaded { client_id, db_name, result } => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    if let Some(database) = client.databases.iter_mut().find(|d| d.name == db_name)
                    {
                        match result {
                            Ok(mut names) => {
                                names.sort_unstable();
                                database.state = DatabaseState::Loaded;
                                database.collections =
                                    names.into_iter().map(CollectionNode::new).collect();
                            }
                            Err(error) => {
                                database.state = DatabaseState::Error(error);
                                database.collections.clear();
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionClicked { client_id, db_name, collection } => {
                let now = Instant::now();
                let is_double = self
                    .last_collection_click
                    .as_ref()
                    .map(|last| {
                        last.client_id == client_id
                            && last.db_name == db_name
                            && last.collection == collection
                            && now.duration_since(last.at) <= DOUBLE_CLICK_INTERVAL
                    })
                    .unwrap_or(false);

                if is_double {
                    self.last_collection_click = None;
                    self.open_collection_tab(client_id, db_name, collection);
                } else {
                    self.last_collection_click =
                        Some(CollectionClick { client_id, db_name, collection, at: now });
                }

                Task::none()
            }
            Message::CollectionEditorAction { tab_id, action } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.editor.perform(action);
                    }
                }
                Task::none()
            }
            Message::CollectionSend(tab_id) => {
                self.collection_query_task(tab_id)
            }
            Message::CollectionSkipChanged { tab_id, value } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.update_skip(value);
                    }
                }
                Task::none()
            }
            Message::CollectionLimitChanged { tab_id, value } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.update_limit(value);
                    }
                }
                Task::none()
            }
            Message::CollectionSkipPrev(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.decrement_skip_by_limit();
                    }
                }
                self.collection_query_task(tab_id)
            }
            Message::CollectionSkipNext(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.increment_skip_by_limit();
                    }
                }
                self.collection_query_task(tab_id)
            }
            Message::CollectionQueryCompleted { tab_id, result, duration } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.last_query_duration = Some(duration);
                        match result {
                            Ok(values) => collection.set_tree_from_bson(values),
                            Err(error) => collection.set_tree_error(error),
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionTreeToggle { tab_id, node_id } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    if let TabKind::Collection(collection) = &mut tab.content {
                        collection.toggle_node(node_id);
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        let menu_bar = self.build_menu_bar();
        let toolbar = self.build_toolbar();

        let content_grid =
            pane_grid::PaneGrid::new(&self.panes, |_, pane_state, _| match pane_state {
                PaneContent::Sidebar => pane_grid::Content::new(self.sidebar_panel()),
                PaneContent::Main => pane_grid::Content::new(self.main_panel()),
            })
            .on_resize(8, Message::PaneResized)
            .spacing(8)
            .height(Length::Fill);

        Column::new()
            .push(menu_bar)
            .push(toolbar)
            .push(content_grid)
            .spacing(0)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }

    fn sidebar_panel(&self) -> Element<Message> {
        let mut list = Column::new().spacing(4);

        if self.clients.is_empty() {
            list = list.push(text("Соединений нет").size(16));
        } else {
            for client in &self.clients {
                list = list.push(self.render_client(client));
            }
        }

        let scrollable = Scrollable::new(list).width(Length::Fill).height(Length::Fill);

        Container::new(scrollable)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Self::pane_style)
            .into()
    }

    fn render_client<'a>(&'a self, client: &'a OMDBClient) -> Element<'a, Message> {
        let indicator = if client.expanded { "v" } else { ">" };
        let status_label = match &client.status {
            ConnectionStatus::Connecting => "Подключение...".to_owned(),
            ConnectionStatus::Ready => "Готово".to_owned(),
            ConnectionStatus::Failed(err) => format!("Ошибка: {err}"),
        };
        let connection_label = client.connection.display_label();

        let header_row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(text(indicator))
            .push(text(&client.name).size(16))
            .push(text(status_label.clone()).size(12));

        let mut column = Column::new()
            .spacing(4)
            .push(Button::new(header_row).padding(4).on_press(Message::ToggleClient(client.id)))
            .push(
                Row::new()
                    .spacing(8)
                    .push(Space::with_width(Length::Fixed(16.0)))
                    .push(text(connection_label).size(12)),
            );

        if matches!(client.status, ConnectionStatus::Failed(_)) {
            column = column.push(
                Row::new()
                    .spacing(8)
                    .push(Space::with_width(Length::Fixed(16.0)))
                    .push(text(status_label).size(12)),
            );
        }

        if client.expanded && matches!(client.status, ConnectionStatus::Ready) {
            if client.databases.is_empty() {
                column = column.push(
                    Row::new()
                        .spacing(8)
                        .push(Space::with_width(Length::Fixed(16.0)))
                        .push(text("Нет баз данных").size(12)),
                );
            } else {
                for database in &client.databases {
                    column = column.push(self.render_database(client.id, database));
                }
            }
        }

        column.into()
    }

    fn render_database<'a>(
        &'a self,
        client_id: ClientId,
        database: &'a DatabaseNode,
    ) -> Element<'a, Message> {
        let indicator = if database.expanded { "v" } else { ">" };

        let db_row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(Space::with_width(Length::Fixed(16.0)))
            .push(text(indicator))
            .push(text(&database.name));

        let mut column = Column::new().spacing(4).push(
            Button::new(db_row)
                .padding(4)
                .on_press(Message::ToggleDatabase { client_id, db_name: database.name.clone() }),
        );

        if database.expanded {
            match &database.state {
                DatabaseState::Idle => {}
                DatabaseState::Loading => {
                    column = column.push(
                        Row::new()
                            .spacing(8)
                            .push(Space::with_width(Length::Fixed(32.0)))
                            .push(text("Загрузка коллекций...").size(12)),
                    );
                }
                DatabaseState::Error(error) => {
                    column = column.push(
                        Row::new()
                            .spacing(8)
                            .push(Space::with_width(Length::Fixed(32.0)))
                            .push(text(format!("Ошибка: {error}")).size(12)),
                    );
                }
                DatabaseState::Loaded => {
                    if database.collections.is_empty() {
                        column = column.push(
                            Row::new()
                                .spacing(8)
                                .push(Space::with_width(Length::Fixed(32.0)))
                                .push(text("Нет коллекций").size(12)),
                        );
                    } else {
                        for collection in &database.collections {
                            column = column.push(self.render_collection(
                                client_id,
                                &database.name,
                                collection,
                            ));
                        }
                    }
                }
            }
        }

        column.into()
    }

    fn render_collection<'a>(
        &'a self,
        client_id: ClientId,
        db_name: &str,
        collection: &'a CollectionNode,
    ) -> Element<'a, Message> {
        let row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(Space::with_width(Length::Fixed(32.0)))
            .push(text(&collection.name).size(12));

        Button::new(row)
            .padding(4)
            .on_press(Message::CollectionClicked {
                client_id,
                db_name: db_name.to_owned(),
                collection: collection.name.clone(),
            })
            .into()
    }

    fn main_panel(&self) -> Element<Message> {
        if self.tabs.is_empty() {
            Container::new(text("Вкладки не открыты"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .padding(16)
                .style(Self::pane_style)
                .into()
        } else {
            let active_id = self
                .active_tab
                .or_else(|| self.tabs.first().map(|tab| tab.id));

            let mut tabs_row = Row::new().spacing(8).align_y(Vertical::Center);

            let active_bg = Color::from_rgb8(0xd5, 0xe4, 0xff);
            let inactive_bg = Color::from_rgb8(0xf2, 0xf4, 0xf8);
            let border_color = Color::from_rgb8(0xc2, 0xc8, 0xd3);

            for tab in &self.tabs {
                let is_active = active_id == Some(tab.id);

                let title_button = Button::new(Text::new(tab.title.clone()).size(14))
                    .padding([4, 12])
                    .on_press(Message::TabSelected(tab.id));

                let close_button = Button::new(Text::new("×").size(14))
                    .padding([4, 8])
                    .on_press(Message::TabClosed(tab.id));

                let tab_inner = Row::new()
                    .spacing(4)
                    .align_y(Vertical::Center)
                    .push(title_button)
                    .push(close_button);

                let tab_container = Container::new(tab_inner)
                    .padding([4, 8])
                    .style(move |_| {
                        if is_active {
                            container::Style {
                                background: Some(active_bg.into()),
                                text_color: Some(Color::BLACK),
                                border: border::rounded(6).width(1).color(border_color),
                                ..Default::default()
                            }
                        } else {
                            container::Style {
                                background: Some(inactive_bg.into()),
                                border: border::rounded(6).width(1).color(border_color),
                                ..Default::default()
                            }
                        }
                    });

                tabs_row = tabs_row.push(tab_container);
            }

            let header_scroll = Scrollable::with_direction(
                tabs_row,
                scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
            )
            .height(Length::Shrink)
            .width(Length::Fill);

            let header = Container::new(header_scroll)
                .width(Length::Fill)
                .padding([0, 4]);

            let content = active_id
                .and_then(|id| self.tabs.iter().find(|tab| tab.id == id))
                .map(|tab| tab.view())
                .unwrap_or_else(|| {
                    Container::new(text("Нет активной вкладки"))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into()
                });

            let layout = Column::new()
                .spacing(8)
                .push(header)
                .push(content)
                .width(Length::Fill)
                .height(Length::Fill);

            Container::new(layout)
                .padding(8)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(Self::pane_style)
                .into()
        }
    }

    fn build_menu_bar(&self) -> MenuBar<'_, Message, Theme, Renderer> {
        MenuBar::new(vec![
            self.menu_root(
                TopMenu::File,
                &[MenuEntry::Action("New"), MenuEntry::Action("Open"), MenuEntry::Action("Save")],
            ),
            self.menu_root(
                TopMenu::View,
                &[MenuEntry::Action("Explorer"), MenuEntry::Action("Refresh")],
            ),
            self.menu_root(
                TopMenu::Options,
                &[MenuEntry::Action("Preferences"), MenuEntry::Action("Settings")],
            ),
            self.menu_root(
                TopMenu::Windows,
                &[MenuEntry::Action("Cascade"), MenuEntry::Action("Tile")],
            ),
            self.menu_root(
                TopMenu::Help,
                &[MenuEntry::Action("Documentation"), MenuEntry::Action("About")],
            ),
        ])
        .width(Length::Fill)
    }

    fn menu_root(
        &self,
        menu: TopMenu,
        entries: &[MenuEntry],
    ) -> MenuItemWidget<'_, Message, Theme, Renderer> {
        let label = text(menu.label()).size(16);
        let root_button = button(label).padding([6, 12]);

        let menu_widget = Menu::new(
            entries
                .iter()
                .map(|entry| {
                    let entry_label = text(entry.label()).size(14);
                    let entry_button = button(entry_label)
                        .on_press(Message::MenuItemSelected(menu, *entry))
                        .padding([6, 12])
                        .width(Length::Fill);
                    MenuItemWidget::new(entry_button)
                })
                .collect(),
        )
        .offset(4.0)
        .max_width(180.0);

        MenuItemWidget::with_menu(root_button, menu_widget)
    }

    fn build_toolbar(&self) -> Row<'_, Message> {
        Row::new()
            .spacing(8)
            .padding([6, 12])
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(self.toolbar_button(Tool::Tool1, "Tool1"))
            .push(self.toolbar_button(Tool::Tool2, "Tool2"))
            .push(self.toolbar_button(Tool::Tool3, "Tool3"))
    }

    fn toolbar_button(&self, tool: Tool, label: &str) -> Element<Message> {
        let mut label_text = label.to_owned();

        if Some(tool) == self.last_tool {
            label_text.push_str(" *");
        }

        Button::new(text(label_text)).on_press(Message::ToolSelected(tool)).padding([6, 16]).into()
    }

    fn pane_style(theme: &Theme) -> iced::widget::container::Style {
        let palette = theme.extended_palette();

        iced::widget::container::Style {
            background: Some(palette.background.weak.color.into()),
            border: border::rounded(6).width(1).color(palette.primary.weak.color),
            ..Default::default()
        }
    }

    fn handle_tool(&mut self, tool: Tool) -> Task<Message> {
        self.last_tool = Some(tool);
        match tool {
            Tool::Tool1 => self.add_default_connection(),
            Tool::Tool2 => {
                self.add_text_tab();
                Task::none()
            }
            Tool::Tool3 => {
                self.add_form_tab();
                Task::none()
            }
        }
    }

    fn add_default_connection(&mut self) -> Task<Message> {
        let connection = OMDBConnection::from_uri(DEFAULT_URI);
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let mut client = OMDBClient::new(client_id, connection.clone());
        client.name = format!("Соединение #{client_id}");
        self.clients.push(client);

        Task::perform(async move { connect_and_discover(connection) }, move |result| {
            Message::ConnectionCompleted { client_id, result }
        })
    }

    fn add_text_tab(&mut self) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(TabData::new_text(id));
        self.active_tab = Some(id);
    }

    fn add_form_tab(&mut self) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(TabData::new_form(id));
        self.active_tab = Some(id);
    }

    fn open_collection_tab(&mut self, client_id: ClientId, db_name: String, collection: String) {
        if let Some(existing) = self.tabs.iter().find(|tab| {
            matches!(
                &tab.content,
                TabKind::Collection(existing)
                    if existing.client_id == client_id
                        && existing.db_name == db_name
                        && existing.collection == collection
            )
        }) {
            self.active_tab = Some(existing.id);
            return;
        }

        let mut client_name = String::from("Неизвестный клиент");
        let mut values = vec![Bson::String(String::from("Нет активного соединения"))];

        if let Some(client) = self.clients.iter().find(|c| c.id == client_id) {
            client_name = client.name.clone();

            if let Some(handle) = client.handle.clone() {
                let database = handle.database(&db_name);
                let collection_handle = database.collection::<Document>(&collection);

                values = match collection_handle
                    .find(Document::new())
                    .skip(DEFAULT_RESULT_SKIP)
                    .limit(DEFAULT_RESULT_LIMIT)
                    .run()
                {
                    Ok(cursor) => cursor
                        .take(DEFAULT_RESULT_LIMIT as usize)
                        .filter_map(|result| result.ok())
                        .map(Bson::Document)
                        .collect::<Vec<_>>(),
                    Err(error) => {
                        vec![Bson::String(format!("Ошибка загрузки документов: {error}"))]
                    }
                };
            }
        }

        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(TabData::new_collection(
            id,
            client_id,
            client_name,
            db_name,
            collection,
            values,
        ));
        self.active_tab = Some(id);
    }

    fn collection_query_task(&mut self, tab_id: TabId) -> Task<Message> {
        let mut request: Option<(ClientId, String, String, Document, u64, u64)> = None;

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            if let TabKind::Collection(collection) = &mut tab.content {
                let query_text = collection.editor.text().to_string();
                match collection.parse_filter(&query_text) {
                    Ok(filter) => {
                        let skip = collection.skip_value();
                        let limit = collection.limit_value();
                        collection.last_query_duration = None;
                        request = Some((
                            collection.client_id,
                            collection.db_name.clone(),
                            collection.collection.clone(),
                            filter,
                            skip,
                            limit,
                        ));
                    }
                    Err(error) => {
                        collection.set_tree_error(error);
                    }
                }
            }
        }

        let Some((client_id, db_name, collection_name, filter, skip, limit)) = request else {
            return Task::none();
        };

        let Some(handle) = self
            .clients
            .iter()
            .find(|client| client.id == client_id)
            .and_then(|client| client.handle.clone())
        else {
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                if let TabKind::Collection(collection) = &mut tab.content {
                    collection.set_tree_error(String::from("Нет активного соединения"));
                }
            }
            return Task::none();
        };

        Task::perform(
            async move {
                let started = Instant::now();
                let result = run_collection_query(
                    handle,
                    db_name,
                    collection_name,
                    filter,
                    skip,
                    limit,
                );
                (result, started.elapsed())
            },
            move |(result, duration)| Message::CollectionQueryCompleted { tab_id, result, duration },
        )
    }
}

impl TabData {
    fn new_text(id: TabId) -> Self {
        Self {
            id,
            title: format!("Text {id}"),
            content: TabKind::TextEditor { text: String::new() },
        }
    }

    fn new_form(id: TabId) -> Self {
        Self {
            id,
            title: format!("Form {id}"),
            content: TabKind::Form { input: String::new(), clicks: 0 },
        }
    }

    fn new_collection(
        id: TabId,
        client_id: ClientId,
        client_name: String,
        db_name: String,
        collection: String,
        values: Vec<Bson>,
    ) -> Self {
        let title = format!("{db_name}.{collection}");
        Self {
            id,
            title,
            content: TabKind::Collection(CollectionTab::new(
                client_id, client_name, db_name, collection, values,
            )),
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.content {
            TabKind::TextEditor { text: value } => {
                let id = self.id;

                let input = text_input("Введите текст", value)
                    .on_input(move |content| Message::TextTabChanged(id, content))
                    .padding([8, 12])
                    .width(Length::Fill);

                Container::new(input)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding([12, 16])
                    .into()
            }
            TabKind::Form { input, clicks } => {
                let id_for_input = self.id;
                let id_for_button = self.id;

                let action_row = Row::new()
                    .spacing(12)
                    .push(
                        Button::new(text("Нажать"))
                            .on_press(Message::FormButtonPressed(id_for_button))
                            .padding([6, 16]),
                    )
                    .push(Text::new(format!("Нажатий: {clicks}")).size(16).width(Length::Fill));

                let input_widget = text_input("Введите текст", input)
                    .on_input(move |value| Message::FormTextChanged(id_for_input, value))
                    .padding([8, 12])
                    .width(Length::Fill);

                Column::new()
                    .spacing(16)
                    .push(action_row)
                    .push(input_widget)
                    .width(Length::Fill)
                    .padding([12, 16])
                    .into()
            }
            TabKind::Collection(collection) => collection.view(self.id),
        }
    }
}

impl TopMenu {
    fn label(self) -> &'static str {
        match self {
            TopMenu::File => "File",
            TopMenu::View => "View",
            TopMenu::Options => "Options",
            TopMenu::Windows => "Windows",
            TopMenu::Help => "Help",
        }
    }
}

impl MenuEntry {
    fn label(self) -> &'static str {
        match self {
            MenuEntry::Action(label) => label,
        }
    }
}

impl OMDBConnection {
    fn from_uri(uri: &str) -> Self {
        Self::Uri(uri.to_owned())
    }

    fn display_label(&self) -> String {
        match self {
            OMDBConnection::Uri(uri) => uri.clone(),
        }
    }
}

impl OMDBClient {
    fn new(id: ClientId, connection: OMDBConnection) -> Self {
        Self {
            id,
            name: connection.display_label(),
            connection,
            status: ConnectionStatus::Connecting,
            expanded: true,
            handle: None,
            databases: Vec::new(),
        }
    }
}

impl DatabaseNode {
    fn new(name: String) -> Self {
        Self { name, expanded: false, state: DatabaseState::Idle, collections: Vec::new() }
    }
}

impl CollectionNode {
    fn new(name: String) -> Self {
        Self { name }
    }
}

fn connect_and_discover(connection: OMDBConnection) -> Result<ConnectionBootstrap, String> {
    match connection {
        OMDBConnection::Uri(uri) => {
            let client = Client::with_uri_str(uri.as_str()).map_err(|err| err.to_string())?;
            let databases = client.list_database_names().run().map_err(|err| err.to_string())?;

            Ok(ConnectionBootstrap { handle: Arc::new(client), databases })
        }
    }
}

fn fetch_collections(client: Arc<Client>, db_name: String) -> Result<Vec<String>, String> {
    let database = client.database(&db_name);
    database.list_collection_names().run().map_err(|err| err.to_string())
}

fn run_collection_query(
    client: Arc<Client>,
    db_name: String,
    collection_name: String,
    filter: Document,
    skip: u64,
    limit: u64,
) -> Result<Vec<Bson>, String> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let database = client.database(&db_name);
    let collection = database.collection::<Document>(&collection_name);

    let mut find_builder = collection.find(filter);

    if skip > 0 {
        find_builder = find_builder.skip(skip);
    }

    let limit_capped = limit.min(i64::MAX as u64) as i64;
    if limit_capped > 0 {
        find_builder = find_builder.limit(limit_capped);
    }

    let cursor = find_builder.run().map_err(|err| err.to_string())?;

    let take_limit = if limit_capped > 0 { limit_capped as usize } else { usize::MAX };
    let mut documents = Vec::new();

    for result in cursor.into_iter().take(take_limit) {
        let document = result.map_err(|err| err.to_string())?;
        documents.push(Bson::Document(document));
    }

    Ok(documents)
}
