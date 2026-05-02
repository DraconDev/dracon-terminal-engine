#![allow(missing_docs)]
//! # Widget Tutorial: Building a Custom ColorPicker Widget
//!
//! This example demonstrates how to build a complete custom widget from scratch
//! using the `Widget` trait. We'll create a `ColorPicker` widget that displays
//! a color swatch the user can click or keyboard-navigate to cycle through colors.
//!
//! ## What You'll Learn
//!
//! 1. **Struct Design** - How to structure your widget's internal state
//! 2. **Widget Trait Implementation** - All 12 methods of the `Widget` trait explained
//! 3. **Builder Pattern** - Using the builder pattern for widget configuration
//! 4. **HitZone Integration** - Using `HitZone` for mouse click handling
//! 5. **Plane Rendering** - Drawing cells with colors and styles
//! 6. **Theme Integration** - Making your widget theme-aware
//! 7. **App Integration** - Adding your widget to an application with multiple instances
//!
//! ## Run This Example
//!
//! ```sh
//! cargo run --example widget_tutorial
//! ```
//!
//! Use arrow keys to navigate between color pickers, left/right to change colors,
//! and Enter or click to cycle through colors.

// ============================================================================
// IMPORTS
// ============================================================================

// We use the public crate name `dracon_terminal_engine` for examples,
// NOT `crate` which refers to the internal crate.
use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::hitzone::{HitZone, HitZoneGroup};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ============================================================================
// PART 1: THE PRESET COLORS
// ============================================================================

/// Represents a single preset color with its display name and RGB values.
/// This is a simple data structure that our ColorPicker will cycle through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetColor {
    name: &'static str,
    r: u8,
    g: u8,
    b: u8,
}

impl PresetColor {
    /// All available preset colors for the ColorPicker.
    /// Each color has a human-readable name and RGB values.
    const PRESETS: &'static [PresetColor] = &[
        PresetColor { name: "Red",    r: 255, g: 0,   b: 0   },
        PresetColor { name: "Green",  r: 0,   g: 255, b: 0   },
        PresetColor { name: "Blue",   r: 0,   g: 0,   b: 255 },
        PresetColor { name: "Yellow", r: 255, g: 255, b: 0   },
        PresetColor { name: "Purple", r: 128, g: 0,   b: 128 },
        PresetColor { name: "Cyan",   r: 0,   g: 255, b: 255 },
        PresetColor { name: "Orange", r: 255, g: 165, b: 0   },
        PresetColor { name: "Pink",   r: 255, g: 192, b: 203 },
    ];

    /// Returns the Color representation for use in Cell rendering.
    fn to_compositor_color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    /// Returns the RGB values as a formatted string for display.
    fn rgb_string(&self) -> String {
        format!("RGB({}, {}, {})", self.r, self.g, self.b)
    }
}

// ============================================================================
// PART 2: WIDGET STRUCT DEFINITION
// ============================================================================

/// The ColorPicker widget - displays a color swatch that cycles through presets.
///
/// # State Fields
///
/// - `id`: Unique identifier assigned by the App framework
/// - `selected_index`: Which preset color is currently selected (0-based)
/// - `theme`: The current theme for colors and styling
/// - `area`: The widget's position and size (stored in a Cell for interior mutability)
/// - `dirty`: Whether the widget needs re-rendering
/// - `hitzones`: Click detection zones for mouse interaction
///
/// # Design Notes
///
/// We use `std::cell::Cell<Rect>` for area because the App framework needs to
/// set the area after construction, but we also need to read it during rendering.
/// Using Cell allows interior mutability without RefCell overhead.
pub struct ColorPicker {
    /// Unique identifier for this widget (assigned by App::add_widget).
    id: WidgetId,

    /// Index into PRESETS array - which color is currently selected.
    selected_index: usize,

    /// Current theme for foreground/background colors.
    /// Updated via on_theme_change() when App's theme changes.
    theme: Theme,

    /// The rectangular area this widget occupies on screen.
    /// Using Cell<Rect> allows mutation even through &self (needed for render).
    area: std::cell::Cell<Rect>,

    /// Whether this widget needs to be re-rendered.
    /// Set to true whenever state changes (color selection, focus, theme).
    dirty: bool,

    /// Hit zones for mouse click detection.
    /// The swatch area is clickable to cycle colors.
    /// Using Option allows us to rebuild zones when area changes.
    hitzones: HitZoneGroup<usize>,
}

impl ColorPicker {
    // ========================================================================
    // PART 3: CONSTRUCTOR AND BUILDER PATTERN
    // ========================================================================

    /// Creates a new ColorPicker with default settings.
    /// Default color is the first in the preset list (Red).
    pub fn new() -> Self {
        Self {
            id: WidgetId::default_id(),
            selected_index: 0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 5)),
            dirty: true,
            hitzones: HitZoneGroup::new(),
        }
    }

    /// Creates a ColorPicker with a specific widget ID.
    /// Used internally when App assigns IDs.
    pub fn with_id(id: WidgetId) -> Self {
        Self {
            id,
            selected_index: 0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 5)),
            dirty: true,
            hitzones: HitZoneGroup::new(),
        }
    }

    /// Builder method: Sets the initial selected color by name.
    /// Returns self for method chaining.
    pub fn initial_color(mut self, color_name: &str) -> Self {
        self.selected_index = PresetColor::PRESETS
            .iter()
            .position(|p| p.name.to_lowercase() == color_name.to_lowercase())
            .unwrap_or(0);
        self
    }

    /// Builder method: Sets the theme for this widget.
    /// Returns self for method chaining.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    // ========================================================================
    // PART 4: STATE ACCESS METHODS
    // ========================================================================

    /// Returns a reference to the currently selected preset color.
    pub fn selected_color(&self) -> &'static PresetColor {
        &PresetColor::PRESETS[self.selected_index]
    }

    /// Cycles to the next color in the preset list (wraps around).
    pub fn cycle_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % PresetColor::PRESETS.len();
        self.dirty = true;
    }

    /// Cycles to the previous color in the preset list (wraps around).
    pub fn cycle_prev(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = PresetColor::PRESETS.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.dirty = true;
    }

    /// Rebuilds the hitzones based on the current area.
    /// Called when area changes to keep click detection accurate.
    fn rebuild_hitzones(&mut self) {
        let area = self.area.get();

        // Clear existing zones and rebuild based on current area.
        // The swatch occupies the left 6 characters.
        // The text info occupies the remaining space.
        self.hitzones = HitZoneGroup::new();

        // Swatch zone: left portion is clickable to cycle color.
        let swatch_zone = HitZone::new(
            0,                              // ID for swatch zone
            area.x,                         // left edge
            area.y,                         // top edge
            6,                              // width: "█████ " = 6 chars
            area.height,                    // full height
        )
        .on_click(|_click_kind| {
            // The actual cycle logic is in handle_mouse, but this
            // shows how to register callbacks.
        });

        self.hitzones.zones_mut().push(swatch_zone);
    }
}

// ========================================================================
// PART 5: THE WIDGET TRAIT IMPLEMENTATION
// ========================================================================

/// Implementing the Widget trait for ColorPicker.
/// This is the core contract every framework widget must fulfill.
impl Widget for ColorPicker {

    // ------------------------------------------------------------------------
    // id() and set_id()
    // ------------------------------------------------------------------------

    /// Returns the unique identifier for this widget.
    /// The App framework uses this to route events and manage focus.
    fn id(&self) -> WidgetId {
        self.id
    }

    /// Sets the widget's ID. Called by App::add_widget after assigning an ID.
    /// We store it in our struct for later use.
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    // ------------------------------------------------------------------------
    // area() and set_area()
    // ------------------------------------------------------------------------

    /// Returns the current area (position and size) of this widget.
    /// The App framework calls this to know where to render and route events.
    fn area(&self) -> Rect {
        self.area.get()
    }

    /// Sets the widget's area. Called by App when adding the widget.
    /// We mark dirty because a new area might mean different hitzone geometry.
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
        self.rebuild_hitzones();
    }

    // ------------------------------------------------------------------------
    // focusable() and z_index()
    // ------------------------------------------------------------------------

    /// ColorPicker can receive keyboard focus for arrow-key navigation.
    fn focusable(&self) -> bool {
        true
    }

    /// Default z-index of 0 means it renders below higher-z widgets.
    fn z_index(&self) -> u16 {
        0
    }

    // ------------------------------------------------------------------------
    // needs_render(), mark_dirty(), clear_dirty()
    // ------------------------------------------------------------------------

    /// Returns true if the widget needs to be rendered.
    /// The render loop skips widgets that return false here.
    fn needs_render(&self) -> bool {
        self.dirty
    }

    /// Marks the widget as dirty (needs re-rendering).
    /// Call this whenever state changes that affect visuals.
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag after rendering.
    /// The render loop calls this automatically after successful render.
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    // ------------------------------------------------------------------------
    // render()
    // ------------------------------------------------------------------------

    /// Renders the widget into a Plane at the given area.
    /// This is the main drawing function - it fills cells with colors and text.
    ///
    /// The Plane is a 2D grid of Cells, each cell has:
    /// - char: the character to display
    /// - fg: foreground color
    /// - bg: background color
    /// - style: text styling (bold, italic, etc.)
    fn render(&self, area: Rect) -> Plane {
        // Create a new plane with the widget's ID and dimensions.
        // Plane is filled with transparent cells by default.
        let mut plane = Plane::new(self.id.0, area.width, area.height);
        plane.z_index = 0;

        // Get the currently selected color's info.
        let color = self.selected_color();
        let color_cell = color.to_compositor_color();

        // ---- ROW 0: Color name header ----
        // Format: "[ ● ] ColorName"
        let header = format!("[ {} ] {}", "●", color.name);

        for (i, c) in header.chars().take(area.width as usize).enumerate() {
            plane.cells[i] = Cell {
                char: c,
                fg: self.theme.fg,
                bg: self.theme.bg,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }

        // ---- ROW 1: Color swatch ----
        // A full-width bar of the current color.
        // We use the block character █ for a solid fill.
        let swatch_row = 1usize;
        for col in 0..area.width as usize {
            let idx = swatch_row * area.width as usize + col;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: '█',
                    fg: color_cell,
                    bg: color_cell,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        // ---- ROW 2: RGB value display ----
        // Shows the numeric RGB values centered below the swatch.
        let rgb_text = color.rgb_string();
        let rgb_row = 2usize;
        let start_x = (area.width as i32 - rgb_text.len() as i32) / 2;
        let start_x = start_x.max(0) as usize;

        for (i, c) in rgb_text.chars().take(area.width as usize - start_x).enumerate() {
            let idx = rgb_row * area.width as usize + start_x + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: color_cell,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        // ---- ROW 3: Instructions (subtle, smaller text) ----
        // Shows how to interact with the widget.
        let instruction_row = 3usize;
        let instruction = if area.width >= 20 { "←/→ or Click to change" } else { "←/→" };
        let instr_len = instruction.len().min(area.width as usize);
        let instr_start = ((area.width as usize - instr_len) / 2).max(0);

        for (i, c) in instruction.chars().take(instr_len).enumerate() {
            let idx = instruction_row * area.width as usize + instr_start + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: self.theme.inactive_fg,
                    bg: self.theme.bg,
                    style: Styles::DIM,
                    transparent: false,
                    skip: false,
                };
            }
        }

        // ---- ROW 4: Color index indicator ----
        // Shows "1/8" style position indicator.
        let index_row = 4usize;
        let index_text = format!("{}/{}", self.selected_index + 1, PresetColor::PRESETS.len());
        let index_start = ((area.width as usize - index_text.len()) / 2).max(0);

        for (i, c) in index_text.chars().take(area.width as usize - index_start).enumerate() {
            let idx = index_row * area.width as usize + index_start + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: self.theme.accent,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    // ------------------------------------------------------------------------
    // on_focus() and on_blur()
    // ------------------------------------------------------------------------

    /// Called when the widget gains focus.
    /// We mark dirty to show the focus indicator (if we had one).
    fn on_focus(&mut self) {
        self.dirty = true;
    }

    /// Called when the widget loses focus.
    fn on_blur(&mut self) {
        self.dirty = true;
    }

    // ------------------------------------------------------------------------
    // on_mount() and on_unmount()
    // ------------------------------------------------------------------------

    /// Called when the widget is added to the application.
    /// Good place for one-time initialization.
    fn on_mount(&mut self) {
        self.dirty = true;
        self.rebuild_hitzones();
    }

    /// Called when the widget is removed from the application.
    fn on_unmount(&mut self) {
        // Clean up any resources if needed.
        // For ColorPicker, no special cleanup needed.
    }

    // ------------------------------------------------------------------------
    // on_theme_change()
    // ------------------------------------------------------------------------

    /// Called when the application theme changes.
    /// We update our cached theme and mark dirty to re-render.
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    // ------------------------------------------------------------------------
    // handle_key()
    // ------------------------------------------------------------------------

    /// Handles keyboard events.
    /// Returns true if the event was consumed, false if it should bubble.
    ///
    /// Arrow keys cycle through colors:
    /// - Left/Right: cycle prev/next color
    /// - Enter/Space: cycle to next color
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Only handle key press events (not release or repeat).
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            // Left arrow: cycle to previous color
            KeyCode::Left => {
                self.cycle_prev();
                true
            }
            // Right arrow: cycle to next color
            KeyCode::Right => {
                self.cycle_next();
                true
            }
            // Enter or Space: cycle to next color
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.cycle_next();
                true
            }
            // Any other key: not consumed
            _ => false,
        }
    }

    // ------------------------------------------------------------------------
    // handle_mouse()
    // ------------------------------------------------------------------------

    /// Handles mouse events within the widget's bounds.
    /// Coordinates are local to the widget (0,0 is top-left of widget area).
    ///
    /// Returns true if the event was consumed.
    fn handle_mouse(
        &mut self,
        kind: MouseEventKind,
        local_col: u16,
        _local_row: u16,
    ) -> bool {
        // Handle left-click on the swatch area (col 0-5) to cycle color.
        if let MouseEventKind::Down(_) = kind {
            // Check if click is in the swatch area (left 6 columns).
            if local_col < 6 {
                self.cycle_next();
                return true;
            }
        }

        false
    }

    // ------------------------------------------------------------------------
    // commands() and apply_command_output()
    // ------------------------------------------------------------------------

    /// Returns the list of commands this widget can execute.
    /// Used by AI to enumerate available actions.
    ///
    /// ColorPicker doesn't bind to external commands, so we return empty.
    fn commands(&self) -> Vec<dracon_terminal_engine::framework::command::BoundCommand> {
        Vec::new()
    }

    /// Applies parsed output from a bound command.
    /// ColorPicker doesn't use commands, so this is a no-op.
    fn apply_command_output(&mut self, _output: &dracon_terminal_engine::framework::command::ParsedOutput) {
        // No-op: ColorPicker doesn't bind to commands.
    }
}

// ============================================================================
// PART 6: DEFAULT AND CONSTRUCTOR TRAITS
// ============================================================================

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PART 7: THE MAIN FUNCTION - PUTTING IT ALL TOGETHER
// ============================================================================

/// Main entry point demonstrating ColorPicker in a real App.
///
/// This function creates an App with multiple ColorPicker widgets
/// arranged in a grid, showing how to:
/// - Add multiple widget instances
/// - Use different themes
/// - Handle keyboard navigation between widgets
fn main() -> std::io::Result<()> {
    // ---- Create the App with builder pattern ----
    let mut app = App::new()?
        .title("Widget Tutorial: ColorPicker")
        .fps(30)
        .theme(Theme::nord());

    // ---- Create multiple ColorPicker instances ----
    //
    // We'll create a 2x2 grid of color pickers with different initial colors
    // to demonstrate that each instance maintains its own state.

    // Row 1: Red and Green pickers
    let red_picker = ColorPicker::new()
        .initial_color("Red")
        .with_theme(Theme::nord());

    let green_picker = ColorPicker::new()
        .initial_color("Green")
        .with_theme(Theme::nord());

    // Row 2: Blue and Yellow pickers
    let blue_picker = ColorPicker::new()
        .initial_color("Blue")
        .with_theme(Theme::nord());

    let yellow_picker = ColorPicker::new()
        .initial_color("Yellow")
        .with_theme(Theme::nord());

    // ---- Add widgets to the app with their areas ----
    //
    // The area is (x, y, width, height) in terminal cells.
    // We're creating a 2x2 grid layout.

    let picker_width = 25u16;
    let picker_height = 6u16;

    // Add each picker to the app - the app assigns IDs and calls on_mount()
    let _id1 = app.add_widget(Box::new(red_picker), Rect::new(0,  0, picker_width, picker_height));
    let _id2 = app.add_widget(Box::new(green_picker), Rect::new(26, 0, picker_width, picker_height));
    let _id3 = app.add_widget(Box::new(blue_picker), Rect::new(0,  7, picker_width, picker_height));
    let _id4 = app.add_widget(Box::new(yellow_picker), Rect::new(26, 7, picker_width, picker_height));

    // ---- Add a header label using the framework's built-in Label widget ----
    let header = dracon_terminal_engine::framework::widgets::Label::new("←/→ to change color | Click swatch to cycle | Tab to navigate");
    let _header_id = app.add_widget(Box::new(header), Rect::new(0, 14, 80, 1));

    // ---- Add a footer showing theme name ----
    let footer = dracon_terminal_engine::framework::widgets::Label::new("Theme: nord | Press Ctrl+C to exit");
    let _footer_id = app.add_widget(Box::new(footer), Rect::new(0, 15, 80, 1));

    // ---- Quit support ----
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    app = app
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        });

    // ---- Run the app ----
    //
    // The run() function starts the event loop:
    // - Reads keyboard/mouse input
    // - Routes events to focused widgets
    // - Calls render() on dirty widgets each frame
    // - Responds to window resize
    //
    // The closure is called each frame (render callback).
    // We don't use it because widgets self-manage via the render loop.
    app.run(|_ctx| {})
}

// ============================================================================
// EPILOGUE: KEY CONCEPTS SUMMARY
// ============================================================================
//
// 1. STRUCT DESIGN
//    Keep internal state private. Use Cell for interior mutability on
//    data that must be readable in const methods (like render).
//
// 2. BUILDER PATTERN
//    Methods like `.with_theme()` and `.initial_color()` return self
//    enabling fluent chaining: ColorPicker::new().initial_color("Red").with_theme(theme)
//
// 3. WIDGET TRAIT
//    All 12 methods must be implemented. Most have sensible defaults.
//    Key methods: id(), area(), render(), handle_key(), handle_mouse()
//
// 4. AREA MANAGEMENT
//    The App sets our area after construction. Using Cell<Rect> allows
//    storing area where both &self (for render) and &mut self (for set_area) exist.
//
// 5. DIRTY TRACKING
//    Mark dirty whenever visual state changes. Clear dirty after render.
//    Only dirty widgets are rendered (optimization).
//
// 6. HITZONES
//    For simple widgets, manual bounds checking in handle_mouse() works.
//    For complex widgets with multiple clickable regions, use HitZoneGroup.
//
// 7. THEME INTEGRATION
//    Store theme in struct. Update via on_theme_change(). Use theme colors
//    in render() for consistent appearance.
//
// 8. PLANE RENDERING
//    Create Plane with id, width, height. Fill cells with chars, fg, bg, style.
//    Cells default to transparent - must explicitly set all fields.
//
// 9. EVENT HANDLING
//    handle_key() for keyboard navigation
//    handle_mouse() for click detection
//    Return true when event is consumed, false to bubble
//
// 10. APP INTEGRATION
//     Box::<dyn Widget>::new(your_widget) to add to App
//     App assigns ID, sets area, calls on_mount()
//     Focus management happens automatically via Tab key
//
// ============================================================================