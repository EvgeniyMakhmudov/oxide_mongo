#![cfg(test)]

use crate::mongo::connection::ConnectionBootstrap;
use crate::mongo::query::{
    QueryResult, parse_collection_query_with_collection, run_collection_query,
};
use crate::mongo::shell::{
    bson_type_name, format_bson_shell, parse_shell_bson_value, parse_shell_json_value,
};
use crate::ui::connections::{ConnectionEntry, ConnectionFormState, ConnectionsWindowState};
use crate::ui::menues::{
    CollectionContextAction, ConnectionContextAction, DatabaseContextAction, MenuEntry, TopMenu,
};
use crate::{App, AppMode, ClientId, DEFAULT_RESULT_LIMIT, Message, TabId, TableContextAction};
use iced::Task;
use iced::futures::{StreamExt, executor::block_on};
use iced::widget::text_editor::Content as TextEditorContent;
use iced_runtime::{Action as RuntimeAction, task as runtime_task};
use mongodb::bson::{self, Bson, Document};
use mongodb::sync::Client;
use std::env;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

struct ConnectionsFileGuard {
    path: PathBuf,
    backup: Option<Vec<u8>>,
}

impl ConnectionsFileGuard {
    fn new(path: PathBuf) -> Self {
        let backup = fs::read(&path).ok();
        let _ = fs::remove_file(&path);
        Self { path, backup }
    }
}

impl Drop for ConnectionsFileGuard {
    fn drop(&mut self) {
        if let Some(ref data) = self.backup {
            let _ = fs::write(&self.path, data);
        } else {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn extract_host_port(uri: &str) -> Result<(String, u16), ParseIntError> {
    let trimmed = uri.strip_prefix("mongodb://").unwrap_or(uri);
    let trimmed = trimmed.split('?').next().unwrap_or(trimmed);
    let host_segment = trimmed.split('/').next().unwrap_or(trimmed);
    let host_segment = host_segment.split('@').last().unwrap_or(host_segment);
    let primary = host_segment.split(',').next().unwrap_or(host_segment);

    if let Some(end_bracket) = primary.find(']') {
        // IPv6 address like [::1]:27017
        let host = primary.trim_start_matches('[').split(']').next().unwrap_or(primary);
        let port_part = primary[end_bracket + 1..].trim_start_matches(':');
        let port = if port_part.is_empty() { 27017 } else { port_part.parse()? };
        Ok((host.to_string(), port))
    } else if let Some((host, port)) = primary.rsplit_once(':') {
        let port = port.parse()?;
        Ok((host.to_string(), port))
    } else {
        Ok((primary.to_string(), 27017))
    }
}

fn get_numeric_i64(document: &Document, key: &str) -> i64 {
    match document.get(key) {
        Some(Bson::Int32(value)) => i64::from(*value),
        Some(Bson::Int64(value)) => *value,
        Some(Bson::Double(value)) => *value as i64,
        Some(other) => panic!("Expected numeric value for '{}', got {:?}", key, other),
        None => panic!("Missing numeric field '{}'", key),
    }
}

fn drive_task(app: &mut App, task: Task<Message>) {
    if let Some(stream) = runtime_task::into_stream(task) {
        block_on(async {
            let mut pending = vec![stream];
            while let Some(mut current) = pending.pop() {
                while let Some(action) = current.next().await {
                    if let RuntimeAction::Output(message) = action {
                        let next_task = app.update(message);
                        if let Some(next_stream) = runtime_task::into_stream(next_task) {
                            pending.push(next_stream);
                        }
                    }
                }
            }
        });
    }
}

impl App {
    pub(crate) fn test_mode(&self) -> AppMode {
        self.mode
    }

    pub(crate) fn test_connections_window(&self) -> Option<&ConnectionsWindowState> {
        self.connections_window.as_ref()
    }

    pub(crate) fn test_connection_form(&self) -> Option<&ConnectionFormState> {
        self.connection_form.as_ref()
    }

    pub(crate) fn test_connections(&self) -> &[ConnectionEntry] {
        &self.connections
    }

    pub(crate) fn test_clear_connections(&mut self) {
        self.connections.clear();
    }

    pub(crate) fn test_clear_clients(&mut self) {
        self.clients.clear();
        self.next_client_id = 1;
    }

    pub(crate) fn test_clients_len(&self) -> usize {
        self.clients.len()
    }

    pub(crate) fn test_last_client_id(&self) -> Option<ClientId> {
        self.clients.last().map(|client| client.id)
    }

    pub(crate) fn test_client_databases(&self, client_id: ClientId) -> Option<Vec<String>> {
        self.clients
            .iter()
            .find(|client| client.id == client_id)
            .map(|client| client.databases.iter().map(|database| database.name.clone()).collect())
    }

    pub(crate) fn test_database_collections(
        &self,
        client_id: ClientId,
        db_name: &str,
    ) -> Option<Vec<String>> {
        self.clients.iter().find(|client| client.id == client_id).and_then(|client| {
            client.databases.iter().find(|database| database.name == db_name).map(|database| {
                database.collections.iter().map(|collection| collection.name.clone()).collect()
            })
        })
    }

    pub(crate) fn test_tabs_len(&self) -> usize {
        self.tabs.len()
    }

    pub(crate) fn test_active_tab_id(&self) -> Option<TabId> {
        self.active_tab
    }

    pub(crate) fn test_set_editor_text(&mut self, tab_id: TabId, text: &str) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.collection.editor = TextEditorContent::with_text(text);
            true
        } else {
            false
        }
    }

    pub(crate) fn test_collection_identifiers(
        &self,
        tab_id: TabId,
    ) -> Option<(ClientId, String, String)> {
        self.tabs.iter().find(|tab| tab.id == tab_id).map(|tab| {
            (
                tab.collection.client_id,
                tab.collection.db_name.clone(),
                tab.collection.collection.clone(),
            )
        })
    }

    pub(crate) fn test_collection_skip_limit(&self, tab_id: TabId) -> Option<(u64, u64)> {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .map(|tab| (tab.collection.skip_value(), tab.collection.limit_value()))
    }

    #[allow(dead_code)]
    pub(crate) fn test_collection_last_result(&self, tab_id: TabId) -> Option<QueryResult> {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .and_then(|tab| tab.collection.last_result.clone())
    }

    pub(crate) fn test_query_timeout(&self) -> Option<Duration> {
        let secs = self.settings.query_timeout_secs;
        if secs == 0 { None } else { Some(Duration::from_secs(secs)) }
    }

    #[allow(dead_code)]
    pub(crate) fn test_first_root_node_id(&self, tab_id: TabId) -> Option<usize> {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .and_then(|tab| tab.collection.bson_tree.first_root_id())
    }

    pub(crate) fn test_root_node_id_at(&self, tab_id: TabId, index: usize) -> Option<usize> {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .and_then(|tab| tab.collection.bson_tree.root_id_at(index))
    }

    pub(crate) fn test_set_document_modal_text(&mut self, text: &str) -> bool {
        if let Some(modal) = self.document_modal.as_mut() {
            modal.editor = TextEditorContent::with_text(text);
            true
        } else {
            false
        }
    }

    pub(crate) fn test_find_node_id_by_path(&self, tab_id: TabId, path: &str) -> Option<usize> {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .and_then(|tab| tab.collection.bson_tree.find_node_id_by_path(path))
    }

    pub(crate) fn test_set_value_modal_text(&mut self, text: &str) -> bool {
        if let Some(modal) = self.value_edit_modal.as_mut() {
            modal.value_editor = TextEditorContent::with_text(text);
            modal.value_input = text.to_string();
            modal.error = None;

            if let Ok(bson) = parse_shell_bson_value(&modal.value_input) {
                modal.value_kind = crate::mongo::bson_edit::ValueEditKind::from_bson(&bson);
                modal.value_label = bson_type_name(&bson).to_string();
            } else if let Some(kind) =
                crate::mongo::bson_edit::ValueEditKind::infer(&modal.value_input)
            {
                modal.value_kind = kind;
                modal.value_label = kind.label().to_string();
            }

            true
        } else {
            false
        }
    }

    pub(crate) fn test_value_modal_label(&self) -> Option<String> {
        self.value_edit_modal.as_ref().map(|modal| modal.value_label.clone())
    }

    pub(crate) fn test_value_modal_context(
        &self,
    ) -> Option<(ClientId, String, String, Document, String)> {
        self.value_edit_modal.as_ref().map(|modal| {
            (
                modal.client_id,
                modal.db_name.clone(),
                modal.collection.clone(),
                modal.filter.clone(),
                modal.path.clone(),
            )
        })
    }
}

#[test]
#[ignore = "needs real MongoDB URI"]
fn connection_flow_via_messages() {
    let uri = env::var("OXIDE_MONGO_TEST_URI")
        .expect("OXIDE_MONGO_TEST_URI must be set for this integration test");

    let (host, port) = extract_host_port(&uri)
        .map(|(host, port)| (host.trim().to_string(), port))
        .expect("failed to parse host/port from MongoDB URI");

    let shared_client = Arc::new(
        Client::with_uri_str(&uri)
            .expect("failed to establish client connection using provided URI"),
    );

    let connection_name = format!("connection-{}", Uuid::new_v4().simple());
    let new_db_name_1 = String::from("debug-db-1");
    let new_db_name_2 = String::from("debug-db-2");
    let collection_name_1 = format!("collection-{}", Uuid::new_v4().simple());
    let collection_name_2 = format!("collection-{}", Uuid::new_v4().simple());

    let _connections_guard = ConnectionsFileGuard::new(PathBuf::from("connections.toml"));

    let (mut app, _) = App::init();
    app.test_clear_connections();
    app.test_clear_clients();

    //
    // Step 1.1: Open the connections window via the File menu.
    //
    let _ = app.update(Message::MenuItemSelected(TopMenu::File, MenuEntry::Action("Connections")));
    assert!(matches!(app.test_mode(), AppMode::Connections));
    assert!(app.test_connections_window().is_some());

    //
    // Step 1.2: Start creating a new connection.
    //
    let _ = app.update(Message::ConnectionsCreate);
    assert!(matches!(app.test_mode(), AppMode::ConnectionForm));
    assert!(app.test_connection_form().is_some());

    //
    // Step 1.3: Fill in the connection form with name, host, and port.
    //
    let _ = app.update(Message::ConnectionFormNameChanged(connection_name.clone()));
    let _ = app.update(Message::ConnectionFormHostChanged(host.clone()));
    let _ = app.update(Message::ConnectionFormPortChanged(port.to_string()));

    //
    // Step 1.4: Save the connection and return to the connections list.
    //
    let _ = app.update(Message::ConnectionFormSave);
    assert!(matches!(app.test_mode(), AppMode::Connections));
    assert_eq!(app.test_connections().len(), 1);

    let saved_connection = app.test_connections().first().expect("connection entry should exist");
    assert_eq!(saved_connection.name, connection_name);
    assert_eq!(saved_connection.host, host.trim());
    assert_eq!(saved_connection.port, port);

    //
    // Step 1.5: Select the new connection and trigger the connect flow.
    //
    let _ = app.update(Message::ConnectionsSelect(0));
    if let Some(window) = app.test_connections_window() {
        assert_eq!(window.selected, Some(0));
    } else {
        panic!("connections window should remain open after selecting entry");
    }

    let _ = app.update(Message::ConnectionsConnect);
    assert!(matches!(app.test_mode(), AppMode::Main));
    assert!(app.test_connections_window().is_none());
    assert_eq!(app.test_clients_len(), 1);

    let client_id =
        app.test_last_client_id().expect("client should have been created during connect flow");

    let bootstrap = ConnectionBootstrap {
        handle: shared_client.clone(),
        databases: Vec::new(),
        ssh_tunnel: None,
    };
    let _ = app.update(Message::ConnectionCompleted { client_id, result: Ok(bootstrap) });

    //
    // Step 2.1: Verify the database list in the sidebar is initially empty.
    //
    let databases_after_connect =
        app.test_client_databases(client_id).expect("client databases should be available");
    assert!(databases_after_connect.is_empty());

    //
    // Step 2.2.1: Initiate database creation via the connection context menu.
    //
    let _ = app.update(Message::ConnectionContextMenu {
        client_id,
        action: ConnectionContextAction::CreateDatabase,
    });
    assert!(matches!(app.test_mode(), AppMode::DatabaseModal));

    //
    // Step 2.2.1: Fill in the database modal with the first database and collection names.
    //
    let _ = app.update(Message::DatabaseModalInputChanged(new_db_name_1.clone()));
    let _ = app.update(Message::DatabaseModalCollectionInputChanged(collection_name_1.clone()));
    let _ = app.update(Message::DatabaseModalConfirm);

    //
    // Step 2.2.1: Simulate successful creation and refresh of the database list.
    //
    let _ = app.update(Message::DatabaseCreateCompleted {
        client_id,
        _db_name: new_db_name_1.clone(),
        result: Ok(()),
    });
    let expected_after_first = vec![new_db_name_1.clone()];
    let _ = app.update(Message::DatabasesRefreshed {
        client_id,
        result: Ok(expected_after_first.clone()),
    });
    let databases_after_first =
        app.test_client_databases(client_id).expect("client databases should be available");
    assert_eq!(databases_after_first, expected_after_first);

    //
    // Step 2.2.2: Repeat the flow for the second database.
    //
    let _ = app.update(Message::ConnectionContextMenu {
        client_id,
        action: ConnectionContextAction::CreateDatabase,
    });
    assert!(matches!(app.test_mode(), AppMode::DatabaseModal));

    let _ = app.update(Message::DatabaseModalInputChanged(new_db_name_2.clone()));
    let _ = app.update(Message::DatabaseModalCollectionInputChanged(collection_name_1.clone()));
    let _ = app.update(Message::DatabaseModalConfirm);

    let _ = app.update(Message::DatabaseCreateCompleted {
        client_id,
        _db_name: new_db_name_2.clone(),
        result: Ok(()),
    });
    let mut expected_after_second = vec![new_db_name_1.clone(), new_db_name_2.clone()];
    expected_after_second.sort_unstable();
    let _ = app.update(Message::DatabasesRefreshed {
        client_id,
        result: Ok(expected_after_second.clone()),
    });
    let databases_after_second =
        app.test_client_databases(client_id).expect("client databases should be available");
    assert_eq!(databases_after_second, expected_after_second);

    //
    // Step 2.3: Expand the first database and ensure the collection list contains the expected entry.
    //
    let _ = app.update(Message::ToggleDatabase { client_id, db_name: new_db_name_1.clone() });
    let _ = app.update(Message::CollectionsLoaded {
        client_id,
        db_name: new_db_name_1.clone(),
        result: Ok(vec![collection_name_1.clone()]),
    });
    let collections = app
        .test_database_collections(client_id, &new_db_name_1)
        .expect("database should be present after expansion");
    assert_eq!(collections, vec![collection_name_1.clone()]);

    //
    // Step 2.4: Create an additional collection via the database context menu.
    //
    let _ = app.update(Message::DatabaseContextMenu {
        client_id,
        db_name: new_db_name_1.clone(),
        action: DatabaseContextAction::CreateCollection,
    });
    assert!(matches!(app.test_mode(), AppMode::CollectionModal));

    let _ = app.update(Message::CollectionModalInputChanged(collection_name_2.clone()));
    let _ = app.update(Message::CollectionModalConfirm);
    let _ = app.update(Message::CollectionCreateCompleted {
        client_id,
        db_name: new_db_name_1.clone(),
        collection: collection_name_2.clone(),
        result: Ok(()),
    });
    assert!(matches!(app.test_mode(), AppMode::Main));

    let mut expected_collections = vec![collection_name_1.clone(), collection_name_2.clone()];
    expected_collections.sort();
    let collections_after_create =
        app.test_database_collections(client_id, &new_db_name_1).expect("collections available");
    assert_eq!(collections_after_create, expected_collections);

    //
    // Step 2.5: Open an empty tab for the primary collection.
    //
    let _ = app.update(Message::CollectionContextMenu {
        client_id,
        db_name: new_db_name_1.clone(),
        collection: collection_name_1.clone(),
        action: CollectionContextAction::OpenEmptyTab,
    });
    assert_eq!(app.test_tabs_len(), 1);

    let primary_tab_id =
        app.test_active_tab_id().expect("a collection tab should be active after opening it");
    let (tab_client_id, tab_db_name, tab_collection_name) =
        app.test_collection_identifiers(primary_tab_id).expect("tab metadata should be available");
    assert_eq!(tab_client_id, client_id);
    assert_eq!(tab_db_name, new_db_name_1);
    assert_eq!(tab_collection_name, collection_name_1);

    let execute_query = |app: &mut App,
                         tab_id: TabId,
                         query: &str,
                         shared_client: &Arc<Client>|
     -> QueryResult {
        assert!(app.test_set_editor_text(tab_id, query), "failed to update request editor text");
        let _ = app.update(Message::CollectionSend(tab_id));

        let (_client, db_name, collection_name) = app
            .test_collection_identifiers(tab_id)
            .expect("collection identifiers must be present");
        let (skip_value, limit_value) =
            app.test_collection_skip_limit(tab_id).expect("skip/limit must parse");

        let (effective_collection, operation) =
            parse_collection_query_with_collection(&db_name, &collection_name, query)
                .expect("query parses");
        let timeout = app.test_query_timeout();
        let result = run_collection_query(
            Arc::clone(shared_client),
            db_name,
            effective_collection,
            operation,
            skip_value,
            limit_value,
            timeout,
        )
        .expect("query should succeed");

        let _ = app.update(Message::CollectionQueryCompleted {
            tab_id,
            result: Ok(result.clone()),
            duration: Duration::from_millis(5),
        });

        result
    };

    //
    // Step 3.1: Count documents before any inserts.
    //
    let baseline_count_query = format!(
        "db.getCollection('{collection}').find({{}}).count({{}})",
        collection = collection_name_1
    );
    let count_zero_result =
        execute_query(&mut app, primary_tab_id, &baseline_count_query, &shared_client);
    match count_zero_result {
        QueryResult::Count { value } => assert_eq!(value, Bson::Int64(0)),
        other => panic!("expected count result with zero documents, got {:?}", other),
    }

    //
    // Step 3.2: Insert a document demonstrating native MongoDB types.
    //
    let insert_one_payload = r#"{
    string1: "Пример строки",
    string2: String("через конструктор String"),
    int32_1: NumberInt(42),
    int32_2: NumberInt("42"),
    long1: NumberLong(9007199254740991),
    long2: NumberLong("9007199254740991"),
    long3: NumberLong(42),
    double1: 3.14159,
    double2: Number(2.5),
    double3: Infinity,
    double4: -Infinity,
    double5: NaN,
    decimal1: NumberDecimal("12345.6789"),
    decimal2: NumberDecimal("1E-28"),
    decimal3: NumberDecimal("0.30000000000000004"),
    bool1: true,
    bool2: Boolean(false),
    date1: new Date(),
    date2: ISODate(),
    date3: ISODate("2025-10-14T15:30:00Z"),
    date4: new Date("2025-10-14T15:30:00Z"),
    date5: new Date(2025, 9, 14, 17, 0, 0, 0),
    date6: new Date(0),
    array1: [1, 2, 3, "текст"],
    array2: new Array(1, 2, 3),
    object1: { a: 1, b: "строка" },
    object2: Object({ x: 10, y: 20 }),
    null1: null,
    regex1: /mongodb/i,
    regex2: new RegExp("mon(go|godb)", "i"),
    objectId1: ObjectId(),
    objectId2: ObjectId("64e9c4a9c2c1b3a5f1d0eabc"),
    objectId3: ObjectId.fromDate(ISODate("2020-01-01T00:00:00Z")),
    binary1: new BinData(0, "YWJjZGVmZw=="),
    binary2: HexData(0, "DEADBEEF"),
    uuid1: UUID(),
    uuid2: UUID("12345678-1234-5678-9abc-123456789abc"),
    timestamp1: Timestamp(1680000000, 1),
    timestamp2: Timestamp(ISODate("2023-03-28T00:00:00Z").getTime()/1000, 5),
    minKey1: MinKey(),
    maxKey1: MaxKey(),
    undefined1: undefined,
    js1: new Code("function() { return 2 + 2; }"),
    js2: function() { return 40 + 2; },
    jsWithScope1: new Code(
      "function(x) { return x + y; }",
      { y: 5 }
      ),
    dbref1: DBRef("users", ObjectId("64e9c4a9c2c1b3a5f1d0eabc")),
    dbref2: DBRef("users", ObjectId(), "otherDb")
}"#;
    let insert_one_query = format!(
        "db.getCollection('{collection}').insertOne({payload})",
        collection = collection_name_1,
        payload = insert_one_payload
    );
    let insert_one_result =
        execute_query(&mut app, primary_tab_id, &insert_one_query, &shared_client);
    match insert_one_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("operation").unwrap_or_default(), "insertOne");
            assert!(
                matches!(document.get("insertedId"), Some(Bson::ObjectId(_))),
                "insertedId should be an ObjectId: {document:?}"
            );
        }
        other => panic!("expected insertOne acknowledgment, got {:?}", other),
    }

    //
    // Step 3.3: Count the documents again to ensure the insert landed.
    //
    let post_insert_count =
        execute_query(&mut app, primary_tab_id, &baseline_count_query, &shared_client);
    match post_insert_count {
        QueryResult::Count { value } => assert_eq!(value, Bson::Int64(1)),
        other => panic!("expected count result with a single document, got {:?}", other),
    }

    //
    // Step 3.4: Insert multiple documents with insertMany.
    //
    let insert_many_query = format!(
        "db.getCollection('{collection}').insertMany([{{value: 10}}, {{value: 11}}])",
        collection = collection_name_1
    );
    let insert_many_result =
        execute_query(&mut app, primary_tab_id, &insert_many_query, &shared_client);
    match insert_many_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("operation").unwrap_or_default(), "insertMany");
            assert_eq!(get_numeric_i64(&document, "insertedCount"), 2);
        }
        other => panic!("expected insertMany acknowledgment, got {:?}", other),
    }

    //
    // Step 3.4 Supplement: Insert one more helper document so that four entries exist before viewing.
    //
    let helper_insert_query = format!(
        "db.getCollection('{collection}').insertOne({{ helper: true }})",
        collection = collection_name_1
    );
    let helper_result =
        execute_query(&mut app, primary_tab_id, &helper_insert_query, &shared_client);
    if let QueryResult::SingleDocument { document } = helper_result {
        assert_eq!(document.get_str("operation").unwrap_or_default(), "insertOne");
    } else {
        panic!("expected insertOne acknowledgment for helper document");
    }

    //
    // Step 3.5: Use the context menu to view documents and verify four rows are shown.
    //
    let _ = app.update(Message::CollectionContextMenu {
        client_id,
        db_name: new_db_name_1.clone(),
        collection: collection_name_1.clone(),
        action: CollectionContextAction::ViewDocuments,
    });
    assert_eq!(app.test_tabs_len(), 2);
    let documents_tab_id =
        app.test_active_tab_id().expect("documents tab should become active after view request");
    let view_query =
        format!("db.getCollection('{collection}').find({{}})", collection = collection_name_1);
    let documents_result = execute_query(&mut app, documents_tab_id, &view_query, &shared_client);
    match &documents_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 4),
        other => panic!("expected documents result with four entries, got {:?}", other),
    }

    //
    // Step 3.6: Open the document modal for the primary document, replace its content, and validate a zero count.
    //
    let (primary_index, primary_document) = match &documents_result {
        QueryResult::Documents(values) => values
            .iter()
            .enumerate()
            .find_map(|(index, value)| {
                let document = value.as_document()?;
                let matches_anchor = document
                    .get_str("string1")
                    .map(|value| value == "Пример строки")
                    .unwrap_or(false);
                if matches_anchor { Some((index, document.clone())) } else { None }
            })
            .or_else(|| {
                values.get(0).and_then(|value| value.as_document().cloned()).map(|doc| (0, doc))
            })
            .expect("at least one document should be present for editing"),
        other => panic!("expected documents result for edit step, got {:?}", other),
    };

    let primary_document_id =
        primary_document.get("_id").cloned().expect("primary document should contain _id field");
    let formatted_primary_id = format_bson_shell(&primary_document_id);

    let updated_data = format!(
        concat!(
            "{{\n",
            "    \"_id\": {id},\n",
            "    \"string1\": \"Пример строки1\",\n",
            "    \"string2\": String(\"через конструктор String1\"),\n",
            "    \"int32_1\": NumberInt(421),\n",
            "    \"int32_2\": NumberInt(\"421\"),\n",
            "    \"long1\": NumberLong(90071992547409911),\n",
            "    \"long2\": NumberLong(\"90071992547409911\"),\n",
            "    \"long3\": NumberLong(421),\n",
            "    \"double1\": 3.141591,\n",
            "    \"double2\": Number(2.51),\n",
            "    \"double3\": -Infinity,\n",
            "    \"double4\": Infinity,\n",
            "    \"decimal1\": NumberDecimal(\"12345.67891\"),\n",
            "    \"decimal2\": NumberDecimal(\"1E-21\"),\n",
            "    \"decimal3\": NumberDecimal(\"0.300000000000000041\"),\n",
            "    \"bool1\": false,\n",
            "    \"bool2\": Boolean(true),\n",
            "    \"date3\": ISODate(\"2025-10-14T15:30:01Z\"),\n",
            "    \"date4\": new Date(\"2025-10-14T15:30:01Z\"),\n",
            "    \"date5\": new Date(2025, 9, 14, 17, 0, 0, 1),\n",
            "    \"date6\": new Date(1),\n",
            "    \"array1\": [1, 2, 3, \"текст1\"],\n",
            "    \"array2\": new Array(1, 2, 31),\n",
            "    \"object1\": {{ \"a\": 11, \"b\": \"строка1\" }},\n",
            "    \"object2\": Object({{ \"x\": 101, \"y\": 201 }}),\n",
            "    \"regex1\": /mongodb1/i,\n",
            "    \"regex2\": new RegExp(\"mon(go|godb)1\", \"i\"),\n",
            "    \"objectId2\": ObjectId(\"64e9c4a9c2c1b3a5f1d0eab1\"),\n",
            "    \"objectId3\": ObjectId.fromDate(ISODate(\"2020-01-01T00:00:01Z\")),\n",
            "    \"binary2\": HexData(0, \"DEADBEEF11\"),\n",
            "    \"uuid2\": UUID(\"12345678-1234-5678-9abc-123456789ab1\"),\n",
            "    \"timestamp1\": Timestamp(1680000001, 1),\n",
            "    \"timestamp2\": Timestamp(ISODate(\"2023-03-21T00:00:00Z\").getTime()/1000, 5)\n",
            "}}"
        ),
        id = formatted_primary_id
    );

    let _parsed_updated_doc = parse_shell_json_value(&updated_data)
        .and_then(|value| {
            let object = value
                .as_object()
                .cloned()
                .ok_or_else(|| String::from("updated_data must be an object"));
            object.map(|obj| bson::to_document(&obj).map_err(|err| err.to_string()))
        })
        .and_then(std::convert::identity)
        .expect("updated_data must parse into a BSON document");

    let root_node_id = app
        .test_root_node_id_at(documents_tab_id, primary_index)
        .expect("root node for primary document should exist");
    let _ = app
        .update(Message::DocumentEditRequested { tab_id: documents_tab_id, node_id: root_node_id });
    assert!(matches!(app.test_mode(), AppMode::DocumentModal));
    assert!(app.test_set_document_modal_text(&updated_data));

    let save_task = app.update(Message::DocumentModalSave);
    drive_task(&mut app, save_task);
    if !matches!(app.test_mode(), AppMode::Main) {
        let modal_error =
            app.document_modal.as_ref().and_then(|modal| modal.error.clone()).unwrap_or_default();
        panic!("document modal did not close after save: {modal_error}");
    }

    let updated_count_query = format!(
        "db.getCollection('{collection}').count({filter})",
        collection = collection_name_1,
        filter = updated_data
    );
    let updated_count =
        execute_query(&mut app, documents_tab_id, &updated_count_query, &shared_client);
    match updated_count {
        QueryResult::Count { value } => assert_eq!(value, Bson::Int64(0)),
        other => panic!("expected zero count after replacement, got {:?}", other),
    }

    //
    // Step 3.7.1: Reload documents and ensure four entries remain available.
    //
    let refreshed_view_query =
        format!("db.getCollection('{collection}').find()", collection = collection_name_1);
    let refreshed_documents =
        execute_query(&mut app, documents_tab_id, &refreshed_view_query, &shared_client);
    if let QueryResult::Documents(values) = &refreshed_documents {
        assert_eq!(values.len(), 4);
    } else {
        panic!(
            "expected documents result with four entries after refresh, got {:?}",
            refreshed_documents
        );
    }

    //
    // Step 3.7.2: Edit the second element of array1 to a Double value of 2.0.
    //
    let array_value_node_id = app
        .test_find_node_id_by_path(documents_tab_id, "array1.1")
        .expect("array1[1] node should exist");
    let _ = app.update(Message::TableContextMenu {
        tab_id: documents_tab_id,
        node_id: array_value_node_id,
        action: TableContextAction::EditValue,
    });
    assert!(matches!(app.test_mode(), AppMode::ValueEditModal));
    assert!(app.test_set_value_modal_text("2.0"));
    let value_label =
        app.test_value_modal_label().expect("value edit modal should expose a value label");
    assert_eq!(value_label, "Double");

    let value_modal_context =
        app.test_value_modal_context().expect("value modal context should be available");
    let (value_client_id, _value_db_name, _value_collection, value_filter, value_path) =
        value_modal_context;
    assert_eq!(value_client_id, client_id);
    assert_eq!(value_path, "array1.1");
    assert!(value_filter.contains_key("_id"));

    let save_task = app.update(Message::ValueEditModalSave);
    drive_task(&mut app, save_task);
    assert!(matches!(app.test_mode(), AppMode::Main));

    //
    // Step 3.7.3: Ensure the updated array entry is queryable as a Double.
    //
    let array_filter_count_query = format!(
        "db.getCollection('{collection}').find({{\"array1.1\": NumberDouble(\"2.0\")}}).count()",
        collection = collection_name_1
    );
    let array_filter_count =
        execute_query(&mut app, documents_tab_id, &array_filter_count_query, &shared_client);
    match array_filter_count {
        QueryResult::Count { value } => assert_eq!(value, Bson::Int64(1)),
        other => panic!("expected count result with single matching array value, got {:?}", other),
    }

    //
    // Step 4.1.1: Close previous tabs and open an empty tab for the second collection.
    //
    for tab_id in [documents_tab_id, primary_tab_id] {
        let _ = app.update(Message::TabClosed(tab_id));
    }
    assert_eq!(app.test_tabs_len(), 0);

    let _ = app.update(Message::CollectionContextMenu {
        client_id,
        db_name: new_db_name_1.clone(),
        collection: collection_name_2.clone(),
        action: CollectionContextAction::OpenEmptyTab,
    });
    assert_eq!(app.test_tabs_len(), 1);

    let secondary_tab_id =
        app.test_active_tab_id().expect("secondary collection tab should be active");
    let (sec_client_id, sec_db_name, sec_collection_name) =
        app.test_collection_identifiers(secondary_tab_id).expect("secondary tab identifiers");
    assert_eq!(sec_client_id, client_id);
    assert_eq!(sec_db_name, new_db_name_1);
    assert_eq!(sec_collection_name, collection_name_2);

    //
    // Step 4.1.1.1: Insert into COLLECTION_NAME_1 from COLLECTION_NAME_2 tab.
    //
    let insert_other_collection = format!(
        "db.getCollection('{collection}').insertOne({{ marker: \"from_other_collection\" }})",
        collection = collection_name_1
    );
    let insert_other_result =
        execute_query(&mut app, secondary_tab_id, &insert_other_collection, &shared_client);
    match insert_other_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("operation").unwrap_or_default(), "insertOne");
        }
        other => panic!("expected insertOne for other collection, got {:?}", other),
    }

    let verify_other_collection = format!(
        "db.getCollection('{collection}').find({{ marker: \"from_other_collection\" }})",
        collection = collection_name_1
    );
    let (verify_other_collection_name, verify_other_op) = parse_collection_query_with_collection(
        &new_db_name_1,
        &collection_name_1,
        &verify_other_collection,
    )
    .expect("query parses");
    let verify_other_result = run_collection_query(
        Arc::clone(&shared_client),
        new_db_name_1.clone(),
        verify_other_collection_name,
        verify_other_op,
        0,
        DEFAULT_RESULT_LIMIT as u64,
        app.test_query_timeout(),
    )
    .expect("query should succeed");
    match &verify_other_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 1),
        other => panic!("expected marker document in other collection, got {:?}", other),
    }

    //
    // Step 4.1.2: Insert documents into COLLECTION_NAME_2.
    //
    let insert_many_collection_2 = format!(
        concat!(
            "db.getCollection('{collection}').insertMany([\n",
            "    {{ \"name\": \"Alex\", \"department\": \"IT\", \"starts\": ISODate(\"2020-02-01\"), \"points\": 10}},\n",
            "    {{ \"name\": \"Alex\", \"department\": \"Support\", \"starts\": ISODate(\"2018-03-10\"), \"points\": 8}},\n",
            "    {{ \"name\": \"Anya\", \"department\": \"IT\", \"starts\": ISODate(\"2020-06-15\"), \"points\": 20}},\n",
            "    {{ \"name\": \"Mark\", \"department\": \"Devops\", \"starts\": ISODate(\"2019-05-01\"), \"points\": 12}}\n",
            "])"
        ),
        collection = collection_name_2
    );
    let insert_many_collection_2_result =
        execute_query(&mut app, secondary_tab_id, &insert_many_collection_2, &shared_client);
    match insert_many_collection_2_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("operation").unwrap_or_default(), "insertMany");
            assert_eq!(get_numeric_i64(&document, "insertedCount"), 4);
        }
        other => panic!("expected insertMany result for collection 2, got {:?}", other),
    }

    //
    // Step 4.2.1: Check find({}) return 4 documents.
    //
    let find_all_collection_2 =
        format!("db.getCollection('{collection}').find({{}})", collection = collection_name_2);
    let find_all_result =
        execute_query(&mut app, secondary_tab_id, &find_all_collection_2, &shared_client);
    match &find_all_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 4),
        other => panic!("expected 4 documents in collection 2, got {:?}", other),
    }

    //
    // Step 4.2.2: Query unknown name should return zero documents.
    //
    let find_unknown = format!(
        "db.getCollection('{collection}').find({{\"name\": \"unknown\"}})",
        collection = collection_name_2
    );
    let find_unknown_result =
        execute_query(&mut app, secondary_tab_id, &find_unknown, &shared_client);
    match &find_unknown_result {
        QueryResult::Documents(values) => assert!(values.is_empty()),
        other => panic!("expected zero documents for unknown name, got {:?}", other),
    }

    //
    // Step 4.2.3: Filter by IT department and specific names, expect 2 documents.
    //
    let find_it_names = format!(
        "db.getCollection('{collection}').find({{\"department\": \"IT\", \"name\": {{\"$in\": [\"Alex\", \"Anya\"]}}}})",
        collection = collection_name_2
    );
    let find_it_result = execute_query(&mut app, secondary_tab_id, &find_it_names, &shared_client);
    match &find_it_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 2),
        other => panic!("expected two IT documents for Alex/Anya, got {:?}", other),
    }

    //
    // Step 4.2.4: Filter by date range in 2020, expect 2 documents.
    //
    let find_date_range = format!(
        "db.getCollection('{collection}').find({{\"starts\": {{\"$gt\": ISODate('2020-01-01'), \"$lt\": ISODate(\"2021-01-01\")}}}})",
        collection = collection_name_2
    );
    let find_date_result =
        execute_query(&mut app, secondary_tab_id, &find_date_range, &shared_client);
    match &find_date_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 2),
        other => panic!("expected two documents in date range, got {:?}", other),
    }

    //
    // Step 4.3: findOne on date range should return a single document, then findOne by _id.
    //
    let find_one_date_range = format!(
        "db.getCollection('{collection}').findOne({{\"starts\": {{\"$gt\": ISODate('2020-01-01'), \"$lt\": ISODate(\"2021-01-01\")}}}})",
        collection = collection_name_2
    );
    let find_one_date_result =
        execute_query(&mut app, secondary_tab_id, &find_one_date_range, &shared_client);
    let find_one_id = match &find_one_date_result {
        QueryResult::SingleDocument { document } => {
            assert!(document.get("starts").is_some(), "findOne result must contain starts field");
            document.get("_id").cloned().expect("findOne result must contain _id field")
        }
        QueryResult::Documents(values) => {
            panic!("expected single document for date range, got list of len {}", values.len())
        }
        other => panic!("expected single document in date range, got {:?}", other),
    };

    let find_one_id_query = format!(
        "db.getCollection('{collection}').findOne({id})",
        collection = collection_name_2,
        id = format_bson_shell(&find_one_id)
    );
    let find_one_id_result =
        execute_query(&mut app, secondary_tab_id, &find_one_id_query, &shared_client);
    match &find_one_id_result {
        QueryResult::SingleDocument { document } => {
            let returned_id =
                document.get("_id").cloned().expect("findOne by _id result must contain _id field");
            assert_eq!(returned_id, find_one_id);
        }
        QueryResult::Documents(values) => {
            panic!("expected single document for _id lookup, got list of len {}", values.len())
        }
        other => panic!("expected single document for _id lookup, got {:?}", other),
    }

    //
    // Step 4.4: Aggregation.
    //
    let aggregate_points = format!(
        concat!(
            "db.getCollection('{collection}').aggregate([",
            "{{\"$match\": {{\"department\": \"IT\"}}}}, ",
            "{{\"$group\": {{\"_id\": null, \"total\": {{\"$sum\": \"$points\"}}, \"count\": {{\"$sum\": 1}}}}}}, ",
            "{{\"$project\": {{\"value\": {{\"$divide\": [\"$total\", \"$count\"]}}}}}}",
            "])"
        ),
        collection = collection_name_2
    );
    let aggregate_result =
        execute_query(&mut app, secondary_tab_id, &aggregate_points, &shared_client);
    match &aggregate_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 1);
            if let Some(doc) = values.first().and_then(|b| b.as_document()) {
                let value_bson = doc.get("value").cloned().unwrap_or(Bson::Null);
                match value_bson {
                    Bson::Int32(v) => assert_eq!(v, 15),
                    Bson::Int64(v) => assert_eq!(v, 15),
                    Bson::Double(v) => assert!((v - 15.0).abs() < f64::EPSILON),
                    other => panic!("expected numeric value 15, got {:?}", other),
                }
            } else {
                panic!("aggregate result missing document payload");
            }
        }
        other => panic!("expected aggregation documents result, got {:?}", other),
    }

    let numeric_field = |document: &Document, key: &str| -> Option<f64> {
        match document.get(key) {
            Some(Bson::Int32(v)) => Some(*v as f64),
            Some(Bson::Int64(v)) => Some(*v as f64),
            Some(Bson::Double(v)) => Some(*v),
            _ => None,
        }
    };

    //
    // Step 4.5: Distinct by name with points filter should return Alex, Anya, and Mark.
    //
    let distinct_names = format!(
        "db.getCollection('{collection}').distinct('name', {{points: {{\"$lte\": 20}}}})",
        collection = collection_name_2
    );
    let distinct_result =
        execute_query(&mut app, secondary_tab_id, &distinct_names, &shared_client);
    match &distinct_result {
        QueryResult::Distinct { field, values } => {
            assert_eq!(field, "name");
            let mut names: Vec<String> =
                values.iter().filter_map(|b| b.as_str().map(|s| s.to_string())).collect();
            names.sort();
            assert_eq!(names, vec!["Alex".to_string(), "Anya".to_string(), "Mark".to_string()]);
        }
        other => panic!("expected distinct result for names, got {:?}", other),
    }

    //
    // Step 4.6.1: count() should report 4 documents.
    //
    let count_all =
        format!("db.getCollection('{collection}').count()", collection = collection_name_2);
    let count_all_result = execute_query(&mut app, secondary_tab_id, &count_all, &shared_client);
    match &count_all_result {
        QueryResult::Count { value } => assert_eq!(*value, Bson::Int64(4)),
        other => panic!("expected count result with value 4, got {:?}", other),
    }

    //
    // Step 4.6.2: countDocuments() should report 4 documents.
    //
    let count_documents = format!(
        "db.getCollection('{collection}').countDocuments()",
        collection = collection_name_2
    );
    let count_documents_result =
        execute_query(&mut app, secondary_tab_id, &count_documents, &shared_client);
    match &count_documents_result {
        QueryResult::Count { value } => assert_eq!(*value, Bson::Int64(4)),
        other => panic!("expected countDocuments result with value 4, got {:?}", other),
    }

    //
    // Step 4.6.3: estimatedDocumentCount() should report 4 documents.
    //
    let estimated_count = format!(
        "db.getCollection('{collection}').estimatedDocumentCount()",
        collection = collection_name_2
    );
    let estimated_count_result =
        execute_query(&mut app, secondary_tab_id, &estimated_count, &shared_client);
    match &estimated_count_result {
        QueryResult::Count { value } => assert_eq!(*value, Bson::Int64(4)),
        other => panic!("expected estimatedDocumentCount result with value 4, got {:?}", other),
    }

    //
    // Step 4.7.1: findOneAndUpdate should return the original document with points 10 for Alex (IT).
    //
    let find_one_and_update = format!(
        "db.getCollection('{collection}').findOneAndUpdate({{name: 'Alex', department: 'IT'}}, {{\"$set\": {{\"points\": 20}}}})",
        collection = collection_name_2
    );
    let fou_result =
        execute_query(&mut app, secondary_tab_id, &find_one_and_update, &shared_client);
    match &fou_result {
        QueryResult::SingleDocument { document } => {
            let points = document
                .get("points")
                .and_then(|b| match b {
                    Bson::Int32(v) => Some(*v as f64),
                    Bson::Int64(v) => Some(*v as f64),
                    Bson::Double(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or_default();
            assert!(
                (points - 10.0).abs() < f64::EPSILON,
                "findOneAndUpdate should return original document, got {}",
                points
            );
        }
        other => panic!("expected single document result for findOneAndUpdate, got {:?}", other),
    }

    //
    // Step 4.7.2: updateOne on Anya should acknowledge and modify one document.
    //
    let update_one = format!(
        "db.getCollection('{collection}').updateOne({{name: 'Anya'}}, {{\"$set\": {{points: 30}}}})",
        collection = collection_name_2
    );
    let update_one_result = execute_query(&mut app, secondary_tab_id, &update_one, &shared_client);
    match &update_one_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_bool("acknowledged").unwrap_or(false), true);
            let matched = numeric_field(document, "matchedCount").unwrap_or_default();
            let modified = numeric_field(document, "modifiedCount").unwrap_or_default();
            assert!(
                (matched - 1.0).abs() < f64::EPSILON,
                "expected matchedCount 1, got {}",
                matched
            );
            assert!(
                (modified - 1.0).abs() < f64::EPSILON,
                "expected modifiedCount 1, got {}",
                modified
            );
        }
        other => panic!("expected updateOne acknowledgment, got {:?}", other),
    }

    //
    // Step 4.7.3: findAndModify should return original Support Alex with points 8.
    //
    let find_and_modify = format!(
        "db.getCollection('{collection}').findAndModify({{query: {{name: 'Alex', department: 'Support'}}, update:{{\"$inc\": {{points: 20}}}}}})",
        collection = collection_name_2
    );
    let fam_result = execute_query(&mut app, secondary_tab_id, &find_and_modify, &shared_client);
    match &fam_result {
        QueryResult::SingleDocument { document } => {
            let points = document
                .get("points")
                .and_then(|b| match b {
                    Bson::Int32(v) => Some(*v as f64),
                    Bson::Int64(v) => Some(*v as f64),
                    Bson::Double(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or_default();
            assert!(
                (points - 8.0).abs() < f64::EPSILON,
                "findAndModify should return original Support Alex points 8, got {}",
                points
            );
        }
        other => panic!("expected findAndModify to return original document, got {:?}", other),
    }

    //
    // Step 4.7.4: findAndModify with new=true should return updated Support Alex with points 18.
    //
    let find_and_modify_new = format!(
        "db.getCollection('{collection}').findAndModify({{query: {{name: 'Alex', department: 'Support'}}, update:{{\"$inc\": {{points: -10}}}}, new: true}})",
        collection = collection_name_2
    );
    let fam_new_result =
        execute_query(&mut app, secondary_tab_id, &find_and_modify_new, &shared_client);
    match &fam_new_result {
        QueryResult::SingleDocument { document } => {
            let points = document
                .get("points")
                .and_then(|b| match b {
                    Bson::Int32(v) => Some(*v as f64),
                    Bson::Int64(v) => Some(*v as f64),
                    Bson::Double(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or_default();
            assert!(
                (points - 18.0).abs() < f64::EPSILON,
                "findAndModify with new=true should return points 18, got {}",
                points
            );
        }
        other => panic!("expected findAndModify with new=true, got {:?}", other),
    }

    //
    // Step 4.7.5: findOneAndReplace should return original Mark document with points 12.
    //
    let find_one_and_replace = format!(
        "db.getCollection('{collection}').findOneAndReplace({{name: 'Mark'}}, {{name: 'Markus', department: 'Devops', \"starts\": ISODate(\"2019-05-01\"), points: 22}})",
        collection = collection_name_2
    );
    let foar_result =
        execute_query(&mut app, secondary_tab_id, &find_one_and_replace, &shared_client);
    match &foar_result {
        QueryResult::SingleDocument { document } => {
            let points = document
                .get("points")
                .and_then(|b| match b {
                    Bson::Int32(v) => Some(*v as f64),
                    Bson::Int64(v) => Some(*v as f64),
                    Bson::Double(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or_default();
            assert!(
                (points - 12.0).abs() < f64::EPSILON,
                "findAndModify with new=true should return points 12, got {}",
                points
            );
        }
        other => panic!("expected original document from findOneAndReplace, got {:?}", other),
    }

    //
    // Step 4.7.6: replaceOne on Markus should acknowledge one modification.
    //
    let replace_one = format!(
        "db.getCollection('{collection}').replaceOne({{name: 'Markus'}}, {{name: 'Markus', department: 'Devops', \"starts\": ISODate(\"2019-05-01\"), points: 25}})",
        collection = collection_name_2
    );
    let replace_one_result =
        execute_query(&mut app, secondary_tab_id, &replace_one, &shared_client);
    match &replace_one_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_bool("acknowledged").unwrap_or(false), true);
            let matched = numeric_field(document, "matchedCount").unwrap_or_default();
            let modified = numeric_field(document, "modifiedCount").unwrap_or_default();
            assert!(
                (matched - 1.0).abs() < f64::EPSILON,
                "expected matchedCount 1, got {}",
                matched
            );
            assert!(
                (modified - 1.0).abs() < f64::EPSILON,
                "expected modifiedCount 1, got {}",
                modified
            );
        }
        other => panic!("expected replaceOne acknowledgment, got {:?}", other),
    }

    //
    // Step 4.7.7: updateMany should touch all documents with points > 0 (increment points by 100).
    //
    let update_many = format!(
        "db.getCollection('{collection}').updateMany({{points: {{\"$gt\": 0}}}}, {{\"$inc\": {{points: 100}}}})",
        collection = collection_name_2
    );
    let update_many_result =
        execute_query(&mut app, secondary_tab_id, &update_many, &shared_client);
    match &update_many_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_bool("acknowledged").unwrap_or(false), true);
            let matched = numeric_field(document, "matchedCount").unwrap_or_default();
            let modified = numeric_field(document, "modifiedCount").unwrap_or_default();
            assert!(
                (matched - 4.0).abs() < f64::EPSILON,
                "expected matchedCount 4, got {}",
                matched
            );
            assert!(
                (modified - 4.0).abs() < f64::EPSILON,
                "expected modifiedCount 4, got {}",
                modified
            );
        }
        other => panic!("expected updateMany acknowledgment, got {:?}", other),
    }

    //
    // Step 4.7.8: Aggregation after updates should produce total points of 493.
    //
    let aggregate_points_after = format!(
        concat!(
            "db.getCollection('{collection}').aggregate([",
            "{{\"$group\": {{\"_id\": null, \"value\": {{\"$sum\": \"$points\"}}}}}}",
            "])"
        ),
        collection = collection_name_2
    );
    let aggregate_after_result =
        execute_query(&mut app, secondary_tab_id, &aggregate_points_after, &shared_client);
    match &aggregate_after_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 1);
            if let Some(doc) = values.first().and_then(|b| b.as_document()) {
                let value_bson = doc.get("value").cloned().unwrap_or(Bson::Null);
                match value_bson {
                    Bson::Double(v) => assert!((v - 493.0).abs() < f64::EPSILON),
                    Bson::Int32(v) => assert_eq!(v, 493),
                    Bson::Int64(v) => assert_eq!(v, 493),
                    other => panic!("expected numeric value 493, got {:?}", other),
                }
            } else {
                panic!("aggregate result missing document payload after updates");
            }
        }
        other => panic!("expected aggregation documents result after updates, got {:?}", other),
    }

    //
    // Step 4.8.1: findOneAndDelete should return IT Alex (highest points among Alex).
    //
    let find_one_and_delete = format!(
        "db.getCollection('{collection}').findOneAndDelete({{name: 'Alex'}}, {{sort: {{points: -1}}}})",
        collection = collection_name_2
    );
    let foad_result =
        execute_query(&mut app, secondary_tab_id, &find_one_and_delete, &shared_client);
    match &foad_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("department").unwrap_or_default(), "IT");
        }
        other => panic!("expected single document from findOneAndDelete, got {:?}", other),
    }

    //
    // Step 4.8.2: deleteOne remaining Alex, then verify counts.
    //
    let delete_one_alex = format!(
        "db.getCollection('{collection}').deleteOne({{name: 'Alex'}})",
        collection = collection_name_2
    );
    let delete_one_result =
        execute_query(&mut app, secondary_tab_id, &delete_one_alex, &shared_client);
    match &delete_one_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_bool("acknowledged").unwrap_or(false), true);
            let deleted = numeric_field(document, "deletedCount").unwrap_or_default();
            assert!(
                (deleted - 1.0).abs() < f64::EPSILON,
                "expected deletedCount 1, got {}",
                deleted
            );
        }
        other => panic!("expected deleteOne acknowledgment, got {:?}", other),
    }

    let find_remaining_alex = format!(
        "db.getCollection('{collection}').find({{name: \"Alex\"}})",
        collection = collection_name_2
    );
    let remaining_alex_result =
        execute_query(&mut app, secondary_tab_id, &find_remaining_alex, &shared_client);
    match &remaining_alex_result {
        QueryResult::Documents(values) => assert!(values.is_empty()),
        other => panic!("expected zero documents for remaining Alex, got {:?}", other),
    }

    let find_after_delete =
        format!("db.getCollection('{collection}').find({{}})", collection = collection_name_2);
    let after_delete_result =
        execute_query(&mut app, secondary_tab_id, &find_after_delete, &shared_client);
    match &after_delete_result {
        QueryResult::Documents(values) => assert_eq!(values.len(), 2),
        other => panic!("expected two documents after deleteOne, got {:?}", other),
    }

    //
    // Step 4.8.3: deleteMany remaining positives and verify empty collection.
    //
    let delete_many = format!(
        "db.getCollection('{collection}').deleteMany({{points: {{\"$gt\": 0}}}})",
        collection = collection_name_2
    );
    let delete_many_result =
        execute_query(&mut app, secondary_tab_id, &delete_many, &shared_client);
    match &delete_many_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_bool("acknowledged").unwrap_or(false), true);
            let deleted = numeric_field(document, "deletedCount").unwrap_or_default();
            assert!(
                (deleted - 2.0).abs() < f64::EPSILON,
                "expected deletedCount 2, got {}",
                deleted
            );
        }
        other => panic!("expected deleteMany acknowledgment, got {:?}", other),
    }

    let find_after_delete_many =
        format!("db.getCollection('{collection}').find({{}})", collection = collection_name_2);
    let after_delete_many_result =
        execute_query(&mut app, secondary_tab_id, &find_after_delete_many, &shared_client);
    match &after_delete_many_result {
        QueryResult::Documents(values) => assert!(values.is_empty()),
        other => panic!("expected zero documents after deleteMany, got {:?}", other),
    }

    //
    // Step 5.1: Create single index and verify via getIndexes.
    //
    let create_index = format!(
        "db.getCollection('{collection}').createIndex({{name: 1}}, {{name: \"name_asc\", expireAfterSeconds: 3600}})",
        collection = collection_name_2
    );
    let create_index_result =
        execute_query(&mut app, secondary_tab_id, &create_index, &shared_client);
    match &create_index_result {
        QueryResult::SingleDocument { .. } => {}
        other => panic!("expected createIndex response document, got {:?}", other),
    }

    let get_indexes =
        format!("db.getCollection('{collection}').getIndexes()", collection = collection_name_2);
    let get_indexes_result =
        execute_query(&mut app, secondary_tab_id, &get_indexes, &shared_client);
    let index_names: Vec<String> = match &get_indexes_result {
        QueryResult::Indexes(values) => values
            .iter()
            .filter_map(|b| b.as_document())
            .filter_map(|doc| doc.get_str("name").ok().map(|s| s.to_string()))
            .collect(),
        other => panic!("expected index list, got {:?}", other),
    };
    assert_eq!(index_names.len(), 2);
    assert!(index_names.contains(&String::from("_id_")));
    assert!(index_names.contains(&String::from("name_asc")));

    //
    // Step 5.2: Reinsert documents for further index tests.
    //
    let insert_many_again = format!(
        concat!(
            "db.getCollection('{collection}').insertMany([\n",
            "    {{ \"name\": \"Alex\", \"department\": \"IT\", \"starts\": ISODate(\"2020-02-01\"), \"points\": 10, \"scores\": 1 }},\n",
            "    {{ \"name\": \"Alex\", \"department\": \"Support\", \"starts\": ISODate(\"2018-03-10\"), \"points\": 8, \"scores\": 1 }},\n",
            "    {{ \"name\": \"Anya\", \"department\": \"IT\", \"starts\": ISODate(\"2020-06-15\"), \"points\": 20, \"scores\": 1 }},\n",
            "    {{ \"name\": \"Mark\", \"department\": \"Devops\", \"starts\": ISODate(\"2019-05-01\"), \"points\": 12, \"scores\": 1 }}\n",
            "])"
        ),
        collection = collection_name_2
    );
    let insert_again_result =
        execute_query(&mut app, secondary_tab_id, &insert_many_again, &shared_client);
    match &insert_again_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_str("operation").unwrap_or_default(), "insertMany");
            assert_eq!(get_numeric_i64(document, "insertedCount"), 4);
        }
        other => panic!("expected insertMany result on reinsertion, got {:?}", other),
    }

    //
    // Step 5.3: Create multiple indexes and verify names list.
    //
    let create_indexes = format!(
        "db.getCollection('{collection}').createIndexes([{{starts: 1}}, {{points: -1}}])",
        collection = collection_name_2
    );
    let create_indexes_result =
        execute_query(&mut app, secondary_tab_id, &create_indexes, &shared_client);
    match &create_indexes_result {
        QueryResult::SingleDocument { .. } => {}
        other => panic!("expected createIndexes response document, got {:?}", other),
    }

    let get_indexes_full =
        format!("db.getCollection('{collection}').getIndexes()", collection = collection_name_2);
    let get_indexes_full_result =
        execute_query(&mut app, secondary_tab_id, &get_indexes_full, &shared_client);
    let mut full_index_names: Vec<String> = match &get_indexes_full_result {
        QueryResult::Indexes(values) => values
            .iter()
            .filter_map(|b| b.as_document())
            .filter_map(|doc| doc.get_str("name").ok().map(|s| s.to_string()))
            .collect(),
        other => panic!("expected index list after createIndexes, got {:?}", other),
    };
    full_index_names.sort();
    assert_eq!(
        full_index_names,
        vec![
            "_id_".to_string(),
            "name_asc".to_string(),
            "points_-1".to_string(),
            "starts_1".to_string()
        ]
    );

    //
    // Step 5.4.1: Explain plan for Mark should use name_asc index.
    //
    let explain_query = format!(
        "db.getCollection('{collection}').find({{\"name\": \"Mark\"}}).explain()",
        collection = collection_name_2
    );
    let explain_result = execute_query(&mut app, secondary_tab_id, &explain_query, &shared_client);
    let plan_doc = match &explain_result {
        QueryResult::SingleDocument { document } => document,
        other => panic!("expected explain to return a single document, got {:?}", other),
    };
    let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
    let winning_plan =
        planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
    let input_stage = winning_plan
        .get_document("inputStage")
        .or_else(|_| {
            winning_plan
                .get_array("inputStage")
                .ok()
                .and_then(|arr| arr.first().and_then(|b| b.as_document()))
                .ok_or(())
        })
        .map(|doc| doc.clone())
        .unwrap_or_else(|_| panic!("inputStage missing in winningPlan: {:?}", winning_plan));

    let stage = input_stage.get_str("stage").unwrap_or_default();
    assert_eq!(stage, "IXSCAN");
    let index_name = input_stage.get_str("indexName").unwrap_or_default();
    assert_eq!(index_name, "name_asc");

    //
    // Step 5.4.2: explain().find(...).finish() should produce the same plan.
    //
    let explain_chain_query = format!(
        "db.getCollection('{collection}').explain().find({{\"name\": \"Mark\"}}).finish()",
        collection = collection_name_2
    );
    let explain_chain_result =
        execute_query(&mut app, secondary_tab_id, &explain_chain_query, &shared_client);
    let plan_doc = match &explain_chain_result {
        QueryResult::SingleDocument { document } => document,
        other => panic!("expected explain chain to return a single document, got {:?}", other),
    };
    let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
    let winning_plan =
        planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
    let input_stage = winning_plan
        .get_document("inputStage")
        .or_else(|_| {
            winning_plan
                .get_array("inputStage")
                .ok()
                .and_then(|arr| arr.first().and_then(|b| b.as_document()))
                .ok_or(())
        })
        .map(|doc| doc.clone())
        .unwrap_or_else(|_| panic!("inputStage missing in winningPlan: {:?}", winning_plan));

    let stage = input_stage.get_str("stage").unwrap_or_default();
    assert_eq!(stage, "IXSCAN");
    let index_name = input_stage.get_str("indexName").unwrap_or_default();
    assert_eq!(index_name, "name_asc");

    //
    // Step 5.5: Hide index and expect COLLSCAN in explain.
    //
    let hide_index = format!(
        "db.getCollection('{collection}').hideIndex(\"name_asc\")",
        collection = collection_name_2
    );
    let hide_index_result = execute_query(&mut app, secondary_tab_id, &hide_index, &shared_client);
    match &hide_index_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_f64("ok").unwrap_or_default(), 1.0);
        }
        other => panic!("expected hideIndex response, got {:?}", other),
    }

    let explain_after_hide = format!(
        "db.getCollection('{collection}').find({{\"name\": \"Mark\"}}).explain()",
        collection = collection_name_2
    );
    let explain_after_hide_result =
        execute_query(&mut app, secondary_tab_id, &explain_after_hide, &shared_client);
    let plan_doc = match &explain_after_hide_result {
        QueryResult::SingleDocument { document } => document,
        other => panic!("expected explain after hide to return a single document, got {:?}", other),
    };
    let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
    let winning_plan =
        planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
    let stage = winning_plan.get_str("stage").unwrap_or_default();
    assert_eq!(stage, "COLLSCAN");

    //
    // Step 5.6: Unhide index and expect IXSCAN on name_asc.
    //
    let unhide_index = format!(
        "db.getCollection('{collection}').unhideIndex(\"name_asc\")",
        collection = collection_name_2
    );
    let unhide_index_result =
        execute_query(&mut app, secondary_tab_id, &unhide_index, &shared_client);
    match &unhide_index_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_f64("ok").unwrap_or_default(), 1.0);
        }
        other => panic!("expected unhideIndex response, got {:?}", other),
    }

    let explain_after_unhide = format!(
        "db.getCollection('{collection}').find({{\"name\": \"Mark\"}}).explain()",
        collection = collection_name_2
    );
    let explain_after_unhide_result =
        execute_query(&mut app, secondary_tab_id, &explain_after_unhide, &shared_client);
    let plan_doc = match &explain_after_unhide_result {
        QueryResult::SingleDocument { document } => document,
        other => {
            panic!("expected explain after unhide to return a single document, got {:?}", other)
        }
    };
    let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
    let winning_plan =
        planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
    let input_stage = winning_plan
        .get_document("inputStage")
        .or_else(|_| {
            winning_plan
                .get_array("inputStage")
                .ok()
                .and_then(|arr| arr.first().and_then(|b| b.as_document()))
                .ok_or(())
        })
        .map(|doc| doc.clone())
        .unwrap_or_else(|_| panic!("inputStage missing in winningPlan: {:?}", winning_plan));

    let stage = input_stage.get_str("stage").unwrap_or_default();
    assert_eq!(stage, "IXSCAN");
    let index_name = input_stage.get_str("indexName").unwrap_or_default();
    assert_eq!(index_name, "name_asc");

    //
    // Step 5.7: dropIndex should remove name_asc and explain should fall back to COLLSCAN.
    //
    let drop_index = format!(
        "db.getCollection('{collection}').dropIndex(\"name_asc\")",
        collection = collection_name_2
    );
    let drop_index_result = execute_query(&mut app, secondary_tab_id, &drop_index, &shared_client);
    match &drop_index_result {
        QueryResult::SingleDocument { document } => {
            assert_eq!(document.get_f64("ok").unwrap_or_default(), 1.0);
        }
        other => panic!("expected dropIndex response, got {:?}", other),
    }

    let explain_after_drop = format!(
        "db.getCollection('{collection}').find({{\"name\": \"Mark\"}}).explain()",
        collection = collection_name_2
    );
    let explain_after_drop_result =
        execute_query(&mut app, secondary_tab_id, &explain_after_drop, &shared_client);
    let plan_doc = match &explain_after_drop_result {
        QueryResult::SingleDocument { document } => document,
        other => panic!("expected explain after drop to return a single document, got {:?}", other),
    };
    let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
    let winning_plan =
        planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
    assert_eq!(winning_plan.get_str("stage").unwrap_or_default(), "COLLSCAN");

    let get_indexes_after_drop =
        format!("db.getCollection('{collection}').getIndexes()", collection = collection_name_2);
    let get_indexes_after_drop_result =
        execute_query(&mut app, secondary_tab_id, &get_indexes_after_drop, &shared_client);
    let mut remaining_indexes: Vec<String> = match &get_indexes_after_drop_result {
        QueryResult::Indexes(values) => values
            .iter()
            .filter_map(|b| b.as_document())
            .filter_map(|doc| doc.get_str("name").ok().map(|s| s.to_string()))
            .collect(),
        other => panic!("expected index list after dropIndex, got {:?}", other),
    };
    remaining_indexes.sort();
    assert_eq!(
        remaining_indexes,
        vec!["_id_".to_string(), "points_-1".to_string(), "starts_1".to_string()]
    );

    //
    // Step 6.1: sort by starts ascending should begin with 2018-03-10 and end with 2020-06-15.
    //
    let sort_starts_asc = format!(
        "db.getCollection('{collection}').find({{}}).sort({{'starts': 1}})",
        collection = collection_name_2
    );
    let sort_starts_asc_result =
        execute_query(&mut app, secondary_tab_id, &sort_starts_asc, &shared_client);
    match &sort_starts_asc_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 4);
            let first = values.first().and_then(|b| b.as_document()).unwrap();
            let last = values.last().and_then(|b| b.as_document()).unwrap();
            let first_start = bson::DateTime::parse_rfc3339_str("2018-03-10T00:00:00Z").unwrap();
            let last_start = bson::DateTime::parse_rfc3339_str("2020-06-15T00:00:00Z").unwrap();
            assert_eq!(first.get_datetime("starts").unwrap(), &first_start);
            assert_eq!(last.get_datetime("starts").unwrap(), &last_start);
        }
        other => panic!("expected sorted documents by starts asc, got {:?}", other),
    }

    //
    // Step 6.2: sort by starts descending should begin with 2020-06-15 and end with 2018-03-10.
    //
    let sort_starts_desc = format!(
        "db.getCollection('{collection}').find({{}}).sort({{'starts': -1}})",
        collection = collection_name_2
    );
    let sort_starts_desc_result =
        execute_query(&mut app, secondary_tab_id, &sort_starts_desc, &shared_client);
    match &sort_starts_desc_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 4);
            let first = values.first().and_then(|b| b.as_document()).unwrap();
            let last = values.last().and_then(|b| b.as_document()).unwrap();
            let first_start = bson::DateTime::parse_rfc3339_str("2020-06-15T00:00:00Z").unwrap();
            let last_start = bson::DateTime::parse_rfc3339_str("2018-03-10T00:00:00Z").unwrap();
            assert_eq!(first.get_datetime("starts").unwrap(), &first_start);
            assert_eq!(last.get_datetime("starts").unwrap(), &last_start);
        }
        other => panic!("expected sorted documents by starts desc, got {:?}", other),
    }

    //
    // Step 6.3: sort by points descending should start with 20 and end with 8.
    //
    let sort_points_desc = format!(
        "db.getCollection('{collection}').find({{}}).sort({{'points': -1}})",
        collection = collection_name_2
    );
    let sort_points_desc_result =
        execute_query(&mut app, secondary_tab_id, &sort_points_desc, &shared_client);
    match &sort_points_desc_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 4);
            let first = values.first().and_then(|b| b.as_document()).unwrap();
            let last = values.last().and_then(|b| b.as_document()).unwrap();
            assert_eq!(get_numeric_i64(first, "points"), 20);
            assert_eq!(get_numeric_i64(last, "points"), 8);
        }
        other => panic!("expected sorted documents by points desc, got {:?}", other),
    }

    //
    // Step 6.4: sort by points ascending for Alex should start with 8 then 10.
    //
    let sort_alex_points = format!(
        "db.getCollection('{collection}').find({{name: \"Alex\"}}).sort({{'points': 1}})",
        collection = collection_name_2
    );
    let sort_alex_points_result =
        execute_query(&mut app, secondary_tab_id, &sort_alex_points, &shared_client);
    match &sort_alex_points_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 2);
            let first = values.first().and_then(|b| b.as_document()).unwrap();
            let last = values.last().and_then(|b| b.as_document()).unwrap();
            assert_eq!(get_numeric_i64(first, "points"), 8);
            assert_eq!(get_numeric_i64(last, "points"), 10);
        }
        other => panic!("expected sorted Alex documents by points asc, got {:?}", other),
    }

    let _extract_input_stage = |plan_doc: &Document| -> Document {
        let planner = plan_doc.get_document("queryPlanner").expect("queryPlanner missing");
        let winning_plan =
            planner.get_document("winningPlan").expect("winningPlan missing in queryPlanner");
        winning_plan
            .get_document("inputStage")
            .or_else(|_| {
                winning_plan
                    .get_array("inputStage")
                    .ok()
                    .and_then(|arr| arr.first().and_then(|b| b.as_document()))
                    .ok_or(())
            })
            .map(|doc| doc.clone())
            .unwrap_or_else(|_| panic!("inputStage missing in winningPlan: {:?}", winning_plan))
    };

    //
    // Step 7.1: hint to points_-1 should order Alex docs by points descending (10 then 8).
    //
    let hint_points_desc = format!(
        "db.getCollection('{collection}').find({{name: \"Alex\"}}).hint('points_-1')",
        collection = collection_name_2
    );
    let hint_points_desc_result =
        execute_query(&mut app, secondary_tab_id, &hint_points_desc, &shared_client);
    match &hint_points_desc_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 2);
            let first = values.first().and_then(|b| b.as_document()).unwrap();
            let last = values.last().and_then(|b| b.as_document()).unwrap();
            assert_eq!(get_numeric_i64(first, "points"), 10);
            assert_eq!(get_numeric_i64(last, "points"), 8);
        }
        other => panic!("expected documents result for hint points_-1, got {:?}", other),
    }

    //
    // Step 7.2: hint to non-existent index should error.
    //
    let bad_hint_query = format!(
        "db.getCollection('{collection}').find({{name: \"Alex\"}}).hint('points_1')",
        collection = collection_name_2
    );
    let (bad_hint_collection, bad_hint_op) =
        parse_collection_query_with_collection(&new_db_name_1, &collection_name_2, &bad_hint_query)
            .unwrap();
    let bad_hint_err = run_collection_query(
        Arc::clone(&shared_client),
        new_db_name_1.clone(),
        bad_hint_collection,
        bad_hint_op,
        0,
        DEFAULT_RESULT_LIMIT as u64,
        app.test_query_timeout(),
    )
    .expect_err("expected hint to non-existent index to fail");
    assert!(
        bad_hint_err.to_lowercase().contains("hint"),
        "expected hint-related error, got {bad_hint_err}"
    );

    //
    // Step 8: comment on find should still return two Alex documents.
    //
    let comment_query = format!(
        "db.getCollection('{collection}').find({{name: \"Alex\"}}).comment('my_comment')",
        collection = collection_name_2
    );
    let comment_result = execute_query(&mut app, secondary_tab_id, &comment_query, &shared_client);
    match &comment_result {
        QueryResult::Documents(values) => {
            assert_eq!(values.len(), 2);
            for doc in values {
                let name = doc.as_document().and_then(|d| d.get_str("name").ok()).unwrap_or("");
                assert_eq!(name, "Alex");
            }
        }
        other => panic!("expected two Alex documents with comment, got {:?}", other),
    }
}
