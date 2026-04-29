use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

pub struct Button {
    label: String,
    is_active: bool,
}

impl Button {
    pub fn new(label: impl Into<String>, is_active: bool) -> Self {
        Self {
            label: label.into(),
            is_active,
        }
    }
}

impl Widget for Button {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.is_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let paragraph = ratatui::widgets::Paragraph::new(self.label.as_str())
            .alignment(ratatui::layout::Alignment::Center)
            .style(style);

        paragraph.render(area, buf);
    }
}