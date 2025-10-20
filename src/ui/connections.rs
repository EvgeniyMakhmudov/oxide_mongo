use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::text_editor::{self, Action as TextEditorAction, Content as TextEditorContent};
use iced::widget::{
    self, Button, Column, Container, Image, Row, Scrollable, Space, Text, button, text_input,
};
use iced::{Color, Element, Length, Shadow, Theme, border};
use serde::{Deserialize, Serialize};

use crate::i18n::tr;
use crate::{
    DOUBLE_CLICK_INTERVAL, ICON_NETWORK_BYTES, ICON_NETWORK_HANDLE, Message, shared_icon_handle,
};

const CONNECTIONS_FILE: &str = "connections.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub include_filter: String,
    pub exclude_filter: String,
}

impl ConnectionEntry {
    pub fn uri(&self) -> String {
        format!("mongodb://{}:{}", self.host.trim(), self.port)
    }
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
    Filter,
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
    pub(crate) include_editor: TextEditorContent,
    pub(crate) exclude_editor: TextEditorContent,
    pub(crate) validation_error: Option<String>,
    pub(crate) test_feedback: Option<TestFeedback>,
    pub(crate) testing: bool,
}

impl ConnectionFormState {
    pub fn new(mode: ConnectionFormMode, entry: Option<&ConnectionEntry>) -> Self {
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
                    String::from(tr("localhost")),
                    String::from(tr("27017")),
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

    pub fn validate(&self) -> Result<ConnectionEntry, String> {
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

        Ok(ConnectionEntry {
            name: name.to_string(),
            host: host.to_string(),
            port,
            include_filter: self.include_editor.text(),
            exclude_filter: self.exclude_editor.text(),
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

pub fn connections_view<'a>(
    state: &'a ConnectionsWindowState,
    connections: &'a [ConnectionEntry],
) -> Element<'a, Message> {
    let border_color = Color::from_rgb8(0xba, 0xc5, 0xd6);
    let selected_bg = Color::from_rgb8(0xe9, 0xf0, 0xfa);
    let normal_bg = Color::from_rgb8(0xfc, 0xfd, 0xfe);
    let accent_bar = Color::from_rgb8(0x41, 0x82, 0xf2);

    let mut entries = Column::new().spacing(4).width(Length::Fill);

    if connections.is_empty() {
        entries = entries.push(
            Container::new(Text::new(tr("No saved connections")).size(16))
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

            let name_text =
                Text::new(entry.name.clone()).size(18).color(Color::from_rgb8(0x17, 0x1a, 0x20));
            let details_text = Text::new(format!("{}:{}", entry.host, entry.port))
                .size(13)
                .color(Color::from_rgb8(0x2f, 0x3b, 0x4b));

            let labels = Column::new().spacing(4).push(name_text).push(details_text);

            let filters_text = if entry.include_filter.trim().is_empty()
                && entry.exclude_filter.trim().is_empty()
            {
                Text::new(tr("No filters configured"))
                    .size(12)
                    .color(Color::from_rgb8(0x8a, 0x95, 0xa5))
            } else {
                Text::new(tr("Collection filters configured"))
                    .size(12)
                    .color(Color::from_rgb8(0x36, 0x71, 0xc9))
            };

            let right_info = Column::new().spacing(4).align_x(Horizontal::Right).push(filters_text);

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
        Button::new(Text::new(tr("Create"))).padding([6, 16]).on_press(Message::ConnectionsCreate),
    );

    let mut edit_button = Button::new(Text::new(tr("Edit"))).padding([6, 16]);
    if state.selected.is_some() {
        edit_button = edit_button.on_press(Message::ConnectionsEdit);
    }
    left_controls = left_controls.push(edit_button);

    let mut delete_button = Button::new(Text::new(tr("Delete"))).padding([6, 16]);
    if state.selected.is_some() {
        delete_button = delete_button.on_press(Message::ConnectionsDelete);
    }
    left_controls = left_controls.push(delete_button);

    let mut connect_button = Button::new(Text::new(tr("Connect"))).padding([6, 16]);
    if state.selected.is_some() {
        connect_button = connect_button.on_press(Message::ConnectionsConnect);
    }

    let right_controls = Row::new()
        .spacing(8)
        .push(
            Button::new(Text::new(tr("Cancel")))
                .padding([6, 16])
                .on_press(Message::ConnectionsCancel),
        )
        .push(connect_button);

    let mut content =
        Column::new().spacing(16).push(Text::new(tr("Connections")).size(24)).push(list);

    if let Some(feedback) = &state.feedback {
        let color = if feedback.starts_with(tr("Save error: ")) {
            Color::from_rgb8(0xd9, 0x53, 0x4f)
        } else {
            Color::from_rgb8(0x1e, 0x88, 0x3a)
        };
        content = content.push(Text::new(feedback.clone()).size(14).color(color));
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
            .push(Text::new(format!("{} \"{}\"?", tr("Delete"), name)).size(14))
            .push(
                Button::new(Text::new(tr("Yes")))
                    .padding([4, 12])
                    .on_press(Message::ConnectionsDeleteConfirmed),
            )
            .push(
                Button::new(Text::new(tr("No")))
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

    let card = Container::new(content).padding(20).width(Length::Fixed(640.0)).style(pane_style);

    Container::new(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| widget::container::Style {
            background: Some(Color::from_rgba8(0x16, 0x1a, 0x1f, 0.55).into()),
            ..Default::default()
        })
        .into()
}

pub fn connection_form_view<'a>(state: &'a ConnectionFormState) -> Element<'a, Message> {
    let title = match state.mode {
        ConnectionFormMode::Create => tr("New connection"),
        ConnectionFormMode::Edit(_) => tr("Edit connection"),
    };

    let bg_active = Color::from_rgb8(0xd6, 0xe8, 0xff);
    let bg_inactive = Color::from_rgb8(0xf6, 0xf7, 0xfa);
    let border_color = Color::from_rgb8(0xc2, 0xc8, 0xd3);

    let general_active = state.active_tab == ConnectionFormTab::General;
    let mut general_button =
        Button::new(Text::new(tr("General")).size(14)).padding([6, 16]).style(move |_, _| {
            button::Style {
                background: Some((if general_active { bg_active } else { bg_inactive }).into()),
                text_color: Color::BLACK,
                border: border::rounded(6).width(1).color(border_color),
                shadow: Shadow::default(),
            }
        });
    if !general_active {
        general_button =
            general_button.on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::General));
    }

    let filter_active = state.active_tab == ConnectionFormTab::Filter;
    let mut filter_button = Button::new(Text::new(tr("Database filter")).size(14))
        .padding([6, 16])
        .style(move |_, _| button::Style {
            background: Some((if filter_active { bg_active } else { bg_inactive }).into()),
            text_color: Color::BLACK,
            border: border::rounded(6).width(1).color(border_color),
            shadow: Shadow::default(),
        });
    if !filter_active {
        filter_button =
            filter_button.on_press(Message::ConnectionFormTabChanged(ConnectionFormTab::Filter));
    }

    let tabs_row = Row::new().spacing(8).push(general_button).push(filter_button);

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

            Column::new()
                .spacing(12)
                .push(Text::new(tr("Name")).size(14))
                .push(name_input)
                .push(Text::new(tr("Address/Host/IP")).size(14))
                .push(host_input)
                .push(Text::new(tr("Port")).size(14))
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
                Button::new(Text::new(tr("Add filter for system databases")).size(14))
                    .padding([6, 16])
                    .on_press(Message::ConnectionFormAddSystemFilters);

            Column::new()
                .spacing(12)
                .push(Text::new(tr("Include")).size(14))
                .push(include_editor)
                .push(Text::new(tr("Exclude")).size(14))
                .push(exclude_editor)
                .push(add_system_filters)
                .into()
        }
    };

    let mut content =
        Column::new().spacing(16).push(Text::new(title).size(24)).push(tabs_row).push(tab_content);

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
        content = content
            .push(Text::new(tr("Testing...")).size(14).color(Color::from_rgb8(0x1e, 0x88, 0x3a)));
    }

    let mut test_button = Button::new(Text::new(tr("Test"))).padding([6, 16]);
    if !state.testing {
        test_button = test_button.on_press(Message::ConnectionFormTest);
    } else {
        test_button = test_button.style(|_, _| button::Style {
            background: Some(Color::from_rgb8(0xe3, 0xe6, 0xeb).into()),
            text_color: Color::from_rgb8(0x8a, 0x93, 0xa3),
            border: border::rounded(6).width(1).color(Color::from_rgb8(0xd7, 0xdb, 0xe2)),
            shadow: Shadow::default(),
        });
    }

    let cancel_button = Button::new(Text::new(tr("Cancel")))
        .padding([6, 16])
        .on_press(Message::ConnectionFormCancel);

    let mut save_button = Button::new(Text::new(tr("Save"))).padding([6, 16]);
    if !state.testing {
        save_button = save_button.on_press(Message::ConnectionFormSave);
    } else {
        save_button = save_button.style(|_, _| button::Style {
            background: Some(Color::from_rgb8(0xe3, 0xe6, 0xeb).into()),
            text_color: Color::from_rgb8(0x8a, 0x93, 0xa3),
            border: border::rounded(6).width(1).color(Color::from_rgb8(0xd7, 0xdb, 0xe2)),
            shadow: Shadow::default(),
        });
    }

    let buttons = Row::new().spacing(12).push(cancel_button).push(test_button).push(save_button);
    content = content.push(buttons);

    let card = Container::new(content).padding(16).width(Length::Fixed(560.0)).style(pane_style);

    Container::new(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| widget::container::Style {
            background: Some(Color::from_rgba8(0x16, 0x1a, 0x1f, 0.55).into()),
            ..Default::default()
        })
        .into()
}

fn pane_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();

    widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        border: border::rounded(6).width(1).color(palette.primary.weak.color),
        ..Default::default()
    }
}

fn connections_file_path() -> PathBuf {
    PathBuf::from(CONNECTIONS_FILE)
}
