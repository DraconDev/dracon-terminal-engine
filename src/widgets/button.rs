use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

/// A button widget that displays a label and can be toggled active/inactive.
pub struct Button {
    /// The text label displayed on the button.
    label: String,
    /// Whether the button is in an active state (rendered in yellow/bold).
    is_active: bool,
}

impl Button {
    /// Creates a new Button with the given label and active state.
    pub fn new(label: impl Into<String>, is_active: bool) -> Self {
        Self {
            label: label.into(),
            is_active,
        }
    }
}

impl Widget for Button {
    /// Renders the button into the given buffer area with active/inactive styling.
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
