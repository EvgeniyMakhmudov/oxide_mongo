use iced::Font;
use iced::alignment::{Horizontal, Vertical};
use iced::border;
use iced::keyboard::{self, key};
use iced::widget::image::Handle;
use iced::widget::pane_grid::ResizeEvent;
use iced::widget::scrollable;
use iced::widget::text_editor::{
    self, Action as TextEditorAction, Binding as TextEditorBinding, Content as TextEditorContent,
};
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Space, Text, button, container, pane_grid,
    text, text_input,
};
use iced::window;
use iced::{
    Color, Element, Length, Renderer, Shadow, Subscription, Task, Theme, application, clipboard,
};
use iced_aw::{
    ContextMenu,
    menu::{Item as MenuItemWidget, Menu, MenuBar},
};
use mongodb::bson::{self, Bson, Document, doc};
use mongodb::options::{Acknowledgment, Collation, Hint, WriteConcern};
use mongodb::sync::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

type TabId = u32;
type ClientId = u32;

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);
const DEFAULT_RESULT_LIMIT: i64 = 50;
const DEFAULT_RESULT_SKIP: u64 = 0;
const WINDOW_ICON_BYTES: &[u8] = include_bytes!("../assests/icons/oxide_mongo_256x256.png");
const ICON_NETWORK_BYTES: &[u8] = include_bytes!("../assests/icons/network_115x128.png");
const ICON_DATABASE_BYTES: &[u8] = include_bytes!("../assests/icons/database_105x128.png");
const ICON_COLLECTION_BYTES: &[u8] = include_bytes!("../assests/icons/collection_108x128.png");
const CONNECTIONS_FILE: &str = "connections.toml";
const MONO_FONT_BYTES: &[u8] = include_bytes!("../assests/fonts/DejaVuSansMono.ttf");
const MONO_FONT: Font = Font::with_name("DejaVu Sans Mono");
static ICON_NETWORK_HANDLE: OnceLock<Handle> = OnceLock::new();
static ICON_DATABASE_HANDLE: OnceLock<Handle> = OnceLock::new();
static ICON_COLLECTION_HANDLE: OnceLock<Handle> = OnceLock::new();

fn main() -> iced::Result {
    let icon = window::icon::from_file_data(WINDOW_ICON_BYTES, None)
        .map_err(|error| iced::Error::WindowCreationFailed(Box::new(error)))?;

    let mut window_settings = window::Settings::default();
    window_settings.icon = Some(icon);
    window_settings.size.width += 280.0;

    application("Oxide Mongo", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(MONO_FONT_BYTES)
        .window(window_settings)
        .run_with(App::init)
}

struct App {
    panes: pane_grid::State<PaneContent>,
    tabs: Vec<TabData>,
    active_tab: Option<TabId>,
    next_tab_id: TabId,
    clients: Vec<OMDBClient>,
    next_client_id: ClientId,
    last_collection_click: Option<CollectionClick>,
    connections: Vec<ConnectionEntry>,
    mode: AppMode,
    connections_window: Option<ConnectionsWindowState>,
    connection_form: Option<ConnectionFormState>,
    collection_modal: Option<CollectionModalState>,
    database_modal: Option<DatabaseModalState>,
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
    collection: CollectionTab,
}

#[derive(Debug, Clone)]
enum Message {
    MenuItemSelected(TopMenu, MenuEntry),
    TabSelected(TabId),
    TabClosed(TabId),
    PaneResized(ResizeEvent),
    ConnectionCompleted {
        client_id: ClientId,
        result: Result<ConnectionBootstrap, String>,
    },
    ToggleClient(ClientId),
    ToggleDatabase {
        client_id: ClientId,
        db_name: String,
    },
    CollectionsLoaded {
        client_id: ClientId,
        db_name: String,
        result: Result<Vec<String>, String>,
    },
    CollectionClicked {
        client_id: ClientId,
        db_name: String,
        collection: String,
    },
    CollectionEditorAction {
        tab_id: TabId,
        action: TextEditorAction,
    },
    CollectionSend(TabId),
    CollectionTreeToggle {
        tab_id: TabId,
        node_id: usize,
    },
    CollectionContextMenu {
        client_id: ClientId,
        db_name: String,
        collection: String,
        action: CollectionContextAction,
    },
    DatabaseContextMenu {
        client_id: ClientId,
        db_name: String,
        action: DatabaseContextAction,
    },
    CollectionSkipChanged {
        tab_id: TabId,
        value: String,
    },
    CollectionLimitChanged {
        tab_id: TabId,
        value: String,
    },
    CollectionPaneResized {
        tab_id: TabId,
        split: pane_grid::Split,
        ratio: f32,
    },
    CollectionSkipPrev(TabId),
    CollectionSkipNext(TabId),
    CollectionQueryCompleted {
        tab_id: TabId,
        result: Result<QueryResult, String>,
        duration: Duration,
    },
    ConnectionsSelect(usize),
    ConnectionsQuickConnect(usize),
    ConnectionsCreate,
    ConnectionsEdit,
    ConnectionsDelete,
    ConnectionsDeleteConfirmed,
    ConnectionsDeleteCancelled,
    ConnectionsConnect,
    ConnectionsCancel,
    ConnectionFormTabChanged(ConnectionFormTab),
    ConnectionFormNameChanged(String),
    ConnectionFormHostChanged(String),
    ConnectionFormPortChanged(String),
    ConnectionFormIncludeAction(TextEditorAction),
    ConnectionFormExcludeAction(TextEditorAction),
    ConnectionFormAddSystemFilters,
    ConnectionFormTest,
    ConnectionFormTestResult(Result<(), String>),
    ConnectionFormSave,
    ConnectionFormCancel,
    TableContextMenu {
        tab_id: TabId,
        node_id: usize,
        action: TableContextAction,
    },
    CollectionModalInputChanged(String),
    CollectionModalConfirm,
    CollectionModalCancel,
    CollectionDeleteAllCompleted {
        client_id: ClientId,
        db_name: String,
        collection: String,
        result: Result<u64, String>,
    },
    CollectionDeleteCollectionCompleted {
        client_id: ClientId,
        db_name: String,
        collection: String,
        result: Result<(), String>,
    },
    CollectionRenameCompleted {
        client_id: ClientId,
        db_name: String,
        old_collection: String,
        new_name: String,
        result: Result<(), String>,
    },
    DatabaseModalInputChanged(String),
    DatabaseModalConfirm,
    DatabaseModalCancel,
    DatabaseDropCompleted {
        client_id: ClientId,
        db_name: String,
        result: Result<(), String>,
    },
    DatabasesRefreshed {
        client_id: ClientId,
        result: Result<Vec<String>, String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableContextAction {
    CopyJson,
    CopyKey,
    CopyValue,
    CopyPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectionContextAction {
    OpenEmptyTab,
    ViewDocuments,
    DeleteTemplate,
    DeleteAllDocuments,
    DeleteCollection,
    RenameCollection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseContextAction {
    Refresh,
    Stats,
    Drop,
}

#[derive(Debug, Clone)]
enum OMDBConnection {
    Uri { uri: String, include_filter: String, exclude_filter: String },
}

#[derive(Debug, Clone)]
struct OMDBClient {
    id: ClientId,
    name: String,
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

impl ConnectionEntry {
    fn uri(&self) -> String {
        format!("mongodb://{}:{}", self.host.trim(), self.port)
    }
}

impl ConnectionsWindowState {
    fn new(selected: Option<usize>) -> Self {
        Self { selected, confirm_delete: false, feedback: None, last_click: None }
    }
}

impl ConnectionFormState {
    fn new(mode: ConnectionFormMode, entry: Option<&ConnectionEntry>) -> Self {
        let (name, host, port, include_filter, exclude_filter) = entry
            .map(|conn| {
                (
                    conn.name.clone(),
                    conn.host.clone(),
                    conn.port.to_string(),
                    conn.include_filter.clone(),
                    conn.exclude_filter.clone(),
                )
            })
            .unwrap_or_else(|| {
                (
                    String::new(),
                    String::from("localhost"),
                    String::from("27017"),
                    String::new(),
                    String::new(),
                )
            });

        Self {
            mode,
            active_tab: ConnectionFormTab::General,
            name,
            host,
            port,
            include_editor: TextEditorContent::with_text(&include_filter),
            exclude_editor: TextEditorContent::with_text(&exclude_filter),
            validation_error: None,
            test_feedback: None,
            testing: false,
        }
    }

    fn validate(&self) -> Result<ConnectionEntry, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(String::from("Название не может быть пустым"));
        }

        let host = self.host.trim();
        if host.is_empty() {
            return Err(String::from("Адрес/Хост/IP не может быть пустым"));
        }

        let port: u16 = self
            .port
            .trim()
            .parse()
            .map_err(|_| String::from("Порт должен быть числом от 0 до 65535"))?;

        Ok(ConnectionEntry {
            name: name.to_string(),
            host: host.to_string(),
            port,
            include_filter: self.include_editor.text(),
            exclude_filter: self.exclude_editor.text(),
        })
    }

    fn include_action(&mut self, action: TextEditorAction) {
        self.include_editor.perform(action);
    }

    fn exclude_action(&mut self, action: TextEditorAction) {
        self.exclude_editor.perform(action);
    }

    fn add_system_filters(&mut self) {
        const SYSTEM_FILTERS: [&str; 4] = ["admin", "local", "config", "$external"];

        let current_text = self.exclude_editor.text();
        let mut lines: Vec<String> = if current_text.is_empty() {
            Vec::new()
        } else {
            current_text.lines().map(|line| line.to_string()).collect()
        };

        let mut existing: HashSet<String> =
            lines.iter().map(|line| line.trim().to_string()).collect();
        let mut added = false;

        for filter in SYSTEM_FILTERS {
            if existing.insert(filter.to_string()) {
                lines.push(filter.to_string());
                added = true;
            }
        }

        if added {
            let new_text = lines.join("\n");
            self.exclude_editor = TextEditorContent::with_text(&new_text);
        }
    }
}

impl CollectionModalState {
    fn new_delete_all(client_id: ClientId, db_name: String, collection: String) -> Self {
        Self {
            client_id,
            db_name,
            collection,
            kind: CollectionModalKind::DeleteAllDocuments,
            input: String::new(),
            error: None,
            processing: false,
        }
    }

    fn new_delete_collection(client_id: ClientId, db_name: String, collection: String) -> Self {
        Self {
            client_id,
            db_name,
            collection,
            kind: CollectionModalKind::DeleteCollection,
            input: String::new(),
            error: None,
            processing: false,
        }
    }

    fn new_rename(client_id: ClientId, db_name: String, collection: String) -> Self {
        Self {
            client_id,
            db_name,
            collection: collection.clone(),
            kind: CollectionModalKind::RenameCollection,
            input: collection,
            error: None,
            processing: false,
        }
    }
}

impl DatabaseModalState {
    fn new_drop(client_id: ClientId, db_name: String) -> Self {
        Self { client_id, db_name, input: String::new(), error: None, processing: false }
    }
}

impl TestFeedback {
    fn message(&self) -> &str {
        match self {
            TestFeedback::Success(msg) | TestFeedback::Failure(msg) => msg.as_str(),
        }
    }

    fn is_success(&self) -> bool {
        matches!(self, TestFeedback::Success(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionEntry {
    name: String,
    host: String,
    port: u16,
    include_filter: String,
    exclude_filter: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConnectionStore {
    connections: Vec<ConnectionEntry>,
}

#[derive(Debug)]
struct ConnectionsWindowState {
    selected: Option<usize>,
    confirm_delete: bool,
    feedback: Option<String>,
    last_click: Option<ListClick>,
}

#[derive(Debug, Clone, Copy)]
struct ListClick {
    index: usize,
    at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Main,
    Connections,
    ConnectionForm,
    CollectionModal,
    DatabaseModal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionFormTab {
    General,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionFormMode {
    Create,
    Edit(usize),
}

#[derive(Debug)]
struct ConnectionFormState {
    mode: ConnectionFormMode,
    active_tab: ConnectionFormTab,
    name: String,
    host: String,
    port: String,
    include_editor: TextEditorContent,
    exclude_editor: TextEditorContent,
    validation_error: Option<String>,
    test_feedback: Option<TestFeedback>,
    testing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectionModalKind {
    DeleteAllDocuments,
    DeleteCollection,
    RenameCollection,
}

#[derive(Debug, Clone)]
struct CollectionModalState {
    client_id: ClientId,
    db_name: String,
    collection: String,
    kind: CollectionModalKind,
    input: String,
    error: Option<String>,
    processing: bool,
}

#[derive(Debug, Clone)]
struct DatabaseModalState {
    client_id: ClientId,
    db_name: String,
    input: String,
    error: Option<String>,
    processing: bool,
}

#[derive(Debug)]
enum TestFeedback {
    Success(String),
    Failure(String),
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

struct TableContextMenu;

impl TableContextMenu {
    fn new<'a>(
        underlay: impl Into<Element<'a, Message>>,
        overlay: impl Fn() -> Element<'a, Message> + 'a,
    ) -> Element<'a, Message> {
        ContextMenu::new(underlay, overlay).into()
    }
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
    ) -> Self {
        let id_value = id.next();
        match value {
            Bson::Document(map) => {
                let children = map
                    .iter()
                    .map(|(k, v)| BsonNode::from_bson(Some(k.clone()), Some(k.clone()), v, id))
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
                        BsonNode::from_bson(Some(display), Some(index.to_string()), item, id)
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
                let (display, ty) = format_bson_scalar(other);
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
        self.display_key.clone().unwrap_or_else(|| String::from("value"))
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
        Bson::Decimal128(d) => (format!("numberDecimal(\"{}\")", d), String::from("Decimal128")),
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

fn format_bson_shell(value: &Bson) -> String {
    format_bson_shell_internal(value, 0)
}

fn format_bson_shell_internal(value: &Bson, level: usize) -> String {
    match value {
        Bson::Document(doc) => format_document_shell(doc, level),
        Bson::Array(items) => format_array_shell(items, level),
        _ => format_bson_shell_scalar(value),
    }
}

fn format_document_shell(doc: &Document, level: usize) -> String {
    if doc.is_empty() {
        return String::from("{}");
    }

    let indent_current = shell_indent(level);
    let indent_child = shell_indent(level + 1);

    let mut entries: Vec<Vec<String>> = Vec::new();
    for (key, value) in doc.iter() {
        let value_repr = format_bson_shell_internal(value, level + 1);
        let value_lines: Vec<&str> = value_repr.lines().collect();
        let mut lines = Vec::new();
        if let Some((first, rest)) = value_lines.split_first() {
            lines.push(format!("{indent_child}\"{key}\": {first}"));
            for line in rest {
                lines.push(line.to_string());
            }
        } else {
            lines.push(format!("{indent_child}\"{key}\": null"));
        }
        entries.push(lines);
    }

    let mut result = String::from("{\n");
    let entry_count = entries.len();
    for (index, mut entry) in entries.into_iter().enumerate() {
        if let Some(last) = entry.last_mut() {
            if index + 1 != entry_count {
                last.push(',');
            }
        }
        for line in entry {
            result.push_str(&line);
            result.push('\n');
        }
    }
    result.push_str(&indent_current);
    result.push('}');
    result
}

fn format_array_shell(items: &[Bson], level: usize) -> String {
    if items.is_empty() {
        return String::from("[]");
    }

    let indent_current = shell_indent(level);
    let indent_child = shell_indent(level + 1);

    let mut result = String::from("[\n");
    let len = items.len();
    for (index, item) in items.iter().enumerate() {
        let value_repr = format_bson_shell_internal(item, level + 1);
        let value_lines: Vec<&str> = value_repr.lines().collect();
        let last_line_index = value_lines.len().saturating_sub(1);
        for (line_index, line) in value_lines.into_iter().enumerate() {
            if line_index == 0 {
                result.push_str(&indent_child);
                result.push_str(line);
            } else {
                result.push_str(line);
            }
            if line_index == last_line_index && index + 1 != len {
                result.push(',');
            }
            result.push('\n');
        }
    }
    result.push_str(&indent_current);
    result.push(']');
    result
}

fn format_bson_shell_scalar(value: &Bson) -> String {
    match value {
        Bson::String(s) => serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s)),
        Bson::Boolean(b) => b.to_string(),
        Bson::Int32(i) => i.to_string(),
        Bson::Int64(i) => i.to_string(),
        Bson::Double(f) => {
            if f.is_finite() {
                format!("{f}")
            } else {
                format!("Double({f})")
            }
        }
        Bson::Decimal128(d) => format!("numberDecimal(\"{}\")", d),
        Bson::DateTime(dt) => match dt.try_to_rfc3339_string() {
            Ok(iso) => format!("ISODate(\"{}\")", iso),
            Err(_) => format!("DateTime({})", dt.timestamp_millis()),
        },
        Bson::ObjectId(oid) => format!("ObjectId(\"{}\")", oid),
        Bson::Binary(bin) => format!("Binary(len={}, subtype={:?})", bin.bytes.len(), bin.subtype),
        Bson::Symbol(sym) => format!("Symbol({sym:?})"),
        Bson::RegularExpression(regex) => {
            if regex.options.is_empty() {
                format!("Regex({:?})", regex.pattern)
            } else {
                format!("Regex({:?}, {:?})", regex.pattern, regex.options)
            }
        }
        Bson::Timestamp(ts) => format!("Timestamp(time={}, increment={})", ts.time, ts.increment),
        Bson::JavaScriptCode(code) => format!("Code({code:?})"),
        Bson::JavaScriptCodeWithScope(code_with_scope) => {
            let scope_len = code_with_scope.scope.len();
            format!("CodeWithScope({:?}, scope_fields={})", code_with_scope.code, scope_len)
        }
        Bson::DbPointer(ptr) => format!("DbPointer({ptr:?})"),
        Bson::Undefined => String::from("undefined"),
        Bson::Null => String::from("null"),
        Bson::MinKey => String::from("MinKey"),
        Bson::MaxKey => String::from("MaxKey"),
        Bson::Document(_) | Bson::Array(_) => unreachable!("containers handled separately"),
    }
}

fn shell_indent(level: usize) -> String {
    const INDENT: usize = 4;
    " ".repeat(level * INDENT)
}

impl BsonTree {
    fn from_values(values: &[Bson]) -> Self {
        let mut id_gen = IdGenerator::default();
        let mut roots = Vec::new();

        if values.is_empty() {
            let info_value = Bson::String("Документы не найдены".into());
            let placeholder =
                BsonNode::from_bson(Some(String::from("info")), None, &info_value, &mut id_gen);
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
                roots.push(BsonNode::from_bson(Some(key), None, value, &mut id_gen));
            }
        }

        let expanded = HashSet::new();

        Self { roots, expanded }
    }

    fn from_error(message: String) -> Self {
        let value = Bson::String(message);
        Self::from_values(std::slice::from_ref(&value))
    }

    fn from_distinct(field: String, values: Vec<Bson>) -> Self {
        let mut id_gen = IdGenerator::default();
        let array_bson = Bson::Array(values);
        let path_key = field.clone();
        let node = BsonNode::from_bson(Some(field), Some(path_key), &array_bson, &mut id_gen);
        let mut expanded = HashSet::new();
        expanded.insert(node.id);

        Self { roots: vec![node], expanded }
    }

    fn from_count(value: Bson) -> Self {
        let mut id_gen = IdGenerator::default();
        let node = BsonNode::from_bson(
            Some(String::from("count")),
            Some(String::from("count")),
            &value,
            &mut id_gen,
        );
        let mut expanded = HashSet::new();
        expanded.insert(node.id);
        Self { roots: vec![node], expanded }
    }

    fn from_document(document: Document) -> Self {
        let mut id_gen = IdGenerator::default();
        let value = Bson::Document(document);
        let mut roots = Vec::new();
        let mut expanded = HashSet::new();

        let key = match &value {
            Bson::Document(doc) => {
                doc.get("_id").map(Self::summarize_id).unwrap_or_else(|| String::from("document"))
            }
            _ => String::from("document"),
        };

        let node = BsonNode::from_bson(Some(key), None, &value, &mut id_gen);
        expanded.insert(node.id);
        roots.push(node);

        Self { roots, expanded }
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
                Container::new(Text::new("Key").size(14).font(MONO_FONT))
                    .width(Length::FillPortion(4))
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
                Container::new(Text::new("Value").size(14).font(MONO_FONT))
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
                Container::new(Text::new("Type").size(14).font(MONO_FONT))
                    .width(Length::FillPortion(3))
                    .padding([6, 8]),
            );

        let header = Container::new(header_row).width(Length::Fill).height(Length::Shrink).style(
            move |_| iced::widget::container::Style {
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
                    let toggle = Button::new(Text::new(indicator))
                        .padding([0, 4])
                        .on_press(Message::CollectionTreeToggle { tab_id, node_id: node.id });
                    key_row = key_row.push(toggle);
                } else {
                    let disabled = Container::new(
                        Text::new(indicator).size(14).color(Color::from_rgb8(0xb5, 0xbc, 0xc6)),
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
            key_row = key_row.push(Text::new(key_label.clone()).size(14).font(MONO_FONT));

            let value_text = node.value_display().unwrap_or_default();
            let type_text = node.type_label();

            let key_cell = Container::new(key_row).width(Length::FillPortion(4)).padding([6, 8]);

            let value_cell = Container::new(Text::new(value_text.clone()).size(14).font(MONO_FONT))
                .width(Length::FillPortion(5))
                .padding([6, 8]);

            let type_cell = Container::new(Text::new(type_text.clone()).size(14).font(MONO_FONT))
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

            let menu_node_id = node.id;
            let menu_tab_id = tab_id;
            let path_enabled = self.node_path(menu_node_id).is_some();

            let row_container = Container::new(row_content).width(Length::Fill).style(move |_| {
                iced::widget::container::Style {
                    background: Some(background.into()),
                    ..Default::default()
                }
            });

            let row_with_menu = TableContextMenu::new(row_container, move || {
                let mut menu = Column::new().spacing(4).padding([4, 6]);

                let copy_json = Button::new(Text::new("Копировать JSON").size(14))
                    .padding([4, 12])
                    .width(Length::Shrink)
                    .on_press(Message::TableContextMenu {
                        tab_id: menu_tab_id,
                        node_id: menu_node_id,
                        action: TableContextAction::CopyJson,
                    });

                let copy_key = Button::new(Text::new("Копировать ключ").size(14))
                    .padding([4, 12])
                    .width(Length::Shrink)
                    .on_press(Message::TableContextMenu {
                        tab_id: menu_tab_id,
                        node_id: menu_node_id,
                        action: TableContextAction::CopyKey,
                    });

                let copy_value = Button::new(Text::new("Копировать значение").size(14))
                    .padding([4, 12])
                    .width(Length::Shrink)
                    .on_press(Message::TableContextMenu {
                        tab_id: menu_tab_id,
                        node_id: menu_node_id,
                        action: TableContextAction::CopyValue,
                    });

                let mut copy_path = Button::new(Text::new("Копировать путь").size(14))
                    .padding([4, 12])
                    .width(Length::Shrink);

                if path_enabled {
                    copy_path = copy_path.on_press(Message::TableContextMenu {
                        tab_id: menu_tab_id,
                        node_id: menu_node_id,
                        action: TableContextAction::CopyPath,
                    });
                }

                menu = menu.push(copy_json);
                menu = menu.push(copy_key);
                menu = menu.push(copy_value);
                menu = menu.push(copy_path);

                menu.into()
            });

            body = body.push(row_with_menu);
        }

        let c_body =
            Container::new(body).width(Length::Fill).height(Length::Shrink).style(move |_| {
                iced::widget::container::Style {
                    background: Some(header_bg.into()),
                    ..Default::default()
                }
            });

        let scrollable_body = Scrollable::new(c_body).width(Length::Fill);

        Column::new().spacing(2).height(Length::Shrink).push(header).push(scrollable_body).into()
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

    fn node_display_key(&self, node_id: usize) -> Option<String> {
        Self::find_node(&self.roots, node_id).map(BsonNode::display_key)
    }

    fn node_value_display(&self, node_id: usize) -> Option<String> {
        Self::find_node(&self.roots, node_id).map(|node| node.value_display().unwrap_or_default())
    }

    fn node_bson(&self, node_id: usize) -> Option<Bson> {
        Self::find_node(&self.roots, node_id).map(|node| node.bson.clone())
    }

    fn node_path(&self, node_id: usize) -> Option<String> {
        let nodes = Self::find_node_path(&self.roots, node_id, &mut Vec::new())?;
        let mut components = Vec::new();
        for node in nodes {
            if let Some(component) = &node.path_key {
                components.push(component.clone());
            }
        }

        if components.is_empty() { None } else { Some(components.join(".")) }
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

impl CollectionTab {
    const REQUEST_EDITOR_LINES: f32 = 4.0;
    const REQUEST_LINE_HEIGHT: f32 = 24.0;
    const REQUEST_VERTICAL_CHROME: f32 = 24.0;
    const RESPONSE_REFERENCE_HEIGHT: f32 = 480.0;
    const MIN_RESPONSE_RATIO: f32 = 0.1;

    fn preferred_request_height() -> f32 {
        Self::REQUEST_EDITOR_LINES * Self::REQUEST_LINE_HEIGHT + Self::REQUEST_VERTICAL_CHROME
    }

    fn min_request_ratio() -> f32 {
        let preferred = Self::preferred_request_height();
        preferred / (preferred + Self::RESPONSE_REFERENCE_HEIGHT)
    }

    fn initial_split_ratio() -> f32 {
        Self::min_request_ratio()
    }

    fn clamp_split_ratio(ratio: f32) -> f32 {
        let min_ratio = Self::min_request_ratio();
        let max_ratio = 1.0 - Self::MIN_RESPONSE_RATIO;
        ratio.clamp(min_ratio, max_ratio)
    }

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
        let initial_ratio = Self::clamp_split_ratio(Self::initial_split_ratio());
        panes.resize(split, initial_ratio);

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

    fn resize_split(&mut self, split: pane_grid::Split, ratio: f32) {
        if !ratio.is_finite() {
            return;
        }

        let clamped = Self::clamp_split_ratio(ratio);
        self.panes.resize(split, clamped);
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
                Image::new(shared_icon_handle(&ICON_NETWORK_HANDLE, ICON_NETWORK_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(Text::new(self.client_name.clone()).size(14));

        let database_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_DATABASE_HANDLE, ICON_DATABASE_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(Text::new(self.db_name.clone()).size(14));

        let collection_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_COLLECTION_HANDLE, ICON_COLLECTION_BYTES))
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
            .push(Container::new(info_labels).width(Length::Fill).padding([0, 4]))
            .push(navigation);

        let panel_bg = Color::from_rgb8(0xef, 0xf1, 0xf5);
        let panel_border = Color::from_rgb8(0xd0, 0xd4, 0xda);

        let info_panel =
            Container::new(info_row).width(Length::Fill).padding([8, 12]).style(move |_| {
                iced::widget::container::Style {
                    background: Some(panel_bg.into()),
                    border: border::rounded(6).width(1).color(panel_border),
                    ..Default::default()
                }
            });

        let resize_tab_id = tab_id;
        let panes = pane_grid::PaneGrid::new(&self.panes, |_, pane, _| match pane {
            CollectionPane::Request => pane_grid::Content::new(self.request_view(tab_id)),
            CollectionPane::Response => pane_grid::Content::new(self.response_view(tab_id)),
        })
        .on_resize(8, move |event| Message::CollectionPaneResized {
            tab_id: resize_tab_id,
            split: event.split,
            ratio: event.ratio,
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

    fn table_context_content(&self, node_id: usize, action: TableContextAction) -> Option<String> {
        match action {
            TableContextAction::CopyJson => {
                self.bson_tree.node_bson(node_id).map(|bson| format_bson_shell(&bson))
            }
            TableContextAction::CopyKey => self.bson_tree.node_display_key(node_id),
            TableContextAction::CopyValue => self.bson_tree.node_value_display(node_id),
            TableContextAction::CopyPath => self.bson_tree.node_path(node_id),
        }
    }

    fn request_view(&self, tab_id: TabId) -> Element<Message> {
        let send_tab_id = tab_id;
        let editor = text_editor::TextEditor::new(&self.editor)
            .key_binding(move |key_press| {
                let is_enter = matches!(key_press.key, keyboard::Key::Named(key::Named::Enter));
                if is_enter && key_press.modifiers.command() {
                    Some(TextEditorBinding::Custom(Message::CollectionSend(send_tab_id)))
                } else {
                    TextEditorBinding::from_key_press(key_press)
                }
            })
            .on_action(move |action| Message::CollectionEditorAction { tab_id, action })
            .height(Length::Fill);

        let send_content =
            Container::new(Text::new("Send")).center_x(Length::Shrink).center_y(Length::Fill);

        let send_button = Button::new(send_content)
            .on_press(Message::CollectionSend(tab_id))
            .padding([4, 12])
            .width(Length::Shrink)
            .height(Length::Fill);

        let controls_row = Row::new()
            .spacing(0)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Container::new(editor).width(Length::FillPortion(9)).height(Length::Fill).style(
                move |_| container::Style {
                    border: border::rounded(4.0).width(1),
                    ..Default::default()
                },
            ))
            .push(
                Container::new(send_button)
                    .width(Length::Shrink)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .style(move |_| container::Style {
                        border: border::rounded(4.0).width(1),
                        ..Default::default()
                    }),
            );

        let content = Column::new().spacing(8).width(Length::Fill).height(Length::Fill).push(
            Container::new(controls_row).width(Length::Fill).height(Length::Fill).style(
                move |_| container::Style {
                    border: border::rounded(4.0).width(1),
                    ..Default::default()
                },
            ),
        );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                border: border::rounded(4.0).width(1),
                ..Default::default()
            })
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

    fn parse_query(&self, text: &str) -> Result<QueryOperation, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(String::from(
                "Запрос должен начинаться с db.<collection>, db.getCollection('<collection>') или поддерживаемого метода базы.",
            ));
        }

        let cleaned = trimmed.trim_end_matches(';').trim();

        if let Some(result) = self.try_parse_database_method(cleaned)? {
            return Ok(result);
        }

        let after_collection = Self::strip_collection_prefix(cleaned)?;

        let (method_name, args, remainder) = Self::extract_primary_method(after_collection)?;
        if !remainder.trim().is_empty() {
            let extra = remainder.trim_start();
            if method_name == "find" && extra.starts_with(".countDocuments(") {
                return Err(String::from(
                    "countDocuments() нужно вызывать непосредственно на коллекции. Цепочки вида db.collection.find(...).countDocuments(...) не поддерживаются.",
                ));
            }
            return Err(String::from(
                "Поддерживается только один вызов метода после указания коллекции.",
            ));
        }

        let args_trimmed = args.trim();
        match method_name.as_str() {
            "countDocuments" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.len() > 2 {
                    return Err(String::from(
                        "countDocuments поддерживает не более двух аргументов: query и options.",
                    ));
                }

                let filter = if let Some(first) = parts.get(0) {
                    if first.is_empty() { Document::new() } else { Self::parse_json_object(first)? }
                } else {
                    Document::new()
                };

                let options = if let Some(second) = parts.get(1) {
                    Self::parse_count_documents_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::CountDocuments { filter, options })
            }
            "estimatedDocumentCount" => {
                let options = if args_trimmed.is_empty() {
                    None
                } else {
                    let parts = Self::split_arguments(args_trimmed);
                    if parts.len() > 1 {
                        return Err(String::from(
                            "estimatedDocumentCount принимает только один аргумент options.",
                        ));
                    }

                    match parts.get(0) {
                        Some(source) if source.trim().is_empty() => None,
                        Some(source) => Self::parse_estimated_count_options(source)?,
                        None => None,
                    }
                };

                Ok(QueryOperation::EstimatedDocumentCount { options })
            }
            "findOne" => {
                let filter = if args_trimmed.is_empty() {
                    Document::new()
                } else {
                    Self::parse_json_object(args_trimmed)?
                };
                Ok(QueryOperation::FindOne { filter })
            }
            "count" => {
                let filter = if args_trimmed.is_empty() {
                    Document::new()
                } else {
                    Self::parse_json_object(args_trimmed)?
                };
                Ok(QueryOperation::Count { filter })
            }
            "distinct" => {
                let parts = if args_trimmed.is_empty() {
                    Vec::new()
                } else {
                    Self::split_arguments(args_trimmed)
                };
                if parts.is_empty() {
                    return Err(String::from("distinct требует как минимум имя поля."));
                }

                let field_value: Value = serde_json::from_str(&parts[0])
                    .map_err(|error| format!("JSON parse error: {error}"))?;
                let field = match field_value {
                    Value::String(s) => s,
                    _ => return Err(String::from("Первый аргумент distinct должен быть строкой.")),
                };

                let filter = if parts.len() > 1 {
                    let filter_value: Value = serde_json::from_str(&parts[1])
                        .map_err(|error| format!("JSON parse error: {error}"))?;
                    if !filter_value.is_object() {
                        return Err(String::from("Фильтр distinct должен быть JSON-объектом."));
                    }
                    bson::to_document(&filter_value)
                        .map_err(|error| format!("BSON conversion error: {error}"))?
                } else {
                    Document::new()
                };

                Ok(QueryOperation::Distinct { field, filter })
            }
            "aggregate" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(
                        "aggregate требует массив стадий в качестве аргумента.",
                    ));
                }

                let value: Value = serde_json::from_str(args_trimmed)
                    .map_err(|error| format!("JSON parse error: {error}"))?;
                let array = value
                    .as_array()
                    .ok_or_else(|| String::from("Аргумент aggregate должен быть массивом."))?;
                let mut pipeline = Vec::new();
                for item in array {
                    let doc = item
                        .as_object()
                        .ok_or_else(|| String::from("Элементы pipeline должны быть объектами."))?;
                    pipeline.push(
                        bson::to_document(doc)
                            .map_err(|error| format!("BSON conversion error: {error}"))?,
                    );
                }
                Ok(QueryOperation::Aggregate { pipeline })
            }
            "insertOne" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(
                        "insertOne требует документ в качестве первого аргумента.",
                    ));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(
                        "insertOne принимает один документ и необязательный объект options.",
                    ));
                }

                let document = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_insert_one_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::InsertOne { document, options })
            }
            "insertMany" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(
                        "insertMany требует массив документов в качестве первого аргумента.",
                    ));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(
                        "insertMany принимает массив документов и необязательный объект options.",
                    ));
                }

                let docs_value: Value = serde_json::from_str(&parts[0])
                    .map_err(|error| format!("JSON parse error: {error}"))?;
                let docs_array = docs_value.as_array().ok_or_else(|| {
                    String::from("Первый аргумент insertMany должен быть массивом документов.")
                })?;
                if docs_array.is_empty() {
                    return Err(String::from(
                        "insertMany требует как минимум один документ в массиве.",
                    ));
                }

                let mut documents = Vec::with_capacity(docs_array.len());
                for (index, entry) in docs_array.iter().enumerate() {
                    let object = entry.as_object().ok_or_else(|| {
                        format!(
                            "Элемент с индексом {index} в insertMany должен быть JSON-объектом."
                        )
                    })?;
                    let doc = bson::to_document(object)
                        .map_err(|error| format!("BSON conversion error: {error}"))?;
                    documents.push(doc);
                }

                let options = if let Some(second) = parts.get(1) {
                    Self::parse_insert_many_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::InsertMany { documents, options })
            }
            "deleteOne" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(
                        "deleteOne требует фильтр в качестве первого аргумента.",
                    ));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(
                        "deleteOne принимает фильтр и необязательный объект options.",
                    ));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_delete_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::DeleteOne { filter, options })
            }
            "deleteMany" => {
                if args_trimmed.is_empty() {
                    return Err(String::from(
                        "deleteMany требует фильтр в качестве первого аргумента.",
                    ));
                }

                let parts = Self::split_arguments(args_trimmed);
                if parts.is_empty() || parts.len() > 2 {
                    return Err(String::from(
                        "deleteMany принимает фильтр и необязательный объект options.",
                    ));
                }

                let filter = Self::parse_json_object(&parts[0])?;
                let options = if let Some(second) = parts.get(1) {
                    Self::parse_delete_options(second)?
                } else {
                    None
                };

                Ok(QueryOperation::DeleteMany { filter, options })
            }
            "find" => {
                if args_trimmed.is_empty() {
                    return Ok(QueryOperation::Find { filter: Document::new() });
                }
                let filter = Self::parse_json_object(args_trimmed)?;
                Ok(QueryOperation::Find { filter })
            }
            other => Err(format!(
                "Метод {other} не поддерживается. Доступны: find, findOne, count, countDocuments, estimatedDocumentCount, distinct, aggregate, insertOne, insertMany, deleteOne, deleteMany.",
            )),
        }
    }

    fn strip_collection_prefix(text: &str) -> Result<&str, String> {
        if let Some(rest) = text.strip_prefix("db.getCollection(") {
            let rest = rest.trim_start();
            let (_, after_literal) = Self::parse_collection_literal(rest)?;
            let after_literal = after_literal.trim_start();
            let after_paren = after_literal.strip_prefix(')').ok_or_else(|| {
                String::from("После имени коллекции в getCollection ожидается ')'.")
            })?;
            let after_paren = after_paren.trim_start();
            if !after_paren.starts_with('.') {
                return Err(String::from("После указания коллекции ожидается вызов метода."));
            }
            Ok(after_paren)
        } else if let Some(rest) = text.strip_prefix("db.") {
            if rest.is_empty() {
                return Err(String::from("После db. ожидается имя коллекции."));
            }

            let bytes = rest.as_bytes();
            let mut index = 0usize;
            while index < bytes.len() {
                let byte = bytes[index];
                if (byte as char).is_ascii_alphanumeric() || byte == b'_' {
                    index += 1;
                    continue;
                }

                if byte == b'.' {
                    if index == 0 {
                        return Err(String::from("После db. ожидается имя коллекции."));
                    }
                    return Ok(&rest[index..]);
                }

                return Err(format!("Недопустимый символ '{}' в имени коллекции.", byte as char));
            }

            Err(String::from("После указания коллекции ожидается вызов метода."))
        } else {
            Err(String::from(
                "Запрос должен начинаться с db.<collection>, db.getCollection('<collection>') или поддерживаемого метода базы.",
            ))
        }
    }

    fn parse_collection_literal(text: &str) -> Result<(&str, &str), String> {
        if text.trim().is_empty() {
            return Err(String::from("Имя коллекции в getCollection не задано."));
        }

        let trimmed = text.trim_start();
        if trimmed.is_empty() {
            return Err(String::from("Имя коллекции в getCollection не задано."));
        }

        let bytes = trimmed.as_bytes();
        let quote = bytes[0];
        if quote != b'\'' && quote != b'"' {
            return Err(String::from(
                "Имя коллекции в getCollection должно быть строкой в кавычках.",
            ));
        }

        let mut index = 1usize;
        while index < bytes.len() {
            match bytes[index] {
                b'\\' => index += 2,
                ch if ch == quote => {
                    let name = &trimmed[1..index];
                    let rest = &trimmed[index + 1..];
                    return Ok((name, rest));
                }
                _ => index += 1,
            }
        }

        Err(String::from("Строка коллекции в getCollection не закрыта."))
    }

    fn extract_primary_method(text: &str) -> Result<(String, String, &str), String> {
        if !text.starts_with('.') {
            return Err(String::from("После указания коллекции ожидается вызов метода."));
        }

        let rest = &text[1..];
        if rest.is_empty() {
            return Err(String::from("После точки ожидается имя метода."));
        }

        let bytes = rest.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            let byte = bytes[index];
            if (byte as char).is_ascii_alphanumeric() || byte == b'_' {
                index += 1;
                continue;
            }

            if byte == b'(' {
                if index == 0 {
                    return Err(String::from("После точки ожидается имя метода."));
                }

                let method_name = &rest[..index];
                let mut depth = 0i32;
                let mut cursor = index + 1;
                while cursor < bytes.len() {
                    match bytes[cursor] {
                        b'(' => depth += 1,
                        b')' => {
                            if depth == 0 {
                                let args = &rest[index + 1..cursor];
                                let remainder = &rest[cursor + 1..];
                                return Ok((method_name.to_string(), args.to_string(), remainder));
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    cursor += 1;
                }

                return Err(String::from("Скобка метода не закрыта."));
            }

            if byte == b'.' {
                return Err(String::from(
                    "Поддерживается только один вызов метода после указания коллекции.",
                ));
            }

            return Err(format!("Недопустимый символ '{}' в имени метода.", byte as char));
        }

        Err(String::from("Ожидается '(' после названия метода."))
    }

    fn parse_count_documents_options(
        source: &str,
    ) -> Result<Option<CountDocumentsParsedOptions>, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from("Опции countDocuments должны быть JSON-объектом."))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = CountDocumentsParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "limit" => {
                    let limit = Self::parse_non_negative_u64(value, "limit")?;
                    options.limit = Some(limit);
                }
                "skip" => {
                    let skip = Self::parse_non_negative_u64(value, "skip")?;
                    options.skip = Some(skip);
                }
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                "hint" => {
                    let hint = match value {
                        Value::String(name) => Hint::Name(name.clone()),
                        Value::Object(map) => {
                            let doc = bson::to_document(map)
                                .map_err(|error| format!("BSON conversion error: {error}"))?;
                            Hint::Keys(doc)
                        }
                        _ => {
                            return Err(String::from(
                                "Параметр 'hint' должен быть строкой или JSON-объектом.",
                            ));
                        }
                    };
                    options.hint = Some(hint);
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается в options countDocuments. Доступны: limit, skip, hint, maxTimeMS.",
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_estimated_count_options(
        source: &str,
    ) -> Result<Option<EstimatedDocumentCountParsedOptions>, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object = value.as_object().ok_or_else(|| {
            String::from("Опции estimatedDocumentCount должны быть JSON-объектом.")
        })?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = EstimatedDocumentCountParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "maxTimeMS" => {
                    let millis = Self::parse_non_negative_u64(value, "maxTimeMS")?;
                    options.max_time = Some(Duration::from_millis(millis));
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается в options estimatedDocumentCount. Доступен только maxTimeMS.",
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn try_parse_database_method(&self, cleaned: &str) -> Result<Option<QueryOperation>, String> {
        if let Some(rest) = cleaned.strip_prefix("db.") {
            let rest = rest.trim();
            if rest.starts_with("getCollection(") {
                return Ok(None);
            }

            if let Some(paren_pos) = rest.find('(') {
                let dot_pos = rest.find('.');
                if dot_pos.is_none() || paren_pos < dot_pos.unwrap() {
                    let synthetic = format!(".{rest}");
                    let (method_name, args, remainder) = Self::extract_primary_method(&synthetic)?;
                    if !remainder.trim().is_empty() {
                        return Err(String::from(
                            "Поддерживается только один вызов метода после указания базы данных.",
                        ));
                    }
                    return self.parse_database_method(&method_name, &args).map(Some);
                }
            }
        }

        Ok(None)
    }

    fn parse_database_method(&self, method: &str, args: &str) -> Result<QueryOperation, String> {
        let args_trimmed = args.trim();

        match method {
            "stats" => {
                let mut command = doc! { "dbStats": 1 };

                if !args_trimmed.is_empty() {
                    if args_trimmed.starts_with('{') {
                        let extras = Self::parse_json_object(args_trimmed)?;
                        for (key, value) in extras {
                            command.insert(key, value);
                        }
                    } else {
                        let value: Value = serde_json::from_str(args_trimmed)
                            .map_err(|error| format!("JSON parse error: {error}"))?;

                        if let Some(number) = value.as_f64() {
                            command.insert("scale", Bson::Double(number));
                        } else if let Some(number) = value.as_i64() {
                            command.insert("scale", Bson::Int64(number));
                        } else if let Some(number) = value.as_u64() {
                            if number <= i64::MAX as u64 {
                                command.insert("scale", Bson::Int64(number as i64));
                            } else {
                                command.insert("scale", Bson::String(number.to_string()));
                            }
                        } else {
                            return Err(String::from(
                                "Аргумент db.stats ожидается числом или объектом с параметрами.",
                            ));
                        }
                    }
                }

                Ok(QueryOperation::DatabaseCommand { db: self.db_name.clone(), command })
            }
            other => Err(format!("Метод db.{other} не поддерживается. Доступен: stats.",)),
        }
    }

    fn parse_insert_one_options(source: &str) -> Result<Option<InsertOneParsedOptions>, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from("Опции insertOne должны быть JSON-объектом."))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = InsertOneParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается в options insertOne. Доступно: writeConcern.",
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_insert_many_options(source: &str) -> Result<Option<InsertManyParsedOptions>, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from("Опции insertMany должны быть JSON-объектом."))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = InsertManyParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                "ordered" => {
                    let ordered = value.as_bool().ok_or_else(|| {
                        String::from(
                            "Параметр 'ordered' в options insertMany должен быть логическим значением.",
                        )
                    })?;
                    options.ordered = Some(ordered);
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается в options insertMany. Доступны: writeConcern, ordered.",
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_delete_options(source: &str) -> Result<Option<DeleteParsedOptions>, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| String::from("Опции deleteOne/deleteMany должны быть JSON-объектом."))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut options = DeleteParsedOptions::default();

        for (key, value) in object {
            match key.as_str() {
                "writeConcern" => {
                    options.write_concern = Self::parse_write_concern_value(value)?;
                }
                "collation" => {
                    options.collation = Some(Self::parse_collation_value(value)?);
                }
                "hint" => {
                    options.hint = Some(Self::parse_hint_value(value)?);
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается в options deleteOne/deleteMany. Доступны: writeConcern, collation, hint.",
                    ));
                }
            }
        }

        if options.has_values() { Ok(Some(options)) } else { Ok(None) }
    }

    fn parse_write_concern_value(value: &Value) -> Result<Option<WriteConcern>, String> {
        let object = value
            .as_object()
            .ok_or_else(|| String::from("writeConcern должен быть JSON-объектом."))?;

        if object.is_empty() {
            return Ok(None);
        }

        let mut write_concern = WriteConcern::default();
        let mut has_values = false;

        for (key, value) in object {
            match key.as_str() {
                "w" => {
                    let ack = match value {
                        Value::String(s) => Acknowledgment::from(s.as_str()),
                        Value::Number(number) => {
                            let raw = number.as_u64().ok_or_else(|| {
                                String::from(
                                    "writeConcern.w должен быть неотрицательным целым числом.",
                                )
                            })?;
                            let nodes = u32::try_from(raw).map_err(|_| {
                                String::from(
                                    "writeConcern.w не должен превышать максимально допустимое значение.",
                                )
                            })?;
                            Acknowledgment::Nodes(nodes)
                        }
                        _ => {
                            return Err(String::from(
                                "writeConcern.w должен быть строкой или числом.",
                            ));
                        }
                    };
                    write_concern.w = Some(ack);
                    has_values = true;
                }
                "j" => {
                    let journal = value.as_bool().ok_or_else(|| {
                        String::from("writeConcern.j должен быть логическим значением.")
                    })?;
                    write_concern.journal = Some(journal);
                    has_values = true;
                }
                "wtimeout" | "wtimeoutMS" => {
                    let millis = value.as_u64().ok_or_else(|| {
                        String::from(
                            "writeConcern.wtimeout должен быть неотрицательным целым числом.",
                        )
                    })?;
                    write_concern.w_timeout = Some(Duration::from_millis(millis));
                    has_values = true;
                }
                other => {
                    return Err(format!(
                        "Параметр '{other}' не поддерживается внутри writeConcern. Доступны: w, j, wtimeout.",
                    ));
                }
            }
        }

        if has_values { Ok(Some(write_concern)) } else { Ok(None) }
    }

    fn parse_collation_value(value: &Value) -> Result<Collation, String> {
        let object = value
            .as_object()
            .ok_or_else(|| String::from("collation должен быть JSON-объектом."))?;
        let document =
            bson::to_document(object).map_err(|error| format!("BSON conversion error: {error}"))?;
        bson::from_document::<Collation>(document)
            .map_err(|error| format!("Collation parse error: {error}"))
    }

    fn parse_hint_value(value: &Value) -> Result<Hint, String> {
        match value {
            Value::String(name) => Ok(Hint::Name(name.clone())),
            Value::Object(map) => {
                let document = bson::to_document(map)
                    .map_err(|error| format!("BSON conversion error: {error}"))?;
                Ok(Hint::Keys(document))
            }
            _ => Err(String::from(
                "hint должен быть строкой или JSON-объектом со спецификацией индекса.",
            )),
        }
    }

    fn parse_non_negative_u64(value: &Value, field: &str) -> Result<u64, String> {
        match value {
            Value::Number(number) => number.as_u64().ok_or_else(|| {
                format!("Параметр '{field}' должен быть неотрицательным целым числом.",)
            }),
            _ => Err(format!("Параметр '{field}' должен быть неотрицательным целым числом.",)),
        }
    }

    fn split_arguments(args: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;

        for ch in args.chars() {
            if in_string {
                current.push(ch);
                if escape {
                    escape = false;
                } else if ch == '\\' {
                    escape = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => {
                    in_string = true;
                    current.push(ch);
                }
                '{' | '[' => {
                    depth += 1;
                    current.push(ch);
                }
                '}' | ']' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    result.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }

        result
    }

    fn parse_json_object(source: &str) -> Result<Document, String> {
        let value: Value =
            serde_json::from_str(source).map_err(|error| format!("JSON parse error: {error}"))?;
        let object =
            value.as_object().ok_or_else(|| String::from("Аргумент должен быть JSON-объектом"))?;
        bson::to_document(object).map_err(|error| format!("BSON conversion error: {error}"))
    }

    fn sanitize_numeric<S: AsRef<str>>(value: S) -> String {
        let filtered: String = value.as_ref().chars().filter(|ch| ch.is_ascii_digit()).collect();
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

    fn set_query_result(&mut self, result: QueryResult) {
        let start = Instant::now();

        let (tree, count) = match result {
            QueryResult::Documents(values) => {
                let count = values.len();
                (BsonTree::from_values(&values), count)
            }
            QueryResult::SingleDocument { document } => (BsonTree::from_document(document), 1),
            QueryResult::Distinct { field, values } => {
                let count = values.len();
                (BsonTree::from_distinct(field, values), count)
            }
            QueryResult::Count { value } => (BsonTree::from_count(value), 1),
        };

        let elapsed = start.elapsed();
        println!(
            "[table] collection='{}' documents={} processed_in_ms={:.3}",
            self.collection,
            count,
            elapsed.as_secs_f64() * 1000.0
        );

        self.bson_tree = tree;
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

        let connections = load_connections_from_disk().unwrap_or_else(|error| {
            eprintln!("Failed to load connections: {error}");
            Vec::new()
        });

        Self {
            panes,
            tabs: Vec::new(),
            active_tab: None,
            next_tab_id: 1,
            clients: Vec::new(),
            next_client_id: 1,
            last_collection_click: None,
            connections,
            mode: AppMode::Main,
            connections_window: None,
            connection_form: None,
            collection_modal: None,
            database_modal: None,
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
                match entry {
                    MenuEntry::Action(label) => {
                        if menu == TopMenu::File && label == "Соединения" {
                            self.open_connections_window();
                        } else {
                            println!("Menu '{menu:?}' entry '{label}' clicked");
                        }
                    }
                }
                Task::none()
            }
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
                    let _ = self.open_collection_tab(client_id, db_name, collection);
                } else {
                    self.last_collection_click =
                        Some(CollectionClick { client_id, db_name, collection, at: now });
                }

                Task::none()
            }
            Message::CollectionEditorAction { tab_id, action } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.editor.perform(action);
                }
                Task::none()
            }
            Message::DatabaseContextMenu { client_id, db_name, action } => match action {
                DatabaseContextAction::Refresh => self.refresh_databases(client_id),
                DatabaseContextAction::Stats => {
                    let tab_id = self.open_database_stats_tab(client_id, db_name.clone());
                    self.collection_query_task(tab_id)
                }
                DatabaseContextAction::Drop => {
                    self.database_modal = Some(DatabaseModalState::new_drop(client_id, db_name));
                    self.mode = AppMode::DatabaseModal;
                    Task::none()
                }
            },
            Message::CollectionContextMenu { client_id, db_name, collection, action } => {
                match action {
                    CollectionContextAction::OpenEmptyTab => {
                        let _ = self.open_collection_tab(client_id, db_name, collection);
                        Task::none()
                    }
                    CollectionContextAction::ViewDocuments => {
                        let tab_id = self.open_collection_tab(
                            client_id,
                            db_name.clone(),
                            collection.clone(),
                        );
                        self.collection_query_task(tab_id)
                    }
                    CollectionContextAction::DeleteTemplate => {
                        let tab_id =
                            self.open_collection_tab(client_id, db_name, collection.clone());
                        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                            let template = format!(
                                "db.getCollection('{collection_name}').deleteMany({{ '': '' }});",
                                collection_name = collection
                            );
                            tab.collection.editor = TextEditorContent::with_text(&template);
                        }
                        Task::none()
                    }
                    CollectionContextAction::DeleteAllDocuments => {
                        self.collection_modal = Some(CollectionModalState::new_delete_all(
                            client_id, db_name, collection,
                        ));
                        self.mode = AppMode::CollectionModal;
                        Task::none()
                    }
                    CollectionContextAction::DeleteCollection => {
                        self.collection_modal = Some(CollectionModalState::new_delete_collection(
                            client_id, db_name, collection,
                        ));
                        self.mode = AppMode::CollectionModal;
                        Task::none()
                    }
                    CollectionContextAction::RenameCollection => {
                        self.collection_modal =
                            Some(CollectionModalState::new_rename(client_id, db_name, collection));
                        self.mode = AppMode::CollectionModal;
                        Task::none()
                    }
                }
            }
            Message::CollectionSend(tab_id) => self.collection_query_task(tab_id),
            Message::CollectionModalInputChanged(value) => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    modal.input = value;
                    modal.error = None;
                }
                Task::none()
            }
            Message::CollectionModalCancel => {
                self.collection_modal = None;
                self.mode = AppMode::Main;
                Task::none()
            }
            Message::CollectionModalConfirm => {
                let Some(modal) = self.collection_modal.as_mut() else {
                    return Task::none();
                };

                if modal.processing {
                    return Task::none();
                }

                let trimmed_input = modal.input.trim().to_string();
                match modal.kind {
                    CollectionModalKind::DeleteAllDocuments
                    | CollectionModalKind::DeleteCollection => {
                        if trimmed_input != modal.collection {
                            modal.error = Some(String::from(
                                "Для подтверждения введите точное имя коллекции.",
                            ));
                            return Task::none();
                        }
                    }
                    CollectionModalKind::RenameCollection => {
                        if trimmed_input.is_empty() {
                            modal.error =
                                Some(String::from("Новое имя коллекции не может быть пустым."));
                            return Task::none();
                        }

                        if trimmed_input == modal.collection {
                            modal.error = Some(String::from(
                                "Новое имя коллекции должно отличаться от текущего.",
                            ));
                            return Task::none();
                        }
                    }
                }

                let client_id = modal.client_id;
                let db_name = modal.db_name.clone();
                let collection = modal.collection.clone();
                let kind = modal.kind;

                let handle = match self
                    .clients
                    .iter()
                    .find(|client| client.id == client_id)
                    .and_then(|client| client.handle.clone())
                {
                    Some(handle) => handle,
                    None => {
                        modal.error = Some(String::from("Нет активного соединения."));
                        return Task::none();
                    }
                };

                modal.processing = true;
                modal.error = None;

                match kind {
                    CollectionModalKind::DeleteAllDocuments => {
                        let future_db = db_name.clone();
                        let future_collection = collection.clone();
                        let message_db = db_name.clone();
                        let message_collection = collection.clone();
                        let handle_task = handle.clone();
                        Task::perform(
                            async move {
                                let database = handle_task.database(&future_db);
                                let coll = database.collection::<Document>(&future_collection);
                                coll.delete_many(Document::new())
                                    .run()
                                    .map(|result| result.deleted_count)
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::CollectionDeleteAllCompleted {
                                client_id,
                                db_name: message_db.clone(),
                                collection: message_collection.clone(),
                                result,
                            },
                        )
                    }
                    CollectionModalKind::DeleteCollection => {
                        let future_db = db_name.clone();
                        let future_collection = collection.clone();
                        let message_db = db_name.clone();
                        let message_collection = collection.clone();
                        let handle_task = handle.clone();
                        Task::perform(
                            async move {
                                let database = handle_task.database(&future_db);
                                let coll = database.collection::<Document>(&future_collection);
                                coll.drop().run().map_err(|error| error.to_string())
                            },
                            move |result| Message::CollectionDeleteCollectionCompleted {
                                client_id,
                                db_name: message_db.clone(),
                                collection: message_collection.clone(),
                                result,
                            },
                        )
                    }
                    CollectionModalKind::RenameCollection => {
                        let new_name = trimmed_input;
                        let future_db = db_name.clone();
                        let future_collection = collection.clone();
                        let future_new = new_name.clone();
                        let message_db = db_name.clone();
                        let message_old = collection.clone();
                        let message_new = new_name.clone();
                        let handle_task = handle.clone();
                        Task::perform(
                            async move {
                                let admin = handle_task.database("admin");
                                let command = doc! {
                                    "renameCollection": format!("{}.{}", future_db, future_collection),
                                    "to": format!("{}.{}", future_db, future_new),
                                    "dropTarget": false,
                                };
                                admin
                                    .run_command(command)
                                    .run()
                                    .map(|_| ())
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::CollectionRenameCompleted {
                                client_id,
                                db_name: message_db.clone(),
                                old_collection: message_old.clone(),
                                new_name: message_new.clone(),
                                result,
                            },
                        )
                    }
                }
            }
            Message::CollectionDeleteAllCompleted { client_id, db_name, collection, result } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if modal.kind == CollectionModalKind::DeleteAllDocuments
                        && modal.client_id == client_id
                        && modal.db_name == db_name
                        && modal.collection == collection
                    {
                        match result {
                            Ok(_) => {
                                self.collection_modal = None;
                                self.mode = AppMode::Main;
                            }
                            Err(error) => {
                                modal.processing = false;
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionDeleteCollectionCompleted {
                client_id,
                db_name,
                collection,
                result,
            } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if modal.kind == CollectionModalKind::DeleteCollection
                        && modal.client_id == client_id
                        && modal.db_name == db_name
                        && modal.collection == collection
                    {
                        match result {
                            Ok(()) => {
                                self.collection_modal = None;
                                self.mode = AppMode::Main;
                                self.remove_collection_from_tree(client_id, &db_name, &collection);
                            }
                            Err(error) => {
                                modal.processing = false;
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionRenameCompleted {
                client_id,
                db_name,
                old_collection,
                new_name,
                result,
            } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if modal.kind == CollectionModalKind::RenameCollection
                        && modal.client_id == client_id
                        && modal.db_name == db_name
                        && modal.collection == old_collection
                    {
                        match result {
                            Ok(()) => {
                                self.collection_modal = None;
                                self.mode = AppMode::Main;
                                self.rename_collection_in_tree(
                                    client_id,
                                    &db_name,
                                    &old_collection,
                                    &new_name,
                                );
                            }
                            Err(error) => {
                                modal.processing = false;
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::DatabaseModalInputChanged(value) => {
                if let Some(modal) = self.database_modal.as_mut() {
                    modal.input = value;
                    modal.error = None;
                }
                Task::none()
            }
            Message::DatabaseModalCancel => {
                self.database_modal = None;
                self.mode = AppMode::Main;
                Task::none()
            }
            Message::DatabaseModalConfirm => {
                let Some(modal) = self.database_modal.as_mut() else {
                    return Task::none();
                };

                if modal.processing {
                    return Task::none();
                }

                if modal.input.trim() != modal.db_name {
                    modal.error =
                        Some(String::from("Для подтверждения введите точное имя базы данных."));
                    return Task::none();
                }

                let client_id = modal.client_id;
                let db_name = modal.db_name.clone();

                let handle = match self
                    .clients
                    .iter()
                    .find(|client| client.id == client_id)
                    .and_then(|client| client.handle.clone())
                {
                    Some(handle) => handle,
                    None => {
                        modal.error = Some(String::from("Нет активного соединения."));
                        return Task::none();
                    }
                };

                modal.processing = true;
                modal.error = None;

                let future_db = db_name.clone();
                let handle_task = handle.clone();
                let message_db = db_name.clone();

                Task::perform(
                    async move {
                        let database = handle_task.database(&future_db);
                        database.drop().run().map_err(|error| error.to_string())
                    },
                    move |result| Message::DatabaseDropCompleted {
                        client_id,
                        db_name: message_db.clone(),
                        result,
                    },
                )
            }
            Message::DatabaseDropCompleted { client_id, db_name, result } => {
                if let Some(modal) = self.database_modal.as_mut() {
                    if modal.client_id == client_id && modal.db_name == db_name {
                        match result {
                            Ok(()) => {
                                self.database_modal = None;
                                self.mode = AppMode::Main;
                                self.remove_database_from_tree(client_id, &db_name);
                            }
                            Err(error) => {
                                modal.processing = false;
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::DatabasesRefreshed { client_id, result } => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    match result {
                        Ok(mut names) => {
                            names.sort_unstable();
                            client.databases = names.into_iter().map(DatabaseNode::new).collect();
                        }
                        Err(error) => {
                            eprintln!("Не удалось обновить список баз данных: {error}");
                            for database in &mut client.databases {
                                database.state = DatabaseState::Error(error.clone());
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionSkipChanged { tab_id, value } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.update_skip(value);
                }
                Task::none()
            }
            Message::CollectionLimitChanged { tab_id, value } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.update_limit(value);
                }
                Task::none()
            }
            Message::CollectionPaneResized { tab_id, split, ratio } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.resize_split(split, ratio);
                }
                Task::none()
            }
            Message::TableContextMenu { tab_id, node_id, action } => {
                let content = self
                    .tabs
                    .iter()
                    .find(|tab| tab.id == tab_id)
                    .and_then(|tab| tab.collection.table_context_content(node_id, action));

                if let Some(text) = content { clipboard::write(text) } else { Task::none() }
            }
            Message::CollectionSkipPrev(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.decrement_skip_by_limit();
                }
                self.collection_query_task(tab_id)
            }
            Message::CollectionSkipNext(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.increment_skip_by_limit();
                }
                self.collection_query_task(tab_id)
            }
            Message::CollectionQueryCompleted { tab_id, result, duration } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    let collection = &mut tab.collection;
                    collection.last_query_duration = Some(duration);
                    match result {
                        Ok(query_result) => collection.set_query_result(query_result),
                        Err(error) => collection.set_tree_error(error),
                    }
                }
                Task::none()
            }
            Message::ConnectionsCancel => {
                self.close_connections_window();
                Task::none()
            }
            Message::ConnectionsSelect(index) => {
                if let Some(state) = self.connections_window.as_mut() {
                    if index < self.connections.len() {
                        state.selected = Some(index);
                        state.confirm_delete = false;
                        state.last_click = Some(ListClick { index, at: Instant::now() });
                    }
                }
                Task::none()
            }
            Message::ConnectionsQuickConnect(index) => {
                if let Some(state) = self.connections_window.as_mut() {
                    state.selected = Some(index);
                }
                if let Some(entry) = self.connections.get(index).cloned() {
                    self.close_connections_window();
                    return self.add_connection_from_entry(entry);
                }
                Task::none()
            }
            Message::ConnectionsCreate => {
                self.open_connection_form(ConnectionFormMode::Create);
                Task::none()
            }
            Message::ConnectionsEdit => {
                if let Some(state) = &self.connections_window {
                    if let Some(index) = state.selected {
                        if index < self.connections.len() {
                            self.open_connection_form(ConnectionFormMode::Edit(index));
                        }
                    }
                }
                Task::none()
            }
            Message::ConnectionsDelete => {
                if let Some(state) = self.connections_window.as_mut() {
                    if state.selected.is_some() {
                        state.confirm_delete = true;
                    }
                }
                Task::none()
            }
            Message::ConnectionsDeleteCancelled => {
                if let Some(state) = self.connections_window.as_mut() {
                    state.confirm_delete = false;
                }
                Task::none()
            }
            Message::ConnectionsDeleteConfirmed => {
                if let Some(state) = self.connections_window.as_mut() {
                    if let Some(index) = state.selected {
                        if index < self.connections.len() {
                            self.connections.remove(index);
                            match save_connections_to_disk(&self.connections) {
                                Ok(()) => state.feedback = Some(String::from("Удалено")),
                                Err(error) => {
                                    state.feedback = Some(format!("Ошибка сохранения: {error}"));
                                }
                            }
                            if self.connections.is_empty() {
                                state.selected = None;
                            } else if index >= self.connections.len() {
                                state.selected = Some(self.connections.len() - 1);
                            }
                        }
                    }
                    state.confirm_delete = false;
                }
                Task::none()
            }
            Message::ConnectionsConnect => {
                if let Some(state) = &self.connections_window {
                    if let Some(index) = state.selected {
                        if let Some(entry) = self.connections.get(index) {
                            let task = self.add_connection_from_entry(entry.clone());
                            self.close_connections_window();
                            return task;
                        }
                    }
                }
                Task::none()
            }
            Message::ConnectionFormTabChanged(tab) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.active_tab = tab;
                }
                Task::none()
            }
            Message::ConnectionFormNameChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.name = value;
                }
                Task::none()
            }
            Message::ConnectionFormHostChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.host = value;
                }
                Task::none()
            }
            Message::ConnectionFormPortChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    let sanitized: String =
                        value.chars().filter(|ch| ch.is_ascii_digit()).take(5).collect();
                    form.port = sanitized;
                }
                Task::none()
            }
            Message::ConnectionFormIncludeAction(action) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.include_action(action);
                }
                Task::none()
            }
            Message::ConnectionFormExcludeAction(action) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.exclude_action(action);
                }
                Task::none()
            }
            Message::ConnectionFormAddSystemFilters => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.add_system_filters();
                }
                Task::none()
            }
            Message::ConnectionFormTest => {
                if let Some(form) = self.connection_form.as_mut() {
                    match form.validate() {
                        Ok(entry) => {
                            form.validation_error = None;
                            form.testing = true;
                            form.test_feedback = None;
                            let uri = entry.uri();
                            return Task::perform(
                                async move {
                                    Client::with_uri_str(&uri)
                                        .map(|_| ())
                                        .map_err(|err| err.to_string())
                                },
                                Message::ConnectionFormTestResult,
                            );
                        }
                        Err(error) => {
                            form.validation_error = Some(error);
                        }
                    }
                }
                Task::none()
            }
            Message::ConnectionFormTestResult(result) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.testing = false;
                    form.test_feedback = Some(match result {
                        Ok(()) => TestFeedback::Success(String::from("Соединение установлено")),
                        Err(error) => TestFeedback::Failure(error),
                    });
                }
                Task::none()
            }
            Message::ConnectionFormSave => {
                if let Some(form) = self.connection_form.as_mut() {
                    match form.validate() {
                        Ok(entry) => {
                            let result = match form.mode {
                                ConnectionFormMode::Create => {
                                    self.connections.push(entry);
                                    Ok(self.connections.len() - 1)
                                }
                                ConnectionFormMode::Edit(index) => {
                                    if let Some(slot) = self.connections.get_mut(index) {
                                        *slot = entry;
                                        Ok(index)
                                    } else {
                                        Err(String::from("Выбранное соединение не найдено"))
                                    }
                                }
                            };

                            match result {
                                Ok(selected_index) => {
                                    if let Err(error) = save_connections_to_disk(&self.connections)
                                    {
                                        if let Some(window) = self.connections_window.as_mut() {
                                            window.feedback =
                                                Some(format!("Ошибка сохранения: {error}"));
                                        }
                                    }

                                    self.open_connections_window();
                                    if let Some(window) = self.connections_window.as_mut() {
                                        window.selected = Some(selected_index);
                                        window.feedback = Some(String::from("Сохранено"));
                                    }
                                }
                                Err(error) => {
                                    form.validation_error = Some(error);
                                    return Task::none();
                                }
                            }
                        }
                        Err(error) => {
                            form.validation_error = Some(error);
                        }
                    }
                }
                Task::none()
            }
            Message::ConnectionFormCancel => {
                self.open_connections_window();
                Task::none()
            }
            Message::CollectionTreeToggle { tab_id, node_id } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.toggle_node(node_id);
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn main_view(&self) -> Element<Message> {
        let menu_bar = self.build_menu_bar();

        let content_grid =
            pane_grid::PaneGrid::new(&self.panes, |_, pane_state, _| match pane_state {
                PaneContent::Sidebar => pane_grid::Content::new(self.sidebar_panel()),
                PaneContent::Main => pane_grid::Content::new(self.main_panel()),
            })
            .on_resize(8, Message::PaneResized)
            .spacing(8)
            .height(Length::Fill);

        Column::new().push(menu_bar).push(content_grid).spacing(0).height(Length::Fill).into()
    }

    fn view(&self) -> Element<Message> {
        match self.mode {
            AppMode::Main => self.main_view(),
            AppMode::Connections => {
                if let Some(state) = &self.connections_window {
                    self.connections_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::ConnectionForm => {
                if let Some(state) = &self.connection_form {
                    self.connection_form_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::CollectionModal => {
                if let Some(state) = &self.collection_modal {
                    self.collection_modal_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::DatabaseModal => {
                if let Some(state) = &self.database_modal {
                    self.database_modal_view(state)
                } else {
                    self.main_view()
                }
            }
        }
    }

    fn connections_view(&self, state: &ConnectionsWindowState) -> Element<Message> {
        let border_color = Color::from_rgb8(0xba, 0xc5, 0xd6);
        let selected_bg = Color::from_rgb8(0xe9, 0xf0, 0xfa);
        let normal_bg = Color::from_rgb8(0xfc, 0xfd, 0xfe);
        let accent_bar = Color::from_rgb8(0x41, 0x82, 0xf2);

        let mut entries = Column::new().spacing(4).width(Length::Fill);

        if self.connections.is_empty() {
            entries = entries.push(
                Container::new(Text::new("Сохранённых соединений нет").size(16))
                    .width(Length::Fill)
                    .padding([12, 8]),
            );
        } else {
            for (index, entry) in self.connections.iter().enumerate() {
                let is_selected = state.selected == Some(index);
                let icon = Container::new(
                    Image::new(shared_icon_handle(&ICON_NETWORK_HANDLE, ICON_NETWORK_BYTES))
                        .width(Length::Fixed(28.0))
                        .height(Length::Fixed(28.0)),
                )
                .width(Length::Fixed(44.0))
                .height(Length::Fixed(44.0))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center);

                let name_text = Text::new(entry.name.clone())
                    .size(18)
                    .color(Color::from_rgb8(0x17, 0x1a, 0x20));
                let details_text = Text::new(format!("{}:{}", entry.host, entry.port))
                    .size(13)
                    .color(Color::from_rgb8(0x2f, 0x3b, 0x4b));

                let labels = Column::new().spacing(4).push(name_text).push(details_text);

                let filters_text = if entry.include_filter.trim().is_empty()
                    && entry.exclude_filter.trim().is_empty()
                {
                    Text::new("Фильтры не заданы")
                        .size(12)
                        .color(Color::from_rgb8(0x8a, 0x95, 0xa5))
                } else {
                    Text::new("Настроены фильтры коллекций")
                        .size(12)
                        .color(Color::from_rgb8(0x36, 0x71, 0xc9))
                };

                let right_info =
                    Column::new().spacing(4).align_x(Horizontal::Right).push(filters_text);

                let row = Row::new()
                    .spacing(16)
                    .align_y(Vertical::Center)
                    .push(icon)
                    .push(labels)
                    .push(Space::with_width(Length::Fill))
                    .push(right_info);

                let container =
                    Container::new(row).padding([8, 12]).width(Length::Fill).style(move |_| {
                        container::Style {
                            background: Some(
                                if is_selected { selected_bg } else { normal_bg }.into(),
                            ),
                            border: border::rounded(10).width(1).color(border_color),
                            shadow: Shadow {
                                color: Color::from_rgba8(0, 0, 0, 0.08),
                                offset: iced::Vector::new(0.0, 1.0),
                                blur_radius: 6.0,
                            },
                            ..Default::default()
                        }
                    });

                let accent = Container::new(Space::with_width(Length::Fixed(4.0)))
                    .height(Length::Fixed(44.0))
                    .style(move |_| container::Style {
                        background: Some(
                            if is_selected { accent_bar } else { Color::TRANSPARENT }.into(),
                        ),
                        ..Default::default()
                    });

                let mut button = Button::new(
                    Row::new().spacing(0).width(Length::Fill).push(accent).push(container),
                )
                .width(Length::Fill)
                .on_press(Message::ConnectionsSelect(index));

                if state.last_click.map_or(false, |last| {
                    last.index == index && last.at.elapsed() <= DOUBLE_CLICK_INTERVAL
                }) {
                    button = button.on_press(Message::ConnectionsQuickConnect(index));
                }

                entries = entries.push(button);
            }
        }

        let list = Scrollable::new(entries).width(Length::Fill).height(Length::Fixed(280.0));

        let mut left_controls = Row::new().spacing(8).push(
            Button::new(Text::new("Создать")).padding([6, 16]).on_press(Message::ConnectionsCreate),
        );

        let mut edit_button = Button::new(Text::new("Редактировать")).padding([6, 16]);
        if state.selected.is_some() {
            edit_button = edit_button.on_press(Message::ConnectionsEdit);
        }
        left_controls = left_controls.push(edit_button);

        let mut delete_button = Button::new(Text::new("Удалить")).padding([6, 16]);
        if state.selected.is_some() {
            delete_button = delete_button.on_press(Message::ConnectionsDelete);
        }
        left_controls = left_controls.push(delete_button);

        let mut connect_button = Button::new(Text::new("Соединить")).padding([6, 16]);
        if state.selected.is_some() {
            connect_button = connect_button.on_press(Message::ConnectionsConnect);
        }

        let right_controls = Row::new()
            .spacing(8)
            .push(
                Button::new(Text::new("Отменить"))
                    .padding([6, 16])
                    .on_press(Message::ConnectionsCancel),
            )
            .push(connect_button);

        let mut content =
            Column::new().spacing(16).push(Text::new("Соединения").size(24)).push(list);

        if let Some(feedback) = &state.feedback {
            let color = if feedback.starts_with("Ошибка") {
                Color::from_rgb8(0xd9, 0x53, 0x4f)
            } else {
                Color::from_rgb8(0x1e, 0x88, 0x3a)
            };
            content = content.push(Text::new(feedback.clone()).size(14).color(color));
        }

        if state.confirm_delete {
            let name = state
                .selected
                .and_then(|index| self.connections.get(index))
                .map(|entry| entry.name.clone())
                .unwrap_or_else(|| String::from("соединение"));
            let confirm_row = Row::new()
                .spacing(12)
                .align_y(Vertical::Center)
                .push(Text::new(format!("Удалить \"{}\"?", name)).size(14))
                .push(
                    Button::new(Text::new("Да"))
                        .padding([4, 12])
                        .on_press(Message::ConnectionsDeleteConfirmed),
                )
                .push(
                    Button::new(Text::new("Нет"))
                        .padding([4, 12])
                        .on_press(Message::ConnectionsDeleteCancelled),
                );
            content = content.push(confirm_row);
        }

        let controls_row = Row::new()
            .spacing(16)
            .align_y(Vertical::Center)
            .push(left_controls)
            .push(Space::with_width(Length::Fill))
            .push(right_controls);

        content = content.push(controls_row);

        let card =
            Container::new(content).padding(20).width(Length::Fixed(640.0)).style(Self::pane_style);

        Container::new(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn collection_modal_view(&self, state: &CollectionModalState) -> Element<Message> {
        let (title, warning, prompt, placeholder, confirm_label) = match state.kind {
            CollectionModalKind::DeleteAllDocuments => (
                "Удаление всех документов",
                format!(
                    "Будут удалены все документы из коллекции \"{}\" базы \"{}\". Это действие нельзя отменить.",
                    state.collection, state.db_name
                ),
                Some(format!(
                    "Подтвердите удаление всех документов введя название коллекции \"{}\".",
                    state.collection
                )),
                state.collection.as_str(),
                "Подтвердить удаление",
            ),
            CollectionModalKind::DeleteCollection => (
                "Удаление коллекции",
                format!(
                    "Коллекция \"{}\" в базе \"{}\" будет удалена вместе со всеми документами. Это действие нельзя отменить.",
                    state.collection, state.db_name
                ),
                Some(format!(
                    "Подтвердите удаление коллекции введя её название \"{}\".",
                    state.collection
                )),
                state.collection.as_str(),
                "Подтвердить удаление",
            ),
            CollectionModalKind::RenameCollection => (
                "Переименовать коллекцию",
                format!(
                    "Введите новое имя для коллекции \"{}\" в базе \"{}\".",
                    state.collection, state.db_name
                ),
                None,
                "Новое имя коллекции",
                "Переименовать",
            ),
        };

        let confirm_ready = match state.kind {
            CollectionModalKind::DeleteAllDocuments | CollectionModalKind::DeleteCollection => {
                state.input.trim() == state.collection && !state.processing
            }
            CollectionModalKind::RenameCollection => {
                let trimmed = state.input.trim();
                !trimmed.is_empty() && trimmed != state.collection && !state.processing
            }
        };

        let mut column = Column::new()
            .spacing(16)
            .push(Text::new(title).size(22).color(Color::from_rgb8(0x17, 0x1a, 0x20)))
            .push(Text::new(warning).size(14).color(Color::from_rgb8(0x31, 0x38, 0x4a)));

        if let Some(prompt) = prompt {
            column =
                column.push(Text::new(prompt).size(13).color(Color::from_rgb8(0x55, 0x5f, 0x73)));
        }

        let input_field = text_input(placeholder, &state.input)
            .padding([6, 10])
            .width(Length::Fill)
            .on_input(Message::CollectionModalInputChanged);

        column = column.push(input_field);

        if let Some(error) = &state.error {
            column = column
                .push(Text::new(error.clone()).size(13).color(Color::from_rgb8(0xd9, 0x53, 0x4f)));
        }

        if state.processing {
            column = column.push(
                Text::new("Выполнение операции...")
                    .size(13)
                    .color(Color::from_rgb8(0x36, 0x71, 0xc9)),
            );
        }

        let cancel_button = Button::new(Text::new("Отмена").size(14))
            .padding([6, 16])
            .on_press(Message::CollectionModalCancel);

        let mut confirm_button = Button::new(Text::new(confirm_label).size(14)).padding([6, 16]);
        if confirm_ready {
            confirm_button = confirm_button.on_press(Message::CollectionModalConfirm);
        } else {
            confirm_button = confirm_button.style(|_, _| button::Style {
                background: Some(Color::from_rgb8(0xe3, 0xe6, 0xeb).into()),
                text_color: Color::from_rgb8(0x8a, 0x93, 0xa3),
                border: border::rounded(6).width(1).color(Color::from_rgb8(0xd7, 0xdb, 0xe2)),
                shadow: Shadow::default(),
            });
        }

        let buttons = Row::new().spacing(12).push(cancel_button).push(confirm_button);

        column = column.push(buttons);

        let modal = Container::new(column).padding(24).width(Length::Fixed(420.0)).style(|_| {
            container::Style {
                background: Some(Color::from_rgb8(0xff, 0xff, 0xff).into()),
                border: border::rounded(12).width(1).color(Color::from_rgb8(0xd0, 0xd5, 0xdc)),
                shadow: Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

        Container::new(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgba8(0x16, 0x1a, 0x1f, 0.55).into()),
                ..Default::default()
            })
            .into()
    }

    fn database_modal_view(&self, state: &DatabaseModalState) -> Element<Message> {
        let warning = format!(
            "База данных \"{}\" будет полностью удалена вместе со всеми коллекциями и документами. Это действие нельзя отменить.",
            state.db_name
        );
        let prompt =
            format!("Подтвердите удаление всех данных, введя название базы \"{}\".", state.db_name);

        let confirm_ready = !state.processing && state.input.trim() == state.db_name;

        let mut column = Column::new()
            .spacing(16)
            .push(
                Text::new("Удаление базы данных")
                    .size(22)
                    .color(Color::from_rgb8(0x17, 0x1a, 0x20)),
            )
            .push(Text::new(warning).size(14).color(Color::from_rgb8(0x31, 0x38, 0x4a)))
            .push(Text::new(prompt).size(13).color(Color::from_rgb8(0x55, 0x5f, 0x73)));

        let input_field = text_input("Название базы данных", &state.input)
            .padding([6, 10])
            .width(Length::Fill)
            .on_input(Message::DatabaseModalInputChanged);

        column = column.push(input_field);

        if let Some(error) = &state.error {
            column = column
                .push(Text::new(error.clone()).size(13).color(Color::from_rgb8(0xd9, 0x53, 0x4f)));
        }

        if state.processing {
            column = column.push(
                Text::new("Выполнение операции...")
                    .size(13)
                    .color(Color::from_rgb8(0x36, 0x71, 0xc9)),
            );
        }

        let cancel_button = Button::new(Text::new("Отмена").size(14))
            .padding([6, 16])
            .on_press(Message::DatabaseModalCancel);

        let mut confirm_button =
            Button::new(Text::new("Подтвердить удаление").size(14)).padding([6, 16]);

        if confirm_ready {
            confirm_button = confirm_button.on_press(Message::DatabaseModalConfirm);
        } else {
            confirm_button = confirm_button.style(|_, _| button::Style {
                background: Some(Color::from_rgb8(0xe3, 0xe6, 0xeb).into()),
                text_color: Color::from_rgb8(0x8a, 0x93, 0xa3),
                border: border::rounded(6).width(1).color(Color::from_rgb8(0xd7, 0xdb, 0xe2)),
                shadow: Shadow::default(),
            });
        }

        let buttons = Row::new().spacing(12).push(cancel_button).push(confirm_button);
        column = column.push(buttons);

        let modal = Container::new(column).padding(24).width(Length::Fixed(420.0)).style(|_| {
            container::Style {
                background: Some(Color::from_rgb8(0xff, 0xff, 0xff).into()),
                border: border::rounded(12).width(1).color(Color::from_rgb8(0xd0, 0xd5, 0xdc)),
                shadow: Shadow {
                    color: Color::from_rgba8(0, 0, 0, 0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

        Container::new(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgba8(0x16, 0x1a, 0x1f, 0.55).into()),
                ..Default::default()
            })
            .into()
    }

    fn connection_form_view<'a>(&'a self, state: &'a ConnectionFormState) -> Element<'a, Message> {
        let title = match state.mode {
            ConnectionFormMode::Create => "Новое соединение",
            ConnectionFormMode::Edit(_) => "Редактирование соединения",
        };

        let bg_active = Color::from_rgb8(0xd6, 0xe8, 0xff);
        let bg_inactive = Color::from_rgb8(0xf6, 0xf7, 0xfa);
        let border_color = Color::from_rgb8(0xc2, 0xc8, 0xd3);

        let general_active = state.active_tab == ConnectionFormTab::General;
        let mut general_button =
            Button::new(Text::new("Общее").size(14)).padding([6, 16]).style(move |_, _| {
                button::Style {
                    background: Some((if general_active { bg_active } else { bg_inactive }).into()),
                    text_color: Color::BLACK,
                    border: border::rounded(6).width(1).color(border_color),
                    shadow: Shadow::default(),
                }
            });
        if !general_active {
            general_button = general_button
                .on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::General));
        }

        let filter_active = state.active_tab == ConnectionFormTab::Filter;
        let mut filter_button = Button::new(Text::new("Фильтр баз данных").size(14))
            .padding([6, 16])
            .style(move |_, _| button::Style {
                background: Some((if filter_active { bg_active } else { bg_inactive }).into()),
                text_color: Color::BLACK,
                border: border::rounded(6).width(1).color(border_color),
                shadow: Shadow::default(),
            });
        if !filter_active {
            filter_button = filter_button
                .on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::Filter));
        }

        let tabs_row = Row::new().spacing(8).push(general_button).push(filter_button);

        let tab_content: Element<_> = match state.active_tab {
            ConnectionFormTab::General => {
                let name_input = text_input("Название", &state.name)
                    .on_input(Message::ConnectionFormNameChanged)
                    .padding([6, 12])
                    .width(Length::Fill);

                let host_input = text_input("Адрес/Хост/IP", &state.host)
                    .on_input(Message::ConnectionFormHostChanged)
                    .padding([6, 12])
                    .width(Length::Fill);

                let port_input = text_input("Порт", &state.port)
                    .on_input(Message::ConnectionFormPortChanged)
                    .padding([6, 12])
                    .align_x(Horizontal::Center)
                    .width(Length::Fixed(120.0));

                Column::new()
                    .spacing(12)
                    .push(Text::new("Название").size(14))
                    .push(name_input)
                    .push(Text::new("Адрес/Хост/IP").size(14))
                    .push(host_input)
                    .push(Text::new("Порт").size(14))
                    .push(port_input)
                    .into()
            }
            ConnectionFormTab::Filter => {
                let include_editor = text_editor::TextEditor::new(&state.include_editor)
                    .on_action(Message::ConnectionFormIncludeAction)
                    .height(Length::Fixed(130.0));

                let exclude_editor = text_editor::TextEditor::new(&state.exclude_editor)
                    .on_action(Message::ConnectionFormExcludeAction)
                    .height(Length::Fixed(130.0));

                let add_system_filters =
                    Button::new(Text::new("Добавить фильтр системные базы данных").size(14))
                        .padding([6, 16])
                        .on_press(Message::ConnectionFormAddSystemFilters);

                Column::new()
                    .spacing(12)
                    .push(Text::new("Включить").size(14))
                    .push(include_editor)
                    .push(Text::new("Исключить").size(14))
                    .push(exclude_editor)
                    .push(add_system_filters)
                    .into()
            }
        };

        let mut content = Column::new()
            .spacing(16)
            .push(Text::new(title).size(24))
            .push(tabs_row)
            .push(tab_content);

        if let Some(error) = &state.validation_error {
            content = content
                .push(Text::new(error.clone()).size(14).color(Color::from_rgb8(0xd9, 0x53, 0x4f)));
        }

        if let Some(feedback) = &state.test_feedback {
            let color = if feedback.is_success() {
                Color::from_rgb8(0x1e, 0x88, 0x3a)
            } else {
                Color::from_rgb8(0xd9, 0x53, 0x4f)
            };
            content = content.push(Text::new(feedback.message()).size(14).color(color));
        }

        if state.testing {
            content = content.push(
                Text::new("Тестирование...").size(14).color(Color::from_rgb8(0x1e, 0x88, 0x3a)),
            );
        }

        let mut test_button = Button::new(Text::new("Тестировать")).padding([6, 16]);
        if !state.testing {
            test_button = test_button.on_press(Message::ConnectionFormTest);
        }

        let left_controls = Row::new().push(test_button);

        let right_controls = Row::new()
            .spacing(8)
            .push(
                Button::new(Text::new("Отменить"))
                    .padding([6, 16])
                    .on_press(Message::ConnectionFormCancel),
            )
            .push(
                Button::new(Text::new("Сохранить"))
                    .padding([6, 16])
                    .on_press(Message::ConnectionFormSave),
            );

        let controls_row = Row::new()
            .spacing(16)
            .align_y(Vertical::Center)
            .push(left_controls)
            .push(Space::with_width(Length::Fill))
            .push(right_controls);

        content = content.push(controls_row);

        let card =
            Container::new(content).padding(16).width(Length::Fixed(560.0)).style(Self::pane_style);

        Container::new(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
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

        let header_row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(text(indicator))
            .push(
                Image::new(shared_icon_handle(&ICON_NETWORK_HANDLE, ICON_NETWORK_BYTES))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0)),
            )
            .push(text(&client.name).size(16))
            .push(text(status_label.clone()).size(12));

        let mut column = Column::new().spacing(4).push(self.sidebar_button(
            header_row,
            0.0,
            Message::ToggleClient(client.id),
        ));

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

        Container::new(column)
            .style(move |_| container::Style {
                border: border::rounded(4.0).width(1),
                ..Default::default()
            })
            .into()
    }

    fn render_database<'a>(
        &'a self,
        client_id: ClientId,
        database: &'a DatabaseNode,
    ) -> Element<'a, Message> {
        let indicator = if database.expanded { "v" } else { ">" };
        let icon_size = 14.0;

        let db_row = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(text(indicator))
            .push(
                Image::new(shared_icon_handle(&ICON_DATABASE_HANDLE, ICON_DATABASE_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(text(&database.name));

        let base_button = self.sidebar_button(
            db_row,
            16.0,
            Message::ToggleDatabase { client_id, db_name: database.name.clone() },
        );

        let db_name_owned = database.name.clone();
        let menu = ContextMenu::new(base_button, move || {
            let mut menu = Column::new().spacing(2).padding([4, 6]);

            let make_button = |label: &str, message: Message| -> Element<Message> {
                Button::new(Text::new(label.to_owned()).size(14))
                    .padding([4, 8])
                    .on_press(message)
                    .into()
            };

            menu = menu.push(make_button(
                "Обновить",
                Message::DatabaseContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    action: DatabaseContextAction::Refresh,
                },
            ));

            menu = menu.push(make_button(
                "Статистика",
                Message::DatabaseContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    action: DatabaseContextAction::Stats,
                },
            ));

            menu = menu.push(make_button(
                "Удалить БД",
                Message::DatabaseContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    action: DatabaseContextAction::Drop,
                },
            ));

            menu.into()
        });

        let mut column = Column::new().spacing(4).push(menu);

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
        let icon_size = 12.0;

        let row = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_COLLECTION_HANDLE, ICON_COLLECTION_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(text(&collection.name).size(14));

        let base_button = self.sidebar_button(
            row,
            32.0,
            Message::CollectionClicked {
                client_id,
                db_name: db_name.to_owned(),
                collection: collection.name.clone(),
            },
        );

        let db_name_owned = db_name.to_owned();
        let collection_name = collection.name.clone();

        ContextMenu::new(base_button, move || {
            let mut menu = Column::new().spacing(2).padding([4, 6]);

            let make_button = |label: &str, message: Message| -> Element<Message> {
                Button::new(Text::new(label.to_owned()).size(14))
                    .padding([4, 8])
                    .on_press(message)
                    .into()
            };

            menu = menu.push(make_button(
                "Открыть пустую вкладку",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::OpenEmptyTab,
                },
            ));

            menu = menu.push(make_button(
                "Посмотреть документы",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::ViewDocuments,
                },
            ));

            menu = menu.push(make_button(
                "Удалить документы...",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::DeleteTemplate,
                },
            ));

            menu = menu.push(make_button(
                "Удалить все документы...",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::DeleteAllDocuments,
                },
            ));

            menu = menu.push(make_button(
                "Переименовать коллекцию...",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::RenameCollection,
                },
            ));

            menu = menu.push(make_button(
                "Удалить коллекцию...",
                Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name_owned.clone(),
                    collection: collection_name.clone(),
                    action: CollectionContextAction::DeleteCollection,
                },
            ));

            menu.into()
        })
        .into()
    }

    fn sidebar_button<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        indent: f32,
        on_press: Message,
    ) -> Element<'a, Message> {
        let button = Button::new(content)
            .padding([4, 4])
            .width(Length::Shrink)
            .height(Length::Shrink)
            .style(Self::sidebar_button_style)
            .on_press(on_press);

        Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(Space::with_width(Length::Fixed(indent.max(0.0))))
            .push(button)
            .into()
    }

    fn sidebar_button_style(theme: &Theme, status: button::Status) -> button::Style {
        let palette = theme.extended_palette();
        let base = Color::from_rgb8(0xf3, 0xf5, 0xfa);
        let hover = Color::from_rgb8(0xe8, 0xec, 0xf5);
        let pressed = Color::from_rgb8(0xdc, 0xe2, 0xef);
        let disabled = palette.background.weak.color;
        let background = match status {
            button::Status::Active => base,
            button::Status::Hovered => hover,
            button::Status::Pressed => pressed,
            button::Status::Disabled => disabled,
        };

        button::Style {
            background: Some(background.into()),
            text_color: Color::from_rgb8(0x22, 0x28, 0x38),
            border: border::rounded(6).width(1).color(Color::from_rgb8(0xc6, 0xcc, 0xd9)),
            shadow: Shadow::default(),
        }
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
            let active_id = self.active_tab.or_else(|| self.tabs.first().map(|tab| tab.id));

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

                let tab_container = Container::new(tab_inner).padding([4, 8]).style(move |_| {
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

            let header = Container::new(header_scroll).width(Length::Fill).padding([0, 4]);

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
        let connections_button = button(text("Соединения").size(16))
            .padding([6, 12])
            .on_press(Message::MenuItemSelected(TopMenu::File, MenuEntry::Action("Соединения")));

        let mut roots = Vec::new();
        roots.push(MenuItemWidget::new(connections_button));
        roots.push(self.menu_root(
            TopMenu::View,
            &[MenuEntry::Action("Explorer"), MenuEntry::Action("Refresh")],
        ));
        roots.push(self.menu_root(
            TopMenu::Options,
            &[MenuEntry::Action("Preferences"), MenuEntry::Action("Settings")],
        ));
        roots.push(self.menu_root(
            TopMenu::Windows,
            &[MenuEntry::Action("Cascade"), MenuEntry::Action("Tile")],
        ));
        roots.push(self.menu_root(
            TopMenu::Help,
            &[MenuEntry::Action("Documentation"), MenuEntry::Action("About")],
        ));

        MenuBar::new(roots).width(Length::Fill)
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

    fn pane_style(theme: &Theme) -> iced::widget::container::Style {
        let palette = theme.extended_palette();

        iced::widget::container::Style {
            background: Some(palette.background.weak.color.into()),
            border: border::rounded(6).width(1).color(palette.primary.weak.color),
            ..Default::default()
        }
    }

    fn open_collection_tab(
        &mut self,
        client_id: ClientId,
        db_name: String,
        collection: String,
    ) -> TabId {
        let mut client_name = String::from("Неизвестный клиент");
        let mut values = vec![Bson::String(String::from(
            "Запрос пока не выполнен. Сформируйте запрос и нажмите Send.",
        ))];

        if let Some(client) = self.clients.iter().find(|c| c.id == client_id) {
            client_name = client.name.clone();

            if client.handle.is_none() {
                values = vec![Bson::String(String::from(
                    "Соединение не активно. Повторите подключение, затем выполните запрос.",
                ))];
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
        id
    }

    fn open_database_stats_tab(&mut self, client_id: ClientId, db_name: String) -> TabId {
        let tab_id =
            self.open_collection_tab(client_id, db_name.clone(), String::from("(database)"));

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.collection.editor = TextEditorContent::with_text("db.stats()");
            tab.title = format!("{}.stats", db_name);
        }

        tab_id
    }

    fn refresh_databases(&mut self, client_id: ClientId) -> Task<Message> {
        let handle = match self
            .clients
            .iter()
            .find(|client| client.id == client_id)
            .and_then(|client| client.handle.clone())
        {
            Some(handle) => handle,
            None => return Task::none(),
        };

        if let Some(client) = self.clients.iter_mut().find(|client| client.id == client_id) {
            for database in &mut client.databases {
                database.state = DatabaseState::Loading;
            }
        }

        Task::perform(
            async move { handle.list_database_names().run().map_err(|error| error.to_string()) },
            move |result| Message::DatabasesRefreshed { client_id, result },
        )
    }

    fn remove_collection_from_tree(
        &mut self,
        client_id: ClientId,
        db_name: &str,
        collection: &str,
    ) {
        if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
            if let Some(database) = client.databases.iter_mut().find(|d| d.name == db_name) {
                database.collections.retain(|node| node.name != collection);
            }
        }

        if let Some(click) = &self.last_collection_click {
            if click.client_id == client_id
                && click.db_name == db_name
                && click.collection == collection
            {
                self.last_collection_click = None;
            }
        }

        let removed: HashSet<TabId> = self
            .tabs
            .iter()
            .filter(|tab| {
                tab.collection.client_id == client_id
                    && tab.collection.db_name == db_name
                    && tab.collection.collection == collection
            })
            .map(|tab| tab.id)
            .collect();

        if !removed.is_empty() {
            self.tabs.retain(|tab| !removed.contains(&tab.id));
            if let Some(active) = self.active_tab {
                if removed.contains(&active) {
                    self.active_tab = self.tabs.last().map(|tab| tab.id);
                }
            }
        }
    }

    fn remove_database_from_tree(&mut self, client_id: ClientId, db_name: &str) {
        if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
            client.databases.retain(|database| database.name != db_name);
        }

        if self
            .last_collection_click
            .as_ref()
            .is_some_and(|click| click.client_id == client_id && click.db_name == db_name)
        {
            self.last_collection_click = None;
        }

        self.tabs.retain(|tab| {
            !(tab.collection.client_id == client_id && tab.collection.db_name == db_name)
        });

        if let Some(active) = self.active_tab {
            if self.tabs.iter().all(|tab| tab.id != active) {
                self.active_tab = self.tabs.last().map(|tab| tab.id);
            }
        }
    }

    fn rename_collection_in_tree(
        &mut self,
        client_id: ClientId,
        db_name: &str,
        old_collection: &str,
        new_name: &str,
    ) {
        if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
            if let Some(database) = client.databases.iter_mut().find(|d| d.name == db_name) {
                if let Some(node) =
                    database.collections.iter_mut().find(|node| node.name == old_collection)
                {
                    node.name = new_name.to_string();
                }
                database.collections.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        for tab in &mut self.tabs {
            let collection = &mut tab.collection;
            if collection.client_id == client_id
                && collection.db_name == db_name
                && collection.collection == old_collection
            {
                collection.collection = new_name.to_string();
                tab.title = format!("{}.{}", db_name, new_name);
            }
        }

        if let Some(click) = &mut self.last_collection_click {
            if click.client_id == client_id
                && click.db_name == db_name
                && click.collection == old_collection
            {
                click.collection = new_name.to_string();
                click.at = Instant::now();
            }
        }
    }

    fn collection_query_task(&mut self, tab_id: TabId) -> Task<Message> {
        let mut request: Option<(ClientId, String, String, QueryOperation, u64, u64)> = None;

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            let collection = &mut tab.collection;
            let query_text = collection.editor.text().to_string();
            match collection.parse_query(&query_text) {
                Ok(operation) => {
                    let skip = collection.skip_value();
                    let limit = collection.limit_value();
                    collection.last_query_duration = None;
                    request = Some((
                        collection.client_id,
                        collection.db_name.clone(),
                        collection.collection.clone(),
                        operation,
                        skip,
                        limit,
                    ));
                }
                Err(error) => {
                    collection.set_tree_error(error);
                }
            }
        }

        let Some((client_id, db_name, collection_name, operation, skip, limit)) = request else {
            return Task::none();
        };

        let Some(handle) = self
            .clients
            .iter()
            .find(|client| client.id == client_id)
            .and_then(|client| client.handle.clone())
        else {
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                tab.collection.set_tree_error(String::from("Нет активного соединения"));
            }
            return Task::none();
        };

        Task::perform(
            async move {
                let started = Instant::now();
                let result =
                    run_collection_query(handle, db_name, collection_name, operation, skip, limit);
                (result, started.elapsed())
            },
            move |(result, duration)| Message::CollectionQueryCompleted {
                tab_id,
                result,
                duration,
            },
        )
    }

    fn open_connections_window(&mut self) {
        let mut state =
            self.connections_window.take().unwrap_or_else(|| ConnectionsWindowState::new(None));

        if let Some(selected) = state.selected {
            if self.connections.is_empty() {
                state.selected = None;
            } else if selected >= self.connections.len() {
                state.selected = Some(self.connections.len() - 1);
            }
        } else if !self.connections.is_empty() {
            state.selected = Some(0);
        }

        state.confirm_delete = false;
        self.connections_window = Some(state);
        self.connection_form = None;
        self.mode = AppMode::Connections;
    }

    fn close_connections_window(&mut self) {
        self.mode = AppMode::Main;
        self.connections_window = None;
        self.connection_form = None;
    }

    fn open_connection_form(&mut self, mode: ConnectionFormMode) {
        if let Some(window) = self.connections_window.as_mut() {
            window.confirm_delete = false;
        }
        let entry = match mode {
            ConnectionFormMode::Create => None,
            ConnectionFormMode::Edit(index) => self.connections.get(index),
        };
        self.connection_form = Some(ConnectionFormState::new(mode, entry));
        self.mode = AppMode::ConnectionForm;
    }

    fn add_connection_from_entry(&mut self, entry: ConnectionEntry) -> Task<Message> {
        let uri = entry.uri();
        let connection =
            OMDBConnection::from_uri(&uri, &entry.include_filter, &entry.exclude_filter);
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let mut client = OMDBClient::new(client_id, connection.clone());
        client.name = entry.name;
        self.clients.push(client);

        Task::perform(async move { connect_and_discover(connection) }, move |result| {
            Message::ConnectionCompleted { client_id, result }
        })
    }
}

impl TabData {
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
            collection: CollectionTab::new(client_id, client_name, db_name, collection, values),
        }
    }

    fn view(&self) -> Element<Message> {
        self.collection.view(self.id)
    }
}

impl TopMenu {
    fn label(self) -> &'static str {
        match self {
            TopMenu::File => "Соединения",
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
    fn from_uri(uri: &str, include_filter: &str, exclude_filter: &str) -> Self {
        Self::Uri {
            uri: uri.to_owned(),
            include_filter: include_filter.to_owned(),
            exclude_filter: exclude_filter.to_owned(),
        }
    }

    fn display_label(&self) -> String {
        match self {
            OMDBConnection::Uri { uri, .. } => uri.clone(),
        }
    }
}

impl OMDBClient {
    fn new(id: ClientId, connection: OMDBConnection) -> Self {
        Self {
            id,
            name: connection.display_label(),
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

fn shared_icon_handle(lock: &OnceLock<Handle>, bytes: &'static [u8]) -> Handle {
    lock.get_or_init(|| Handle::from_bytes(bytes.to_vec())).clone()
}

fn connect_and_discover(connection: OMDBConnection) -> Result<ConnectionBootstrap, String> {
    match connection {
        OMDBConnection::Uri { uri, include_filter, exclude_filter } => {
            let client = Client::with_uri_str(&uri).map_err(|err| err.to_string())?;
            let mut databases =
                client.list_database_names().run().map_err(|err| err.to_string())?;

            let include_items: Vec<_> =
                include_filter.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if !include_items.is_empty() {
                let include_set: HashSet<_> = include_items.into_iter().collect();
                databases.retain(|db| include_set.contains(db.as_str()));
            } else {
                let exclude_items: Vec<_> =
                    exclude_filter.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if !exclude_items.is_empty() {
                    let exclude_set: HashSet<_> = exclude_items.into_iter().collect();
                    databases.retain(|db| !exclude_set.contains(db.as_str()));
                }
            }

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
    operation: QueryOperation,
    skip: u64,
    limit: u64,
) -> Result<QueryResult, String> {
    let database = client.database(&db_name);
    let collection = database.collection::<Document>(&collection_name);

    match operation {
        QueryOperation::Find { filter } => {
            if limit == 0 {
                return Ok(QueryResult::Documents(Vec::new()));
            }

            let mut builder = collection.find(filter);
            if skip > 0 {
                builder = builder.skip(skip);
            }

            let limit_capped = limit.min(i64::MAX as u64) as i64;
            if limit_capped > 0 {
                builder = builder.limit(limit_capped);
            }

            let cursor = builder.run().map_err(|err| err.to_string())?;
            let take_limit = if limit_capped > 0 { limit_capped as usize } else { usize::MAX };
            let mut documents = Vec::new();

            for result in cursor.into_iter().take(take_limit) {
                let document = result.map_err(|err| err.to_string())?;
                documents.push(Bson::Document(document));
            }

            Ok(QueryResult::Documents(documents))
        }
        QueryOperation::FindOne { filter } => {
            let mut builder = collection.find(filter);
            if skip > 0 {
                builder = builder.skip(skip);
            }
            builder = builder.limit(1);

            let cursor = builder.run().map_err(|err| err.to_string())?;
            if let Some(result) = cursor.into_iter().next() {
                let document = result.map_err(|err| err.to_string())?;
                Ok(QueryResult::SingleDocument { document })
            } else {
                Ok(QueryResult::Documents(Vec::new()))
            }
        }
        QueryOperation::Count { filter } => {
            let count = collection.count_documents(filter).run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::CountDocuments { filter, options } => {
            let mut builder = collection.count_documents(filter);

            if let Some(opts) = options {
                if let Some(limit) = opts.limit {
                    builder = builder.limit(limit);
                }
                if let Some(skip) = opts.skip {
                    builder = builder.skip(skip);
                }
                if let Some(max_time) = opts.max_time {
                    builder = builder.max_time(max_time);
                }
                if let Some(hint) = opts.hint {
                    builder = builder.hint(hint);
                }
            }

            let count = builder.run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::EstimatedDocumentCount { options } => {
            let mut builder = collection.estimated_document_count();

            if let Some(opts) = options {
                if let Some(max_time) = opts.max_time {
                    builder = builder.max_time(max_time);
                }
            }

            let count = builder.run().map_err(|err| err.to_string())?;

            let count_value = if count <= i64::MAX as u64 {
                Bson::Int64(count as i64)
            } else {
                Bson::String(count.to_string())
            };

            Ok(QueryResult::Count { value: count_value })
        }
        QueryOperation::Distinct { field, filter } => {
            let values =
                collection.distinct(field.clone(), filter).run().map_err(|err| err.to_string())?;

            Ok(QueryResult::Distinct { field, values })
        }
        QueryOperation::Aggregate { mut pipeline } => {
            if skip > 0 {
                let skip_i64 = i64::try_from(skip).unwrap_or(i64::MAX);
                pipeline.push(doc! { "$skip": skip_i64 });
            }

            if limit > 0 {
                let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
                pipeline.push(doc! { "$limit": limit_i64 });
            }

            let cursor = collection.aggregate(pipeline).run().map_err(|err| err.to_string())?;

            let mut documents = Vec::new();
            for result in cursor {
                let document = result.map_err(|err| err.to_string())?;
                documents.push(Bson::Document(document));
            }

            Ok(QueryResult::Documents(documents))
        }
        QueryOperation::InsertOne { document, options } => {
            let mut action = collection.insert_one(document);
            if let Some(opts) = options {
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from("insertOne")));
            response.insert("insertedId", result.inserted_id);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::InsertMany { documents, options } => {
            let mut action = collection.insert_many(documents);
            if let Some(opts) = options {
                if let Some(ordered) = opts.ordered {
                    action = action.ordered(ordered);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let mut pairs: Vec<(usize, Bson)> = result.inserted_ids.into_iter().collect();
            pairs.sort_by_key(|(index, _)| *index);

            let mut ids_document = Document::new();
            for (index, id) in pairs {
                ids_document.insert(index.to_string(), id);
            }

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from("insertMany")));
            response.insert("insertedCount", Bson::Int64(ids_document.len() as i64));
            response.insert("insertedIds", Bson::Document(ids_document));

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::DeleteOne { filter, options } => {
            let mut action = collection.delete_one(filter);
            if let Some(opts) = options {
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let deleted_count = result.deleted_count;
            let deleted_bson = if deleted_count <= i64::MAX as u64 {
                Bson::Int64(deleted_count as i64)
            } else {
                Bson::String(deleted_count.to_string())
            };

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from("deleteOne")));
            response.insert("deletedCount", deleted_bson);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::DeleteMany { filter, options } => {
            let mut action = collection.delete_many(filter);
            if let Some(opts) = options {
                if let Some(collation) = opts.collation {
                    action = action.collation(collation);
                }
                if let Some(hint) = opts.hint {
                    action = action.hint(hint);
                }
                if let Some(write_concern) = opts.write_concern {
                    action = action.write_concern(write_concern);
                }
            }

            let result = action.run().map_err(|err| err.to_string())?;
            let deleted_count = result.deleted_count;
            let deleted_bson = if deleted_count <= i64::MAX as u64 {
                Bson::Int64(deleted_count as i64)
            } else {
                Bson::String(deleted_count.to_string())
            };

            let mut response = Document::new();
            response.insert("operation", Bson::String(String::from("deleteMany")));
            response.insert("deletedCount", deleted_bson);

            Ok(QueryResult::SingleDocument { document: response })
        }
        QueryOperation::DatabaseCommand { db, command } => {
            let database = client.database(&db);
            let document = database.run_command(command).run().map_err(|err| err.to_string())?;
            Ok(QueryResult::SingleDocument { document })
        }
    }
}

fn connections_file_path() -> PathBuf {
    PathBuf::from(CONNECTIONS_FILE)
}

fn load_connections_from_disk() -> Result<Vec<ConnectionEntry>, String> {
    let path = connections_file_path();
    let data = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.to_string()),
    };

    let store: ConnectionStore = toml::from_str(&data).map_err(|err| err.to_string())?;
    Ok(store.connections)
}

fn save_connections_to_disk(connections: &[ConnectionEntry]) -> Result<(), String> {
    let store = ConnectionStore { connections: connections.to_vec() };
    let data = toml::to_string_pretty(&store).map_err(|err| err.to_string())?;
    let path = connections_file_path();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
    }
    let mut file = fs::File::create(path).map_err(|err| err.to_string())?;
    file.write_all(data.as_bytes()).map_err(|err| err.to_string())
}

#[derive(Debug, Clone, Default)]
struct CountDocumentsParsedOptions {
    limit: Option<u64>,
    skip: Option<u64>,
    hint: Option<Hint>,
    max_time: Option<Duration>,
}

impl CountDocumentsParsedOptions {
    fn has_values(&self) -> bool {
        self.limit.is_some()
            || self.skip.is_some()
            || self.hint.is_some()
            || self.max_time.is_some()
    }
}

#[derive(Debug, Clone, Default)]
struct EstimatedDocumentCountParsedOptions {
    max_time: Option<Duration>,
}

impl EstimatedDocumentCountParsedOptions {
    fn has_values(&self) -> bool {
        self.max_time.is_some()
    }
}

#[derive(Debug, Clone, Default)]
struct InsertOneParsedOptions {
    write_concern: Option<WriteConcern>,
}

impl InsertOneParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some()
    }
}

#[derive(Debug, Clone, Default)]
struct InsertManyParsedOptions {
    write_concern: Option<WriteConcern>,
    ordered: Option<bool>,
}

impl InsertManyParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some() || self.ordered.is_some()
    }
}

#[derive(Debug, Clone, Default)]
struct DeleteParsedOptions {
    write_concern: Option<WriteConcern>,
    collation: Option<Collation>,
    hint: Option<Hint>,
}

impl DeleteParsedOptions {
    fn has_values(&self) -> bool {
        self.write_concern.is_some() || self.collation.is_some() || self.hint.is_some()
    }
}

#[derive(Debug, Clone)]
enum QueryOperation {
    Find { filter: Document },
    FindOne { filter: Document },
    Count { filter: Document },
    CountDocuments { filter: Document, options: Option<CountDocumentsParsedOptions> },
    EstimatedDocumentCount { options: Option<EstimatedDocumentCountParsedOptions> },
    Distinct { field: String, filter: Document },
    Aggregate { pipeline: Vec<Document> },
    InsertOne { document: Document, options: Option<InsertOneParsedOptions> },
    InsertMany { documents: Vec<Document>, options: Option<InsertManyParsedOptions> },
    DeleteOne { filter: Document, options: Option<DeleteParsedOptions> },
    DeleteMany { filter: Document, options: Option<DeleteParsedOptions> },
    DatabaseCommand { db: String, command: Document },
}

#[derive(Debug, Clone)]
enum QueryResult {
    Documents(Vec<Bson>),
    SingleDocument { document: Document },
    Distinct { field: String, values: Vec<Bson> },
    Count { value: Bson },
}
