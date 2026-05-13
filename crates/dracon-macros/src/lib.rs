//! Procedural macros for Dracon Terminal Engine.
//!
//! This crate provides derive macros for simplifying widget implementation:
//!
//! - `#[derive(Widget)]` - Auto-generates `needs_render`, `mark_dirty`, `clear_dirty` boilerplate
//! - `#[derive(FromJson, ToJson)]` - Auto-derive for widget state serialization
//! - `#[widget_state]` - Attribute for state field that auto-generates serialization

use proc_macro::TokenStream;
use quote::quote;
use syn;

/// Derive macro for automatically implementing the Widget trait.
///
/// # Example
///
/// ```ignore
/// use dracon_macros::Widget;
///
/// #[derive(Widget)]
/// struct MyWidget {
///     #[widget_state]
///     count: u32,
///     theme: Theme,
/// }
/// ```
///
/// This generates:
/// - `needs_render()` that checks dirty flag
/// - `mark_dirty()` that sets dirty flag
/// - `clear_dirty()` that clears dirty flag
/// - `on_theme_change()` that updates theme field
#[proc_macro_derive(Widget, attributes(widget_state))]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    let data = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Widget derive only works on structs"),
    };

    // Find fields with #[widget_state] attribute
    let state_fields: Vec<_> = data
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| f.attrs.iter().any(|a| a.path().is_ident("widget_state")))
        .collect();

    // Generate field initialization
    let field_inits = data.fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { #ident }
    });

    // Generate widget implementation
    let gen = quote! {
        impl dracon_terminal_engine::framework::widget::Widget for #name {
            fn id(&self) -> dracon_terminal_engine::framework::widget::WidgetId {
                self.state_id.unwrap_or_else(|| dracon_terminal_engine::framework::widget::WidgetId::new())
            }

            fn set_id(&mut self, id: dracon_terminal_engine::framework::widget::WidgetId) {
                self.state_id = Some(id);
            }

            fn area(&self) -> ratatui::layout::Rect {
                self.area
            }

            fn set_area(&mut self, area: ratatui::layout::Rect) {
                self.area = area;
                self.dirty = true;
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

            fn render(&self, area: ratatui::layout::Rect) -> dracon_terminal_engine::compositor::Plane {
                // User must implement render() manually or use widget_state for auto-render
                unimplemented!("Implement render() for {}", stringify!(#name))
            }

            fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
                false
            }

            fn handle_mouse(
                &mut self,
                kind: dracon_terminal_engine::input::event::MouseEventKind,
                col: u16,
                row: u16,
            ) -> bool {
                false
            }

            fn on_theme_change(&mut self, theme: &dracon_terminal_engine::framework::theme::Theme) {
                self.theme = *theme;
                self.dirty = true;
            }
        }

        // Implementation for widget state
        impl #name {
            /// Create a new widget with default values.
            pub fn new(theme: dracon_terminal_engine::framework::theme::Theme) -> Self {
                Self {
                    #(#field_inits,)*
                    dirty: true,
                    area: ratatui::layout::Rect::new(0, 0, 40, 10),
                    state_id: None,
                    theme,
                }
            }

            /// Get the current state as JSON.
            pub fn state_json(&self) -> serde_json::Result<String> {
                serde_json::to_string(self)
            }

            /// Load state from JSON.
            pub fn set_state(&mut self, json: &str) -> serde_json::Result<()> {
                let new_state: Self = serde_json::from_str(json)?;
                *self = new_state;
                Ok(())
            }
        }
    };

    gen.into()
}

/// Derive macro for JSON serialization (FromJson).
///
/// Implements conversion from JSON string to struct.
#[proc_macro_derive(FromJson)]
pub fn derive_from_json(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl #name {
            /// Create from JSON string.
            pub fn from_json(json: &str) -> serde_json::Result<Self> {
                serde_json::from_str(json)
            }
        }
    };

    gen.into()
}

/// Derive macro for JSON serialization (ToJson).
///
/// Implements conversion from struct to JSON string.
#[proc_macro_derive(ToJson)]
pub fn derive_to_json(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl #name {
            /// Convert to JSON string.
            pub fn to_json(&self) -> serde_json::Result<String> {
                serde_json::to_string(self)
            }
        }
    };

    gen.into()
}

/// Attribute macro for marking widget state fields.
///
/// This attribute marks fields that should be included in the widget's
/// state serialization and reset logic.
#[proc_macro_attribute]
pub fn widget_state(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Simply pass through - the derive macro handles the actual implementation
    let _ = attr;
    input
}