use dracon_terminal_engine::framework::widgets::color_picker::ColorPicker;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use ratatui::layout::Rect;

fn main() {
    let picker = ColorPicker::with_color(Color::Rgb(255, 0, 0))
        .with_theme(Theme::nord());
    
    let area = Rect::new(0, 0, 40, 20);
    let plane = picker.render(area);
    
    eprintln!("Color Picker Render (40x20):");
    eprintln!("============================");
    
    for y in 0..area.height as usize {
        let mut line = String::new();
        for x in 0..area.width as usize {
            let idx = y * area.width as usize + x;
            if idx < plane.cells.len() {
                let cell = &plane.cells[idx];
                if cell.char == ' ' && cell.bg == Color::Reset {
                    line.push('.');
                } else if cell.char == ' ' {
                    line.push('#');
                } else {
                    line.push(cell.char);
                }
            }
        }
        eprintln!("{:2}: {}", y, line);
    }
}
