use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use iced::alignment::{Horizontal, Vertical};
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
                fonts::primary_text(format!("{}:{}", entry.host, entry.port), Some(-1.0))
                    .color(muted_text);

            let labels = Column::new().spacing(4).push(name_text).push(details_text);

            let filters_text = if entry.include_filter.trim().is_empty()
                && entry.exclude_filter.trim().is_empty()
            {
                fonts::primary_text(tr("No filters configured"), Some(-2.0)).color(muted_text)
            } else {
                fonts::primary_text(tr("Collection filters configured"), Some(-2.0))
                    .color(accent_text)
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
                .push(fonts::primary_text(tr("Name"), None).color(text_color))
                .push(name_input)
                .push(fonts::primary_text(tr("Address/Host/IP"), None).color(text_color))
                .push(host_input)
                .push(fonts::primary_text(tr("Port"), None).color(text_color))
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
