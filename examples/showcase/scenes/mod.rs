pub mod accessibility_scene;
pub mod autocomplete_scene;
pub mod calendar_scene;
pub mod cell_pool_scene;
pub mod form_demo;
pub mod modal_demo;
pub mod notification_center_scene;
pub mod rich_text_scene;
pub mod theme_switcher;
pub mod tree_navigator;
pub mod widget_gallery;

/// Which action a scene wants the router to take.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SceneAction {
    /// Stay on this scene (normal operation).
    None,
    /// Pop back to the previous scene.
    Pop,
    /// Push a new scene by ID.
    Push(String),
    /// Quit the entire application.
    Quit,
}
