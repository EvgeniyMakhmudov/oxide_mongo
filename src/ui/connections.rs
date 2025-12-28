use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::checkbox::Checkbox;
use iced::widget::pick_list::PickList;
use iced::widget::text_editor::{self, Action as TextEditorAction, Content as TextEditorContent};
use iced::widget::{
    self, Button, Column, Container, Image, Row, Scrollable, Space, button, text_input,
};
use iced::{Color, Element, Length, Shadow, Theme, border};
use serde::{Deserialize, Serialize};

use crate::fonts;
use crate::i18n::tr;
use crate::settings::ThemePalette;
use crate::ui::modal::modal_layout;
use crate::{
    DOUBLE_CLICK_INTERVAL, ICON_NETWORK_BYTES, ICON_NETWORK_HANDLE, Message, shared_icon_handle,
};

const CONNECTIONS_FILE: &str = "connections.toml";
const PASSWORD_STORAGE_OPTIONS: &[PasswordStorage] =
    &[PasswordStorage::Prompt, PasswordStorage::File];
const AUTH_MECHANISM_OPTIONS: &[AuthMechanismChoice] = &[
    AuthMechanismChoice::ScramSha256,
    AuthMechanismChoice::ScramSha1,
    AuthMechanismChoice::MongodbX509,
];
const SSH_AUTH_METHOD_OPTIONS: &[SshAuthMethod] =
    &[SshAuthMethod::Password, SshAuthMethod::PrivateKey];
const CONNECTION_TYPE_OPTIONS: &[ConnectionType] =
    &[ConnectionType::Direct, ConnectionType::ReplicaSet];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasswordStorage {
    Prompt,
    File,
}

impl Default for PasswordStorage {
    fn default() -> Self {
        Self::Prompt
    }
}

impl std::fmt::Display for PasswordStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            PasswordStorage::Prompt => tr("Prompt for password"),
            PasswordStorage::File => tr("Store in file"),
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMechanismChoice {
    ScramSha256,
    ScramSha1,
    Plain,
    MongodbX509,
    Gssapi,
    MongodbAws,
}

impl Default for AuthMechanismChoice {
    fn default() -> Self {
        Self::ScramSha256
    }
}

impl AuthMechanismChoice {
    pub fn label(self) -> &'static str {
        match self {
            AuthMechanismChoice::ScramSha256 => "SCRAM-SHA-256",
            AuthMechanismChoice::ScramSha1 => "SCRAM-SHA-1",
            AuthMechanismChoice::Plain => "PLAIN",
            AuthMechanismChoice::MongodbX509 => "MONGODB-X509",
            AuthMechanismChoice::Gssapi => "GSSAPI",
            AuthMechanismChoice::MongodbAws => "MONGODB-AWS",
        }
    }
}

impl std::fmt::Display for AuthMechanismChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SshAuthMethod {
    Password,
    PrivateKey,
}

impl Default for SshAuthMethod {
    fn default() -> Self {
        Self::Password
    }
}

impl SshAuthMethod {
    pub fn label(self) -> &'static str {
        match self {
            SshAuthMethod::Password => tr("Password"),
            SshAuthMethod::PrivateKey => tr("Private key"),
        }
    }
}

impl std::fmt::Display for SshAuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

fn default_auth_database() -> String {
    String::from("admin")
}

fn default_ssh_port() -> u16 {
    22
}

pub(crate) fn looks_like_private_key(value: &str) -> bool {
    let trimmed = value.trim_start();
    trimmed.starts_with("-----BEGIN ") && trimmed.contains("PRIVATE KEY-----")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    #[serde(default)]
    pub use_auth: bool,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub password_storage: PasswordStorage,
    #[serde(default)]
    pub mechanism: AuthMechanismChoice,
    #[serde(default = "default_auth_database")]
    pub database: String,
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            use_auth: false,
            username: String::new(),
            password: None,
            password_storage: PasswordStorage::Prompt,
            mechanism: AuthMechanismChoice::ScramSha256,
            database: default_auth_database(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub auth_method: SshAuthMethod,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub private_key: Option<String>,
    #[serde(default)]
    pub passphrase: Option<String>,
}

impl Default for SshTunnelSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_ssh_port(),
            username: String::new(),
            auth_method: SshAuthMethod::default(),
            password: None,
            private_key: None,
            passphrase: None,
        }
    }
}

impl SshTunnelSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.host.trim().is_empty() {
            return Err(String::from(tr("SSH server address cannot be empty")));
        }

        if self.username.trim().is_empty() {
            return Err(String::from(tr("SSH username cannot be empty")));
        }

        match self.auth_method {
            SshAuthMethod::Password => {
                if self.password.as_deref().unwrap_or_default().trim().is_empty() {
                    return Err(String::from(tr("SSH password cannot be empty")));
                }
            }
            SshAuthMethod::PrivateKey => {
                let key_value = self.private_key.as_deref().unwrap_or_default().trim();
                if key_value.is_empty() {
                    return Err(String::from(tr("SSH private key cannot be empty")));
                }
                if !looks_like_private_key(key_value) && !Path::new(key_value).exists() {
                    return Err(String::from(tr("SSH private key file not found")));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub connection_type: ConnectionType,
    pub include_filter: String,
    pub exclude_filter: String,
    #[serde(default)]
    pub auth: AuthSettings,
    #[serde(default)]
    pub ssh_tunnel: SshTunnelSettings,
}

impl ConnectionEntry {
    pub fn address_label(&self) -> String {
        Self::address_label_for(&self.host, self.port)
    }

    pub fn address_label_for(host: &str, port: u16) -> String {
        format!("{}:{}", host.trim(), port)
    }

    pub fn uri(&self) -> Result<String, String> {
        self.uri_for_host_port(&self.host, self.port)
    }

    pub fn uri_for_host_port(&self, host: &str, port: u16) -> Result<String, String> {
        self.uri_with_host_port(host, port, self.auth.password.as_deref())
    }

    pub fn uri_with_host_port(
        &self,
        host: &str,
        port: u16,
        password_override: Option<&str>,
    ) -> Result<String, String> {
        let mut uri = String::from("mongodb://");
        let mut query_params: Vec<(String, String)> = Vec::new();
        query_params.push((
            String::from("directConnection"),
            if self.connection_type == ConnectionType::Direct {
                String::from("true")
            } else {
                String::from("false")
            },
        ));

        if self.auth.use_auth {
            let username = self.auth.username.trim();
            let password = password_override.unwrap_or_default().trim();

            if username.is_empty() {
                return Err(String::from(tr("Login cannot be empty")));
            }

            if password.is_empty() {
                return Err(String::from(tr("Password cannot be empty")));
            }

            uri.push_str(&percent_encode(username));
            uri.push(':');
            uri.push_str(&percent_encode(password));
            uri.push('@');
        }

        uri.push_str(&Self::address_label_for(host, port));

        if self.auth.use_auth {
            let database = self.auth.database.trim();
            let database =
                if database.is_empty() { default_auth_database() } else { database.to_string() };

            uri.push('/');
            uri.push_str(&percent_encode(&database));
            query_params
                .push((String::from("authMechanism"), self.auth.mechanism.label().to_string()));
            query_params.push((String::from("authSource"), database));
        }

        if !query_params.is_empty() {
            uri.push('?');
            let joined = query_params
                .into_iter()
                .map(|(key, value)| format!("{}={}", percent_encode(&key), percent_encode(&value)))
                .collect::<Vec<_>>()
                .join("&");
            uri.push_str(&joined);
        }

        Ok(uri)
    }

    pub fn sanitized_for_storage(&self) -> Self {
        let mut cloned = self.clone();
        if cloned.auth.password_storage == PasswordStorage::Prompt {
            cloned.auth.password = None;
        }
        cloned
    }
}

fn percent_encode(input: &str) -> String {
    input
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{:02X}", byte),
        })
        .collect()
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConnectionStore {
    connections: Vec<ConnectionEntry>,
}

#[derive(Debug)]
pub struct ConnectionsWindowState {
    pub(crate) selected: Option<usize>,
    pub(crate) confirm_delete: bool,
    pub(crate) feedback: Option<String>,
    pub(crate) last_click: Option<ListClick>,
}

impl ConnectionsWindowState {
    pub fn new(selected: Option<usize>) -> Self {
        Self { selected, confirm_delete: false, feedback: None, last_click: None }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ListClick {
    pub(crate) index: usize,
    pub(crate) at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFormTab {
    General,
    Authorization,
    SshTunnel,
    Filter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionType {
    ReplicaSet,
    Direct,
}

impl Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::ReplicaSet
    }
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ConnectionType::ReplicaSet => tr("ReplicaSet"),
            ConnectionType::Direct => tr("Direct connection"),
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFormMode {
    Create,
    Edit(usize),
}

#[derive(Debug)]
pub struct ConnectionFormState {
    pub(crate) mode: ConnectionFormMode,
    pub(crate) active_tab: ConnectionFormTab,
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) connection_type: ConnectionType,
    pub(crate) auth: AuthFormState,
    pub(crate) ssh: SshTunnelFormState,
    pub(crate) include_editor: TextEditorContent,
    pub(crate) exclude_editor: TextEditorContent,
    pub(crate) validation_error: Option<String>,
    pub(crate) test_feedback: Option<TestFeedback>,
    pub(crate) testing: bool,
}

#[derive(Debug, Clone)]
pub struct AuthFormState {
    pub(crate) use_auth: bool,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) password_storage: PasswordStorage,
    pub(crate) mechanism: AuthMechanismChoice,
    pub(crate) database: String,
}

#[derive(Debug, Clone)]
pub struct SshTunnelFormState {
    pub(crate) use_ssh: bool,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) username: String,
    pub(crate) auth_method: SshAuthMethod,
    pub(crate) password: String,
    pub(crate) private_key: String,
    pub(crate) passphrase: String,
}

impl Default for AuthFormState {
    fn default() -> Self {
        let defaults = AuthSettings::default();
        Self::from_settings(&defaults)
    }
}

impl Default for SshTunnelFormState {
    fn default() -> Self {
        let defaults = SshTunnelSettings::default();
        Self::from_settings(&defaults)
    }
}

impl AuthFormState {
    fn from_settings(settings: &AuthSettings) -> Self {
        Self {
            use_auth: settings.use_auth,
            username: settings.username.clone(),
            password: settings.password.clone().unwrap_or_default(),
            password_storage: settings.password_storage,
            mechanism: settings.mechanism,
            database: settings.database.clone(),
        }
    }

    fn to_settings(&self, require_password: bool) -> Result<AuthSettings, String> {
        if !self.use_auth {
            let database = self.database.trim();
            return Ok(AuthSettings {
                use_auth: false,
                username: self.username.trim().to_string(),
                password: None,
                password_storage: self.password_storage,
                mechanism: self.mechanism,
                database: if database.is_empty() {
                    default_auth_database()
                } else {
                    database.to_string()
                },
            });
        }

        let username = self.username.trim();
        if username.is_empty() {
            return Err(String::from(tr("Login cannot be empty")));
        }

        let database = self.database.trim();
        if database.is_empty() {
            return Err(String::from(tr("Database cannot be empty")));
        }

        let password_value = self.password.clone();
        if (require_password || self.password_storage == PasswordStorage::File)
            && password_value.trim().is_empty()
        {
            return Err(String::from(tr("Password cannot be empty")));
        }

        Ok(AuthSettings {
            use_auth: true,
            username: username.to_string(),
            password: (!password_value.is_empty()).then_some(password_value),
            password_storage: self.password_storage,
            mechanism: self.mechanism,
            database: database.to_string(),
        })
    }
}

impl SshTunnelFormState {
    fn from_settings(settings: &SshTunnelSettings) -> Self {
        Self {
            use_ssh: settings.enabled,
            host: settings.host.clone(),
            port: settings.port.to_string(),
            username: settings.username.clone(),
            auth_method: settings.auth_method,
            password: settings.password.clone().unwrap_or_default(),
            private_key: settings.private_key.clone().unwrap_or_default(),
            passphrase: settings.passphrase.clone().unwrap_or_default(),
        }
    }

    fn to_settings(&self) -> Result<SshTunnelSettings, String> {
        let port_str = self.port.trim();
        let port = if self.use_ssh {
            port_str
                .parse()
                .map_err(|_| String::from(tr("SSH port must be a number between 0 and 65535")))?
        } else if port_str.is_empty() {
            default_ssh_port()
        } else {
            port_str.parse().unwrap_or_else(|_| default_ssh_port())
        };

        let host = self.host.trim();
        let username = self.username.trim();

        if self.use_ssh {
            if host.is_empty() {
                return Err(String::from(tr("SSH server address cannot be empty")));
            }
            if username.is_empty() {
                return Err(String::from(tr("SSH username cannot be empty")));
            }
            match self.auth_method {
                SshAuthMethod::Password => {
                    if self.password.trim().is_empty() {
                        return Err(String::from(tr("SSH password cannot be empty")));
                    }
                }
                SshAuthMethod::PrivateKey => {
                    if self.private_key.trim().is_empty() {
                        return Err(String::from(tr("SSH private key cannot be empty")));
                    }
                }
            }
        }

        Ok(SshTunnelSettings {
            enabled: self.use_ssh,
            host: host.to_string(),
            port,
            username: username.to_string(),
            auth_method: self.auth_method,
            password: (!self.password.trim().is_empty()).then_some(self.password.clone()),
            private_key: (!self.private_key.trim().is_empty()).then_some(self.private_key.clone()),
            passphrase: (!self.passphrase.trim().is_empty()).then_some(self.passphrase.clone()),
        })
    }
}

impl ConnectionFormState {
    pub fn new(mode: ConnectionFormMode, entry: Option<&ConnectionEntry>) -> Self {
        let (name, host, port, connection_type, include_filter, exclude_filter, auth, ssh) = entry
            .map(|conn| {
                (
                    conn.name.clone(),
                    conn.host.clone(),
                    conn.port.to_string(),
                    conn.connection_type,
                    conn.include_filter.clone(),
                    conn.exclude_filter.clone(),
                    AuthFormState::from_settings(&conn.auth),
                    SshTunnelFormState::from_settings(&conn.ssh_tunnel),
                )
            })
            .unwrap_or_else(|| {
                (
                    String::new(),
                    String::from(tr("localhost")),
                    String::from(tr("27017")),
                    ConnectionType::default(),
                    String::new(),
                    String::new(),
                    AuthFormState::default(),
                    SshTunnelFormState::default(),
                )
            });

        Self {
            mode,
            active_tab: ConnectionFormTab::General,
            name,
            host,
            port,
            connection_type,
            auth,
            ssh,
            include_editor: TextEditorContent::with_text(&include_filter),
            exclude_editor: TextEditorContent::with_text(&exclude_filter),
            validation_error: None,
            test_feedback: None,
            testing: false,
        }
    }

    pub fn validate(&self, require_password: bool) -> Result<ConnectionEntry, String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(String::from(tr("Name cannot be empty")));
        }

        let host = self.host.trim();
        if host.is_empty() {
            return Err(String::from(tr("Address/Host/IP cannot be empty")));
        }

        let port: u16 = self
            .port
            .trim()
            .parse()
            .map_err(|_| String::from(tr("Port must be a number between 0 and 65535")))?;

        let auth = self.auth.to_settings(require_password)?;
        let ssh = self.ssh.to_settings()?;

        Ok(ConnectionEntry {
            name: name.to_string(),
            host: host.to_string(),
            port,
            connection_type: self.connection_type,
            include_filter: self.include_editor.text(),
            exclude_filter: self.exclude_editor.text(),
            auth,
            ssh_tunnel: ssh,
        })
    }

    pub fn include_action(&mut self, action: TextEditorAction) {
        self.include_editor.perform(action);
    }

    pub fn exclude_action(&mut self, action: TextEditorAction) {
        self.exclude_editor.perform(action);
    }

    pub fn add_system_filters(&mut self) {
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

#[derive(Debug)]
pub enum TestFeedback {
    Success(String),
    Failure(String),
}

impl TestFeedback {
    pub fn message(&self) -> &str {
        match self {
            TestFeedback::Success(msg) | TestFeedback::Failure(msg) => msg.as_str(),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, TestFeedback::Success(_))
    }
}

pub fn load_connections_from_disk() -> Result<Vec<ConnectionEntry>, String> {
    let path = connections_file_path();
    let data = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.to_string()),
    };

    let store: ConnectionStore = toml::from_str(&data).map_err(|err| err.to_string())?;
    Ok(store.connections)
}

pub fn save_connections_to_disk(connections: &[ConnectionEntry]) -> Result<(), String> {
    let sanitized: Vec<_> =
        connections.iter().map(ConnectionEntry::sanitized_for_storage).collect();
    let store = ConnectionStore { connections: sanitized };
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

pub fn connections_view<'a>(
    state: &'a ConnectionsWindowState,
    connections: &'a [ConnectionEntry],
    palette: &ThemePalette,
) -> Element<'a, Message> {
    let palette = palette.clone();
    let border_color = palette.widget_border_color();
    let normal_bg = palette.widget_background_color();
    let selected_bg = palette.subtle_buttons.hover.to_color();
    let accent_bar = palette.primary_buttons.active.to_color();
    let primary_text = palette.text_primary.to_color();
    let muted_text = palette.text_muted.to_color();
    let accent_text = palette.primary_buttons.active.to_color();
    let tag_bg = palette.subtle_buttons.active.to_color();
    let tag_border = palette.widget_border_color();

    let mut entries = Column::new().spacing(4).width(Length::Fill);

    if connections.is_empty() {
        entries = entries.push(
            Container::new(
                fonts::primary_text(tr("No saved connections"), Some(2.0)).color(muted_text),
            )
            .width(Length::Fill)
            .padding([12, 8]),
        );
    } else {
        for (index, entry) in connections.iter().enumerate() {
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

            let name_text = fonts::primary_text(entry.name.clone(), Some(4.0)).color(primary_text);
            let details_text =
                fonts::primary_text(entry.address_label(), Some(-1.0)).color(muted_text);

            let labels = Column::new().spacing(4).push(name_text).push(details_text);

            let filters_text = if entry.include_filter.trim().is_empty()
                && entry.exclude_filter.trim().is_empty()
            {
                fonts::primary_text(tr("No filters configured"), Some(-2.0)).color(muted_text)
            } else {
                fonts::primary_text(tr("Collection filters configured"), Some(-2.0))
                    .color(accent_text)
            };

            let tag = |label: &str| {
                Container::new(fonts::primary_text(label, Some(-3.0)).color(muted_text))
                    .padding([2, 6])
                    .style(move |_| widget::container::Style {
                        background: Some(tag_bg.into()),
                        border: border::rounded(6).width(1).color(tag_border),
                        ..Default::default()
                    })
            };

            let mut tags_row = Row::new().spacing(6).align_y(Vertical::Center);
            let mut has_tags = false;

            if entry.auth.use_auth {
                tags_row = tags_row.push(tag(tr("Auth")));
                has_tags = true;
            }
            if entry.ssh_tunnel.enabled {
                tags_row = tags_row.push(tag(tr("SSH")));
                has_tags = true;
            }

            let mut right_info =
                Column::new().spacing(4).align_x(Horizontal::Right).push(filters_text);
            if has_tags {
                right_info = right_info.push(tags_row);
            }

            let row = Row::new()
                .spacing(16)
                .align_y(Vertical::Center)
                .push(icon)
                .push(labels)
                .push(Space::with_width(Length::Fill))
                .push(right_info);

            let container =
                Container::new(row).padding([8, 12]).width(Length::Fill).style(move |_| {
                    widget::container::Style {
                        background: Some(if is_selected { selected_bg } else { normal_bg }.into()),
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
                .style(move |_| widget::container::Style {
                    background: Some(
                        if is_selected { accent_bar } else { Color::TRANSPARENT }.into(),
                    ),
                    ..Default::default()
                });

            let mut button =
                Button::new(Row::new().spacing(0).width(Length::Fill).push(accent).push(container))
                    .width(Length::Fill)
                    .style(subtle_button_style(palette.clone(), 6.0))
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
        Button::new(fonts::primary_text(tr("Create"), None))
            .padding([6, 16])
            .style(primary_button_style(palette.clone(), 6.0))
            .on_press(Message::ConnectionsCreate),
    );

    let mut edit_button = Button::new(fonts::primary_text(tr("Edit"), None))
        .padding([6, 16])
        .style(primary_button_style(palette.clone(), 6.0));
    if state.selected.is_some() {
        edit_button = edit_button.on_press(Message::ConnectionsEdit);
    }
    left_controls = left_controls.push(edit_button);

    let mut delete_button = Button::new(fonts::primary_text(tr("Delete"), None))
        .padding([6, 16])
        .style(primary_button_style(palette.clone(), 6.0));
    if state.selected.is_some() {
        delete_button = delete_button.on_press(Message::ConnectionsDelete);
    }
    left_controls = left_controls.push(delete_button);

    let mut connect_button = Button::new(fonts::primary_text(tr("Connect"), None))
        .padding([6, 16])
        .style(primary_button_style(palette.clone(), 6.0));
    if state.selected.is_some() {
        connect_button = connect_button.on_press(Message::ConnectionsConnect);
    }

    let right_controls = Row::new()
        .spacing(8)
        .push(
            Button::new(fonts::primary_text(tr("Cancel"), None))
                .padding([6, 16])
                .style(primary_button_style(palette.clone(), 6.0))
                .on_press(Message::ConnectionsCancel),
        )
        .push(connect_button);

    let mut content = Column::new()
        .spacing(16)
        .push(fonts::primary_text(tr("Connections"), Some(10.0)).color(primary_text))
        .push(list);

    if let Some(feedback) = &state.feedback {
        let error_color = Color::from_rgb8(0xd9, 0x53, 0x4f);
        let color =
            if feedback.starts_with(tr("Save error: ")) { error_color } else { accent_text };
        content = content.push(fonts::primary_text(feedback.clone(), None).color(color));
    }

    if state.confirm_delete {
        let name = state
            .selected
            .and_then(|index| connections.get(index))
            .map(|entry| entry.name.clone())
            .unwrap_or_else(|| String::from(tr("connection")));
        let confirm_row = Row::new()
            .spacing(12)
            .align_y(Vertical::Center)
            .push(
                fonts::primary_text(format!("{} \"{}\"?", tr("Delete"), name), None)
                    .color(primary_text),
            )
            .push(
                Button::new(fonts::primary_text(tr("Yes"), None))
                    .padding([4, 12])
                    .style(primary_button_style(palette.clone(), 6.0))
                    .on_press(Message::ConnectionsDeleteConfirmed),
            )
            .push(
                Button::new(fonts::primary_text(tr("No"), None))
                    .padding([4, 12])
                    .style(primary_button_style(palette.clone(), 6.0))
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

    let card_element: Element<Message> = content.into();
    modal_layout(palette, card_element, Length::Fixed(700.0), 20, 6.0)
}

fn subtle_button_style(
    palette: ThemePalette,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| palette.subtle_button_style(radius, status)
}

fn primary_button_style(
    palette: ThemePalette,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| palette.primary_button_style(radius, status)
}

fn disabled_primary_button_style(
    palette: ThemePalette,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let disabled_background = palette.subtle_buttons.active.to_color();
    let disabled_text = palette.text_muted.to_color();
    let border_color = palette.widget_border_color();
    move |_, status| match status {
        button::Status::Disabled => button::Style {
            background: Some(disabled_background.into()),
            text_color: disabled_text,
            border: border::rounded(radius).width(1).color(border_color),
            shadow: Shadow::default(),
            ..Default::default()
        },
        _ => palette.primary_button_style(radius, status),
    }
}

pub fn connection_form_view<'a>(
    state: &'a ConnectionFormState,
    palette: &ThemePalette,
) -> Element<'a, Message> {
    let title = match state.mode {
        ConnectionFormMode::Create => tr("New connection"),
        ConnectionFormMode::Edit(_) => tr("Edit connection"),
    };

    let palette = palette.clone();
    let border_color = palette.widget_border_color();
    let text_color = palette.text_primary.to_color();
    let muted_text = palette.text_muted.to_color();
    let accent_color = palette.primary_buttons.active.to_color();
    let tab_active_bg = palette.subtle_buttons.hover.to_color();
    let tab_inactive_bg = palette.subtle_buttons.active.to_color();
    let error_color = Color::from_rgb8(0xd9, 0x53, 0x4f);

    let general_active = state.active_tab == ConnectionFormTab::General;
    let general_label_color = if general_active { text_color } else { muted_text };
    let mut general_button =
        Button::new(fonts::primary_text(tr("General"), None).color(general_label_color))
            .padding([6, 16])
            .style({
                let border_color = border_color;
                let active_bg = tab_active_bg;
                let inactive_bg = tab_inactive_bg;
                move |_, _| button::Style {
                    background: Some((if general_active { active_bg } else { inactive_bg }).into()),
                    text_color: general_label_color,
                    border: border::rounded(6).width(1).color(border_color),
                    shadow: Shadow::default(),
                }
            });
    if !general_active {
        general_button =
            general_button.on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::General));
    }

    let authorization_active = state.active_tab == ConnectionFormTab::Authorization;
    let authorization_label_color = if authorization_active { text_color } else { muted_text };
    let mut authorization_button = Button::new(
        fonts::primary_text(tr("Authorization"), None).color(authorization_label_color),
    )
    .padding([6, 16])
    .style({
        let border_color = border_color;
        let active_bg = tab_active_bg;
        let inactive_bg = tab_inactive_bg;
        move |_, _| button::Style {
            background: Some((if authorization_active { active_bg } else { inactive_bg }).into()),
            text_color: authorization_label_color,
            border: border::rounded(6).width(1).color(border_color),
            shadow: Shadow::default(),
        }
    });
    if !authorization_active {
        authorization_button = authorization_button
            .on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::Authorization));
    }

    let ssh_active = state.active_tab == ConnectionFormTab::SshTunnel;
    let ssh_label_color = if ssh_active { text_color } else { muted_text };
    let mut ssh_button =
        Button::new(fonts::primary_text(tr("SSH tunnel"), None).color(ssh_label_color))
            .padding([6, 16])
            .style({
                let border_color = border_color;
                let active_bg = tab_active_bg;
                let inactive_bg = tab_inactive_bg;
                move |_, _| button::Style {
                    background: Some((if ssh_active { active_bg } else { inactive_bg }).into()),
                    text_color: ssh_label_color,
                    border: border::rounded(6).width(1).color(border_color),
                    shadow: Shadow::default(),
                }
            });
    if !ssh_active {
        ssh_button =
            ssh_button.on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::SshTunnel));
    }

    let filter_active = state.active_tab == ConnectionFormTab::Filter;
    let filter_label_color = if filter_active { text_color } else { muted_text };
    let mut filter_button =
        Button::new(fonts::primary_text(tr("Database filter"), None).color(filter_label_color))
            .padding([6, 16])
            .style({
                let border_color = border_color;
                let active_bg = tab_active_bg;
                let inactive_bg = tab_inactive_bg;
                move |_, _| button::Style {
                    background: Some((if filter_active { active_bg } else { inactive_bg }).into()),
                    text_color: filter_label_color,
                    border: border::rounded(6).width(1).color(border_color),
                    shadow: Shadow::default(),
                }
            });
    if !filter_active {
        filter_button =
            filter_button.on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::Filter));
    }

    let tabs_row = Row::new()
        .spacing(8)
        .push(general_button)
        .push(authorization_button)
        .push(ssh_button)
        .push(filter_button);

    let tab_content: Element<_> = match state.active_tab {
        ConnectionFormTab::General => {
            let name_input = text_input(tr("Name"), &state.name)
                .on_input(Message::ConnectionFormNameChanged)
                .padding([6, 12])
                .width(Length::Fill);

            let host_input = text_input(tr("Address/Host/IP"), &state.host)
                .on_input(Message::ConnectionFormHostChanged)
                .padding([6, 12])
                .width(Length::Fill);

            let port_input = text_input(tr("Port"), &state.port)
                .on_input(Message::ConnectionFormPortChanged)
                .padding([6, 12])
                .align_x(Horizontal::Center)
                .width(Length::Fixed(120.0));

            let connection_type = PickList::new(
                CONNECTION_TYPE_OPTIONS,
                Some(state.connection_type),
                Message::ConnectionFormTypeChanged,
            )
            .width(Length::FillPortion(4));

            Column::new()
                .spacing(12)
                .push(fonts::primary_text(tr("Name"), None).color(text_color))
                .push(name_input)
                .push(fonts::primary_text(tr("Address/Host/IP"), None).color(text_color))
                .push(host_input)
                .push(fonts::primary_text(tr("Port"), None).color(text_color))
                .push(port_input)
                .push(
                    Row::new()
                        .spacing(12)
                        .align_y(Vertical::Center)
                        .push(
                            fonts::primary_text(tr("Connection type"), None)
                                .color(text_color)
                                .width(Length::FillPortion(2)),
                        )
                        .push(connection_type)
                        .push(Space::with_width(Length::FillPortion(1))),
                )
                .into()
        }
        ConnectionFormTab::Authorization => {
            let use_auth = Checkbox::new(tr("Use"), state.auth.use_auth)
                .on_toggle(Message::ConnectionFormAuthUseChanged);

            let login_input = text_input(tr("Login"), &state.auth.username)
                .on_input(Message::ConnectionFormAuthLoginChanged)
                .padding([6, 12])
                .width(Length::Fill);

            let mut password_input = text_input(tr("Password"), &state.auth.password)
                .on_input(Message::ConnectionFormAuthPasswordChanged)
                .padding([6, 12])
                .width(Length::FillPortion(3));
            #[allow(deprecated)]
            {
                password_input = password_input.secure(true);
            }

            let password_storage = PickList::new(
                PASSWORD_STORAGE_OPTIONS,
                Some(state.auth.password_storage),
                Message::ConnectionFormPasswordStorageChanged,
            )
            .width(Length::FillPortion(2));

            let password_row = Row::new().spacing(12).push(password_input).push(password_storage);

            let mechanism = PickList::new(
                AUTH_MECHANISM_OPTIONS,
                Some(state.auth.mechanism),
                Message::ConnectionFormAuthMechanismChanged,
            )
            .width(Length::FillPortion(3));

            let mechanism_row = Row::new()
                .spacing(12)
                .align_y(Vertical::Center)
                .push(
                    fonts::primary_text(tr("Authentication mechanism"), None)
                        .color(text_color)
                        .width(Length::FillPortion(2)),
                )
                .push(mechanism)
                .push(Space::with_width(Length::FillPortion(1)));

            let database_input = text_input(tr("Database"), &state.auth.database)
                .on_input(Message::ConnectionFormAuthDatabaseChanged)
                .padding([6, 12])
                .width(Length::FillPortion(3));

            Column::new()
                .spacing(12)
                .push(use_auth)
                .push(fonts::primary_text(tr("Login"), None).color(text_color))
                .push(login_input)
                .push(fonts::primary_text(tr("Password"), None).color(text_color))
                .push(password_row)
                .push(mechanism_row)
                .push(fonts::primary_text(tr("Database"), None).color(text_color))
                .push(database_input)
                .into()
        }
        ConnectionFormTab::SshTunnel => {
            let use_ssh = Checkbox::new(tr("Use"), state.ssh.use_ssh)
                .on_toggle(Message::ConnectionFormSshUseChanged);

            let host_input = text_input(tr("Server address"), &state.ssh.host)
                .on_input(Message::ConnectionFormSshHostChanged)
                .padding([6, 12])
                .width(Length::Fill);

            let port_input = text_input(tr("Server port"), &state.ssh.port)
                .on_input(Message::ConnectionFormSshPortChanged)
                .padding([6, 12])
                .align_x(Horizontal::Center)
                .width(Length::Fixed(120.0));

            let user_input = text_input(tr("Username"), &state.ssh.username)
                .on_input(Message::ConnectionFormSshUsernameChanged)
                .padding([6, 12])
                .width(Length::Fill);

            let auth_method = PickList::new(
                SSH_AUTH_METHOD_OPTIONS,
                Some(state.ssh.auth_method),
                Message::ConnectionFormSshAuthMethodChanged,
            )
            .width(Length::FillPortion(3));

            let auth_method_row = Row::new()
                .spacing(12)
                .align_y(Vertical::Center)
                .push(
                    fonts::primary_text(tr("Authentication method"), None)
                        .color(text_color)
                        .width(Length::FillPortion(2)),
                )
                .push(auth_method)
                .push(Space::with_width(Length::FillPortion(1)));

            let mut column = Column::new()
                .spacing(12)
                .push(use_ssh)
                .push(fonts::primary_text(tr("Server address"), None).color(text_color))
                .push(host_input)
                .push(fonts::primary_text(tr("Server port"), None).color(text_color))
                .push(port_input)
                .push(fonts::primary_text(tr("Username"), None).color(text_color))
                .push(user_input)
                .push(auth_method_row);

            match state.ssh.auth_method {
                SshAuthMethod::Password => {
                    let mut password_input = text_input(tr("Password"), &state.ssh.password)
                        .on_input(Message::ConnectionFormSshPasswordChanged)
                        .padding([6, 12])
                        .width(Length::Fill);
                    #[allow(deprecated)]
                    {
                        password_input = password_input.secure(true);
                    }
                    column = column
                        .push(fonts::primary_text(tr("Password"), None).color(text_color))
                        .push(password_input);
                }
                SshAuthMethod::PrivateKey => {
                    let private_key_input = text_input(tr("Text key"), &state.ssh.private_key)
                        .on_input(Message::ConnectionFormSshPrivateKeyChanged)
                        .padding([6, 12])
                        .width(Length::Fill);

                    let browse_button = Button::new(fonts::primary_text("...", None))
                        .padding([6, 10])
                        .style(subtle_button_style(palette.clone(), 6.0))
                        .on_press(Message::ConnectionFormSshPrivateKeyBrowse)
                        .width(Length::Fixed(36.0));

                    let mut passphrase_input = text_input(tr("Passphrase"), &state.ssh.passphrase)
                        .on_input(Message::ConnectionFormSshPassphraseChanged)
                        .padding([6, 12])
                        .width(Length::Fill);
                    #[allow(deprecated)]
                    {
                        passphrase_input = passphrase_input.secure(true);
                    }

                    column = column
                        .push(fonts::primary_text(tr("Private key"), None).color(text_color))
                        .push(
                            Row::new()
                                .spacing(8)
                                .align_y(Vertical::Center)
                                .push(private_key_input)
                                .push(browse_button),
                        )
                        .push(fonts::primary_text(tr("Passphrase"), None).color(text_color))
                        .push(passphrase_input);
                }
            }

            column.into()
        }
        ConnectionFormTab::Filter => {
            let include_editor = text_editor::TextEditor::new(&state.include_editor)
                .on_action(Message::ConnectionFormIncludeAction)
                .height(Length::Fixed(130.0));

            let exclude_editor = text_editor::TextEditor::new(&state.exclude_editor)
                .on_action(Message::ConnectionFormExcludeAction)
                .height(Length::Fixed(130.0));

            let add_system_filters =
                Button::new(fonts::primary_text(tr("Add filter for system databases"), None))
                    .padding([6, 16])
                    .style(primary_button_style(palette.clone(), 6.0))
                    .on_press(Message::ConnectionFormAddSystemFilters);

            Column::new()
                .spacing(12)
                .push(fonts::primary_text(tr("Include"), None).color(text_color))
                .push(include_editor)
                .push(fonts::primary_text(tr("Exclude"), None).color(text_color))
                .push(exclude_editor)
                .push(add_system_filters)
                .into()
        }
    };

    let mut content = Column::new()
        .spacing(16)
        .push(fonts::primary_text(title, Some(10.0)).color(text_color))
        .push(tabs_row)
        .push(tab_content);

    if let Some(error) = &state.validation_error {
        content = content.push(fonts::primary_text(error.clone(), None).color(error_color));
    }

    if let Some(feedback) = &state.test_feedback {
        let color = if feedback.is_success() { accent_color } else { error_color };
        content = content.push(fonts::primary_text(feedback.message(), None).color(color));
    }

    if state.testing {
        content = content.push(fonts::primary_text(tr("Testing..."), None).color(accent_color));
    }

    let mut test_button = Button::new(fonts::primary_text(tr("Test"), None)).padding([6, 16]);
    if !state.testing {
        test_button = test_button
            .on_press(Message::ConnectionFormTest)
            .style(primary_button_style(palette.clone(), 6.0));
    } else {
        test_button = test_button.style(disabled_primary_button_style(palette.clone(), 6.0));
    }

    let cancel_button = Button::new(fonts::primary_text(tr("Cancel"), None))
        .padding([6, 16])
        .style(primary_button_style(palette.clone(), 6.0))
        .on_press(Message::ConnectionFormCancel);

    let mut save_button = Button::new(fonts::primary_text(tr("Save"), None)).padding([6, 16]);
    if !state.testing {
        save_button = save_button
            .on_press(Message::ConnectionFormSave)
            .style(primary_button_style(palette.clone(), 6.0));
    } else {
        save_button = save_button.style(disabled_primary_button_style(palette.clone(), 6.0));
    }

    let buttons = Row::new().spacing(12).push(cancel_button).push(test_button).push(save_button);
    content = content.push(buttons);

    let card_element: Element<Message> = content.into();
    modal_layout(palette, card_element, Length::Fixed(560.0), 16, 6.0)
}

fn connections_file_path() -> PathBuf {
    PathBuf::from(CONNECTIONS_FILE)
}
