use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use mongodb::sync::Client;

use crate::mongo::ssh_tunnel::SshTunnel;
use crate::ui::connections::ConnectionEntry;

#[derive(Debug, Clone)]
pub enum OMDBConnection {
    Entry { entry: ConnectionEntry },
}

#[derive(Debug, Clone)]
pub struct ConnectionBootstrap {
    pub handle: Arc<Client>,
    pub databases: Vec<String>,
    pub ssh_tunnel: Option<Arc<Mutex<SshTunnel>>>,
}

impl OMDBConnection {
    pub fn from_entry(entry: ConnectionEntry) -> Self {
        Self::Entry { entry }
    }
}

pub fn connect_and_discover(connection: OMDBConnection) -> Result<ConnectionBootstrap, String> {
    match connection {
        OMDBConnection::Entry { entry } => {
            let include_filter = entry.include_filter.clone();
            let exclude_filter = entry.exclude_filter.clone();
            let mut ssh_tunnel: Option<Arc<Mutex<SshTunnel>>> = None;

            let uri = if entry.ssh_tunnel.enabled {
                let tunnel = SshTunnel::start(&entry.ssh_tunnel, &entry.host, entry.port)?;
                let uri = entry.uri_for_host_port("127.0.0.1", tunnel.local_port())?;
                ssh_tunnel = Some(Arc::new(Mutex::new(tunnel)));
                uri
            } else {
                entry.uri()?
            };

            let client = Client::with_uri_str(&uri).map_err(|err| err.to_string())?;
            let databases = filter_databases(
                client.list_database_names().run().map_err(|err| err.to_string())?,
                &include_filter,
                &exclude_filter,
            );
            Ok(ConnectionBootstrap { handle: Arc::new(client), databases, ssh_tunnel })
        }
    }
}

fn filter_databases(
    mut databases: Vec<String>,
    include_filter: &str,
    exclude_filter: &str,
) -> Vec<String> {
    let include_items: Vec<_> =
        include_filter.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if !include_items.is_empty() {
        let include_set: HashSet<_> = include_items.into_iter().collect();
        databases.retain(|db| include_set.contains(db.as_str()));
        return databases;
    }

    let exclude_items: Vec<_> =
        exclude_filter.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if !exclude_items.is_empty() {
        let exclude_set: HashSet<_> = exclude_items.into_iter().collect();
        databases.retain(|db| !exclude_set.contains(db.as_str()));
    }

    databases
}

pub fn fetch_collections(client: Arc<Client>, db_name: String) -> Result<Vec<String>, String> {
    let database = client.database(&db_name);
    database.list_collection_names().run().map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::filter_databases;

    fn to_vec(items: &[&str]) -> Vec<String> {
        items.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn include_filter_takes_precedence() {
        let databases = to_vec(&["admin", "app", "local"]);
        let filtered = filter_databases(databases, "app\nadmin", "local");
        assert_eq!(filtered, to_vec(&["admin", "app"]));
    }

    #[test]
    fn exclude_filter_applies_when_include_missing() {
        let databases = to_vec(&["admin", "app", "local"]);
        let filtered = filter_databases(databases, "", "local\nadmin");
        assert_eq!(filtered, to_vec(&["app"]));
    }

    #[test]
    fn whitespace_only_filters_are_ignored() {
        let databases = to_vec(&["admin", "app"]);
        let filtered = filter_databases(databases.clone(), "  \n\t  ", "  ");
        assert_eq!(filtered, databases);
    }

    #[test]
    fn duplicate_entries_in_filters_are_handled() {
        let databases = to_vec(&["admin", "analytics", "app"]);
        let filtered = filter_databases(databases, "admin\n\nadmin\n\n\napp", "analytics");
        assert_eq!(filtered, to_vec(&["admin", "app"]));
    }
}
