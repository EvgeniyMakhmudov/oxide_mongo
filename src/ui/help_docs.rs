use iced::alignment::Vertical;
use iced::font::Weight;
use iced::widget::text::Wrapping;
use iced::widget::{Button, Column, Container, Row, Scrollable, Space, text_input};
use iced::{Element, Font, Length, Shadow, border};

use crate::Message;
use crate::fonts;
use crate::i18n::{docs, tr};
use crate::settings::ThemePalette;
use crate::ui::modal::modal_layout;

pub struct HelpDocsState {
    pub selected_index: usize,
    pub search: String,
}

impl HelpDocsState {
    pub fn new() -> Self {
        Self { selected_index: 0, search: String::new() }
    }
}

pub fn help_docs_view<'a>(palette: ThemePalette, state: &'a HelpDocsState) -> Element<'a, Message> {
    let text_primary = palette.text_primary.to_color();
    let muted = palette.text_muted.to_color();
    let title = fonts::primary_text(tr("Documentation"), Some(6.0)).color(text_primary);
    let close_button = Button::new(fonts::primary_text(tr("Close"), None))
        .padding([6, 16])
        .on_press(Message::HelpDocsClose)
        .style({
            let palette = palette.clone();
            move |_, status| palette.subtle_button_style(6.0, status)
        });

    let header = Row::new()
        .align_y(Vertical::Center)
        .push(title)
        .push(Space::with_width(Length::Fill))
        .push(close_button);

    let nav_column = build_nav_column(&palette, state.selected_index);
    let nav_scroll = Scrollable::new(nav_column).height(Length::Fill);
    let nav_container =
        Container::new(nav_scroll).padding([6, 8]).width(Length::Fixed(220.0)).style({
            let palette = palette.clone();
            move |_| {
                let border_color = palette.widget_border_color();
                iced::widget::container::Style {
                    background: Some(palette.widget_background_color().into()),
                    border: border::rounded(6.0).width(1).color(border_color),
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            }
        });

    let slugs = docs::doc_section_order();
    let selected_slug = slugs
        .get(state.selected_index)
        .copied()
        .or_else(|| slugs.first().copied())
        .unwrap_or("general");
    let markdown = docs::doc_section(selected_slug).map(|section| section.markdown).unwrap_or("");
    let blocks = parse_markdown(markdown);
    let search_query = state.search.trim();
    let matches = count_matches(&blocks, search_query);

    let search_input = text_input(tr("Search"), &state.search)
        .padding([4, 6])
        .on_input(Message::HelpDocsSearchChanged)
        .width(Length::Fill);
    let matches_label =
        fonts::primary_text(format!("{}: {}", tr("Matches"), matches), Some(-1.0)).color(muted);
    let search_row =
        Row::new().spacing(8).align_y(Vertical::Center).push(search_input).push(matches_label);

    let content_column = build_markdown_column(&palette, &blocks, search_query);
    let content_scroll = Scrollable::new(content_column).width(Length::Fill).height(Length::Fill);

    let content =
        Row::new().spacing(16).height(Length::Fixed(520.0)).push(nav_container).push(
            Column::new().spacing(12).width(Length::Fill).push(search_row).push(content_scroll),
        );

    let layout: Element<Message> = Column::new().spacing(16).push(header).push(content).into();
    modal_layout(palette, layout, Length::Fixed(860.0), 20, 12.0)
}

fn build_nav_column(palette: &ThemePalette, selected_index: usize) -> Column<'static, Message> {
    let mut column = Column::new().spacing(6);
    let slugs = docs::doc_section_order();
    for (index, slug) in slugs.iter().enumerate() {
        let selected = index == selected_index;
        let label_color = if selected {
            palette.primary_buttons.text.to_color()
        } else {
            palette.text_primary.to_color()
        };
        let title = docs::doc_section(slug).map(|section| section.title).unwrap_or(*slug);
        let label = fonts::primary_text(title, Some(-1.0)).color(label_color);
        let palette = palette.clone();
        let button = Button::new(label)
            .padding([6, 10])
            .width(Length::Fill)
            .on_press(Message::HelpDocsSectionSelected(index))
            .style(move |_, status| {
                if selected {
                    palette.primary_button_style(6.0, status)
                } else {
                    palette.subtle_button_style(6.0, status)
                }
            });
        column = column.push(button);
    }
    column
}

#[derive(Debug)]
enum MarkdownBlock {
    Heading1(String),
    Heading2(String),
    Bullet(String),
    CodeBlock(String),
    Paragraph(String),
}

fn parse_markdown(source: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut paragraph = String::new();
    let mut code_lines: Vec<String> = Vec::new();

    let flush_paragraph = |paragraph: &mut String, blocks: &mut Vec<MarkdownBlock>| {
        let text = paragraph.trim();
        if !text.is_empty() {
            blocks.push(MarkdownBlock::Paragraph(text.to_string()));
            paragraph.clear();
        }
    };

    let flush_code_block = |code_lines: &mut Vec<String>, blocks: &mut Vec<MarkdownBlock>| {
        if !code_lines.is_empty() {
            let joined = code_lines.join("\n");
            blocks.push(MarkdownBlock::CodeBlock(joined));
            code_lines.clear();
        }
    };

    for line in source.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            flush_paragraph(&mut paragraph, &mut blocks);
            flush_code_block(&mut code_lines, &mut blocks);
            continue;
        }

        if let Some(code_line) = trimmed.strip_prefix("    ").or_else(|| trimmed.strip_prefix('\t'))
        {
            flush_paragraph(&mut paragraph, &mut blocks);
            code_lines.push(code_line.to_string());
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("# ") {
            flush_paragraph(&mut paragraph, &mut blocks);
            flush_code_block(&mut code_lines, &mut blocks);
            blocks.push(MarkdownBlock::Heading1(rest.trim().to_string()));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("## ") {
            flush_paragraph(&mut paragraph, &mut blocks);
            flush_code_block(&mut code_lines, &mut blocks);
            blocks.push(MarkdownBlock::Heading2(rest.trim().to_string()));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("- ") {
            flush_paragraph(&mut paragraph, &mut blocks);
            flush_code_block(&mut code_lines, &mut blocks);
            blocks.push(MarkdownBlock::Bullet(rest.trim().to_string()));
            continue;
        }

        flush_code_block(&mut code_lines, &mut blocks);
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed.trim());
    }

    flush_paragraph(&mut paragraph, &mut blocks);
    flush_code_block(&mut code_lines, &mut blocks);
    blocks
}

fn build_markdown_column(
    palette: &ThemePalette,
    blocks: &[MarkdownBlock],
    search_query: &str,
) -> Column<'static, Message> {
    let text_primary = palette.text_primary.to_color();
    let highlight = palette.primary_buttons.active.to_color();
    let fonts_state = fonts::active_fonts();
    let bold_font = Font { weight: Weight::Bold, ..fonts_state.primary_font };
    let mut column = Column::new().spacing(8);

    for block in blocks {
        let (text, kind) = match block {
            MarkdownBlock::Heading1(text) => (text, "h1"),
            MarkdownBlock::Heading2(text) => (text, "h2"),
            MarkdownBlock::Bullet(text) => (text, "bullet"),
            MarkdownBlock::CodeBlock(text) => (text, "code"),
            MarkdownBlock::Paragraph(text) => (text, "p"),
        };
        let is_match = matches_query(text, search_query);
        let color = if search_query.is_empty() || !is_match { text_primary } else { highlight };

        let element: Element<Message> = match kind {
            "h1" => {
                fonts::primary_text(text.clone(), Some(6.0)).font(bold_font).color(color).into()
            }
            "h2" => {
                fonts::primary_text(text.clone(), Some(4.0)).font(bold_font).color(color).into()
            }
            "bullet" => Row::new()
                .spacing(6)
                .push(fonts::primary_text("-".to_string(), None).color(color))
                .push(
                    fonts::primary_text(text.clone(), None)
                        .color(color)
                        .wrapping(Wrapping::Word)
                        .width(Length::Fill),
                )
                .into(),
            "code" => Row::new()
                .push(Space::with_width(Length::Fixed(12.0)))
                .push(
                    fonts::result_text(text.clone(), Some(-1.0))
                        .color(color)
                        .wrapping(Wrapping::Word)
                        .width(Length::Fill),
                )
                .into(),
            _ => fonts::primary_text(text.clone(), None)
                .color(color)
                .wrapping(Wrapping::Word)
                .width(Length::Fill)
                .into(),
        };

        column = column.push(element);
    }

    column
}

fn matches_query(text: &str, query: &str) -> bool {
    if query.trim().is_empty() {
        return false;
    }
    text.to_lowercase().contains(&query.to_lowercase())
}

fn count_matches(blocks: &[MarkdownBlock], query: &str) -> usize {
    if query.trim().is_empty() {
        return 0;
    }
    blocks
        .iter()
        .filter(|block| match block {
            MarkdownBlock::Heading1(text)
            | MarkdownBlock::Heading2(text)
            | MarkdownBlock::Bullet(text)
            | MarkdownBlock::CodeBlock(text)
            | MarkdownBlock::Paragraph(text) => matches_query(text, query),
        })
        .count()
}
