use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

/// A panel widget with a title and customizable border color.
pub struct Panel {
    /// Title displayed at the top of the panel.
    title: String,
    /// Color of the panel border.
    border_color: Color,
}

impl Panel {
    /// Creates a new Panel with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            border_color: Color::Cyan,
        }
    }

    /// Sets the border color of the panel.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Returns the inner area of the panel (excluding borders).
    pub fn inner(&self, area: Rect) -> Rect {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(self.title.as_str());
        block.inner(area)
    }
}

impl Widget for Panel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border_color))
            .title(self.title.as_str())
            .title_style(Style::default().fg(Color::Yellow));

        block.render(area, buf);
    }
}