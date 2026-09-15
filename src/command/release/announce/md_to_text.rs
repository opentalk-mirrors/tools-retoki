// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use pulldown_cmark::{Alignment, Event, Options, Parser, Tag, TagEnd};
use tera::{Kwargs, State};
use unicode_width::UnicodeWidthStr;

/// Tera filter that converts Markdown to plain text while retaining links.
pub fn md_to_text_filter(value: &str, _kwargs: Kwargs, _state: &State) -> String {
    md_to_text(value)
}

/// Render Markdown as plain text suitable for plain text emails.
///
/// Links are emitted as `text <url>`, GFM tables are rendered as aligned
/// tables and footnote definitions are collected and appended at the end.
pub fn md_to_text(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    Renderer::default().run(Parser::new_ext(markdown, options))
}

#[derive(Default)]
struct Renderer {
    /// Stack of active sinks; the bottom is the document body, inner entries
    /// capture inline content for links, table cells and footnote definitions.
    stack: Vec<String>,
    footnotes: Vec<String>,
    footnote_labels: Vec<String>,
    /// URL of the link currently being captured, if any.
    link_urls: Vec<String>,
    table: Option<Table>,
    in_table_cell: bool,
    /// Stack of list markers: `Some(counter)` for ordered lists, `None` for bullet lists.
    lists: Vec<Option<u64>>,
}

#[derive(Default)]
struct Table {
    alignments: Vec<Alignment>,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    in_header: bool,
}

impl Renderer {
    fn run(mut self, parser: Parser) -> String {
        self.stack.push(String::new());
        for event in parser {
            self.event(event);
        }
        let mut body = self.stack.pop().unwrap_or_default();
        if !self.footnotes.is_empty() {
            let body_trimmed = body.trim_end();
            body = format!("{body_trimmed}\n\n{}", self.footnotes.join("\n"));
        }
        body.trim().to_string()
    }

    fn write(&mut self, text: &str) {
        if let Some(buffer) = self.stack.last_mut() {
            buffer.push_str(text);
        }
    }

    fn ends_with_newline(&self) -> bool {
        self.stack
            .last()
            .map(|buffer| buffer.is_empty() || buffer.ends_with('\n'))
            .unwrap_or(true)
    }

    fn ensure_newline(&mut self) {
        if !self.ends_with_newline() {
            self.write("\n");
        }
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) | Event::Code(text) => self.write(&text),
            Event::SoftBreak => self.write(if self.in_table_cell { " " } else { "\n" }),
            Event::HardBreak => self.write("\n"),
            Event::Rule => self.write(&format!("\n{}\n\n", "-".repeat(70))),
            Event::TaskListMarker(checked) => self.write(if checked { "[x] " } else { "[ ] " }),
            Event::FootnoteReference(label) => self.write(&format!("[^{label}]")),
            Event::Html(html) | Event::InlineHtml(html) => {
                tracing::warn!(
                    "HTML to TXT is unsupported! Check the output for plain HTML content."
                );
                self.write(&html)
            }
            Event::InlineMath(text) | Event::DisplayMath(text) => self.write(&text),
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { .. } | Tag::Paragraph | Tag::CodeBlock(_) => self.ensure_newline(),
            Tag::List(start) => {
                self.ensure_newline();
                self.lists.push(start);
            }
            Tag::Item => {
                self.ensure_newline();
                let depth = self.lists.len().saturating_sub(1);
                self.write(&"  ".repeat(depth));
                match self.lists.last_mut() {
                    Some(Some(counter)) => {
                        let marker = format!("{counter}. ");
                        *counter += 1;
                        self.write(&marker);
                    }
                    _ => self.write("- "),
                }
            }
            Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. } => {
                self.link_urls.push(dest_url.to_string());
                self.stack.push(String::new());
            }
            Tag::FootnoteDefinition(label) => {
                self.footnote_labels.push(label.to_string());
                self.stack.push(String::new());
            }
            Tag::Table(alignments) => {
                self.table = Some(Table {
                    alignments,
                    ..Table::default()
                });
            }
            Tag::TableHead => {
                if let Some(table) = self.table.as_mut() {
                    table.in_header = true;
                }
            }
            Tag::TableCell => {
                self.in_table_cell = true;
                self.stack.push(String::new());
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) => self.write("\n\n"),
            TagEnd::CodeBlock => {
                self.ensure_newline();
                self.write("\n");
            }
            TagEnd::Item => self.ensure_newline(),
            TagEnd::List(_) => {
                self.lists.pop();
                if self.lists.is_empty() {
                    self.write("\n");
                }
            }
            TagEnd::Link | TagEnd::Image => {
                let text = self.stack.pop().unwrap_or_default();
                let url = self.link_urls.pop().unwrap_or_default();
                let text = text.trim();
                if text.is_empty() || text == url {
                    self.write(&format!("<{url}>"));
                } else {
                    self.write(&format!("{text} <{url}>"));
                }
            }
            TagEnd::FootnoteDefinition => {
                let definition = self.stack.pop().unwrap_or_default();
                let label = self.footnote_labels.pop().unwrap_or_default();
                self.footnotes
                    .push(format!("[^{label}]: {}", definition.trim()));
            }
            TagEnd::TableCell => {
                self.in_table_cell = false;
                let cell = self.stack.pop().unwrap_or_default().trim().to_string();
                if let Some(table) = self.table.as_mut() {
                    table.current_row.push(cell);
                }
            }
            TagEnd::TableHead => {
                if let Some(table) = self.table.as_mut() {
                    table.header = std::mem::take(&mut table.current_row);
                    table.in_header = false;
                }
            }
            TagEnd::TableRow => {
                if let Some(table) = self.table.as_mut() {
                    let row = std::mem::take(&mut table.current_row);
                    table.rows.push(row);
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    let rendered = render_table(&table);
                    self.ensure_newline();
                    self.write(&rendered);
                    self.write("\n");
                }
            }
            _ => {}
        }
    }
}

fn cell(row: &[String], column: usize) -> &str {
    row.get(column).map(String::as_str).unwrap_or("")
}

fn render_table(table: &Table) -> String {
    let column_count = std::iter::once(table.header.len())
        .chain(table.rows.iter().map(Vec::len))
        .max()
        .unwrap_or(0);
    if column_count == 0 {
        return String::new();
    }

    let widths: Vec<usize> = (0..column_count)
        .map(|column| {
            let header_width = table.header.get(column).map(|s| s.width()).unwrap_or(0);
            let body_width = table
                .rows
                .iter()
                .map(|row| cell(row, column).width())
                .max()
                .unwrap_or(0);
            header_width.max(body_width).max(3)
        })
        .collect();

    let alignment = |column: usize| {
        table
            .alignments
            .get(column)
            .copied()
            .unwrap_or(Alignment::None)
    };

    let mut lines = Vec::new();
    lines.push(format_row(&table.header, &widths, column_count, alignment));
    lines.push(format_separator(&widths));
    for row in &table.rows {
        lines.push(format_row(row, &widths, column_count, alignment));
    }
    lines.join("\n")
}

fn format_row(
    row: &[String],
    widths: &[usize],
    column_count: usize,
    alignment: impl Fn(usize) -> Alignment,
) -> String {
    let cells: Vec<String> = (0..column_count)
        .map(|column| {
            let content = row.get(column).map(String::as_str).unwrap_or("");
            pad_cell(content, widths[column], alignment(column))
        })
        .collect();
    cells.join("  ").trim_end().to_string()
}

fn format_separator(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|&width| "-".repeat(width))
        .collect::<Vec<_>>()
        .join("  ")
}

fn pad_cell(content: &str, width: usize, alignment: Alignment) -> String {
    let padding = width.saturating_sub(content.width());
    match alignment {
        Alignment::Right => format!("{}{content}", " ".repeat(padding)),
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{content}{}", " ".repeat(left), " ".repeat(right))
        }
        Alignment::Left | Alignment::None => format!("{content}{}", " ".repeat(padding)),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::md_to_text;

    #[test]
    fn links_are_retained_in_angle_brackets() {
        let input = "See the [release notes](https://example.com/notes) for details.";
        assert_snapshot!(md_to_text(input), @"See the release notes <https://example.com/notes> for details.");
    }

    #[test]
    fn bare_link_is_not_duplicated() {
        let input = "<https://example.com>";
        assert_snapshot!(md_to_text(input), @"<https://example.com>");
    }

    #[test]
    fn emphasis_and_code_markers_are_dropped() {
        let input = "A **bold** and *italic* value like `25.2.0`.";
        assert_snapshot!(md_to_text(input), @"A bold and italic value like 25.2.0.");
    }

    #[test]
    fn headings_and_lists_render_as_plain_text() {
        let input = "### Changes\n\n- first item\n- second [item](https://example.com)\n";
        assert_snapshot!(md_to_text(input), @"
        Changes

        - first item
        - second item <https://example.com>
        ");
    }

    #[test]
    fn tables_and_footnotes_render_as_plain_text() {
        let input = r#"### Browser compatibility

Starting with `25.2.0` we plan to maintain a browser compatibility
matrix. This is a draft.

| Browser | System  | Version   | Camera | Audio          |
| ------- | ------- | --------- | ------ | -------------- |
| Chrome  | Desktop | 136       | y      | y              |
| Safari  | MacOS   | 18.5      | y      | warn[^1]       |
| Safari  | iOS     | 18.5      | y      | none[^2]       |

[^1]: no track update on unfocused tabs ('mute' participants)
[^2]: mobile layout is flawed
"#;
        assert_snapshot!(md_to_text(input), @"
        Browser compatibility

        Starting with 25.2.0 we plan to maintain a browser compatibility
        matrix. This is a draft.

        Browser  System   Version  Camera  Audio
        -------  -------  -------  ------  --------
        Chrome   Desktop  136      y       y
        Safari   MacOS    18.5     y       warn[^1]
        Safari   iOS      18.5     y       none[^2]

        [^1]: no track update on unfocused tabs ('mute' participants)
        [^2]: mobile layout is flawed
        ");
    }

    #[test]
    fn emoji_table_is_rendered_with_footnotes_and_links() {
        let input = r#"### Browser compatibility

| Browser | System  | Camera | Screen Share | Audio   | Docs                          |
| ------- | ------- | ------ | ------------ | ------- | ----------------------------- |
| Chrome  | Desktop | ✔️     | ✔️ | ✔️      | [chrome](https://example.com) |
| Chrome  | Android | ✔️     | ❌ | ✔️      | [android](https://example.com/a) |
| Safari  | MacOS   | ✔️     | ✔️ | ⚠️[^1]  |                               |
| Safari  | iOS     | ✔️     | ❌ | ❔      |                               |

[^1]: no track update on unfocused tabs ('mute' participants)
"#;
        assert_snapshot!(md_to_text(input), @"
        Browser compatibility

        Browser  System   Camera  Screen Share  Audio   Docs
        -------  -------  ------  ------------  ------  -------------------------------
        Chrome   Desktop  ✔️      ✔️            ✔️      chrome <https://example.com>
        Chrome   Android  ✔️      ❌            ✔️      android <https://example.com/a>
        Safari   MacOS    ✔️      ✔️            ⚠️[^1]
        Safari   iOS      ✔️      ❌            ❔

        [^1]: no track update on unfocused tabs ('mute' participants)
        ");
    }
}
