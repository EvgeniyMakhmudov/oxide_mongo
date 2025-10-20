use std::collections::HashSet;
use std::sync::Arc;

use mongodb::sync::Client;

#[derive(Debug, Clone)]
pub enum OMDBConnection {
    Uri { uri: String, include_filter: String, exclude_filter: String },
}

#[derive(Debug, Clone)]
pub struct ConnectionBootstrap {
    pub handle: Arc<Client>,
    pub databases: Vec<String>,
}

impl OMDBConnection {
    pub fn from_uri(uri: &str, include_filter: &str, exclude_filter: &str) -> Self {
        Self::Uri {
            uri: uri.to_owned(),
            include_filter: include_filter.to_owned(),
            exclude_filter: exclude_filter.to_owned(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            OMDBConnection::Uri { uri, .. } => uri.clone(),
        }
    }
}

pub fn connect_and_discover(connection: OMDBConnection) -> Result<ConnectionBootstrap, String> {
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

pub fn fetch_collections(client: Arc<Client>, db_name: String) -> Result<Vec<String>, String> {
    let database = client.database(&db_name);
    database.list_collection_names().run().map_err(|err| err.to_string())
}
