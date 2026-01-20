#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod fonts;
mod i18n;
mod logging;
mod mongo;
mod settings;
mod ui;

use crate::fonts::{MONO_FONT, MONO_FONT_BYTES};
use i18n::{tr, tr_format};
use iced::alignment::{Horizontal, Vertical};
use iced::border;
use iced::font::Weight;
use iced::futures::stream;
use iced::keyboard::{self, key};
use iced::theme::{Base, Mode};
use iced::widget::image::Handle;
use iced::widget::operation::{focus, focus_next};
use iced::widget::pane_grid::ResizeEvent;
use iced::widget::scrollable;
use iced::widget::text::Wrapping;
use iced::widget::text_editor::{
    self, Action as TextEditorAction, Binding as TextEditorBinding, Content as TextEditorContent,
};
use iced::widget::{
    Button, Column, Container, Id, Image, Row, Scrollable, Space, button, container, mouse_area,
    pane_grid, text_input,
};
use iced::window;
use iced::{
    Color, Element, Font, Length, Padding, Shadow, Size, Subscription, Task, Theme, application,
    clipboard,
};
use iced_aw::{ColorPicker, ContextMenu};
use iced_fonts::REQUIRED_FONT_BYTES;
use mongo::bson_edit::ValueEditKind;
use mongo::bson_tree::{BsonTree, BsonTreeOptions};
use mongo::connection::{
    ConnectionBootstrap, OMDBConnection, connect_and_discover, fetch_collections, filter_databases,
};
use mongo::query::{
    QueryOperation, QueryResult, ReplicaSetCommand, WatchTarget, open_change_stream,
    parse_collection_query_with_collection, run_collection_query,
};
use mongo::shell;
use mongo::ssh_tunnel::SshTunnel;
use mongodb::bson::{self, Bson, Document, doc};
use mongodb::change_stream::event::ChangeStreamEvent;
use mongodb::options::ReturnDocument;
use mongodb::sync::Client;
use rfd::FileDialog;
use settings::{AppSettings, LogLevel, ThemeChoice, ThemePalette};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ui::connections::{
    AuthMechanismChoice, ConnectionEntry, ConnectionFormMode, ConnectionFormState,
    ConnectionFormTab, ConnectionType, ConnectionsWindowState, ListClick, PasswordStorage,
    SshAuthMethod, TestFeedback, connection_form_view, connections_view,
    load_connections_from_disk, save_connections_to_disk,
};
use ui::help_docs::{HelpDocsState, help_docs_view};
use ui::menues::{
    self, CollectionContextAction, ConnectionContextAction, DatabaseContextAction, MenuEntry,
    TopMenu,
};
use ui::modal::{color_luminance, error_accent_color, modal_layout, success_accent_color};
use ui::settings::{SettingsTab, SettingsWindowState, ThemeColorField, settings_view};
pub(crate) type TabId = u32;
pub(crate) type ClientId = u32;

pub(crate) const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);
const DEFAULT_RESULT_LIMIT: i64 = 50;
const DEFAULT_RESULT_SKIP: u64 = 0;
const PANE_GRID_SPACING: f32 = 8.0;
const PANE_GRID_MIN_SIZE: f32 = 0.0;
const MAIN_PANEL_PADDING: f32 = 8.0;
const TAB_HEADER_PADDING_X: f32 = 4.0;
const TAB_ROW_SPACING: f32 = 8.0;
const TAB_INNER_SPACING: f32 = 4.0;
const TAB_TITLE_PADDING_X: f32 = 12.0;
const TAB_CONTAINER_PADDING_X: f32 = 8.0;
const TAB_CLOSE_PADDING_X: f32 = 8.0;
const TAB_CHAR_WIDTH_FACTOR: f32 = 0.6;
const TAB_WIDTH_TOLERANCE: f32 = 12.0;
const TAB_SCROLLBAR_PADDING: f32 = 10.0;
const WINDOW_ICON_BYTES: &[u8] = include_bytes!("../assests/icons/oxide_mongo_256x256.png");
pub(crate) const ICON_NETWORK_BYTES: &[u8] = include_bytes!("../assests/icons/network_115x128.png");
const ICON_DATABASE_BYTES: &[u8] = include_bytes!("../assests/icons/database_105x128.png");
const ICON_COLLECTION_BYTES: &[u8] = include_bytes!("../assests/icons/collection_108x128.png");
pub(crate) static ICON_NETWORK_HANDLE: OnceLock<Handle> = OnceLock::new();
static ICON_APP_HANDLE: OnceLock<Handle> = OnceLock::new();
static ICON_DATABASE_HANDLE: OnceLock<Handle> = OnceLock::new();
static ICON_COLLECTION_HANDLE: OnceLock<Handle> = OnceLock::new();

fn main() -> iced::Result {
    let icon = window::icon::from_file_data(WINDOW_ICON_BYTES, None)
        .map_err(|error| iced::Error::WindowCreationFailed(Box::new(error)))?;

    let mut window_settings = window::Settings::default();
    window_settings.icon = Some(icon);
    window_settings.size.width += 280.0;

    application(App::init, App::update, App::view)
        .title("Oxide Mongo")
        .subscription(App::subscription)
        .theme(App::theme)
        .font(MONO_FONT_BYTES)
        .font(REQUIRED_FONT_BYTES)
        .window(window_settings)
        .run()
}

pub(crate) struct App {
    panes: pane_grid::State<PaneContent>,
    tabs: Vec<TabData>,
    active_tab: Option<TabId>,
    next_tab_id: TabId,
    clients: Vec<OMDBClient>,
    next_client_id: ClientId,
    last_collection_click: Option<CollectionClick>,
    connections: Vec<ConnectionEntry>,
    settings: AppSettings,
    mode: AppMode,
    connections_window: Option<ConnectionsWindowState>,
    connection_form: Option<ConnectionFormState>,
    settings_window: Option<SettingsWindowState>,
    settings_error_modal: Option<SettingsErrorModalState>,
    help_docs_state: Option<HelpDocsState>,
    collection_modal: Option<CollectionModalState>,
    database_modal: Option<DatabaseModalState>,
    document_modal: Option<DocumentModalState>,
    value_edit_modal: Option<ValueEditModalState>,
    window_size: Option<Size>,
    tab_color_picker: Option<TabId>,
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
    color: Option<Color>,
}

#[derive(Debug, Clone)]
struct SettingsErrorModalState {
    message: String,
}

impl SettingsErrorModalState {
    fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    MenuItemSelected(TopMenu, MenuEntry),
    TabSelected(TabId),
    TabClosed(TabId),
    CloseActiveTab,
    DuplicateTab(TabId),
    TabColorPickerOpened(TabId),
    TabColorPickerCanceled,
    TabColorChanged {
        tab_id: TabId,
        color: Color,
    },
    TabColorReset(TabId),
    WindowEvent(window::Event),
    KeyboardEvent(keyboard::Event),
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
    ConnectionContextMenu {
        client_id: ClientId,
        action: ConnectionContextAction,
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
    CollectionTextCopyJson {
        tab_id: TabId,
    },
    DocumentEditRequested {
        tab_id: TabId,
        node_id: usize,
    },
    ValueEditModalEditorAction(TextEditorAction),
    ValueEditModalSave,
    ValueEditModalCancel,
    ValueEditModalCompleted {
        tab_id: TabId,
        result: Result<Document, String>,
    },
    AboutModalClose,
    LicensesModalClose,
    HelpDocsClose,
    HelpDocsSectionSelected(usize),
    HelpDocsSearchChanged(String),
    OpenUrl(&'static str),
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
    CollectionWatchProgress {
        tab_id: TabId,
        documents: Vec<Bson>,
    },
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
    ConnectionFormTypeChanged(ConnectionType),
    ConnectionFormAuthUseChanged(bool),
    ConnectionFormAuthLoginChanged(String),
    ConnectionFormAuthPasswordChanged(String),
    ConnectionFormPasswordStorageChanged(PasswordStorage),
    ConnectionFormAuthMechanismChanged(AuthMechanismChoice),
    ConnectionFormAuthDatabaseChanged(String),
    ConnectionFormSshUseChanged(bool),
    ConnectionFormSshHostChanged(String),
    ConnectionFormSshPortChanged(String),
    ConnectionFormSshUsernameChanged(String),
    ConnectionFormSshAuthMethodChanged(SshAuthMethod),
    ConnectionFormSshPasswordChanged(String),
    ConnectionFormSshPrivateKeyChanged(String),
    ConnectionFormSshPrivateKeyBrowse,
    ConnectionFormSshPrivateKeyPicked(Option<String>),
    ConnectionFormSshPassphraseChanged(String),
    ConnectionFormIncludeAction(TextEditorAction),
    ConnectionFormExcludeAction(TextEditorAction),
    ConnectionFormAddSystemFilters,
    ConnectionFormTest,
    ConnectionFormTestResult(Result<(), String>),
    ConnectionFormSave,
    ConnectionFormCancel,
    SettingsOpen,
    SettingsTabChanged(SettingsTab),
    SettingsToggleExpandFirstResult(bool),
    SettingsQueryTimeoutChanged(String),
    SettingsToggleSortFields(bool),
    SettingsToggleSortIndexes(bool),
    SettingsToggleCloseTabsOnDbClose(bool),
    SettingsToggleLogging(bool),
    SettingsLogLevelChanged(LogLevel),
    SettingsLogPathChanged(String),
    SettingsLanguageChanged(i18n::Language),
    SettingsPrimaryFontDropdownToggled,
    SettingsPrimaryFontChanged(String),
    SettingsPrimaryFontSizeChanged(String),
    SettingsResultFontDropdownToggled,
    SettingsResultFontChanged(String),
    SettingsResultFontSizeChanged(String),
    SettingsThemeChanged(ThemeChoice),
    SettingsColorPickerOpened(ThemeColorField),
    SettingsColorPickerCanceled,
    SettingsColorChanged(ThemeColorField, Color),
    SettingsThemeColorsReset,
    SettingsApply,
    SettingsSave,
    SettingsCancel,
    SettingsLoadErrorExit,
    SettingsLoadErrorUseDefaults,
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
    CollectionCreateCompleted {
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
    CollectionDropIndexCompleted {
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        index_name: String,
        result: Result<(), String>,
    },
    CollectionHideIndexCompleted {
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        index_name: String,
        result: Result<(), String>,
    },
    CollectionUnhideIndexCompleted {
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        index_name: String,
        result: Result<(), String>,
    },
    DatabaseModalInputChanged(String),
    DatabaseModalCollectionInputChanged(String),
    DatabaseModalConfirm,
    DatabaseModalCancel,
    DatabaseDropCompleted {
        client_id: ClientId,
        db_name: String,
        result: Result<(), String>,
    },
    DatabaseCreateCompleted {
        client_id: ClientId,
        _db_name: String,
        result: Result<(), String>,
    },
    DocumentModalEditorAction(TextEditorAction),
    DocumentModalSave,
    DocumentModalCancel,
    DocumentModalCompleted {
        tab_id: TabId,
        result: Result<Document, String>,
    },
    DatabasesRefreshed {
        client_id: ClientId,
        result: Result<Vec<String>, String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableContextAction {
    CopyJson,
    CopyKey,
    CopyValue,
    CopyPath,
    EditValue,
    DeleteIndex,
    HideIndex,
    UnhideIndex,
    ExpandHierarchy,
    CollapseHierarchy,
    ExpandHierarchyAll,
    CollapseHierarchyAll,
}

#[derive(Debug)]
struct OMDBClient {
    id: ClientId,
    name: String,
    status: ConnectionStatus,
    expanded: bool,
    handle: Option<Arc<Client>>,
    databases: Vec<DatabaseNode>,
    ssh_tunnel: Option<Arc<Mutex<SshTunnel>>>,
    entry: ConnectionEntry,
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

impl CollectionModalState {
    fn new_create(client_id: ClientId, db_name: String) -> Self {
        Self {
            client_id,
            db_name,
            collection: String::new(),
            kind: CollectionModalKind::CreateCollection,
            input: String::new(),
            error: None,
            processing: false,
            origin_tab: None,
        }
    }

    fn new_delete_all(client_id: ClientId, db_name: String, collection: String) -> Self {
        Self {
            client_id,
            db_name,
            collection,
            kind: CollectionModalKind::DeleteAllDocuments,
            input: String::new(),
            error: None,
            processing: false,
            origin_tab: None,
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
            origin_tab: None,
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
            origin_tab: None,
        }
    }

    fn new_drop_index(
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        index_name: String,
    ) -> Self {
        Self {
            client_id,
            db_name,
            collection,
            kind: CollectionModalKind::DropIndex { index_name },
            input: String::new(),
            error: None,
            processing: false,
            origin_tab: Some(tab_id),
        }
    }
}

impl DatabaseModalState {
    fn new_drop(client_id: ClientId, db_name: String) -> Self {
        Self {
            client_id,
            mode: DatabaseModalMode::Drop { db_name },
            input: String::new(),
            collection_input: String::new(),
            error: None,
            processing: false,
        }
    }

    fn new_create(client_id: ClientId) -> Self {
        Self {
            client_id,
            mode: DatabaseModalMode::Create,
            input: String::new(),
            collection_input: String::new(),
            error: None,
            processing: false,
        }
    }
}

impl DocumentModalState {
    fn new_collection_document(
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        document: Document,
    ) -> Option<Self> {
        let original_id = document.get("_id")?.clone();
        let filter = doc! { "_id": original_id.clone() };
        let text = shell::format_bson_shell(&Bson::Document(document.clone()));

        Some(Self {
            tab_id,
            client_id,
            db_name,
            collection,
            kind: DocumentModalKind::CollectionDocument { filter, original_id },
            editor: TextEditorContent::with_text(&text),
            error: None,
            processing: false,
        })
    }

    fn new_index(
        tab_id: TabId,
        client_id: ClientId,
        db_name: String,
        collection: String,
        document: Document,
    ) -> Option<Self> {
        if !document.contains_key("expireAfterSeconds") {
            return None;
        }
        let name = document.get("name")?.as_str()?.to_string();
        let text = shell::format_bson_shell(&Bson::Document(document.clone()));

        Some(Self {
            tab_id,
            client_id,
            db_name,
            collection,
            kind: DocumentModalKind::Index { name },
            editor: TextEditorContent::with_text(&text),
            error: None,
            processing: false,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppMode {
    Main,
    Connections,
    ConnectionForm,
    Settings,
    SettingsLoadError,
    About,
    Licenses,
    HelpDocs,
    CollectionModal,
    DatabaseModal,
    DocumentModal,
    ValueEditModal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CollectionModalKind {
    CreateCollection,
    DeleteAllDocuments,
    DeleteCollection,
    RenameCollection,
    DropIndex { index_name: String },
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
    origin_tab: Option<TabId>,
}

#[derive(Debug, Clone)]
struct DatabaseModalState {
    client_id: ClientId,
    mode: DatabaseModalMode,
    input: String,
    collection_input: String,
    error: Option<String>,
    processing: bool,
}

#[derive(Debug, Clone)]
enum DatabaseModalMode {
    Drop { db_name: String },
    Create,
}

#[derive(Debug)]
struct DocumentModalState {
    tab_id: TabId,
    client_id: ClientId,
    db_name: String,
    collection: String,
    kind: DocumentModalKind,
    editor: TextEditorContent,
    error: Option<String>,
    processing: bool,
}

#[derive(Debug, Clone)]
enum DocumentModalKind {
    CollectionDocument { filter: Document, original_id: Bson },
    Index { name: String },
}

#[derive(Debug, Clone)]
struct ValueEditContext {
    path: String,
    filter: Document,
    current_value: Bson,
}

#[derive(Debug)]
struct ValueEditModalState {
    tab_id: TabId,
    client_id: ClientId,
    db_name: String,
    collection: String,
    filter: Document,
    path: String,
    value_input: String,
    value_editor: TextEditorContent,
    value_kind: ValueEditKind,
    value_label: String,
    error: Option<String>,
    processing: bool,
}

impl ValueEditModalState {
    fn new(tab_id: TabId, collection: &CollectionTab, context: ValueEditContext) -> Self {
        let value_input = Self::initial_value_input(&context.current_value);
        let value_kind = ValueEditKind::from_bson(&context.current_value);
        let value_label = shell::bson_type_name(&context.current_value).to_string();
        Self {
            tab_id,
            client_id: collection.client_id,
            db_name: collection.db_name.clone(),
            collection: collection.collection.clone(),
            filter: context.filter,
            path: context.path,
            value_editor: TextEditorContent::with_text(&value_input),
            value_input,
            value_kind,
            value_label,
            error: None,
            processing: false,
        }
    }

    fn initial_value_input(value: &Bson) -> String {
        match value {
            Bson::String(text) => text.clone(),
            _ => shell::format_shell_value(value),
        }
    }

    fn apply_editor_action(&mut self, action: TextEditorAction) {
        self.value_editor.perform(action);
        self.value_input = self.value_editor.text().to_string();
        self.recalculate_kind_and_label();
        self.error = None;
    }

    fn recalculate_kind_and_label(&mut self) {
        if let Ok(bson) = shell::parse_shell_bson_value(&self.value_input) {
            self.value_kind = ValueEditKind::from_bson(&bson);
            self.value_label = shell::bson_type_name(&bson).to_string();
        } else if let Some(kind) = ValueEditKind::infer(&self.value_input) {
            self.value_kind = kind;
            self.value_label = kind.label().to_string();
        }
    }

    fn prepare_value(&mut self) -> Result<Bson, String> {
        if let Ok(bson) = shell::parse_shell_bson_value(&self.value_input) {
            self.value_kind = ValueEditKind::from_bson(&bson);
            self.value_label = shell::bson_type_name(&bson).to_string();
            return Ok(bson);
        }

        if let Some(kind) = ValueEditKind::infer(&self.value_input) {
            self.value_kind = kind;
            self.value_label = kind.label().to_string();
        }

        let bson = self.value_kind.parse(&self.value_input)?;
        self.value_label = shell::bson_type_name(&bson).to_string();
        Ok(bson)
    }
}

struct CollectionClick {
    client_id: ClientId,
    db_name: String,
    collection: String,
    at: Instant,
}

#[derive(Debug)]
enum WatchStreamEvent {
    Progress(Vec<Bson>),
    Finished { result: Result<QueryResult, String>, duration: Duration },
}

struct WatchStreamState {
    change_stream: Option<mongodb::sync::ChangeStream<ChangeStreamEvent<Document>>>,
    documents: Vec<Bson>,
    limit: usize,
    started: Instant,
    finished: bool,
    error: Option<String>,
}

#[derive(Debug)]
struct CollectionTab {
    client_id: ClientId,
    client_name: String,
    db_name: String,
    collection: String,
    pending_collection: Option<String>,
    editor: TextEditorContent,
    focus_anchor: Id,
    panes: pane_grid::State<CollectionPane>,
    bson_tree: BsonTree,
    response_view_mode: ResponseViewMode,
    text_result: Option<TextResultView>,
    skip_input: String,
    limit_input: String,
    query_in_progress: bool,
    last_query_duration: Option<Duration>,
    last_result: Option<QueryResult>,
    palette: ThemePalette,
}

#[derive(Debug)]
struct TextResultView {
    documents: Vec<String>,
}

impl TextResultView {
    fn from_documents(documents: &[Bson]) -> Option<Self> {
        if documents.is_empty() {
            return None;
        }
        let docs_json: Vec<String> = documents.iter().map(shell::format_bson_shell).collect();
        Some(Self { documents: docs_json })
    }

    fn from_query_result(result: &QueryResult) -> Option<Self> {
        match result {
            QueryResult::Documents(values) => Self::from_documents(values),
            QueryResult::SingleDocument { document } => {
                let bson = Bson::Document(document.clone());
                Self::from_documents(&[bson])
            }
            QueryResult::Indexes(values) => Self::from_documents(values),
            _ => None,
        }
    }

    fn as_json_list(&self) -> String {
        if self.documents.is_empty() {
            return String::from("[]");
        }

        let joined = self.documents.join(",\n");
        format!("[\n{joined}\n]")
    }
}

fn position_cursor_in_find(editor: &mut TextEditorContent, text: &str) {
    let Some(index) = text.find("{}") else {
        return;
    };
    let target = index + 1;
    editor.perform(TextEditorAction::Move(text_editor::Motion::DocumentStart));
    for _ in 0..target {
        editor.perform(TextEditorAction::Move(text_editor::Motion::Right));
    }
}

impl CollectionTab {
    fn build_text_result(&mut self, result: &QueryResult) -> Duration {
        let start = Instant::now();
        self.text_result = TextResultView::from_query_result(result);
        start.elapsed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectionPane {
    Request,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseViewMode {
    Table,
    Text,
}

impl ResponseViewMode {
    fn label(self) -> &'static str {
        match self {
            ResponseViewMode::Table => tr("Table"),
            ResponseViewMode::Text => tr("Text"),
        }
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
        settings: &AppSettings,
    ) -> Self {
        let (mut panes, top) = pane_grid::State::new(CollectionPane::Request);
        let (_, split) = panes
            .split(pane_grid::Axis::Horizontal, top, CollectionPane::Response)
            .expect("failed to split collection tab panes");
        let initial_ratio = Self::clamp_split_ratio(Self::initial_split_ratio());
        panes.resize(split, initial_ratio);

        let palette = settings.active_palette().clone();
        let options = BsonTreeOptions::from(settings);
        let bson_tree = BsonTree::from_values(&values, options);
        let editor_text = format!(
            "db.getCollection('{collection_name}').find({{}})",
            collection_name = collection.as_str()
        );
        let text_result = None;
        let mut editor = TextEditorContent::with_text(&editor_text);
        position_cursor_in_find(&mut editor, &editor_text);

        let mut instance = Self {
            client_id,
            client_name,
            db_name,
            collection,
            pending_collection: None,
            editor,
            focus_anchor: Id::unique(),
            panes,
            bson_tree,
            response_view_mode: ResponseViewMode::Table,
            text_result,
            skip_input: DEFAULT_RESULT_SKIP.to_string(),
            limit_input: DEFAULT_RESULT_LIMIT.to_string(),
            query_in_progress: false,
            last_query_duration: None,
            last_result: Some(QueryResult::Documents(values)),
            palette,
        };

        instance.apply_behavior_settings(settings);
        instance
    }

    fn resize_split(&mut self, split: pane_grid::Split, ratio: f32) {
        if !ratio.is_finite() {
            return;
        }

        let clamped = Self::clamp_split_ratio(ratio);
        self.panes.resize(split, clamped);
    }

    fn view(&self, tab_id: TabId) -> Element<'_, Message> {
        let skip_tab_id = tab_id;
        let limit_tab_id = tab_id;
        let skip_prev_tab_id = tab_id;
        let skip_next_tab_id = tab_id;

        let duration_text = if self.query_in_progress {
            String::from(tr("Executing..."))
        } else {
            self.last_query_duration
                .map(Self::format_duration)
                .unwrap_or_else(|| String::from(tr("—")))
        };

        let icon_size = fonts::active_fonts().primary_size * 1.5;

        let skip_input = text_input(tr("skip"), &self.skip_input)
            .padding([4, 6])
            .align_x(Horizontal::Center)
            .on_input(move |value| Message::CollectionSkipChanged { tab_id: skip_tab_id, value })
            .width(Length::Fixed(52.0));

        let limit_input = text_input(tr("limit"), &self.limit_input)
            .padding([4, 6])
            .align_x(Horizontal::Center)
            .on_input(move |value| Message::CollectionLimitChanged { tab_id: limit_tab_id, value })
            .width(Length::Fixed(52.0));

        let skip_prev = Button::new(fonts::primary_text(tr("◀"), Some(6.0)))
            .on_press(Message::CollectionSkipPrev(skip_prev_tab_id))
            .padding([2, 6])
            .style({
                let palette = self.palette.clone();
                move |_, status| palette.subtle_button_style(4.0, status)
            });

        let skip_next = Button::new(fonts::primary_text(tr("▶"), Some(6.0)))
            .on_press(Message::CollectionSkipNext(skip_next_tab_id))
            .padding([2, 6])
            .style({
                let palette = self.palette.clone();
                move |_, status| palette.subtle_button_style(4.0, status)
            });

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
            .push(fonts::primary_text(self.client_name.clone(), None));

        let database_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_DATABASE_HANDLE, ICON_DATABASE_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(fonts::primary_text(self.db_name.clone(), None));

        let collection_label = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_COLLECTION_HANDLE, ICON_COLLECTION_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(fonts::primary_text(self.collection.clone(), None));

        let info_labels = Row::new()
            .spacing(12)
            .align_y(Vertical::Center)
            .push(connection_label)
            .push(database_label)
            .push(collection_label)
            .push(fonts::primary_text(format!("{} {}", tr("Duration:"), duration_text), None));

        let info_row = Row::new()
            .spacing(16)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(Container::new(info_labels).width(Length::Fill).padding([0, 4]))
            .push(navigation);

        let panel_bg = self.palette.widget_background_color();
        let panel_border = self.palette.widget_border_color();

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
                self.bson_tree.node_bson(node_id).map(|bson| shell::format_bson_shell(&bson))
            }
            TableContextAction::CopyKey => self.bson_tree.node_display_key(node_id),
            TableContextAction::CopyValue => self.bson_tree.node_value_display(node_id),
            TableContextAction::CopyPath => self.bson_tree.node_path(node_id),
            TableContextAction::EditValue => None,
            TableContextAction::DeleteIndex
            | TableContextAction::HideIndex
            | TableContextAction::UnhideIndex
            | TableContextAction::ExpandHierarchy
            | TableContextAction::CollapseHierarchy
            | TableContextAction::ExpandHierarchyAll
            | TableContextAction::CollapseHierarchyAll => None,
        }
    }

    fn value_edit_context(&self, node_id: usize) -> Option<ValueEditContext> {
        self.bson_tree.value_edit_context(node_id)
    }

    fn request_view(&self, tab_id: TabId) -> Element<'_, Message> {
        let send_tab_id = tab_id;
        let focus_anchor = Container::new(
            text_input("", "")
                .id(self.focus_anchor.clone())
                .size(1)
                .padding(0)
                .width(Length::Fixed(0.0)),
        )
        .width(Length::Fixed(0.0))
        .height(Length::Fixed(0.0));
        let editor = text_editor::TextEditor::new(&self.editor)
            .key_binding(move |key_press| {
                let is_enter = matches!(key_press.key, keyboard::Key::Named(key::Named::Enter));
                let is_delete = matches!(key_press.key, keyboard::Key::Named(key::Named::Delete));
                if is_enter && key_press.modifiers.command() {
                    Some(TextEditorBinding::Custom(Message::CollectionSend(send_tab_id)))
                } else if is_delete {
                    Some(TextEditorBinding::Delete)
                } else {
                    TextEditorBinding::from_key_press(key_press)
                }
            })
            .on_action(move |action| Message::CollectionEditorAction { tab_id, action })
            .height(Length::Fill);

        let send_content = Container::new(fonts::primary_text(tr("Send"), None))
            .center_x(Length::Shrink)
            .center_y(Length::Fill);

        let send_button = Button::new(send_content)
            .on_press(Message::CollectionSend(tab_id))
            .padding([4, 12])
            .width(Length::Shrink)
            .height(Length::Fill)
            .style({
                let palette = self.palette.clone();
                move |_, status| palette.primary_button_style(4.0, status)
            });

        let controls_row = Row::new()
            .spacing(0)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(focus_anchor)
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

    fn response_view(&self, tab_id: TabId) -> Element<'_, Message> {
        match self.response_view_mode {
            ResponseViewMode::Table => self.bson_tree.view(tab_id),
            ResponseViewMode::Text => self.text_result_view(tab_id),
        }
    }

    fn text_result_view(&self, tab_id: TabId) -> Element<'_, Message> {
        if let Some(text_result) = &self.text_result {
            let mut column = Column::new().spacing(12).width(Length::Fill);
            for doc in &text_result.documents {
                column = column.push(
                    Container::new(fonts::result_text(doc.clone(), None).wrapping(Wrapping::Word))
                        .padding([8, 10])
                        .width(Length::Fill)
                        .style(move |_| container::Style {
                            border: border::rounded(6.0).width(1),
                            ..Default::default()
                        }),
                );
            }

            let scroll = Scrollable::new(column).width(Length::Fill).height(Length::Fill);
            let menu_palette = self.palette.clone();
            let menu_palette_border = self.palette.clone();
            let menu = move || {
                let item_palette = menu_palette.clone();
                let border_palette = menu_palette_border.clone();
                let button = Button::new(fonts::primary_text(tr("Copy JSON"), None))
                    .padding([4, 8])
                    .on_press(Message::CollectionTextCopyJson { tab_id })
                    .style(move |_, status| item_palette.menu_button_style(6.0, status));
                Container::new(button)
                    .style(move |_| iced::widget::container::Style {
                        background: Some(border_palette.menu.background.to_color().into()),
                        border: border::rounded(6.0)
                            .width(1)
                            .color(border_palette.widget_border_color()),
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                            offset: iced::Vector::new(0.0, 3.0),
                            blur_radius: 10.0,
                        },
                        ..Default::default()
                    })
                    .padding([4, 6])
                    .into()
            };

            ContextMenu::new(scroll, menu).into()
        } else {
            Container::new(
                fonts::result_text(tr("Text view is available only for document results"), None)
                    .color(self.palette.text_muted.to_color()),
            )
            .padding([8, 12])
            .width(Length::Fill)
            .into()
        }
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

    fn parse_query(&self, text: &str) -> Result<(String, QueryOperation), String> {
        parse_collection_query_with_collection(&self.db_name, &self.collection, text)
    }

    fn sanitize_numeric<S: AsRef<str>>(value: S) -> String {
        let filtered: String = value.as_ref().chars().filter(|ch| ch.is_ascii_digit()).collect();
        let trimmed = filtered.trim_start_matches('0');
        if trimmed.is_empty() { String::from(tr("0")) } else { trimmed.to_string() }
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

    fn set_query_result(&mut self, result: QueryResult, settings: &AppSettings) {
        self.palette = settings.active_palette().clone();

        let cached = result.clone();
        self.last_result = Some(cached);

        if self.response_view_mode == ResponseViewMode::Text {
            let _ = self.build_text_result(&result);
        } else {
            self.text_result = None;
        };

        let options = BsonTreeOptions::from(settings);

        let tree = match result {
            QueryResult::Documents(values) => BsonTree::from_values(&values, options),
            QueryResult::Indexes(values) => BsonTree::from_indexes(&values, options),
            QueryResult::SingleDocument { document } => BsonTree::from_document(document, options),
            QueryResult::Distinct { field, values } => {
                BsonTree::from_distinct(field, values, options)
            }
            QueryResult::Count { value } => BsonTree::from_count(value, options),
        };

        self.bson_tree = tree;
        self.apply_behavior_settings(settings);
    }

    fn set_tree_error(&mut self, error: String) {
        log::error!("{error}");
        self.bson_tree = BsonTree::from_error(error);
        self.bson_tree.set_table_colors(self.palette.table.clone());
        self.bson_tree.set_menu_colors(self.palette.menu.clone());
        self.bson_tree.set_text_color(self.palette.text_primary);
        self.bson_tree.set_button_colors(self.palette.subtle_buttons.clone());
        self.last_result = None;
        self.text_result = None;
    }

    fn apply_behavior_settings(&mut self, settings: &AppSettings) {
        if settings.expand_first_result {
            if let Some(root_id) = self.bson_tree.first_root_id() {
                self.bson_tree.expand_node(root_id);
            }
        }
    }

    fn refresh_with_settings(&mut self, settings: &AppSettings) {
        self.palette = settings.active_palette().clone();
        self.bson_tree.set_table_colors(self.palette.table.clone());
        self.bson_tree.set_menu_colors(self.palette.menu.clone());
        self.bson_tree.set_text_color(self.palette.text_primary);
        self.bson_tree.set_button_colors(self.palette.subtle_buttons.clone());
        if let Some(result) = self.last_result.clone() {
            if self.response_view_mode == ResponseViewMode::Text {
                self.set_query_result(result, settings);
            }
        } else if settings.expand_first_result {
            if let Some(root_id) = self.bson_tree.first_root_id() {
                self.bson_tree.expand_node(root_id);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(AppSettings::default())
    }
}

impl App {
    fn handle_hotkey(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        match key.as_ref() {
            keyboard::Key::Named(key::Named::F2) => Some(Message::MenuItemSelected(
                TopMenu::View,
                MenuEntry::ViewMode(ResponseViewMode::Table),
            )),
            keyboard::Key::Named(key::Named::F4) => Some(Message::MenuItemSelected(
                TopMenu::View,
                MenuEntry::ViewMode(ResponseViewMode::Text),
            )),
            keyboard::Key::Character("w") if modifiers.command() => Some(Message::CloseActiveTab),
            _ => None,
        }
    }

    fn new(settings: AppSettings) -> Self {
        fonts::set_active_fonts(
            &settings.primary_font,
            settings.primary_font_size as f32,
            &settings.result_font,
            settings.result_font_size as f32,
        );
        let (mut panes, sidebar) = pane_grid::State::new(PaneContent::Sidebar);
        let (_content_pane, split) = panes
            .split(pane_grid::Axis::Vertical, sidebar, PaneContent::Main)
            .expect("failed to split pane grid");
        panes.resize(split, 0.20);

        let connections = load_connections_from_disk().unwrap_or_else(|error| {
            log::warn!("Failed to load connections: {error}");
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
            settings,
            mode: AppMode::Main,
            connections_window: None,
            connection_form: None,
            settings_window: None,
            settings_error_modal: None,
            help_docs_state: None,
            collection_modal: None,
            database_modal: None,
            document_modal: None,
            value_edit_modal: None,
            window_size: None,
            tab_color_picker: None,
        }
    }

    pub(crate) fn init() -> (Self, Task<Message>) {
        let settings_result = settings::load_from_disk();

        let (settings, load_error) = match settings_result {
            Ok(settings) => (settings, None),
            Err(error) => (AppSettings::default(), Some(error)),
        };

        fonts::set_active_fonts(
            &settings.primary_font,
            settings.primary_font_size as f32,
            &settings.result_font,
            settings.result_font_size as f32,
        );
        settings::initialize(settings.clone());
        i18n::init_language(settings.language);
        logging::apply_settings(
            settings.logging_enabled,
            settings.logging_level.to_level_filter(),
            &settings.logging_path,
        );

        let mut app = Self::new(settings);

        if let Some(error) = load_error {
            let message = format!("{} {}", tr("Failed to load settings:"), error);
            log::error!("{message}");
            app.settings_error_modal = Some(SettingsErrorModalState::new(message));
            app.mode = AppMode::SettingsLoadError;
        }

        (app, Task::none())
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MenuItemSelected(menu, entry) => {
                match entry {
                    MenuEntry::Action(label) => {
                        if menu == TopMenu::File && label == "Connections" {
                            self.open_connections_window();
                        } else if menu == TopMenu::Help && label == "Documentation" {
                            self.open_help_docs_window();
                        } else if menu == TopMenu::Help && label == "About" {
                            self.open_about_modal();
                        } else if menu == TopMenu::Help && label == "Licenses" {
                            self.open_licenses_modal();
                        } else {
                            log::debug!("Menu '{menu:?}' entry '{label}' clicked");
                        }
                    }
                    MenuEntry::ViewMode(mode) => {
                        if let Some(active_id) = self.active_tab {
                            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == active_id)
                            {
                                tab.collection.response_view_mode = mode;
                                log::debug!("View mode set to {:?} tab_id={}", mode, active_id);
                                if mode == ResponseViewMode::Text {
                                    if let Some(result) = tab.collection.last_result.clone() {
                                        tab.collection.build_text_result(&result);
                                    }
                                }
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::TabSelected(id) => {
                if self.tabs.iter().any(|tab| tab.id == id) {
                    self.active_tab = Some(id);
                    log::debug!("Tab selected id={}", id);
                }
                Task::none()
            }
            Message::TabClosed(id) => {
                if let Some(position) = self.tabs.iter().position(|tab| tab.id == id) {
                    self.tabs.remove(position);
                    if self.tab_color_picker == Some(id) {
                        self.tab_color_picker = None;
                    }
                    if self.active_tab == Some(id) {
                        self.active_tab = self
                            .tabs
                            .get(position.saturating_sub(1))
                            .or_else(|| self.tabs.get(position))
                            .map(|tab| tab.id);
                    }
                    log::debug!("Tab closed id={}", id);
                }
                Task::none()
            }
            Message::CloseActiveTab => {
                if let Some(active_id) = self.active_tab {
                    if let Some(position) = self.tabs.iter().position(|tab| tab.id == active_id) {
                        self.tabs.remove(position);
                        if self.tab_color_picker == Some(active_id) {
                            self.tab_color_picker = None;
                        }
                        self.active_tab = self
                            .tabs
                            .get(position.saturating_sub(1))
                            .or_else(|| self.tabs.get(position))
                            .map(|tab| tab.id);
                        log::debug!("Tab closed id={}", active_id);
                    }
                }
                Task::none()
            }
            Message::DuplicateTab(tab_id) => {
                log::debug!("Tab duplicated id={}", tab_id);
                self.duplicate_collection_tab(tab_id);
                Task::none()
            }
            Message::TabColorPickerOpened(tab_id) => {
                self.tab_color_picker = Some(tab_id);
                Task::none()
            }
            Message::TabColorPickerCanceled => {
                self.tab_color_picker = None;
                Task::none()
            }
            Message::TabColorChanged { tab_id, color } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.color = Some(color);
                }
                self.tab_color_picker = None;
                Task::none()
            }
            Message::TabColorReset(tab_id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.color = None;
                }
                self.tab_color_picker = None;
                Task::none()
            }
            Message::WindowEvent(event) => {
                match event {
                    window::Event::Opened { size, .. } | window::Event::Resized(size) => {
                        self.window_size = Some(size);
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::KeyboardEvent(event) => {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    if let Some(message) = Self::handle_hotkey(key, modifiers) {
                        return self.update(message);
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
                        Ok(ConnectionBootstrap { handle, mut databases, ssh_tunnel }) => {
                            databases.sort_unstable();
                            log::debug!(
                                "Connection established client_id={} databases={}",
                                client_id,
                                databases.len()
                            );
                            client.status = ConnectionStatus::Ready;
                            client.handle = Some(handle);
                            client.databases =
                                databases.into_iter().map(DatabaseNode::new).collect();
                            client.expanded = true;
                            client.ssh_tunnel = ssh_tunnel;
                        }
                        Err(error) => {
                            log::error!("{error}");
                            client.status = ConnectionStatus::Failed(error);
                            client.databases.clear();
                            client.handle = None;
                            client.ssh_tunnel = None;
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
                                        let message = String::from(tr("No active connection"));
                                        log::error!("{message}");
                                        database.state = DatabaseState::Error(message);
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
                                log::error!("{error}");
                                database.state = DatabaseState::Error(error);
                                database.collections.clear();
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::ConnectionContextMenu { client_id, action } => match action {
                ConnectionContextAction::CreateDatabase => {
                    let ready = self.clients.iter().any(|client| {
                        client.id == client_id && matches!(client.status, ConnectionStatus::Ready)
                    });
                    if ready {
                        self.database_modal = Some(DatabaseModalState::new_create(client_id));
                        self.mode = AppMode::DatabaseModal;
                    }
                    Task::none()
                }
                ConnectionContextAction::Refresh => self.refresh_databases(client_id),
                ConnectionContextAction::ServerStatus => {
                    if let Some(task) = self.open_server_status_tab(client_id) {
                        task
                    } else {
                        Task::none()
                    }
                }
                ConnectionContextAction::Close => {
                    self.close_client_connection(client_id);
                    Task::none()
                }
            },
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
                    let tab_id = self.open_collection_tab(client_id, db_name, collection);
                    self.focus_collection_editor(tab_id)
                } else {
                    self.last_collection_click =
                        Some(CollectionClick { client_id, db_name, collection, at: now });
                    Task::none()
                }
            }
            Message::CollectionEditorAction { tab_id, action } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.editor.perform(action);
                }
                Task::none()
            }
            Message::CollectionTextCopyJson { tab_id } => {
                if let Some(tab) = self.tabs.iter().find(|tab| tab.id == tab_id) {
                    if let Some(text_result) = &tab.collection.text_result {
                        return clipboard::write(text_result.as_json_list());
                    }
                }
                Task::none()
            }
            Message::DatabaseContextMenu { client_id, db_name, action } => match action {
                DatabaseContextAction::OpenEmptyTab => {
                    let _ = self.open_database_empty_tab(client_id, db_name);
                    Task::none()
                }
                DatabaseContextAction::CreateCollection => {
                    self.collection_modal =
                        Some(CollectionModalState::new_create(client_id, db_name.clone()));
                    self.mode = AppMode::CollectionModal;
                    Task::none()
                }
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
                    CollectionContextAction::ChangeStream => {
                        let tab_id = self.open_collection_tab(
                            client_id,
                            db_name.clone(),
                            collection.clone(),
                        );
                        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                            let query = format!(
                                "db.getCollection('{collection_name}').watch()",
                                collection_name = collection
                            );
                            tab.collection.editor = TextEditorContent::with_text(&query);
                        }
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
                    CollectionContextAction::Stats => {
                        let tab_id = self.open_collection_stats_tab(
                            client_id,
                            db_name.clone(),
                            collection.clone(),
                        );
                        self.collection_query_task(tab_id)
                    }
                    CollectionContextAction::CreateIndex => {
                        let _ = self.open_collection_create_index_tab(
                            client_id,
                            db_name.clone(),
                            collection.clone(),
                        );
                        Task::none()
                    }
                    CollectionContextAction::Indexes => {
                        let tab_id = self.open_collection_indexes_tab(
                            client_id,
                            db_name.clone(),
                            collection.clone(),
                        );
                        self.collection_query_task(tab_id)
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
                    CollectionModalKind::CreateCollection => {
                        if trimmed_input.is_empty() {
                            let message = String::from(tr("Collection name cannot be empty."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }
                    }
                    CollectionModalKind::DeleteAllDocuments
                    | CollectionModalKind::DeleteCollection => {
                        if trimmed_input != modal.collection {
                            let message =
                                String::from(tr("Enter the exact collection name to confirm."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }
                    }
                    CollectionModalKind::RenameCollection => {
                        if trimmed_input.is_empty() {
                            let message = String::from(tr("New collection name cannot be empty."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        if trimmed_input == modal.collection {
                            let message = String::from(tr(
                                "New collection name must differ from the current one.",
                            ));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }
                    }
                    CollectionModalKind::DropIndex { ref index_name } => {
                        if trimmed_input != *index_name {
                            let message =
                                String::from(tr("Enter the exact index name to confirm."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }
                    }
                }

                let client_id = modal.client_id;
                let db_name = modal.db_name.clone();
                let collection = modal.collection.clone();
                let kind = modal.kind.clone();
                let origin_tab = modal.origin_tab;

                let handle = match self
                    .clients
                    .iter()
                    .find(|client| client.id == client_id)
                    .and_then(|client| client.handle.clone())
                {
                    Some(handle) => handle,
                    None => {
                        let message = String::from(tr("No active connection."));
                        log::error!("{message}");
                        modal.error = Some(message);
                        return Task::none();
                    }
                };

                if matches!(modal.kind, CollectionModalKind::CreateCollection) {
                    if let Some(client) = self.clients.iter().find(|client| client.id == client_id)
                    {
                        if let Some(database) =
                            client.databases.iter().find(|db| db.name == db_name)
                        {
                            if database.collections.iter().any(|node| node.name == trimmed_input) {
                                let message =
                                    String::from(tr("A collection with this name already exists."));
                                log::error!("{message}");
                                modal.error = Some(message);
                                return Task::none();
                            }
                        }
                    }
                }

                modal.processing = true;
                modal.error = None;

                match kind {
                    CollectionModalKind::CreateCollection => {
                        let new_collection = trimmed_input.clone();
                        modal.collection = new_collection.clone();
                        log::debug!(
                            "Create collection requested db={} collection={}",
                            db_name,
                            new_collection
                        );

                        let future_db = db_name.clone();
                        let message_db = db_name.clone();
                        let message_collection = new_collection.clone();
                        let handle_task = handle.clone();

                        Task::perform(
                            async move {
                                handle_task
                                    .database(&future_db)
                                    .create_collection(&new_collection)
                                    .run()
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::CollectionCreateCompleted {
                                client_id,
                                db_name: message_db.clone(),
                                collection: message_collection.clone(),
                                result,
                            },
                        )
                    }
                    CollectionModalKind::DeleteAllDocuments => {
                        let future_db = db_name.clone();
                        let future_collection = collection.clone();
                        let message_db = db_name.clone();
                        let message_collection = collection.clone();
                        log::debug!(
                            "Delete all documents requested db={} collection={}",
                            db_name,
                            collection
                        );
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
                        log::debug!(
                            "Drop collection requested db={} collection={}",
                            db_name,
                            collection
                        );
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
                        log::debug!(
                            "Rename collection requested db={} from={} to={}",
                            db_name,
                            collection,
                            new_name
                        );
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
                    CollectionModalKind::DropIndex { index_name } => {
                        let Some(tab_id_value) = origin_tab else {
                            modal.processing = false;
                            let message =
                                String::from(tr("Failed to determine the tab to refresh indexes."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        };

                        let future_db = db_name.clone();
                        let future_collection = collection.clone();
                        let future_index = index_name.clone();
                        let message_db = db_name.clone();
                        let message_collection = collection.clone();
                        let message_index = index_name.clone();
                        log::debug!(
                            "Drop index requested db={} collection={} index={}",
                            db_name,
                            collection,
                            index_name
                        );
                        let handle_task = handle.clone();

                        Task::perform(
                            async move {
                                let command = doc! {
                                    "dropIndexes": future_collection,
                                    "index": future_index,
                                };
                                handle_task
                                    .database(&future_db)
                                    .run_command(command)
                                    .run()
                                    .map(|_| ())
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::CollectionDropIndexCompleted {
                                tab_id: tab_id_value,
                                client_id,
                                db_name: message_db.clone(),
                                collection: message_collection.clone(),
                                index_name: message_index.clone(),
                                result,
                            },
                        )
                    }
                }
            }
            Message::CollectionDeleteAllCompleted { client_id, db_name, collection, result } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if matches!(modal.kind, CollectionModalKind::DeleteAllDocuments)
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
                                log::error!("{error}");
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
                    if matches!(modal.kind, CollectionModalKind::DeleteCollection)
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
                                log::error!("{error}");
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionCreateCompleted { client_id, db_name, collection, result } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if matches!(modal.kind, CollectionModalKind::CreateCollection)
                        && modal.client_id == client_id
                        && modal.db_name == db_name
                        && modal.collection == collection
                    {
                        match result {
                            Ok(()) => {
                                self.collection_modal = None;
                                self.mode = AppMode::Main;
                                self.add_collection_to_tree(client_id, &db_name, &collection);
                            }
                            Err(error) => {
                                modal.processing = false;
                                log::error!("{error}");
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
                    if matches!(modal.kind, CollectionModalKind::RenameCollection)
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
                                log::error!("{error}");
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionDropIndexCompleted {
                tab_id,
                client_id,
                db_name,
                collection,
                index_name,
                result,
            } => {
                if let Some(modal) = self.collection_modal.as_mut() {
                    if matches!(modal.kind, CollectionModalKind::DropIndex { .. })
                        && modal.client_id == client_id
                        && modal.db_name == db_name
                        && modal.collection == collection
                    {
                        match result {
                            Ok(()) => {
                                self.collection_modal = None;
                                self.mode = AppMode::Main;
                                return self.collection_query_task(tab_id);
                            }
                            Err(error) => {
                                modal.processing = false;
                                let message = format!(
                                    "{} \"{}\": {}",
                                    tr("Failed to delete index"),
                                    index_name,
                                    error
                                );
                                log::error!("{message}");
                                modal.error = Some(message);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::CollectionHideIndexCompleted {
                tab_id,
                client_id,
                db_name,
                collection,
                index_name,
                result,
            } => {
                match result {
                    Ok(()) => return self.collection_query_task(tab_id),
                    Err(error) => {
                        log::error!(
                            "hideIndex failed: client_id={} db={} collection={} index={} error={}",
                            client_id,
                            db_name,
                            collection,
                            index_name,
                            error
                        );
                    }
                }
                Task::none()
            }
            Message::CollectionUnhideIndexCompleted {
                tab_id,
                client_id,
                db_name,
                collection,
                index_name,
                result,
            } => {
                match result {
                    Ok(()) => return self.collection_query_task(tab_id),
                    Err(error) => {
                        log::error!(
                            "unhideIndex failed: client_id={} db={} collection={} index={} error={}",
                            client_id,
                            db_name,
                            collection,
                            index_name,
                            error
                        );
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
            Message::DatabaseModalCollectionInputChanged(value) => {
                if let Some(modal) = self.database_modal.as_mut() {
                    modal.collection_input = value;
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

                let client_id = modal.client_id;
                match &modal.mode {
                    DatabaseModalMode::Drop { db_name } => {
                        if modal.input.trim() != db_name {
                            let message =
                                String::from(tr("Enter the exact database name to confirm."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        let handle = match self
                            .clients
                            .iter()
                            .find(|client| client.id == client_id)
                            .and_then(|client| client.handle.clone())
                        {
                            Some(handle) => handle,
                            None => {
                                let message = String::from(tr("No active connection."));
                                log::error!("{message}");
                                modal.error = Some(message);
                                return Task::none();
                            }
                        };

                        modal.processing = true;
                        modal.error = None;

                        log::debug!("Drop database requested db={}", db_name);
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
                    DatabaseModalMode::Create => {
                        let db_name_input = modal.input.trim();
                        let collection_name_input = modal.collection_input.trim();

                        if db_name_input.is_empty() {
                            let message = String::from(tr("Provide a database name."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        if collection_name_input.is_empty() {
                            let message = String::from(tr(
                                "Enter the name of the first collection for the new database.",
                            ));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        let (handle, exists) = self
                            .clients
                            .iter()
                            .find(|client| client.id == client_id)
                            .map(|client| {
                                let exists =
                                    client.databases.iter().any(|db| db.name == db_name_input);
                                (client.handle.clone(), exists)
                            })
                            .unwrap_or((None, false));

                        if exists {
                            let message =
                                String::from(tr("A database with this name already exists."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        let Some(handle) = handle else {
                            let message = String::from(tr("No active connection."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        };

                        modal.processing = true;
                        modal.error = None;

                        log::debug!(
                            "Create database requested db={} collection={}",
                            db_name_input,
                            collection_name_input
                        );
                        let db_name = db_name_input.to_string();
                        let collection_name = collection_name_input.to_string();
                        let handle_task = handle.clone();
                        let message_db = db_name.clone();
                        let collection_for_task = collection_name.clone();

                        Task::perform(
                            async move {
                                let database = handle_task.database(&db_name);
                                database
                                    .create_collection(&collection_for_task)
                                    .run()
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::DatabaseCreateCompleted {
                                client_id,
                                _db_name: message_db.clone(),
                                result,
                            },
                        )
                    }
                }
            }
            Message::DatabaseDropCompleted { client_id, db_name, result } => {
                if let Some(modal) = self.database_modal.as_mut() {
                    if modal.client_id == client_id {
                        if let DatabaseModalMode::Drop { db_name: modal_db } = &modal.mode {
                            if modal_db == &db_name {
                                match result {
                                    Ok(()) => {
                                        self.database_modal = None;
                                        self.mode = AppMode::Main;
                                        self.remove_database_from_tree(client_id, &db_name);
                                    }
                                    Err(error) => {
                                        modal.processing = false;
                                        log::error!("{error}");
                                        modal.error = Some(error);
                                    }
                                }
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::DatabaseCreateCompleted { client_id, _db_name: _, result } => {
                if let Some(modal) = self.database_modal.as_mut() {
                    if modal.client_id == client_id
                        && matches!(modal.mode, DatabaseModalMode::Create)
                    {
                        match result {
                            Ok(()) => {
                                self.database_modal = None;
                                self.mode = AppMode::Main;
                                return self.refresh_databases(client_id);
                            }
                            Err(error) => {
                                modal.processing = false;
                                log::error!("{error}");
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::DocumentModalEditorAction(action) => {
                if let Some(modal) = self.document_modal.as_mut() {
                    modal.editor.perform(action);
                }
                Task::none()
            }
            Message::DocumentModalCancel => {
                self.document_modal = None;
                self.mode = AppMode::Main;
                Task::none()
            }
            Message::DocumentModalSave => {
                let Some(modal) = self.document_modal.as_mut() else {
                    return Task::none();
                };

                if modal.processing {
                    return Task::none();
                }

                let editor_text = modal.editor.text().to_string();
                let document = match shell::parse_shell_json_value(&editor_text) {
                    Ok(value) => {
                        let object = match value.as_object() {
                            Some(obj) => obj,
                            None => {
                                let message = String::from(tr("Document must be a JSON object."));
                                log::error!("{message}");
                                modal.error = Some(message);
                                return Task::none();
                            }
                        };
                        match bson::to_document(object) {
                            Ok(doc) => doc,
                            Err(error) => {
                                let message = format!("BSON conversion error: {error}");
                                log::error!("{message}");
                                modal.error = Some(message);
                                return Task::none();
                            }
                        }
                    }
                    Err(error) => {
                        log::error!("{error}");
                        modal.error = Some(error);
                        return Task::none();
                    }
                };

                let client_handle = self
                    .clients
                    .iter()
                    .find(|client| client.id == modal.client_id)
                    .and_then(|client| client.handle.clone());

                let Some(handle) = client_handle else {
                    let message = String::from(tr("No active connection."));
                    log::error!("{message}");
                    modal.error = Some(message);
                    return Task::none();
                };

                modal.processing = true;
                modal.error = None;

                let tab_id = modal.tab_id;
                let db_name = modal.db_name.clone();
                let collection_name = modal.collection.clone();
                let kind = modal.kind.clone();

                match kind {
                    DocumentModalKind::CollectionDocument { filter, original_id } => {
                        let mut replacement = document.clone();
                        if !replacement.contains_key("_id") {
                            replacement.insert("_id", original_id);
                        }

                        log::debug!(
                            "Document save requested db={} collection={}",
                            db_name,
                            collection_name
                        );
                        let filter_clone = filter;
                        let replacement_clone = replacement.clone();
                        let handle_task = handle.clone();
                        let db_name_clone = db_name.clone();
                        let collection_clone = collection_name.clone();

                        Task::perform(
                            async move {
                                let collection = handle_task
                                    .database(&db_name_clone)
                                    .collection::<Document>(&collection_clone);
                                let result = collection
                                    .find_one_and_replace(filter_clone, replacement_clone)
                                    .return_document(ReturnDocument::After)
                                    .run()
                                    .map_err(|error| error.to_string())?;
                                result.ok_or_else(|| {
                                    String::from(tr(
                                        "Document not found. It may have been deleted or the change was not applied.",
                                    ))
                                })
                            },
                            move |result| Message::DocumentModalCompleted { tab_id, result },
                        )
                    }
                    DocumentModalKind::Index { name } => {
                        let index_doc = document.clone();
                        let Some(name_value) = index_doc
                            .get("name")
                            .and_then(|value| value.as_str())
                            .map(|value| value.to_string())
                        else {
                            modal.processing = false;
                            let message = String::from(tr(
                                "Index document must contain a string field named name.",
                            ));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        };

                        if name_value != name {
                            modal.processing = false;
                            let message =
                                String::from(tr("Index name cannot be changed via collMod."));
                            log::error!("{message}");
                            modal.error = Some(message);
                            return Task::none();
                        }

                        log::debug!(
                            "Index update requested db={} collection={} index={}",
                            db_name,
                            collection_name,
                            name
                        );
                        let handle_task = handle.clone();
                        let db_name_clone = db_name.clone();
                        let collection_clone = collection_name.clone();
                        let command_index = index_doc.clone();

                        Task::perform(
                            async move {
                                let command = doc! {
                                    "collMod": collection_clone.clone(),
                                    "index": Bson::Document(command_index),
                                };
                                handle_task
                                    .database(&db_name_clone)
                                    .run_command(command)
                                    .run()
                                    .map_err(|error| error.to_string())
                            },
                            move |result| Message::DocumentModalCompleted { tab_id, result },
                        )
                    }
                }
            }
            Message::DocumentModalCompleted { tab_id, result } => match result {
                Ok(_) => {
                    self.document_modal = None;
                    self.mode = AppMode::Main;
                    return self.collection_query_task(tab_id);
                }
                Err(error) => {
                    if let Some(modal) = self.document_modal.as_mut() {
                        modal.processing = false;
                        log::error!("{error}");
                        modal.error = Some(error);
                    }
                    Task::none()
                }
            },
            Message::ValueEditModalEditorAction(action) => {
                if let Some(modal) = self.value_edit_modal.as_mut() {
                    modal.apply_editor_action(action);
                }
                Task::none()
            }
            Message::ValueEditModalCancel => {
                self.value_edit_modal = None;
                self.mode = AppMode::Main;
                Task::none()
            }
            Message::ValueEditModalSave => {
                let Some(modal) = self.value_edit_modal.as_mut() else {
                    return Task::none();
                };

                if modal.processing {
                    return Task::none();
                }

                let new_value = match modal.prepare_value() {
                    Ok(value) => value,
                    Err(error) => {
                        log::error!("{error}");
                        modal.error = Some(error);
                        return Task::none();
                    }
                };

                let Some(handle) = self
                    .clients
                    .iter()
                    .find(|client| client.id == modal.client_id)
                    .and_then(|client| client.handle.clone())
                else {
                    let message = String::from(tr("No active connection."));
                    log::error!("{message}");
                    modal.error = Some(message);
                    return Task::none();
                };

                let mut set_doc = Document::new();
                set_doc.insert(modal.path.clone(), new_value);

                let mut update_doc = Document::new();
                update_doc.insert("$set", Bson::Document(set_doc));

                modal.processing = true;
                modal.error = None;

                let tab_id = modal.tab_id;
                let db_name = modal.db_name.clone();
                let collection_name = modal.collection.clone();
                let filter_clone = modal.filter.clone();
                let update_clone = update_doc.clone();
                let handle_task = handle.clone();

                log::debug!(
                    "Value edit requested db={} collection={} path={}",
                    db_name,
                    collection_name,
                    modal.path
                );
                Task::perform(
                    async move {
                        let collection =
                            handle_task.database(&db_name).collection::<Document>(&collection_name);
                        let result = collection
                            .find_one_and_update(filter_clone, update_clone)
                            .return_document(ReturnDocument::After)
                            .run()
                            .map_err(|error| error.to_string())?;
                        result.ok_or_else(|| {
                            String::from(tr(
                                "Document not found. It may have been deleted or the change was not applied.",
                            ))
                        })
                    },
                    move |result| Message::ValueEditModalCompleted { tab_id, result },
                )
            }
            Message::ValueEditModalCompleted { tab_id, result } => match result {
                Ok(_) => {
                    self.value_edit_modal = None;
                    self.mode = AppMode::Main;
                    return self.collection_query_task(tab_id);
                }
                Err(error) => {
                    if let Some(modal) = self.value_edit_modal.as_mut() {
                        modal.processing = false;
                        log::error!("{error}");
                        modal.error = Some(error);
                    }
                    Task::none()
                }
            },
            Message::DatabasesRefreshed { client_id, result } => {
                if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
                    match result {
                        Ok(mut names) => {
                            names.sort_unstable();
                            client.databases = names.into_iter().map(DatabaseNode::new).collect();
                        }
                        Err(error) => {
                            log::error!("{} {}", tr("Failed to refresh database list:"), error);
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
            Message::TableContextMenu { tab_id, node_id, action } => match action {
                TableContextAction::EditValue => {
                    let modal_state =
                        self.tabs.iter().find(|tab| tab.id == tab_id).and_then(|tab| {
                            tab.collection.value_edit_context(node_id).map(|context| {
                                ValueEditModalState::new(tab_id, &tab.collection, context)
                            })
                        });

                    if let Some(state) = modal_state {
                        self.value_edit_modal = Some(state);
                        self.mode = AppMode::ValueEditModal;
                    }

                    Task::none()
                }
                TableContextAction::DeleteIndex => {
                    let context = self.tabs.iter().find(|tab| tab.id == tab_id).and_then(|tab| {
                        if !tab.collection.bson_tree.is_indexes_view() {
                            return None;
                        }
                        let index_name = tab.collection.bson_tree.node_index_name(node_id)?;
                        Some((
                            tab.collection.client_id,
                            tab.collection.db_name.clone(),
                            tab.collection.collection.clone(),
                            index_name,
                        ))
                    });

                    if let Some((client_id, db_name, collection, index_name)) = context {
                        if index_name != "_id_" {
                            self.collection_modal = Some(CollectionModalState::new_drop_index(
                                tab_id, client_id, db_name, collection, index_name,
                            ));
                            self.mode = AppMode::CollectionModal;
                        }
                    }

                    Task::none()
                }
                TableContextAction::HideIndex => {
                    let context = self.tabs.iter().find(|tab| tab.id == tab_id).and_then(|tab| {
                        if !tab.collection.bson_tree.is_indexes_view() {
                            return None;
                        }
                        let index_name = tab.collection.bson_tree.node_index_name(node_id)?;
                        let hidden = tab.collection.bson_tree.node_index_hidden(node_id);
                        Some((
                            tab.collection.client_id,
                            tab.collection.db_name.clone(),
                            tab.collection.collection.clone(),
                            index_name,
                            hidden.unwrap_or(false),
                        ))
                    });

                    if let Some((client_id, db_name, collection, index_name, hidden)) = context {
                        if hidden {
                            return Task::none();
                        }

                        if let Some(handle) = self
                            .clients
                            .iter()
                            .find(|client| client.id == client_id)
                            .and_then(|client| client.handle.clone())
                        {
                            let future_db = db_name.clone();
                            let future_collection = collection.clone();
                            let future_index = index_name.clone();
                            let message_db = db_name.clone();
                            let message_collection = collection.clone();
                            let message_index = index_name.clone();
                            let handle_task = handle.clone();
                            return Task::perform(
                                async move {
                                    let command = doc! {
                                        "hideIndex": future_collection,
                                        "index": future_index,
                                    };
                                    handle_task
                                        .database(&future_db)
                                        .run_command(command)
                                        .run()
                                        .map(|_| ())
                                        .map_err(|error| error.to_string())
                                },
                                move |result| Message::CollectionHideIndexCompleted {
                                    tab_id,
                                    client_id,
                                    db_name: message_db.clone(),
                                    collection: message_collection.clone(),
                                    index_name: message_index.clone(),
                                    result,
                                },
                            );
                        }
                    }

                    Task::none()
                }
                TableContextAction::UnhideIndex => {
                    let context = self.tabs.iter().find(|tab| tab.id == tab_id).and_then(|tab| {
                        if !tab.collection.bson_tree.is_indexes_view() {
                            return None;
                        }
                        let index_name = tab.collection.bson_tree.node_index_name(node_id)?;
                        let hidden = tab.collection.bson_tree.node_index_hidden(node_id);
                        Some((
                            tab.collection.client_id,
                            tab.collection.db_name.clone(),
                            tab.collection.collection.clone(),
                            index_name,
                            hidden.unwrap_or(false),
                        ))
                    });

                    if let Some((client_id, db_name, collection, index_name, hidden)) = context {
                        if !hidden {
                            return Task::none();
                        }

                        if let Some(handle) = self
                            .clients
                            .iter()
                            .find(|client| client.id == client_id)
                            .and_then(|client| client.handle.clone())
                        {
                            let future_db = db_name.clone();
                            let future_collection = collection.clone();
                            let future_index = index_name.clone();
                            let message_db = db_name.clone();
                            let message_collection = collection.clone();
                            let message_index = index_name.clone();
                            let handle_task = handle.clone();
                            return Task::perform(
                                async move {
                                    let command = doc! {
                                        "unhideIndex": future_collection,
                                        "index": future_index,
                                    };
                                    handle_task
                                        .database(&future_db)
                                        .run_command(command)
                                        .run()
                                        .map(|_| ())
                                        .map_err(|error| error.to_string())
                                },
                                move |result| Message::CollectionUnhideIndexCompleted {
                                    tab_id,
                                    client_id,
                                    db_name: message_db.clone(),
                                    collection: message_collection.clone(),
                                    index_name: message_index.clone(),
                                    result,
                                },
                            );
                        }
                    }

                    Task::none()
                }
                TableContextAction::ExpandHierarchy => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.collection.bson_tree.expand_recursive(node_id);
                    }
                    Task::none()
                }
                TableContextAction::CollapseHierarchy => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.collection.bson_tree.collapse_recursive(node_id);
                    }
                    Task::none()
                }
                TableContextAction::ExpandHierarchyAll => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.collection.bson_tree.expand_all();
                    }
                    Task::none()
                }
                TableContextAction::CollapseHierarchyAll => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                        tab.collection.bson_tree.collapse_all();
                    }
                    Task::none()
                }
                _ => {
                    let content = self
                        .tabs
                        .iter()
                        .find(|tab| tab.id == tab_id)
                        .and_then(|tab| tab.collection.table_context_content(node_id, action));

                    if let Some(text) = content { clipboard::write(text) } else { Task::none() }
                }
            },
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
            Message::CollectionWatchProgress { tab_id, documents } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    let result = QueryResult::Documents(documents);
                    tab.collection.set_query_result(result, &self.settings);
                }
                Task::none()
            }
            Message::CollectionQueryCompleted { tab_id, result, duration } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    let collection = &mut tab.collection;
                    collection.query_in_progress = false;
                    collection.last_query_duration = Some(duration);
                    match result {
                        Ok(query_result) => {
                            if let Some(pending) = collection.pending_collection.take() {
                                if pending != collection.collection {
                                    collection.collection = pending.clone();
                                    tab.title = pending;
                                }
                            }
                            let (kind, count) = Self::query_result_metrics(&query_result);
                            log::debug!(
                                "Query completed tab_id={} db={} collection={} result={} count={} duration_ms={:.3}",
                                tab_id,
                                collection.db_name,
                                collection.collection,
                                kind,
                                count,
                                duration.as_secs_f64() * 1000.0
                            );
                            collection.set_query_result(query_result, &self.settings)
                        }
                        Err(error) => {
                            collection.pending_collection = None;
                            collection.set_tree_error(error);
                        }
                    }
                }
                Task::none()
            }
            Message::AboutModalClose => {
                self.close_about_modal();
                Task::none()
            }
            Message::LicensesModalClose => {
                self.close_licenses_modal();
                Task::none()
            }
            Message::HelpDocsClose => {
                self.close_help_docs_window();
                Task::none()
            }
            Message::HelpDocsSectionSelected(index) => {
                if let Some(state) = self.help_docs_state.as_mut() {
                    state.selected_index = index;
                }
                Task::none()
            }
            Message::HelpDocsSearchChanged(value) => {
                if let Some(state) = self.help_docs_state.as_mut() {
                    state.search = value;
                }
                Task::none()
            }
            Message::OpenUrl(url) => {
                if let Err(error) = webbrowser::open(url) {
                    log::warn!("Failed to open url {url}: {error}");
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
                    match self.add_connection_from_entry(entry) {
                        Ok(task) => {
                            self.close_connections_window();
                            return task;
                        }
                        Err(error) => {
                            if let Some(window) = self.connections_window.as_mut() {
                                log::error!("{error}");
                                window.feedback = Some(error);
                            }
                        }
                    }
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
                            let removed_name =
                                self.connections.get(index).map(|entry| entry.name.clone());
                            self.connections.remove(index);
                            if let Some(name) = removed_name {
                                log::debug!("Connection deleted name={}", name);
                            }
                            match save_connections_to_disk(&self.connections) {
                                Ok(()) => state.feedback = Some(String::from(tr("Deleted"))),
                                Err(error) => {
                                    log::error!("{error}");
                                    state.feedback =
                                        Some(format!("{}{}", tr("Save error: "), error));
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
                            match self.add_connection_from_entry(entry.clone()) {
                                Ok(task) => {
                                    self.close_connections_window();
                                    return task;
                                }
                                Err(error) => {
                                    if let Some(window) = self.connections_window.as_mut() {
                                        log::error!("{error}");
                                        window.feedback = Some(error);
                                    }
                                }
                            }
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
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormTypeChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.connection_type = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormPortChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    let sanitized: String =
                        value.chars().filter(|ch| ch.is_ascii_digit()).take(5).collect();
                    form.port = sanitized;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormAuthUseChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.use_auth = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormAuthLoginChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.username = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormAuthPasswordChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.password = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormPasswordStorageChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.password_storage = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormAuthMechanismChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.mechanism = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormAuthDatabaseChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.auth.database = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshUseChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.use_ssh = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshHostChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.host = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshPortChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    let sanitized: String =
                        value.chars().filter(|ch| ch.is_ascii_digit()).take(5).collect();
                    form.ssh.port = sanitized;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshUsernameChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.username = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshAuthMethodChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.auth_method = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshPasswordChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.password = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshPrivateKeyChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.private_key = value;
                    form.validation_error = None;
                }
                Task::none()
            }
            Message::ConnectionFormSshPrivateKeyBrowse => {
                return Task::perform(
                    async {
                        FileDialog::new().pick_file().map(|path| path.to_string_lossy().to_string())
                    },
                    Message::ConnectionFormSshPrivateKeyPicked,
                );
            }
            Message::ConnectionFormSshPrivateKeyPicked(path) => {
                if let Some(form) = self.connection_form.as_mut() {
                    if let Some(path) = path {
                        form.ssh.private_key = path;
                        form.validation_error = None;
                    }
                }
                Task::none()
            }
            Message::ConnectionFormSshPassphraseChanged(value) => {
                if let Some(form) = self.connection_form.as_mut() {
                    form.ssh.passphrase = value;
                    form.validation_error = None;
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
                    match form.validate(true) {
                        Ok(entry) => {
                            form.validation_error = None;
                            form.testing = true;
                            form.test_feedback = None;
                            log::debug!(
                                "Connection test started name={} host={} port={} ssh_tunnel={}",
                                entry.name,
                                entry.host,
                                entry.port,
                                entry.ssh_tunnel.enabled
                            );

                            let auth_db = if form.auth.use_auth {
                                form.auth.database.trim()
                            } else {
                                "admin"
                            };
                            let auth_db = if auth_db.is_empty() { "admin" } else { auth_db };
                            let auth_db = auth_db.to_string();

                            return Task::perform(
                                async move {
                                    let tunnel_guard = if entry.ssh_tunnel.enabled {
                                        Some(SshTunnel::start(
                                            &entry.ssh_tunnel,
                                            &entry.host,
                                            entry.port,
                                        )?)
                                    } else {
                                        None
                                    };

                                    let uri = if let Some(ref tunnel) = tunnel_guard {
                                        entry.uri_for_host_port("127.0.0.1", tunnel.local_port())?
                                    } else {
                                        entry.uri()?
                                    };

                                    let client = Client::with_uri_str(&uri)
                                        .map_err(|err| err.to_string())?;
                                    let database = client.database(&auth_db);
                                    database
                                        .run_command(doc! { "ping": 1 })
                                        .run()
                                        .map_err(|err| err.to_string())?;
                                    database
                                        .list_collection_names()
                                        .run()
                                        .map(|_| ())
                                        .map_err(|err| err.to_string())
                                },
                                Message::ConnectionFormTestResult,
                            );
                        }
                        Err(error) => {
                            log::error!("{error}");
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
                        Ok(()) => {
                            log::debug!("Connection test succeeded");
                            TestFeedback::Success(String::from(tr("Connection established")))
                        }
                        Err(error) => {
                            log::error!("{error}");
                            TestFeedback::Failure(error)
                        }
                    });
                }
                Task::none()
            }
            Message::ConnectionFormSave => {
                if let Some(form) = self.connection_form.as_mut() {
                    let require_password = form.auth.password_storage == PasswordStorage::File;
                    match form.validate(require_password) {
                        Ok(entry) => {
                            let entry_name = entry.name.clone();
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
                                        Err(String::from(tr("Selected connection not found")))
                                    }
                                }
                            };

                            match result {
                                Ok(selected_index) => {
                                    log::debug!("Connection saved name={}", entry_name);
                                    if let Err(error) = save_connections_to_disk(&self.connections)
                                    {
                                        if let Some(window) = self.connections_window.as_mut() {
                                            log::error!("{error}");
                                            window.feedback =
                                                Some(format!("{}{}", tr("Save error: "), error));
                                        }
                                    }

                                    self.open_connections_window();
                                    if let Some(window) = self.connections_window.as_mut() {
                                        window.selected = Some(selected_index);
                                        window.feedback = Some(String::from(tr("Saved")));
                                    }
                                }
                                Err(error) => {
                                    log::error!("{error}");
                                    form.validation_error = Some(error);
                                    return Task::none();
                                }
                            }
                        }
                        Err(error) => {
                            log::error!("{error}");
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
            Message::SettingsOpen => {
                self.open_settings_window();
                Task::none()
            }
            Message::SettingsTabChanged(tab) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.active_tab = tab;
                }
                Task::none()
            }
            Message::SettingsToggleExpandFirstResult(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.expand_first_result = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsQueryTimeoutChanged(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.query_timeout_secs = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsToggleSortFields(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.sort_fields_alphabetically = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsToggleSortIndexes(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.sort_index_names_alphabetically = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsToggleCloseTabsOnDbClose(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.close_tabs_on_database_close = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsToggleLogging(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.logging_enabled = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsLogLevelChanged(level) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.logging_level = level;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsLogPathChanged(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.logging_path = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsLanguageChanged(language) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.language = language;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsPrimaryFontDropdownToggled => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.primary_font_open = !state.primary_font_open;
                    if state.primary_font_open {
                        state.result_font_open = false;
                    }
                }
                Task::none()
            }
            Message::SettingsPrimaryFontChanged(choice) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.primary_font_id =
                        if state.font_options.iter().any(|option| option.id == choice) {
                            choice
                        } else {
                            fonts::default_font_id().to_string()
                        };
                    state.primary_font_open = false;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsPrimaryFontSizeChanged(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.primary_font_size = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsResultFontDropdownToggled => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.result_font_open = !state.result_font_open;
                    if state.result_font_open {
                        state.primary_font_open = false;
                    }
                }
                Task::none()
            }
            Message::SettingsResultFontChanged(choice) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.result_font_id =
                        if state.font_options.iter().any(|option| option.id == choice) {
                            choice
                        } else {
                            fonts::default_font_id().to_string()
                        };
                    state.result_font_open = false;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsResultFontSizeChanged(value) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.result_font_size = value;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsThemeChanged(choice) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.theme_choice = choice;
                    state.active_color_picker = None;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsColorPickerOpened(field) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.active_color_picker = Some(field);
                }
                Task::none()
            }
            Message::SettingsColorPickerCanceled => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.active_color_picker = None;
                }
                Task::none()
            }
            Message::SettingsColorChanged(field, color) => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.set_color_for_field(field, color);
                    state.active_color_picker = None;
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsThemeColorsReset => {
                if let Some(state) = self.settings_window.as_mut() {
                    state.reset_theme_colors();
                    state.validation_error = None;
                }
                Task::none()
            }
            Message::SettingsApply => {
                if let Some(mut state) = self.settings_window.take() {
                    if let Err(error) = self.apply_settings_from_state(&mut state) {
                        log::error!("{error}");
                        state.validation_error = Some(error);
                    }
                    self.settings_window = Some(state);
                }
                self.close_settings_window();
                Task::none()
            }
            Message::SettingsSave => {
                let mut state = match self.settings_window.take() {
                    Some(state) => state,
                    None => return Task::none(),
                };

                if let Err(error) = self.apply_settings_from_state(&mut state) {
                    log::error!("{error}");
                    state.validation_error = Some(error);
                    self.settings_window = Some(state);
                    return Task::none();
                }

                self.settings_window = Some(state);

                match settings::save_to_disk(&self.settings) {
                    Ok(()) => {
                        log::debug!("Settings saved");
                        self.close_settings_window();
                    }
                    Err(error) => {
                        if let Some(mut state) = self.settings_window.take() {
                            let message = format!("{} {}", tr("Save error: "), error);
                            log::error!("{message}");
                            state.validation_error = Some(message);
                            self.settings_window = Some(state);
                        }
                    }
                }

                Task::none()
            }
            Message::SettingsCancel => {
                self.close_settings_window();
                Task::none()
            }
            Message::SettingsLoadErrorExit => {
                std::process::exit(1);
            }
            Message::SettingsLoadErrorUseDefaults => {
                let defaults = AppSettings::default();

                if let Err(error) = self.apply_settings_to_runtime(&defaults) {
                    let message = format!("{} {}", tr("Failed to apply settings:"), error);
                    log::error!("{message}");
                    self.settings_error_modal = Some(SettingsErrorModalState::new(message));
                    self.mode = AppMode::SettingsLoadError;
                    return Task::none();
                }

                settings::replace(defaults.clone());
                self.settings = defaults;

                match settings::save_to_disk(&self.settings) {
                    Ok(()) => {
                        self.settings_error_modal = None;
                        self.mode = AppMode::Main;
                    }
                    Err(error) => {
                        let message = format!("{} {}", tr("Save error: "), error);
                        log::error!("{message}");
                        self.settings_error_modal = Some(SettingsErrorModalState::new(message));
                        self.mode = AppMode::SettingsLoadError;
                    }
                }

                Task::none()
            }
            Message::CollectionTreeToggle { tab_id, node_id } => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection.toggle_node(node_id);
                }
                Task::none()
            }
            Message::DocumentEditRequested { tab_id, node_id } => {
                let doc_state = self.tabs.iter().find(|tab| tab.id == tab_id).and_then(|tab| {
                    if !tab.collection.bson_tree.is_root_node(node_id) {
                        return None;
                    }
                    let bson = tab.collection.bson_tree.node_bson(node_id)?;
                    let document = match bson {
                        Bson::Document(doc) => doc,
                        _ => return None,
                    };

                    if tab.collection.bson_tree.is_indexes_view() {
                        DocumentModalState::new_index(
                            tab_id,
                            tab.collection.client_id,
                            tab.collection.db_name.clone(),
                            tab.collection.collection.clone(),
                            document,
                        )
                    } else {
                        DocumentModalState::new_collection_document(
                            tab_id,
                            tab.collection.client_id,
                            tab.collection.db_name.clone(),
                            tab.collection.collection.clone(),
                            document,
                        )
                    }
                });

                if let Some(state) = doc_state {
                    self.document_modal = Some(state);
                    self.mode = AppMode::DocumentModal;
                }

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::events().map(|(_id, event)| Message::WindowEvent(event)),
            keyboard::listen().map(Message::KeyboardEvent),
        ])
    }

    fn main_view(&self) -> Element<'_, Message> {
        let menu_bar = menues::build_menu_bar(self.active_palette());

        let content_grid =
            pane_grid::PaneGrid::new(&self.panes, |_, pane_state, _| match pane_state {
                PaneContent::Sidebar => pane_grid::Content::new(self.sidebar_panel()),
                PaneContent::Main => pane_grid::Content::new(self.main_panel()),
            })
            .on_resize(8, Message::PaneResized)
            .spacing(PANE_GRID_SPACING)
            .height(Length::Fill);

        Column::new().push(menu_bar).push(content_grid).spacing(0).height(Length::Fill).into()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.mode {
            AppMode::Main => self.main_view(),
            AppMode::Connections => {
                if let Some(state) = &self.connections_window {
                    let palette = self.active_palette();
                    connections_view(state, &self.connections, &palette)
                } else {
                    self.main_view()
                }
            }
            AppMode::ConnectionForm => {
                if let Some(state) = &self.connection_form {
                    let palette = self.active_palette();
                    connection_form_view(state, &palette)
                } else {
                    self.main_view()
                }
            }
            AppMode::Settings => {
                if let Some(state) = &self.settings_window {
                    settings_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::SettingsLoadError => {
                if let Some(state) = &self.settings_error_modal {
                    self.settings_error_modal_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::About => {
                let palette = self.active_palette();
                let icon_handle = shared_icon_handle(&ICON_APP_HANDLE, WINDOW_ICON_BYTES);
                ui::about::about_modal_view(palette, icon_handle)
            }
            AppMode::Licenses => {
                let palette = self.active_palette();
                ui::about::licenses_modal_view(palette)
            }
            AppMode::HelpDocs => {
                if let Some(state) = &self.help_docs_state {
                    let palette = self.active_palette();
                    help_docs_view(palette, state)
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
            AppMode::DocumentModal => {
                if let Some(state) = &self.document_modal {
                    self.document_modal_view(state)
                } else {
                    self.main_view()
                }
            }
            AppMode::ValueEditModal => {
                if let Some(state) = &self.value_edit_modal {
                    self.value_edit_modal_view(state)
                } else {
                    self.main_view()
                }
            }
        }
    }

    fn settings_error_modal_view(&self, state: &SettingsErrorModalState) -> Element<'_, Message> {
        let palette = self.active_palette();
        let title = fonts::primary_text(tr("Settings Error"), Some(6.0))
            .color(palette.text_primary.to_color());
        let message =
            fonts::primary_text(state.message.clone(), None).color(palette.text_primary.to_color());

        let exit_button = Button::new(fonts::primary_text(tr("Exit"), None))
            .padding([6, 16])
            .on_press(Message::SettingsLoadErrorExit)
            .style({
                let palette = palette.clone();
                move |_, status| palette.subtle_button_style(6.0, status)
            });

        let continue_button =
            Button::new(fonts::primary_text(tr("Continue with default settings"), None))
                .padding([6, 16])
                .on_press(Message::SettingsLoadErrorUseDefaults)
                .style({
                    let palette = palette.clone();
                    move |_, status| palette.primary_button_style(6.0, status)
                });

        let buttons = Row::new().spacing(12).push(exit_button).push(continue_button);

        let content: Element<Message> =
            Column::new().spacing(16).push(title).push(message).push(buttons).into();

        modal_layout(palette, content, Length::Fixed(520.0), 24, 12.0)
    }

    fn collection_modal_view(&self, state: &CollectionModalState) -> Element<'_, Message> {
        let palette = self.active_palette();
        let text_primary = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();
        let error_color = error_accent_color(&palette);
        let accent_color = success_accent_color(&palette);

        let (title, warning, prompt, placeholder, confirm_label) = match state.kind {
            CollectionModalKind::CreateCollection => (
                tr("Create Collection"),
                tr_format(
                    "Enter a name for the new collection in database \"{}\".",
                    &[state.db_name.as_str()],
                ),
                None,
                tr("Collection Name"),
                tr("Create"),
            ),
            CollectionModalKind::DeleteAllDocuments => (
                tr("Delete All Documents"),
                tr_format(
                    "All documents from collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                    &[state.collection.as_str(), state.db_name.as_str()],
                ),
                Some(tr_format(
                    "Confirm deletion of all documents by entering the collection name \"{}\".",
                    &[state.collection.as_str()],
                )),
                state.collection.as_str(),
                tr("Confirm Deletion"),
            ),
            CollectionModalKind::DeleteCollection => (
                tr("Delete Collection"),
                tr_format(
                    "Collection \"{}\" in database \"{}\" will be deleted along with all documents. This action cannot be undone.",
                    &[state.collection.as_str(), state.db_name.as_str()],
                ),
                Some(tr_format(
                    "Confirm deletion of the collection by entering its name \"{}\".",
                    &[state.collection.as_str()],
                )),
                state.collection.as_str(),
                tr("Confirm Deletion"),
            ),
            CollectionModalKind::RenameCollection => (
                tr("Rename Collection"),
                tr_format(
                    "Enter a new name for collection \"{}\" in database \"{}\".",
                    &[state.collection.as_str(), state.db_name.as_str()],
                ),
                None,
                tr("New Collection Name"),
                tr("Rename"),
            ),
            CollectionModalKind::DropIndex { ref index_name } => (
                tr("Delete Index"),
                tr_format(
                    "Index \"{}\" of collection \"{}\" in database \"{}\" will be deleted. This action cannot be undone.",
                    &[index_name.as_str(), state.collection.as_str(), state.db_name.as_str()],
                ),
                Some(tr_format(
                    "Confirm index deletion by entering its name \"{}\".",
                    &[index_name.as_str()],
                )),
                index_name.as_str(),
                tr("Delete Index"),
            ),
        };

        let confirm_ready = match state.kind {
            CollectionModalKind::CreateCollection => {
                !state.input.trim().is_empty() && !state.processing
            }
            CollectionModalKind::DeleteAllDocuments | CollectionModalKind::DeleteCollection => {
                state.input.trim() == state.collection && !state.processing
            }
            CollectionModalKind::RenameCollection => {
                let trimmed = state.input.trim();
                !trimmed.is_empty() && trimmed != state.collection && !state.processing
            }
            CollectionModalKind::DropIndex { ref index_name } => {
                state.input.trim() == index_name && !state.processing
            }
        };

        let mut column = Column::new()
            .spacing(16)
            .push(fonts::primary_text(title, Some(6.0)).color(text_primary))
            .push(fonts::primary_text(warning, None).color(muted_color));

        if let Some(prompt) = prompt {
            column = column.push(fonts::primary_text(prompt, Some(-1.0)).color(muted_color));
        }

        let input_field = text_input(placeholder, &state.input)
            .padding([6, 10])
            .width(Length::Fill)
            .on_input(Message::CollectionModalInputChanged);

        column = column.push(input_field);

        if let Some(error) = &state.error {
            column = column.push(fonts::primary_text(error.clone(), Some(-1.0)).color(error_color));
        }

        if state.processing {
            column = column
                .push(fonts::primary_text(tr("Processing..."), Some(-1.0)).color(accent_color));
        }

        let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
            .padding([6, 16])
            .on_press(Message::CollectionModalCancel)
            .style({
                let palette = palette.clone();
                move |_, status| palette.subtle_button_style(6.0, status)
            });

        let mut confirm_button =
            Button::new(fonts::primary_text(confirm_label, None)).padding([6, 16]);
        if confirm_ready {
            confirm_button = confirm_button
                .style({
                    let palette = palette.clone();
                    move |_, status| palette.primary_button_style(6.0, status)
                })
                .on_press(Message::CollectionModalConfirm);
        } else {
            confirm_button = confirm_button.style({
                let palette = palette.clone();
                move |_, _| palette.primary_button_style(6.0, button::Status::Disabled)
            });
        }

        let buttons = Row::new().spacing(12).push(cancel_button).push(confirm_button);

        column = column.push(buttons);

        let content: Element<Message> = column.into();
        modal_layout(palette, content, Length::Fixed(420.0), 24, 12.0)
    }

    fn database_modal_view(&self, state: &DatabaseModalState) -> Element<'_, Message> {
        let palette = self.active_palette();
        let text_primary = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();
        let error_color = error_accent_color(&palette);
        let accent_color = success_accent_color(&palette);

        let base = match &state.mode {
            DatabaseModalMode::Drop { db_name } => {
                let warning = tr_format(
                    "Database \"{}\" will be deleted along with all collections and documents. This action cannot be undone.",
                    &[db_name.as_str()],
                );
                let prompt = tr_format(
                    "Confirm deletion of all data by entering the database name \"{}\".",
                    &[db_name.as_str()],
                );

                let confirm_ready = !state.processing && state.input.trim() == db_name;

                let mut column = Column::new()
                    .spacing(16)
                    .push(fonts::primary_text(tr("Delete Database"), Some(6.0)).color(text_primary))
                    .push(fonts::primary_text(warning, None).color(muted_color))
                    .push(fonts::primary_text(prompt, Some(-1.0)).color(muted_color));

                let input_field = text_input(tr("Database name"), &state.input)
                    .padding([6, 10])
                    .width(Length::Fill)
                    .on_input(Message::DatabaseModalInputChanged);

                column = column.push(input_field);

                if let Some(error) = &state.error {
                    column = column
                        .push(fonts::primary_text(error.clone(), Some(-1.0)).color(error_color));
                }

                if state.processing {
                    column = column.push(
                        fonts::primary_text(tr("Processing..."), Some(-1.0)).color(accent_color),
                    );
                }

                let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
                    .padding([6, 16])
                    .on_press(Message::DatabaseModalCancel)
                    .style({
                        let palette = palette.clone();
                        move |_, status| palette.subtle_button_style(6.0, status)
                    });

                let mut confirm_button =
                    Button::new(fonts::primary_text(tr("Confirm Deletion"), None)).padding([6, 16]);

                if confirm_ready {
                    confirm_button = confirm_button
                        .style({
                            let palette = palette.clone();
                            move |_, status| palette.primary_button_style(6.0, status)
                        })
                        .on_press(Message::DatabaseModalConfirm);
                } else {
                    confirm_button = confirm_button.style({
                        let palette = palette.clone();
                        move |_, _| palette.primary_button_style(6.0, button::Status::Disabled)
                    });
                }

                let buttons = Row::new().spacing(12).push(cancel_button).push(confirm_button);
                column = column.push(buttons);

                column
            }
            DatabaseModalMode::Create => {
                let confirm_ready = !state.processing
                    && !state.input.trim().is_empty()
                    && !state.collection_input.trim().is_empty();

                let mut column = Column::new()
                    .spacing(16)
                    .push(fonts::primary_text(tr("Create Database"), Some(6.0)).color(text_primary))
                    .push(fonts::primary_text(tr(
                            "MongoDB creates a database only when the first collection is created. Provide the database name and the first collection to create immediately."
                        ), Some(-1.0)).color(muted_color));

                let input_field = text_input(tr("Database name"), &state.input)
                    .padding([6, 10])
                    .width(Length::Fill)
                    .on_input(Message::DatabaseModalInputChanged);

                let collection_field =
                    text_input(tr("First collection name"), &state.collection_input)
                        .padding([6, 10])
                        .width(Length::Fill)
                        .on_input(Message::DatabaseModalCollectionInputChanged);

                column = column.push(input_field).push(collection_field);

                if let Some(error) = &state.error {
                    column = column
                        .push(fonts::primary_text(error.clone(), Some(-1.0)).color(error_color));
                }

                if state.processing {
                    column = column.push(
                        fonts::primary_text(tr("Creating database..."), Some(-1.0))
                            .color(accent_color),
                    );
                }

                let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
                    .padding([6, 16])
                    .on_press(Message::DatabaseModalCancel)
                    .style({
                        let palette = palette.clone();
                        move |_, status| palette.subtle_button_style(6.0, status)
                    });

                let mut confirm_button =
                    Button::new(fonts::primary_text(tr("Create"), None)).padding([6, 16]);

                if confirm_ready {
                    confirm_button = confirm_button
                        .style({
                            let palette = palette.clone();
                            move |_, status| palette.primary_button_style(6.0, status)
                        })
                        .on_press(Message::DatabaseModalConfirm);
                } else {
                    confirm_button = confirm_button.style({
                        let palette = palette.clone();
                        move |_, _| palette.primary_button_style(6.0, button::Status::Disabled)
                    });
                }

                let buttons = Row::new().spacing(12).push(cancel_button).push(confirm_button);
                column = column.push(buttons);

                column
            }
        };

        let content: Element<Message> = base.into();
        modal_layout(palette, content, Length::Fixed(420.0), 24, 12.0)
    }

    fn document_modal_view<'a>(&self, state: &'a DocumentModalState) -> Element<'a, Message> {
        let palette = self.active_palette();
        let text_primary = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();
        let error_color = error_accent_color(&palette);
        let accent_color = success_accent_color(&palette);

        let (title_text, hint_text, saving_text) = match &state.kind {
            DocumentModalKind::CollectionDocument { .. } => (
                tr("Edit Document"),
                tr(
                    "Edit the JSON representation of the document. The document will be fully replaced on save.",
                ),
                tr("Saving document..."),
            ),
            DocumentModalKind::Index { .. } => (
                tr("Edit TTL Index"),
                tr(
                    "Only the \"expireAfterSeconds\" field value can be changed. Other parameters will be ignored.",
                ),
                tr("Saving index..."),
            ),
        };

        let title = fonts::primary_text(title_text, Some(6.0)).color(text_primary);

        let hint = fonts::primary_text(hint_text, Some(-1.0)).color(muted_color);

        let editor = text_editor::TextEditor::new(&state.editor)
            .font(MONO_FONT)
            .wrapping(Wrapping::Glyph)
            .height(Length::Shrink)
            .on_action(Message::DocumentModalEditorAction);

        let editor_scroll = Scrollable::new(editor).width(Length::Fill).height(Length::Fill);

        let editor_container =
            Container::new(editor_scroll).width(Length::Fill).height(Length::Fill).style({
                let palette = palette.clone();
                move |_| container::Style {
                    border: border::rounded(8).width(1).color(palette.widget_border_color()),
                    background: Some(palette.widget_background_color().into()),
                    ..Default::default()
                }
            });

        let mut column = Column::new().spacing(16).push(title).push(hint).push(editor_container);

        if let Some(error) = &state.error {
            column = column.push(fonts::primary_text(error.clone(), Some(-1.0)).color(error_color));
        }

        if state.processing {
            column = column.push(fonts::primary_text(saving_text, Some(-1.0)).color(accent_color));
        }

        let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
            .padding([6, 16])
            .on_press(Message::DocumentModalCancel)
            .style({
                let palette = palette.clone();
                move |_, status| palette.subtle_button_style(6.0, status)
            });

        let mut save_button = Button::new(fonts::primary_text(tr("Save"), None)).padding([6, 16]);
        if state.processing {
            save_button = save_button.style({
                let palette = palette.clone();
                move |_, _| palette.primary_button_style(6.0, button::Status::Disabled)
            });
        } else {
            save_button = save_button
                .style({
                    let palette = palette.clone();
                    move |_, status| palette.primary_button_style(6.0, status)
                })
                .on_press(Message::DocumentModalSave);
        }

        let buttons = Row::new().spacing(12).push(cancel_button).push(save_button);
        column = column.push(buttons);
        let content: Element<Message> = column.into();
        modal_layout(palette, content, Length::Fixed(600.0), 24, 12.0)
    }

    fn value_edit_modal_view<'a>(&self, state: &'a ValueEditModalState) -> Element<'a, Message> {
        let palette = self.active_palette();
        let text_primary = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();
        let error_color = error_accent_color(&palette);
        let accent_color = success_accent_color(&palette);
        let fonts_state = fonts::active_fonts();
        let bold_font = Font { weight: Weight::Bold, ..fonts_state.primary_font };

        let description = Column::new()
            .spacing(4)
            .push(
                fonts::primary_text(tr("Field value will be modified"), None)
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill),
            )
            .push(
                fonts::primary_text(state.path.clone(), None)
                    .wrapping(Wrapping::Word)
                    .width(Length::Fill)
                    .font(bold_font),
            );

        let editor = text_editor::TextEditor::new(&state.value_editor)
            .font(MONO_FONT)
            .wrapping(Wrapping::Glyph)
            .height(Length::Shrink)
            .on_action(Message::ValueEditModalEditorAction);

        let editor_scroll =
            Scrollable::new(editor).width(Length::Fill).height(Length::Fixed(220.0));

        let value_editor = Container::new(editor_scroll)
            .width(Length::FillPortion(5))
            .height(Length::Fixed(220.0))
            .style({
                let palette = palette.clone();
                move |_| container::Style {
                    border: border::rounded(6).width(1).color(palette.widget_border_color()),
                    background: Some(palette.widget_background_color().into()),
                    ..Default::default()
                }
            });

        let type_indicator = Container::new(
            fonts::primary_text(state.value_label.clone(), None)
                .color(text_primary)
                .wrapping(Wrapping::Word)
                .width(Length::Fill),
        )
        .padding([6, 10])
        .width(Length::FillPortion(2))
        .style({
            let palette = palette.clone();
            move |_| container::Style {
                border: border::rounded(6).width(1).color(palette.widget_border_color()),
                background: Some(palette.widget_background_color().into()),
                ..Default::default()
            }
        });

        let inputs_row = Row::new().spacing(12).push(value_editor);

        let type_label = Column::new()
            .spacing(4)
            .push(
                fonts::primary_text(tr("Value Type"), None)
                    .color(muted_color)
                    .wrapping(Wrapping::Word)
                    .width(Length::Shrink),
            )
            .push(type_indicator);

        let type_row = Row::new().spacing(12).push(type_label);

        let mut column =
            Column::new().spacing(16).push(description).push(inputs_row).push(type_row);

        if let Some(error) = &state.error {
            column = column.push(fonts::primary_text(error.clone(), Some(-1.0)).color(error_color));
        }

        if state.processing {
            column = column
                .push(fonts::primary_text(tr("Saving value..."), Some(-1.0)).color(accent_color));
        }

        let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
            .padding([6, 16])
            .on_press(Message::ValueEditModalCancel)
            .style({
                let palette = palette.clone();
                move |_, status| palette.subtle_button_style(6.0, status)
            });

        let mut save_button = Button::new(fonts::primary_text(tr("Save"), None)).padding([6, 16]);
        if state.processing {
            save_button = save_button.style({
                let palette = palette.clone();
                move |_, _| palette.primary_button_style(6.0, button::Status::Disabled)
            });
        } else {
            save_button = save_button
                .style({
                    let palette = palette.clone();
                    move |_, status| palette.primary_button_style(6.0, status)
                })
                .on_press(Message::ValueEditModalSave);
        }

        let buttons = Row::new()
            .spacing(12)
            .push(Space::new().width(Length::Fill))
            .push(cancel_button)
            .push(save_button);
        column = column.push(buttons);

        let content: Element<Message> = column.into();
        modal_layout(palette, content, Length::Fixed(480.0), 24, 12.0)
    }

    fn theme(&self) -> Theme {
        match self.settings.theme_choice {
            ThemeChoice::System => Theme::default(Mode::None),
            ThemeChoice::Light => Theme::Light,
            ThemeChoice::Dark => Theme::Dark,
            ThemeChoice::SolarizedLight => Theme::Light,
            ThemeChoice::SolarizedDark => Theme::Dark,
            ThemeChoice::NordLight => Theme::Light,
            ThemeChoice::NordDark => Theme::Dark,
            ThemeChoice::GruvboxLight => Theme::Light,
            ThemeChoice::GruvboxDark => Theme::Dark,
            ThemeChoice::OneLight => Theme::Light,
            ThemeChoice::OneDark => Theme::Dark,
        }
    }

    fn sidebar_panel(&self) -> Element<'_, Message> {
        let mut list = Column::new().spacing(4);
        let palette = self.active_palette();
        let muted_color = palette.text_muted.to_color();

        if self.clients.is_empty() {
            list =
                list.push(fonts::primary_text(tr("No connections"), Some(6.0)).color(muted_color));
        } else {
            for client in &self.clients {
                list = list.push(self.render_client(client));
            }
        }

        let scrollable = Scrollable::new(list).width(Length::Fill).height(Length::Fill);

        let pane_bg = palette.widget_background_color();
        let pane_border = palette.widget_border_color();

        Container::new(scrollable)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(pane_bg.into()),
                border: border::rounded(6).width(1).color(pane_border),
                ..Default::default()
            })
            .into()
    }

    fn render_client<'a>(&'a self, client: &'a OMDBClient) -> Element<'a, Message> {
        let icon_size = fonts::active_fonts().primary_size * 1.5;
        let indicator = if client.expanded { "v" } else { ">" };
        let status_label = match &client.status {
            ConnectionStatus::Connecting => tr("Connecting...").to_owned(),
            ConnectionStatus::Ready => tr("Ready").to_owned(),
            ConnectionStatus::Failed(err) => format!("{} {}", tr("Error:"), err),
        };

        let palette = self.active_palette();
        let text_color = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();

        let header_row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(fonts::primary_text(indicator, None).color(muted_color))
            .push(
                Image::new(shared_icon_handle(&ICON_NETWORK_HANDLE, ICON_NETWORK_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(fonts::primary_text(client.name.clone(), Some(6.0)).color(text_color))
            .push(fonts::primary_text(status_label.clone(), Some(6.0)).color(muted_color));

        let base_button =
            self.sidebar_button(header_row, 0.0, Message::ToggleClient(client.id), None);

        let context_client_id = client.id;
        let is_ready = matches!(client.status, ConnectionStatus::Ready);

        let menu = menues::connection_context_menu(
            base_button,
            palette.clone(),
            context_client_id,
            is_ready,
        );

        let mut column = Column::new().spacing(4).push(menu);

        if matches!(client.status, ConnectionStatus::Failed(_)) {
            column = column.push(
                Row::new().spacing(8).push(Space::new().width(Length::Fixed(16.0))).push(
                    fonts::primary_text(status_label, Some(6.0))
                        .color(Color::from_rgb8(0xd9, 0x53, 0x4f)),
                ),
            );
        }

        if client.expanded && matches!(client.status, ConnectionStatus::Ready) {
            if client.databases.is_empty() {
                column = column.push(
                    Row::new().spacing(8).push(Space::new().width(Length::Fixed(16.0))).push(
                        fonts::primary_text(tr("No databases"), Some(6.0)).color(muted_color),
                    ),
                );
            } else {
                for database in &client.databases {
                    column = column.push(self.render_database(client.id, database));
                }
            }
        }

        Container::new(column).into()
    }

    fn render_database<'a>(
        &'a self,
        client_id: ClientId,
        database: &'a DatabaseNode,
    ) -> Element<'a, Message> {
        let primary_font_size = fonts::active_fonts().primary_size;
        let indicator = if database.expanded { "v" } else { ">" };
        let icon_size = primary_font_size * 1.5;
        let palette = self.active_palette();
        let text_color = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();

        let db_row = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(fonts::primary_text(indicator, None).color(muted_color))
            .push(
                Image::new(shared_icon_handle(&ICON_DATABASE_HANDLE, ICON_DATABASE_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(fonts::primary_text(database.name.clone(), None).color(text_color));

        let base_button = self.sidebar_button(
            db_row,
            16.0,
            Message::ToggleDatabase { client_id, db_name: database.name.clone() },
            None,
        );

        let db_name_owned = database.name.clone();
        let menu = menues::database_context_menu(
            base_button,
            palette.clone(),
            client_id,
            db_name_owned.clone(),
        );

        let mut column = Column::new().spacing(4).push(menu);

        if database.expanded {
            match &database.state {
                DatabaseState::Idle => {}
                DatabaseState::Loading => {
                    column = column.push(
                        Row::new().spacing(8).push(Space::new().width(Length::Fixed(32.0))).push(
                            fonts::primary_text(tr("Loading collections..."), Some(6.0))
                                .color(muted_color),
                        ),
                    );
                }
                DatabaseState::Error(error) => {
                    column = column.push(
                        Row::new().spacing(8).push(Space::new().width(Length::Fixed(32.0))).push(
                            fonts::primary_text(format!("{} {}", tr("Error:"), error), Some(6.0)),
                        ),
                    );
                }
                DatabaseState::Loaded => {
                    if database.collections.is_empty() {
                        column = column.push(
                            Row::new()
                                .spacing(8)
                                .push(Space::new().width(Length::Fixed(32.0)))
                                .push(
                                    fonts::primary_text(tr("No collections"), Some(6.0))
                                        .color(muted_color),
                                ),
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
        let icon_size = fonts::active_fonts().primary_size * 1.5;
        let palette = self.active_palette();
        let text_color = palette.text_primary.to_color();
        let _muted_color = palette.text_muted.to_color();

        let row = Row::new()
            .spacing(6)
            .align_y(Vertical::Center)
            .push(
                Image::new(shared_icon_handle(&ICON_COLLECTION_HANDLE, ICON_COLLECTION_BYTES))
                    .width(Length::Fixed(icon_size))
                    .height(Length::Fixed(icon_size)),
            )
            .push(fonts::primary_text(collection.name.clone(), None).color(text_color));

        let db_name_owned = db_name.to_owned();
        let collection_name = collection.name.clone();

        let base_button = self.sidebar_button(
            row,
            32.0,
            Message::CollectionClicked {
                client_id,
                db_name: db_name_owned.clone(),
                collection: collection_name.clone(),
            },
            Some(Message::CollectionContextMenu {
                client_id,
                db_name: db_name_owned.clone(),
                collection: collection_name.clone(),
                action: CollectionContextAction::ViewDocuments,
            }),
        );

        menues::collection_context_menu(
            base_button,
            palette.clone(),
            client_id,
            db_name_owned,
            collection_name,
        )
    }

    fn sidebar_button<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        indent: f32,
        on_press: Message,
        middle_press: Option<Message>,
    ) -> Element<'a, Message> {
        let palette = self.active_palette();
        let button = Button::new(content)
            .padding([4, 4])
            .width(Length::Shrink)
            .height(Length::Shrink)
            .style(move |_, status| palette.subtle_button_style(6.0, status))
            .on_press(on_press);

        let row: Element<Message> = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(Space::new().width(Length::Fixed(indent.max(0.0))))
            .push(button)
            .into();

        if let Some(message) = middle_press {
            mouse_area(row).on_middle_press(message).into()
        } else {
            row
        }
    }

    fn tab_bar_available_width(&self) -> Option<f32> {
        let window_size = self.window_size?;
        let regions =
            self.panes.layout().pane_regions(PANE_GRID_SPACING, PANE_GRID_MIN_SIZE, window_size);
        let main_pane = self
            .panes
            .iter()
            .find(|(_, content)| **content == PaneContent::Main)
            .map(|(pane, _)| *pane)?;
        let bounds = regions.get(&main_pane)?;
        let mut width = bounds.width - (MAIN_PANEL_PADDING * 2.0) - (TAB_HEADER_PADDING_X * 2.0);
        if width < 0.0 {
            width = 0.0;
        }
        Some(width)
    }

    fn estimate_tab_width(&self, title: &str) -> f32 {
        let font_size = fonts::active_fonts().primary_size;
        let char_width = font_size * TAB_CHAR_WIDTH_FACTOR;
        let title_width = title.chars().count() as f32 * char_width;
        let close_width = char_width;
        let padding = (TAB_TITLE_PADDING_X + TAB_CONTAINER_PADDING_X + TAB_CLOSE_PADDING_X) * 2.0;
        title_width + close_width + padding + TAB_INNER_SPACING
    }

    fn estimate_tabs_row_width(&self) -> f32 {
        let mut width = 0.0;
        for (index, tab) in self.tabs.iter().enumerate() {
            if index > 0 {
                width += TAB_ROW_SPACING;
            }
            width += self.estimate_tab_width(&tab.title);
        }
        width
    }

    fn should_reserve_tab_scrollbar(&self) -> bool {
        let Some(available) = self.tab_bar_available_width() else {
            return false;
        };
        let threshold =
            if available > TAB_WIDTH_TOLERANCE { available - TAB_WIDTH_TOLERANCE } else { 0.0 };
        self.estimate_tabs_row_width() > threshold
    }

    fn main_panel(&self) -> Element<'_, Message> {
        let palette = self.active_palette();
        let pane_bg = palette.widget_background_color();
        let pane_border = palette.widget_border_color();
        let text_color = palette.text_primary.to_color();
        let muted_color = palette.text_muted.to_color();
        if self.tabs.is_empty() {
            Container::new(fonts::primary_text(tr("No tabs opened"), None).color(muted_color))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .padding(16)
                .style(move |_| container::Style {
                    background: Some(pane_bg.into()),
                    border: border::rounded(6).width(1).color(pane_border),
                    ..Default::default()
                })
                .into()
        } else {
            let active_id = self.active_tab.or_else(|| self.tabs.first().map(|tab| tab.id));

            let mut tabs_row = Row::new().spacing(TAB_ROW_SPACING).align_y(Vertical::Center);

            let active_bg = palette.subtle_buttons.hover.to_color();
            let inactive_bg = palette.subtle_buttons.active.to_color();
            let border_color = palette.subtle_buttons.border.to_color();

            for tab in &self.tabs {
                let is_active = active_id == Some(tab.id);
                let tab_background =
                    tab.color.unwrap_or_else(|| if is_active { active_bg } else { inactive_bg });
                let tab_text_color = if let Some(custom_color) = tab.color {
                    if color_luminance(custom_color) > 0.5 { Color::BLACK } else { Color::WHITE }
                } else {
                    text_color
                };

                let title_label = Container::new(fonts::primary_text(tab.title.clone(), None))
                    .padding([4.0, TAB_TITLE_PADDING_X]);

                let title_area = mouse_area(title_label).on_press(Message::TabSelected(tab.id));

                let close_button = Button::new(fonts::primary_text(tr("×"), None))
                    .padding([4.0, TAB_CLOSE_PADDING_X])
                    .on_press(Message::TabClosed(tab.id));

                let tab_inner = Row::new()
                    .spacing(TAB_INNER_SPACING)
                    .align_y(Vertical::Center)
                    .push(title_area)
                    .push(close_button);

                let tab_container = Container::new(tab_inner)
                    .padding([4.0, TAB_CONTAINER_PADDING_X])
                    .style(move |_| container::Style {
                        background: Some(tab_background.into()),
                        text_color: Some(tab_text_color),
                        border: border::rounded(6).width(1).color(border_color),
                        ..Default::default()
                    });
                let tab_container =
                    mouse_area(tab_container).on_middle_press(Message::TabClosed(tab.id));

                let menu_palette = palette.clone();
                let menu_border = palette.clone();
                let menu_tab_id = tab.id;
                let menu = move || {
                    let item_palette = menu_palette.clone();
                    let border_palette = menu_border.clone();
                    let duplicate_button =
                        Button::new(fonts::primary_text(tr("Duplicate Tab"), None))
                            .padding([4, 8])
                            .on_press(Message::DuplicateTab(menu_tab_id))
                            .style(move |_, status| item_palette.menu_button_style(6.0, status));
                    let color_palette = menu_palette.clone();
                    let color_button = Button::new(fonts::primary_text(tr("Tab Color"), None))
                        .padding([4, 8])
                        .on_press(Message::TabColorPickerOpened(menu_tab_id))
                        .style(move |_, status| color_palette.menu_button_style(6.0, status));
                    let reset_palette = menu_palette.clone();
                    let reset_button =
                        Button::new(fonts::primary_text(tr("Reset Tab Color"), None))
                            .padding([4, 8])
                            .on_press(Message::TabColorReset(menu_tab_id))
                            .style(move |_, status| reset_palette.menu_button_style(6.0, status));
                    let content = Column::new()
                        .spacing(6)
                        .push(color_button)
                        .push(reset_button)
                        .push(duplicate_button);
                    Container::new(content)
                        .style(move |_| iced::widget::container::Style {
                            background: Some(border_palette.menu.background.to_color().into()),
                            border: border::rounded(6.0)
                                .width(1)
                                .color(border_palette.widget_border_color()),
                            shadow: Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                                offset: iced::Vector::new(0.0, 3.0),
                                blur_radius: 10.0,
                            },
                            ..Default::default()
                        })
                        .padding([4, 6])
                        .into()
                };

                let tab_with_menu = ContextMenu::new(tab_container, menu);
                let show_picker = self.tab_color_picker == Some(tab.id);
                let picker_color = tab_background;
                let tab_with_picker = ColorPicker::new(
                    show_picker,
                    picker_color,
                    tab_with_menu,
                    Message::TabColorPickerCanceled,
                    move |selected| Message::TabColorChanged {
                        tab_id: menu_tab_id,
                        color: selected,
                    },
                );
                tabs_row = tabs_row.push(tab_with_picker);
            }

            let tabs_content = if self.should_reserve_tab_scrollbar() {
                Container::new(tabs_row).padding(Padding::default().bottom(TAB_SCROLLBAR_PADDING))
            } else {
                Container::new(tabs_row)
            };

            let header_scroll = Scrollable::with_direction(
                tabs_content,
                scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
            )
            .height(Length::Shrink)
            .width(Length::Fill);

            let header = Container::new(header_scroll)
                .width(Length::Fill)
                .padding([0.0, TAB_HEADER_PADDING_X]);

            let content = active_id
                .and_then(|id| self.tabs.iter().find(|tab| tab.id == id))
                .map(|tab| tab.view())
                .unwrap_or_else(|| {
                    Container::new(
                        fonts::primary_text(tr("No active tab"), None).color(muted_color),
                    )
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
                .padding(MAIN_PANEL_PADDING)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(pane_bg.into()),
                    border: border::rounded(6).width(1).color(pane_border),
                    ..Default::default()
                })
                .into()
        }
    }

    fn open_collection_tab(
        &mut self,
        client_id: ClientId,
        db_name: String,
        collection: String,
    ) -> TabId {
        let mut client_name = String::from(tr("Unknown client"));
        let mut values = vec![Bson::String(String::from(tr(
            "Query not yet executed. Compose a query and press Send.",
        )))];

        if let Some(client) = self.clients.iter().find(|c| c.id == client_id) {
            client_name = client.name.clone();

            if client.handle.is_none() {
                values = vec![Bson::String(String::from(tr(
                    "Connection inactive. Reconnect and run the query again.",
                )))];
            }
        }

        log::debug!("Open tab client_id={} db={} collection={}", client_id, db_name, collection);
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(TabData::new_collection(
            id,
            client_id,
            client_name,
            db_name,
            collection,
            values,
            &self.settings,
        ));
        self.active_tab = Some(id);
        id
    }

    fn focus_collection_editor(&self, tab_id: TabId) -> Task<Message> {
        let Some(tab) = self.tabs.iter().find(|tab| tab.id == tab_id) else {
            return Task::none();
        };

        focus::<Message>(tab.collection.focus_anchor.clone()).chain(focus_next())
    }

    fn duplicate_collection_tab(&mut self, tab_id: TabId) {
        let snapshot = self.tabs.iter().find(|tab| tab.id == tab_id).map(|tab| {
            (
                tab.collection.client_id,
                tab.collection.db_name.clone(),
                tab.collection.collection.clone(),
                tab.collection.editor.text().to_string(),
                tab.color,
            )
        });

        if let Some((client_id, db_name, collection, query_text, color)) = snapshot {
            let new_tab_id = self.open_collection_tab(client_id, db_name, collection);
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == new_tab_id) {
                tab.collection.editor = TextEditorContent::with_text(&query_text);
                tab.color = color;
            }
            log::debug!("Tab duplicated from={} to={}", tab_id, new_tab_id);
        }
    }

    fn open_database_stats_tab(&mut self, client_id: ClientId, db_name: String) -> TabId {
        let tab_id =
            self.open_collection_tab(client_id, db_name.clone(), String::from(tr("(database)")));

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.collection.editor = TextEditorContent::with_text(tr("db.stats()"));
            tab.title = String::from(tr("stats"));
        }

        tab_id
    }

    fn open_database_empty_tab(&mut self, client_id: ClientId, db_name: String) -> TabId {
        let tab_id =
            self.open_collection_tab(client_id, db_name.clone(), String::from(tr("(database)")));

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.collection.editor = TextEditorContent::with_text("");
            tab.title = db_name;
        }

        tab_id
    }

    fn open_collection_stats_tab(
        &mut self,
        client_id: ClientId,
        db_name: String,
        collection: String,
    ) -> TabId {
        let tab_id =
            self.open_collection_tab(client_id, db_name.clone(), format!("{collection} (stats)"));

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            let command = format!(
                "db.runCommand({{ \"collStats\": \"{collection}\" }})",
                collection = collection
            );
            tab.collection.editor = TextEditorContent::with_text(&command);
            tab.title = String::from(tr("collStats"));
        }

        tab_id
    }

    fn open_collection_indexes_tab(
        &mut self,
        client_id: ClientId,
        db_name: String,
        collection: String,
    ) -> TabId {
        let tab_id = self.open_collection_tab(client_id, db_name, collection.clone());

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            let command = format!(
                "db.getCollection('{collection_name}').getIndexes()",
                collection_name = collection
            );
            tab.collection.editor = TextEditorContent::with_text(&command);
            tab.title = String::from(tr("indexes"));
        }

        tab_id
    }

    fn open_collection_create_index_tab(
        &mut self,
        client_id: ClientId,
        db_name: String,
        collection: String,
    ) -> TabId {
        let tab_id = self.open_collection_tab(client_id, db_name, collection.clone());

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            let template = format!(
                "db.getCollection('{collection_name}').createIndex(\n    {{ \"field\": 1 }},\n    {{ \"name\": \"field_1\" }}\n)",
                collection_name = collection
            );
            tab.collection.editor = TextEditorContent::with_text(&template);
            tab.title = format!("{} createIndex", collection);
        }

        tab_id
    }

    fn open_server_status_tab(&mut self, client_id: ClientId) -> Option<Task<Message>> {
        let (client_name, handle) = self
            .clients
            .iter()
            .find(|client| client.id == client_id)
            .map(|client| (client.name.clone(), client.handle.clone()))?;

        let handle = handle?;

        let db_name = String::from(tr("admin"));
        let collection_label = String::from(tr("serverStatus"));
        let placeholder = vec![Bson::String(String::from(tr("Loading serverStatus...")))];

        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let mut tab = TabData::new_collection(
            id,
            client_id,
            client_name.clone(),
            db_name,
            collection_label,
            placeholder,
            &self.settings,
        );

        tab.title = String::from(tr("serverStatus"));
        tab.collection.editor =
            TextEditorContent::with_text(tr("db.runCommand({ serverStatus: 1 })"));

        self.tabs.push(tab);
        self.active_tab = Some(id);

        log::debug!("Open serverStatus tab client_id={} tab_id={}", client_id, id);
        Some(Self::server_status_task(handle, id))
    }

    fn server_status_task(handle: Arc<Client>, tab_id: TabId) -> Task<Message> {
        Task::perform(
            async move {
                let start = Instant::now();
                let result = handle
                    .database("admin")
                    .run_command(doc! { "serverStatus": 1 })
                    .run()
                    .map_err(|error| error.to_string());
                (result, start.elapsed())
            },
            move |(result, duration)| {
                let mapped = result.map(|document| QueryResult::SingleDocument { document });
                Message::CollectionQueryCompleted { tab_id, result: mapped, duration }
            },
        )
    }

    fn refresh_databases(&mut self, client_id: ClientId) -> Task<Message> {
        let Some(client) = self.clients.iter_mut().find(|client| client.id == client_id) else {
            return Task::none();
        };

        log::debug!(
            "Refresh databases client_id={} ssh_tunnel={}",
            client_id,
            client.entry.ssh_tunnel.enabled
        );
        if client.entry.ssh_tunnel.enabled {
            client.status = ConnectionStatus::Connecting;
            client.handle = None;
            client.ssh_tunnel = None;
            for database in &mut client.databases {
                database.state = DatabaseState::Loading;
            }

            let connection = OMDBConnection::from_entry(client.entry.clone());
            return Task::perform(async move { connect_and_discover(connection) }, move |result| {
                Message::ConnectionCompleted { client_id, result }
            });
        }

        let Some(handle) = client.handle.clone() else {
            return Task::none();
        };

        for database in &mut client.databases {
            database.state = DatabaseState::Loading;
        }

        let include_filter = client.entry.include_filter.clone();
        let exclude_filter = client.entry.exclude_filter.clone();

        Task::perform(
            async move {
                let names =
                    handle.list_database_names().run().map_err(|error| error.to_string())?;
                Ok(filter_databases(names, &include_filter, &exclude_filter))
            },
            move |result| Message::DatabasesRefreshed { client_id, result },
        )
    }

    fn add_collection_to_tree(&mut self, client_id: ClientId, db_name: &str, collection: &str) {
        if let Some(client) = self.clients.iter_mut().find(|c| c.id == client_id) {
            if let Some(database) = client.databases.iter_mut().find(|d| d.name == db_name) {
                if database.collections.iter().any(|node| node.name == collection) {
                    return;
                }
                database.collections.push(CollectionNode::new(collection.to_string()));
                database.collections.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
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

        if self.settings.close_tabs_on_database_close {
            self.tabs.retain(|tab| {
                !(tab.collection.client_id == client_id && tab.collection.db_name == db_name)
            });

            if let Some(active) = self.active_tab {
                if self.tabs.iter().all(|tab| tab.id != active) {
                    self.active_tab = self.tabs.last().map(|tab| tab.id);
                }
            }
        }
    }

    fn close_client_connection(&mut self, client_id: ClientId) {
        log::debug!("Close connection client_id={}", client_id);
        self.clients.retain(|client| client.id != client_id);

        if self.last_collection_click.as_ref().is_some_and(|click| click.client_id == client_id) {
            self.last_collection_click = None;
        }

        if self.document_modal.as_ref().is_some_and(|modal| modal.client_id == client_id) {
            self.document_modal = None;
            self.mode = AppMode::Main;
        }

        if self.settings.close_tabs_on_database_close {
            let removed: HashSet<TabId> = self
                .tabs
                .iter()
                .filter(|tab| tab.collection.client_id == client_id)
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
                tab.title = new_name.to_string();
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

    fn query_operation_label(operation: &QueryOperation) -> &'static str {
        match operation {
            QueryOperation::Find { .. } => "find",
            QueryOperation::FindOne { .. } => "findOne",
            QueryOperation::Count { .. } => "count",
            QueryOperation::CountDocuments { .. } => "countDocuments",
            QueryOperation::EstimatedDocumentCount { .. } => "estimatedDocumentCount",
            QueryOperation::Distinct { .. } => "distinct",
            QueryOperation::Aggregate { .. } => "aggregate",
            QueryOperation::Watch { target, .. } => match target {
                WatchTarget::Collection => "watch(collection)",
                WatchTarget::Database => "watch(database)",
            },
            QueryOperation::InsertOne { .. } => "insertOne",
            QueryOperation::InsertMany { .. } => "insertMany",
            QueryOperation::BulkWrite { .. } => "bulkWrite",
            QueryOperation::DeleteOne { .. } => "deleteOne",
            QueryOperation::DeleteMany { .. } => "deleteMany",
            QueryOperation::UpdateOne { .. } => "updateOne",
            QueryOperation::UpdateMany { .. } => "updateMany",
            QueryOperation::ReplaceOne { .. } => "replaceOne",
            QueryOperation::FindOneAndUpdate { .. } => "findOneAndUpdate",
            QueryOperation::FindOneAndReplace { .. } => "findOneAndReplace",
            QueryOperation::FindOneAndDelete { .. } => "findOneAndDelete",
            QueryOperation::ListIndexes => "getIndexes",
            QueryOperation::ReplicaSetCommand { command } => match command {
                ReplicaSetCommand::Status => "rs.status",
                ReplicaSetCommand::Config => "rs.conf",
                ReplicaSetCommand::IsMaster => "rs.isMaster",
                ReplicaSetCommand::Hello => "rs.hello",
                ReplicaSetCommand::PrintReplicationInfo => "rs.printReplicationInfo",
                ReplicaSetCommand::PrintSecondaryReplicationInfo => {
                    "rs.printSecondaryReplicationInfo"
                }
                ReplicaSetCommand::Initiate { .. } => "rs.initiate",
                ReplicaSetCommand::Reconfig { .. } => "rs.reconfig",
                ReplicaSetCommand::StepDown { .. } => "rs.stepDown",
                ReplicaSetCommand::Freeze { .. } => "rs.freeze",
                ReplicaSetCommand::Add { arbiter: true, .. } => "rs.addArb",
                ReplicaSetCommand::Add { arbiter: false, .. } => "rs.add",
                ReplicaSetCommand::Remove { .. } => "rs.remove",
                ReplicaSetCommand::SyncFrom { .. } => "rs.syncFrom",
                ReplicaSetCommand::SlaveOk => "rs.slaveOk",
            },
            QueryOperation::DatabaseCommand { .. } => "db.command",
        }
    }

    fn query_result_metrics(result: &QueryResult) -> (&'static str, usize) {
        match result {
            QueryResult::Documents(values) => ("documents", values.len()),
            QueryResult::Indexes(values) => ("indexes", values.len()),
            QueryResult::SingleDocument { .. } => ("document", 1),
            QueryResult::Distinct { values, .. } => ("distinct", values.len()),
            QueryResult::Count { .. } => ("count", 1),
        }
    }

    fn collection_query_task(&mut self, tab_id: TabId) -> Task<Message> {
        let mut request: Option<(ClientId, String, String, QueryOperation, u64, u64)> = None;

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            let collection = &mut tab.collection;
            let query_text = collection.editor.text().to_string();
            match collection.parse_query(&query_text) {
                Ok((effective_collection, operation)) => {
                    let skip = collection.skip_value();
                    let limit = collection.limit_value();
                    let op_label = Self::query_operation_label(&operation);
                    log::debug!(
                        "Query start tab_id={} client_id={} db={} collection={} op={} skip={} limit={}",
                        tab_id,
                        collection.client_id,
                        collection.db_name,
                        effective_collection,
                        op_label,
                        skip,
                        limit
                    );
                    collection.query_in_progress = true;
                    collection.last_query_duration = None;
                    collection.pending_collection = Some(effective_collection.clone());
                    request = Some((
                        collection.client_id,
                        collection.db_name.clone(),
                        effective_collection,
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
                tab.collection.set_tree_error(String::from(tr("No active connection")));
                tab.collection.query_in_progress = false;
            }
            return Task::none();
        };

        let timeout_secs = self.settings.query_timeout_secs;
        let timeout =
            if timeout_secs == 0 { None } else { Some(Duration::from_secs(timeout_secs)) };

        match operation {
            QueryOperation::Watch { pipeline, target } => {
                let target_label = match target {
                    WatchTarget::Collection => "collection",
                    WatchTarget::Database => "database",
                };
                log::debug!(
                    "Watch started tab_id={} db={} collection={} target={} limit={}",
                    tab_id,
                    db_name,
                    collection_name,
                    target_label,
                    limit
                );
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.collection
                        .set_query_result(QueryResult::Documents(Vec::new()), &self.settings);
                }
                self.collection_watch_task(
                    tab_id,
                    handle,
                    db_name,
                    collection_name,
                    target,
                    pipeline,
                    limit,
                )
            }
            operation => Task::perform(
                async move {
                    let started = Instant::now();
                    let result = run_collection_query(
                        handle,
                        db_name,
                        collection_name,
                        operation,
                        skip,
                        limit,
                        timeout,
                    );
                    (result, started.elapsed())
                },
                move |(result, duration)| Message::CollectionQueryCompleted {
                    tab_id,
                    result,
                    duration,
                },
            ),
        }
    }

    fn collection_watch_task(
        &self,
        tab_id: TabId,
        handle: Arc<Client>,
        db_name: String,
        collection_name: String,
        target: WatchTarget,
        pipeline: Vec<Document>,
        limit: u64,
    ) -> Task<Message> {
        let started = Instant::now();
        let capped_limit = if limit > usize::MAX as u64 { usize::MAX } else { limit as usize };

        let state = match open_change_stream(&handle, &db_name, &collection_name, target, pipeline)
        {
            Ok(change_stream) => WatchStreamState {
                change_stream: Some(change_stream),
                documents: Vec::new(),
                limit: capped_limit,
                started,
                finished: false,
                error: None,
            },
            Err(error) => WatchStreamState {
                change_stream: None,
                documents: Vec::new(),
                limit: capped_limit,
                started,
                finished: false,
                error: Some(error),
            },
        };

        let stream = stream::unfold(state, |mut state| async move {
            if state.finished {
                return None;
            }

            if let Some(error) = state.error.take() {
                state.finished = true;
                let duration = state.started.elapsed();
                return Some((WatchStreamEvent::Finished { result: Err(error), duration }, state));
            }

            if state.limit == 0 {
                state.finished = true;
                let duration = state.started.elapsed();
                let result = Ok(QueryResult::Documents(Vec::new()));
                return Some((WatchStreamEvent::Finished { result, duration }, state));
            }

            let mut change_stream = match state.change_stream.take() {
                Some(stream) => stream,
                None => {
                    state.finished = true;
                    let duration = state.started.elapsed();
                    let documents = std::mem::take(&mut state.documents);
                    let result = Ok(QueryResult::Documents(documents));
                    return Some((WatchStreamEvent::Finished { result, duration }, state));
                }
            };

            let next_event = change_stream.next();
            state.change_stream = Some(change_stream);

            match next_event {
                Some(Ok(event)) => {
                    let bson = match bson::to_bson(&event) {
                        Ok(bson) => bson,
                        Err(error) => {
                            state.finished = true;
                            let duration = state.started.elapsed();
                            let message = format!("BSON conversion error: {error}");
                            return Some((
                                WatchStreamEvent::Finished { result: Err(message), duration },
                                state,
                            ));
                        }
                    };

                    state.documents.push(bson);
                    if state.documents.len() >= state.limit {
                        state.finished = true;
                        let duration = state.started.elapsed();
                        let documents = std::mem::take(&mut state.documents);
                        let result = Ok(QueryResult::Documents(documents));
                        Some((WatchStreamEvent::Finished { result, duration }, state))
                    } else {
                        let snapshot = state.documents.clone();
                        Some((WatchStreamEvent::Progress(snapshot), state))
                    }
                }
                Some(Err(error)) => {
                    state.finished = true;
                    let duration = state.started.elapsed();
                    Some((
                        WatchStreamEvent::Finished { result: Err(error.to_string()), duration },
                        state,
                    ))
                }
                None => {
                    state.finished = true;
                    let duration = state.started.elapsed();
                    let documents = std::mem::take(&mut state.documents);
                    let result = Ok(QueryResult::Documents(documents));
                    Some((WatchStreamEvent::Finished { result, duration }, state))
                }
            }
        });

        Task::run(stream, move |event| match event {
            WatchStreamEvent::Progress(documents) => {
                Message::CollectionWatchProgress { tab_id, documents }
            }
            WatchStreamEvent::Finished { result, duration } => {
                Message::CollectionQueryCompleted { tab_id, result, duration }
            }
        })
    }

    fn open_connections_window(&mut self) {
        log::debug!("Open connections window");
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
        log::debug!("Close connections window");
        self.mode = AppMode::Main;
        self.connections_window = None;
        self.connection_form = None;
    }

    fn open_connection_form(&mut self, mode: ConnectionFormMode) {
        log::debug!("Open connection form mode={:?}", mode);
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

    fn open_settings_window(&mut self) {
        log::debug!("Open settings window");
        let previous_tab = self.settings_window.as_ref().map(|state| state.active_tab);

        let mut state = SettingsWindowState::from_app_settings(&self.settings);
        if let Some(tab) = previous_tab {
            state.active_tab = tab;
        }

        self.settings_window = Some(state);
        self.connections_window = None;
        self.connection_form = None;
        self.mode = AppMode::Settings;
    }

    fn open_about_modal(&mut self) {
        log::debug!("Open about modal");
        self.mode = AppMode::About;
    }

    fn open_licenses_modal(&mut self) {
        log::debug!("Open licenses modal");
        self.mode = AppMode::Licenses;
    }

    fn open_help_docs_window(&mut self) {
        log::debug!("Open help docs window");
        if self.help_docs_state.is_none() {
            self.help_docs_state = Some(HelpDocsState::new());
        }
        self.mode = AppMode::HelpDocs;
    }

    fn close_settings_window(&mut self) {
        log::debug!("Close settings window");
        self.settings_window = None;
        self.mode = AppMode::Main;
    }

    fn close_about_modal(&mut self) {
        log::debug!("Close about modal");
        self.mode = AppMode::Main;
    }

    fn close_licenses_modal(&mut self) {
        log::debug!("Close licenses modal");
        self.mode = AppMode::Main;
    }

    fn close_help_docs_window(&mut self) {
        log::debug!("Close help docs window");
        self.mode = AppMode::Main;
    }

    fn active_palette(&self) -> ThemePalette {
        self.settings.active_palette().clone()
    }

    fn apply_settings_to_runtime(&mut self, settings: &AppSettings) -> Result<(), String> {
        i18n::set_language(settings.language);
        fonts::set_active_fonts(
            &settings.primary_font,
            settings.primary_font_size as f32,
            &settings.result_font,
            settings.result_font_size as f32,
        );
        logging::apply_settings(
            settings.logging_enabled,
            settings.logging_level.to_level_filter(),
            &settings.logging_path,
        );

        for tab in &mut self.tabs {
            tab.collection.refresh_with_settings(settings);
        }

        Ok(())
    }

    fn apply_settings_from_state(&mut self, state: &mut SettingsWindowState) -> Result<(), String> {
        let active_tab = state.active_tab;
        let new_settings = state.to_app_settings()?;

        self.apply_settings_to_runtime(&new_settings)?;

        settings::replace(new_settings.clone());
        self.settings = new_settings;

        *state = SettingsWindowState::from_app_settings(&self.settings);
        state.active_tab = active_tab;

        log::debug!("Settings applied");
        Ok(())
    }

    fn add_connection_from_entry(
        &mut self,
        entry: ConnectionEntry,
    ) -> Result<Task<Message>, String> {
        entry.uri()?;
        entry.ssh_tunnel.validate()?;
        log::debug!(
            "Connect requested name={} host={} port={} ssh_tunnel={}",
            entry.name,
            entry.host,
            entry.port,
            entry.ssh_tunnel.enabled
        );
        let connection = OMDBConnection::from_entry(entry.clone());
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let client = OMDBClient::new(client_id, entry);
        self.clients.push(client);

        Ok(Task::perform(async move { connect_and_discover(connection) }, move |result| {
            Message::ConnectionCompleted { client_id, result }
        }))
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
        settings: &AppSettings,
    ) -> Self {
        let title = collection.clone();
        Self {
            id,
            title,
            collection: CollectionTab::new(
                client_id,
                client_name,
                db_name,
                collection,
                values,
                settings,
            ),
            color: None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.collection.view(self.id)
    }
}

impl OMDBClient {
    fn new(id: ClientId, entry: ConnectionEntry) -> Self {
        Self {
            id,
            name: entry.name.clone(),
            status: ConnectionStatus::Connecting,
            expanded: true,
            handle: None,
            databases: Vec::new(),
            ssh_tunnel: None,
            entry,
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

pub(crate) fn shared_icon_handle(lock: &OnceLock<Handle>, bytes: &'static [u8]) -> Handle {
    lock.get_or_init(|| Handle::from_bytes(bytes.to_vec())).clone()
}
