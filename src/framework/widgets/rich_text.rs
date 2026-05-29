//! RichText widget — renders minimal Markdown with styling and word wrapping.
//!
//! Supports: `# Header`, `**bold**`, `*italic*`, `` `code` ``, `[link](url)`, `- list item`,
//! and plain paragraphs.

use crate::compositor::{Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId, WidgetState};
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthChar;

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Block {
    Header(u8, Vec<Inline>),
    Paragraph(Vec<Inline>),
    ListItem(Vec<Inline>),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Code(String),
    Link { text: String, url: String },
}

// ---------------------------------------------------------------------------
// Parser (~100 lines)
// ---------------------------------------------------------------------------

fn parse_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            blocks.push(Block::Header(1, parse_inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            blocks.push(Block::Header(2, parse_inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            blocks.push(Block::Header(3, parse_inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("#### ") {
            blocks.push(Block::Header(4, parse_inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("##### ") {
            blocks.push(Block::Header(5, parse_inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("###### ") {
            blocks.push(Block::Header(6, parse_inline(rest)));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            blocks.push(Block::ListItem(parse_inline(&trimmed[2..])));
        } else if trimmed.is_empty() {
            // skip empty lines
        } else {
            let mut para = trimmed.to_string();
            while let Some(next) = lines.peek() {
                if next.trim().is_empty() {
                    break;
                }
                para.push(' ');
                para.push_str(next.trim());
                lines.next();
            }
            blocks.push(Block::Paragraph(parse_inline(&para)));
        }
    }
    blocks
}

fn parse_inline(text: &str) -> Vec<Inline> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // bold: **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            i += 2;
            let start = i;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            result.push(Inline::Bold(parse_inline(&inner)));
            i += 2;
        }
        // italic: *text* or _text_
        else if chars[i] == '*' || chars[i] == '_' {
            let delim = chars[i];
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != delim {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            result.push(Inline::Italic(parse_inline(&inner)));
            i += 1;
        }
        // code: `text`
        else if chars[i] == '`' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            result.push(Inline::Code(inner));
            i += 1;
        }
        // link: [text](url)
        else if chars[i] == '[' {
            i += 1;
            let text_start = i;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            let link_text: String = chars[text_start..i].iter().collect();
            i += 1; // skip ']'
            if i < chars.len() && chars[i] == '(' {
                i += 1;
                let url_start = i;
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
                let url: String = chars[url_start..i].iter().collect();
                i += 1; // skip ')'
                result.push(Inline::Link {
                    text: link_text,
                    url,
                });
            } else {
                result.push(Inline::Text(format!("[{}]", link_text)));
            }
        }
        // plain text
        else {
            let start = i;
            while i < chars.len() && !['*', '_', '`', '['].contains(&chars[i]) {
                i += 1;
            }
            let inner: String = chars[start..i].iter().collect();
            if !inner.is_empty() {
                result.push(Inline::Text(inner));
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Render state
// ---------------------------------------------------------------------------

struct RenderState {
    x: u16,
    y: u16,
}

/// Word render params: (plane, word, width, height, fg, bg, style)
type WriteWordParams<'a> = (&'a mut Plane, &'a str, u16, u16, Color, Color, Styles);

/// Inline render params: (plane, inlines, theme, state, width, height, fg, bg, style)
type InlineRenderParams<'a> = (
    &'a mut Plane,
    &'a [Inline],
    &'a Theme,
    &'a mut RenderState,
    u16,
    u16,
    Color,
    Color,
    Styles,
);

impl RenderState {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    fn write_word(&mut self, params: WriteWordParams) -> bool {
        let (plane, word, width, height, fg, bg, style) = params;
        let word_width = word.chars().map(|c| c.width().unwrap_or(0)).sum::<usize>() as u16;

        if self.x + word_width > width && self.x > 0 && word_width <= width {
            self.x = 0;
            self.y += 1;
            if self.y >= height {
                return false;
            }
        }

        for c in word.chars() {
            let cw = c.width().unwrap_or(0) as u16;
            if cw == 0 {
                continue;
            }
            if self.x + cw > width {
                self.x = 0;
                self.y += 1;
                if self.y >= height {
                    return false;
                }
            }
            plane.set_style(self.x, self.y, fg, bg, style);
            plane.put_char(self.x, self.y, c);
            self.x += cw;
        }
        true
    }
}

fn render_inline(params: InlineRenderParams) {
    let (plane, inlines, theme, state, width, height, fg, bg, style) = params;
    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                for word in text.split_inclusive(' ') {
                    if !state.write_word((plane, word, width, height, fg, bg, style)) {
                        return;
                    }
                }
            }
            Inline::Bold(children) => {
                render_inline((
                    plane,
                    children,
                    theme,
                    state,
                    width,
                    height,
                    theme.fg,
                    bg,
                    style | Styles::BOLD,
                ));
            }
            Inline::Italic(children) => {
                render_inline((
                    plane,
                    children,
                    theme,
                    state,
                    width,
                    height,
                    theme.fg,
                    bg,
                    style | Styles::ITALIC,
                ));
            }
            Inline::Code(text) => {
                for word in text.split_inclusive(' ') {
                    if !state.write_word((
                        plane,
                        word,
                        width,
                        height,
                        theme.fg,
                        theme.secondary,
                        style,
                    )) {
                        return;
                    }
                }
            }
            Inline::Link { text, .. } => {
                for word in text.split_inclusive(' ') {
                    if !state.write_word((
                        plane,
                        word,
                        width,
                        height,
                        theme.info,
                        bg,
                        style | Styles::UNDERLINE,
                    )) {
                        return;
                    }
                }
            }
        }
    }
}

fn render_block(
    plane: &mut Plane,
    block: &Block,
    theme: &Theme,
    state: &mut RenderState,
    width: u16,
    height: u16,
) {
    if state.y >= height {
        return;
    }

    match block {
        Block::Header(level, inlines) => {
            let header_fg = theme.primary;
            let header_style = Styles::BOLD;
            let indent = (*level - 1).min(2) as u16;
            if state.x < indent {
                state.x = indent;
            }
            render_inline((
                plane,
                inlines,
                theme,
                state,
                width,
                height,
                header_fg,
                theme.bg,
                header_style,
            ));
            state.x = 0;
            state.y += 1;
            // blank line after header
            if state.y < height {
                state.y += 1;
            }
        }
        Block::Paragraph(inlines) => {
            render_inline((
                plane,
                inlines,
                theme,
                state,
                width,
                height,
                theme.fg,
                theme.bg,
                Styles::empty(),
            ));
            state.x = 0;
            state.y += 1;
        }
        Block::ListItem(inlines) => {
            let prefix = "- ";
            if state.x == 0 {
                for c in prefix.chars() {
                    if state.x >= width {
                        break;
                    }
                    plane.set_style(state.x, state.y, theme.fg, theme.bg, Styles::empty());
                    plane.put_char(state.x, state.y, c);
                    state.x += 1;
                }
            }
            render_inline((
                plane,
                inlines,
                theme,
                state,
                width,
                height,
                theme.fg,
                theme.bg,
                Styles::empty(),
            ));
            state.x = 0;
            state.y += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Widget
// ---------------------------------------------------------------------------

/// A widget that renders minimal Markdown with styling and word wrapping.
pub struct RichText {
    id: WidgetId,
    content: String,
    blocks: Vec<Block>,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl RichText {
    /// Creates a new RichText widget with the given markdown content.
    pub fn new(content: &str) -> Self {
        let blocks = parse_blocks(content);
        Self {
            id: WidgetId::next(),
            content: content.to_string(),
            blocks,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
        }
    }

    /// Creates a new RichText with the given widget ID and content.
    pub fn with_id(id: WidgetId, content: &str) -> Self {
        let mut s = Self::new(content);
        s.id = id;
        s
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Updates the markdown content and re-parses it.
    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.blocks = parse_blocks(content);
        self.dirty = true;
    }

    /// Returns the raw markdown content.
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Widget for RichText {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    fn focusable(&self) -> bool {
        false
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let mut state = RenderState::new();
        for block in &self.blocks {
            render_block(
                &mut plane,
                block,
                &self.theme,
                &mut state,
                area.width,
                area.height,
            );
            if state.y >= area.height {
                break;
            }
        }

        plane
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for RichText {
    fn state_id(&self) -> Option<&str> {
        None
    }
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    fn apply_json(&mut self, _json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        Ok(())
    }
}
