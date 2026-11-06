//! Toolbar: tool groups and active tool selection.

use eframe::egui::{Align, Layout, Ui};

use crate::domain::{MarkupKind, Tool, ToolGroup, ToolSet};
use crate::ui::icon_button::{
    draw_arrow_icon, draw_ellipse_icon, draw_highlight_icon, draw_length_icon, draw_line_icon,
    draw_rectangle_icon, draw_select_icon, draw_text_icon, icon_button,
};

/// Map a tool to its icon drawing function.
fn tool_icon(tool: Tool) -> fn(&eframe::egui::Painter, eframe::egui::Rect) {
    match tool {
        Tool::Select => draw_select_icon,
        Tool::Highlight => draw_highlight_icon,
        Tool::Rectangle => draw_rectangle_icon,
        Tool::Ellipse => draw_ellipse_icon,
        Tool::Line => draw_line_icon,
        Tool::Arrow => draw_arrow_icon,
        Tool::Text => draw_text_icon,
        Tool::MeasureLength => draw_length_icon,
        _ => draw_select_icon,
    }
}

fn tool_button(ui: &mut Ui, active: &mut Tool, tool: Tool, label: &str) -> bool {
    let is_active = *active == tool;
    let icon = tool_icon(tool);
    let mut response = icon_button(ui, icon, label, is_active);
    if let Some(key) = tool.shortcut_key() {
        response = response.on_hover_text(format!("{} ({})", label, key.to_ascii_uppercase()));
    }
    if response.clicked() {
        *active = tool;
        true
    } else {
        false
    }
}

pub(crate) fn draw_toolbar(ui: &mut Ui, active: &mut Tool, _status: &mut String) {
    ui.horizontal(|ui| {
        ui.label("Tool:");
        let kinds = [
            Tool::Select,
            Tool::Highlight,
            Tool::Rectangle,
            Tool::Ellipse,
            Tool::Line,
            Tool::Arrow,
            Tool::Ink,
            Tool::Text,
            Tool::MeasureLength,
        ];
        for t in kinds {
            tool_button(ui, active, t, t.label());
        }
    });
}

pub(crate) fn draw_tool_groups(
    ui: &mut Ui,
    active: &mut Tool,
    _mode: &mut super::app::InspectorMode,
    hint: &mut String,
) {
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        ui.heading("Tools");
        ui.separator();

        // Select first
        tool_button(ui, active, Tool::Select, Tool::Select.label());

        // Review
        ui.collapsing("Review", |ui| {
            let review = [
                Tool::Highlight,
                Tool::Strikeout,
                Tool::Underline,
                Tool::Squiggly,
                Tool::Text,
                Tool::Callout,
                Tool::Note,
                Tool::Stamp,
            ];
            for t in &review {
                if tool_button(ui, active, *t, t.label()) {
                    *hint = format!("Using {}", t.label());
                }
            }
        });

        // Shapes
        ui.collapsing("Shapes", |ui| {
            let shapes = [
                Tool::Rectangle,
                Tool::Ellipse,
                Tool::Cloud,
                Tool::Polygon,
                Tool::Polyline,
            ];
            for t in &shapes {
                if tool_button(ui, active, *t, t.label()) {
                    *hint = format!("Using {}", t.label());
                }
            }
        });

        // Paths
        ui.collapsing("Paths", |ui| {
            let paths = [Tool::Line, Tool::Arrow, Tool::Ink];
            for t in &paths {
                if tool_button(ui, active, *t, t.label()) {
                    *hint = format!("Measurement: {}", t.label());
                }
            }
        });

        // Measure
        ui.collapsing("Measure", |ui| {
            let meas = [
                Tool::MeasureLength,
                Tool::MeasurePerimeter,
                Tool::MeasureArea,
                Tool::MeasureAngle,
                Tool::MeasureDiameter,
                Tool::MeasureRadius,
                Tool::MeasureCount,
            ];
            for t in &meas {
                if tool_button(ui, active, *t, t.label()) {
                    *hint = format!("Measurement: {}", t.label());
                }
            }
        });

        ui.separator();
        ui.small("Click canvas to place shapes; drag to move selection.");
    });
}

/// Build a tool set from a group (for future reuse).
#[allow(dead_code)]
fn tools_for_group(g: ToolGroup) -> Vec<MarkupKind> {
    match g {
        ToolGroup::Review => ToolSet::default_review().tools,
        ToolGroup::Shapes => ToolSet::default_shapes().tools,
        ToolGroup::Paths => vec![MarkupKind::Line, MarkupKind::Arrow, MarkupKind::Ink],
        ToolGroup::Measure => ToolSet::default_measure().tools,
    }
}
