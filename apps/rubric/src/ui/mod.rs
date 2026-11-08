//! UI module.

pub mod app;
pub mod app_canvas;
pub mod app_dialogs;
pub mod app_inspector;
pub mod app_markups_list;
pub mod app_menu;
pub mod app_pages;
pub mod app_status;
pub mod app_tool_chest;
pub mod app_topbar;
pub mod canvas;
pub use canvas::{draw_canvas, commit_text_edit};
pub mod canvas_draw;
pub mod canvas_geom;
pub mod canvas_hit;
pub mod icon_button;
pub mod icons;
pub mod inspector;
pub mod markups_list;
pub mod menu;
pub mod pages;
pub mod theme;
pub mod tool_chest;
pub mod toolbar;

pub use app::RubricApp;
