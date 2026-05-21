//! Quick size check
#![allow(dead_code)]
use std::mem::size_of;
use super::{Cell, Color, Styles};

pub fn check_sizes() {
    println!("Cell: {}", size_of::<Cell>());
    println!("Color: {}", size_of::<Color>());
    println!("Styles: {}", size_of::<Styles>());
}
