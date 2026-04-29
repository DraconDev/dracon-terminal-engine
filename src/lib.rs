#![warn(missing_docs)]

pub mod backend;
pub mod compositor;
pub mod contracts;
pub mod core;
pub mod input;
pub mod integration;
pub mod layout;
pub(crate) mod system;

pub mod utils;
pub mod visuals;
pub mod widgets;

pub use core::terminal::Terminal;
