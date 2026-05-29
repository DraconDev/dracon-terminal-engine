//! Quick size check
#![allow(dead_code)]
use dracon_terminal_engine::compositor::{Cell, Color, Styles};
use std::mem::size_of;

pub fn check_sizes() {
    println!("Cell: {}", size_of::<Cell>());
    println!("Color: {}", size_of::<Color>());
    println!("Styles: {}", size_of::<Styles>());
}
