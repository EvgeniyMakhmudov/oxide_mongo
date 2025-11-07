#![cfg(test)]

use crate::mongo::connection::ConnectionBootstrap;
use crate::ui::connections::{ConnectionEntry, ConnectionFormState, ConnectionsWindowState};
use crate::ui::menues::{ConnectionContextAction, MenuEntry, TopMenu};
use crate::{App, AppMode, ClientId, Message};
use mongodb::sync::Client;
use std::env;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::Arc;
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
}

#[test]
// #[ignore]
fn connection_flow_via_messages() {
    let uri = match env::var("OXIDE_MONGO_TEST_URI") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("skipping connection_flow_via_messages: OXIDE_MONGO_TEST_URI not provided");
            return;
        }
    };

    let (host, port) = extract_host_port(&uri)
        .map(|(host, port)| (host.trim().to_string(), port))
        .expect("failed to parse host/port from MongoDB URI");

    let shared_client = Arc::new(
        Client::with_uri_str(&uri)
            .expect("failed to establish client connection using provided URI"),
    );

    let connection_name = format!("connection-{}", Uuid::new_v4().simple());
    let new_db_name_1 = format!("db-{}", Uuid::new_v4().simple());
    let new_db_name_2 = format!("db-{}", Uuid::new_v4().simple());
    let collection_name_1 = format!("collection-{}", Uuid::new_v4().simple());

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

    let bootstrap = ConnectionBootstrap { handle: shared_client.clone(), databases: Vec::new() };
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

    let saved_connection = app.test_connections().first().expect("connection entry should exist");
    assert_eq!(saved_connection.name, connection_name);
    assert_eq!(saved_connection.host, host.trim());
    assert_eq!(saved_connection.port, port);
}
