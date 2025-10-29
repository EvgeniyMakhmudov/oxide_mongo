use iced::widget::{Button, Column, Container, button};
use iced::{Color, Element, Length, Renderer, Shadow, Theme, Vector, border};
use iced_aw::{
    ContextMenu,
    menu::{Item as MenuItemWidget, Menu, MenuBar},
};

use crate::fonts;
use crate::i18n::tr;
use crate::settings::ThemePalette;
use crate::{ClientId, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopMenu {
    File,
    View,
    Windows,
    Help,
}

impl TopMenu {
    pub(crate) fn label(self) -> &'static str {
        match self {
            TopMenu::File => "File",
            TopMenu::View => "View",
            TopMenu::Windows => "Windows",
            TopMenu::Help => "Help",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuEntry {
    Action(&'static str),
}

impl MenuEntry {
    pub(crate) fn label(self) -> &'static str {
        match self {
            MenuEntry::Action(label) => label,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionContextAction {
    CreateDatabase,
    Refresh,
    ServerStatus,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CollectionContextAction {
    OpenEmptyTab,
    ViewDocuments,
    DeleteTemplate,
    DeleteAllDocuments,
    DeleteCollection,
    RenameCollection,
    Stats,
    Indexes,
    CreateIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DatabaseContextAction {
    Refresh,
    Stats,
    Drop,
}

pub(crate) fn build_menu_bar<'a>(palette: ThemePalette) -> MenuBar<'a, Message, Theme, Renderer> {
    let connections_palette = palette.clone();
    let connections_button = button(fonts::primary_text(tr("Connections"), Some(1.0)))
        .padding([6, 12])
        .on_press(Message::MenuItemSelected(TopMenu::File, MenuEntry::Action("Connections")))
        .style(move |_, status| connections_palette.menu_button_style(6.0, status));

    let settings_palette = palette.clone();
    let settings_button = button(fonts::primary_text(tr("Settings"), Some(1.0)))
        .padding([6, 12])
        .on_press(Message::SettingsOpen)
        .style(move |_, status| settings_palette.menu_button_style(6.0, status));

    let mut roots = Vec::new();
    roots.push(MenuItemWidget::new(connections_button));
    roots.push(MenuItemWidget::new(settings_button));
    roots.push(menu_root(
        &palette,
        TopMenu::View,
        &[MenuEntry::Action("Explorer"), MenuEntry::Action("Refresh")],
    ));
    roots.push(menu_root(
        &palette,
        TopMenu::Windows,
        &[MenuEntry::Action("Cascade"), MenuEntry::Action("Tile")],
    ));
    roots.push(menu_root(
        &palette,
        TopMenu::Help,
        &[MenuEntry::Action("Documentation"), MenuEntry::Action("About")],
    ));

    MenuBar::new(roots).width(Length::Fill)
}

fn menu_root<'a>(
    palette: &ThemePalette,
    menu: TopMenu,
    entries: &[MenuEntry],
) -> MenuItemWidget<'a, Message, Theme, Renderer> {
    let label = fonts::primary_text(tr(menu.label()), Some(1.0));
    let root_palette = palette.clone();
    let root_button = button(label)
        .padding([6, 12])
        .style(move |_, status| root_palette.menu_button_style(6.0, status));

    let menu_palette = palette.clone();
    let menu_widget = Menu::new(
        entries
            .iter()
            .map(move |entry| {
                let entry_label = fonts::primary_text(entry.label(), None);
                let entry_palette = menu_palette.clone();
                let entry_button = button(entry_label)
                    .on_press(Message::MenuItemSelected(menu, *entry))
                    .padding([6, 12])
                    .width(Length::Fill)
                    .style(move |_, status| entry_palette.menu_button_style(6.0, status));
                MenuItemWidget::new(entry_button)
            })
            .collect(),
    )
    .offset(4.0)
    .max_width(180.0);

    MenuItemWidget::with_menu(root_button, menu_widget)
}

pub(crate) fn connection_context_menu<'a>(
    base_button: Element<'a, Message>,
    palette: ThemePalette,
    client_id: ClientId,
    is_ready: bool,
) -> Element<'a, Message> {
    ContextMenu::new(base_button, move || {
        let mut menu = Column::new().spacing(4).padding([4, 6]);

        let make_button = |label: &str, action: ConnectionContextAction, enabled: bool| {
            let mut button =
                Button::new(fonts::primary_text(label.to_owned(), None)).padding([4, 8]);
            if enabled {
                button = button.on_press(Message::ConnectionContextMenu { client_id, action });
            }
            let item_palette = palette.clone();
            let styled_button =
                button.style(move |_, status| item_palette.menu_button_style(6.0, status));
            apply_item_container(styled_button.into(), palette.clone())
        };

        menu = menu.push(make_button(
            tr("Create Database"),
            ConnectionContextAction::CreateDatabase,
            is_ready,
        ));
        menu = menu.push(make_button(tr("Refresh"), ConnectionContextAction::Refresh, is_ready));
        menu = menu.push(make_button(
            tr("Server Status"),
            ConnectionContextAction::ServerStatus,
            is_ready,
        ));
        menu = menu.push(make_button(tr("Close"), ConnectionContextAction::Close, true));

        menu.into()
    })
    .into()
}

pub(crate) fn database_context_menu<'a>(
    base_button: Element<'a, Message>,
    palette: ThemePalette,
    client_id: ClientId,
    db_name: String,
) -> Element<'a, Message> {
    ContextMenu::new(base_button, move || {
        let mut menu = Column::new().spacing(4).padding([4, 6]);

        let make_button = |label: &str, action: DatabaseContextAction| {
            let item_palette = palette.clone();
            let button = Button::new(fonts::primary_text(label.to_owned(), None))
                .padding([4, 8])
                .on_press(Message::DatabaseContextMenu {
                    client_id,
                    db_name: db_name.clone(),
                    action,
                })
                .style(move |_, status| item_palette.menu_button_style(6.0, status));
            apply_item_container(button.into(), palette.clone())
        };

        menu = menu.push(make_button(tr("Refresh"), DatabaseContextAction::Refresh));
        menu = menu.push(make_button(tr("Statistics"), DatabaseContextAction::Stats));
        menu = menu.push(make_button(tr("Drop Database"), DatabaseContextAction::Drop));

        menu.into()
    })
    .into()
}

pub(crate) fn collection_context_menu<'a>(
    base_button: Element<'a, Message>,
    palette: ThemePalette,
    client_id: ClientId,
    db_name: String,
    collection_name: String,
) -> Element<'a, Message> {
    ContextMenu::new(base_button, move || {
        let mut menu = Column::new().spacing(4).padding([4, 6]);

        let make_button = |label: &str, action: CollectionContextAction| {
            let item_palette = palette.clone();
            let button = Button::new(fonts::primary_text(label.to_owned(), None))
                .padding([4, 8])
                .on_press(Message::CollectionContextMenu {
                    client_id,
                    db_name: db_name.clone(),
                    collection: collection_name.clone(),
                    action,
                })
                .style(move |_, status| item_palette.menu_button_style(6.0, status));
            apply_item_container(button.into(), palette.clone())
        };

        menu = menu.push(make_button(tr("Open Empty Tab"), CollectionContextAction::OpenEmptyTab));
        menu = menu.push(make_button(tr("View Documents"), CollectionContextAction::ViewDocuments));
        menu = menu
            .push(make_button(tr("Delete Documents..."), CollectionContextAction::DeleteTemplate));
        menu = menu.push(make_button(
            tr("Delete All Documents..."),
            CollectionContextAction::DeleteAllDocuments,
        ));
        menu = menu.push(make_button(
            tr("Rename Collection..."),
            CollectionContextAction::RenameCollection,
        ));
        menu = menu
            .push(make_button(tr("Drop Collection..."), CollectionContextAction::DeleteCollection));
        menu = menu.push(make_button(tr("Statistics"), CollectionContextAction::Stats));
        menu = menu.push(make_button(tr("Create Index"), CollectionContextAction::CreateIndex));
        menu = menu.push(make_button(tr("Indexes"), CollectionContextAction::Indexes));

        menu.into()
    })
    .into()
}

fn apply_item_container(
    content: Element<'_, Message>,
    palette: ThemePalette,
) -> Element<'_, Message> {
    let background = palette.menu.background.to_color();
    let luminance = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
    let shadow_color = if luminance > 0.5 {
        Color::from_rgba(0.0, 0.0, 0.0, 0.75)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.3)
    };

    Container::new(content)
        .style(move |_| iced::widget::container::Style {
            background: Some(background.into()),
            border: border::rounded(6.0).width(1).color(palette.widget_border_color()),
            shadow: Shadow {
                color: shadow_color,
                offset: Vector::new(0.0, 3.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        })
        .into()
}
