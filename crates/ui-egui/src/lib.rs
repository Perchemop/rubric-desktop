//! UI egui crate for Rubric.
//!
//! This crate provides the egui-based UI layer for the Rubric PDF markup application.
//! It is divided into modular sub-crates:
//! - `ui-egui-design` — tokens, palette, contrast auditing, focus ring.
//! - `ui-egui-keyboard` — keyboard shortcuts and view navigation.

#![allow(clippy::too_many_arguments)]

pub mod keyboard;
pub mod design;
pub use keyboard::*;
pub use design::*;