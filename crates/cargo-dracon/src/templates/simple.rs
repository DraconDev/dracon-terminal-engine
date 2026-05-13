//! Simple Dracon Terminal Engine Application

use dracon_terminal_engine::prelude::*;
use ratatui::layout::Rect;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()?
        .title("My Dracon App")
        .fps(30)
        .on_tick(|ctx, _| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);
            let mut plane = Plane::new(0, w, h);
            plane.fill_bg(ctx.theme().bg);

            // Render a simple greeting
            let text = "Hello, Dracon!";
            let x = (w.saturating_sub(text.len() as u16)) / 2;
            let y = h / 2;
            plane.put_str(x, y, text);

            ctx.add_plane(plane);
        })
        .on_input(|key| {
            // Handle keyboard input
            if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
                return true; // Quit on 'q'
            }
            false
        })
        .run();

    Ok(())
}