use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// A hotkey hint component for rendering keyboard shortcut indicators.
pub struct HotkeyHint;

impl HotkeyHint {
    /// Renders a hotkey hint with a colored key badge and label.
    pub fn render<'a>(key: &'a str, label: &'a str, color: Color) -> Vec<Span<'a>> {
        vec![
            Span::styled(
                format!(" {} ", key),
                Style::default()
                    .bg(color)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {} ", label), Style::default().fg(Color::White)),
            Span::raw("  "),
        ]
    }
}
