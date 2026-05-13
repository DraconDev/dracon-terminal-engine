#![allow(missing_docs)]
//! Demonstrates framework widgets working together.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Checkbox, Form, ProgressBar, Radio, SearchInput, Select, Slider, Spinner, Toast, ToastKind,
    Toggle, Tooltip,
};

fn main() {
    println!("Framework Widgets Demo");
    println!("========================");
    println!();

    let _theme = Theme::dark();

    let cb = Checkbox::new(WidgetId::new(1), "Enable feature");
    let plane = cb.render(ratatui::layout::Rect::new(0, 0, 40, 3));
    println!(
        "Checkbox rendered (width={}, height={})",
        plane.width, plane.height
    );

    let mut toggle = Toggle::new(WidgetId::new(2), "Mode");
    toggle.toggle();
    println!("Toggle is on: {}", toggle.is_on());

    let pb = ProgressBar::new(WidgetId::new(3));
    let mut pb_clone = pb;
    pb_clone.set_progress(0.75);
    println!("ProgressBar at: {}", pb_clone.progress());

    let sp = Spinner::new(WidgetId::new(4));
    println!("Spinner frame: '{}'", sp.current_frame());

    let radio = Radio::new(WidgetId::new(5), "Option A");
    println!("Radio selected: {}", radio.is_selected());

    let slider = Slider::new(WidgetId::new(6)).with_range(0.0, 100.0);
    let mut slider_clone = slider;
    slider_clone.set_value(50.0);
    println!("Slider value: {}", slider_clone.value());

    let search = SearchInput::new(WidgetId::new(7));
    println!("Search input query: '{}'", search.query());

    let select =
        Select::new(WidgetId::new(8)).with_options(vec!["One".to_string(), "Two".to_string()]);
    println!("Select label: {:?}", select.selected_label());

    let toast = Toast::new(WidgetId::new(10), "Operation complete").with_kind(ToastKind::Success);
    println!("Toast message: '{}'", toast.message());

    let tooltip = Tooltip::new(WidgetId::new(11), "Help text here");
    println!("Tooltip text: '{}'", tooltip.text());

    let form = Form::new(WidgetId::new(13))
        .add_field("Username")
        .add_field("Password");
    println!("Form widget created with id: {:?}", form.id());

    println!();
    println!("All Phase 1-3 widgets initialized successfully!");
}
