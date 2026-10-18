//! Toolbar: tool groups and active tool selection.

use eframe::egui::{Align, Layout, Ui};

use crate::domain::{MarkupKind, Tool, ToolGroup, ToolSet};

fn tool_button(ui: &mut Ui, active: &mut Tool, tool: Tool, label: &str) -> bool {
    let is_active = *active == tool;
    let btn = ui.selectable_label(is_active, label);
    if btn.clicked() {
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
            (Tool::Select, "Select"),
            (Tool::Highlight, "Highlight"),
            (Tool::Rectangle, "Rect"),
            (Tool::Ellipse, "Ellipse"),
            (Tool::Line, "Line"),
            (Tool::Arrow, "Arrow"),
            (Tool::Ink, "Ink"),
            (Tool::Text, "Text"),
            (Tool::MeasureLength, "Length"),
        ];
        for (t, name) in kinds {
            tool_button(ui, active, t, name);
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
        tool_button(ui, active, Tool::Select, "Select");

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
                let k = t.to_kind().unwrap_or(MarkupKind::Highlight);
                if tool_button(ui, active, *t, k.label()) {
                    *hint = format!("Using {}", k.label());
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
                let k = t.to_kind().unwrap_or(MarkupKind::Rectangle);
                if tool_button(ui, active, *t, k.label()) {
                    *hint = format!("Using {}", k.label());
                }
            }
        });

        // Paths
        ui.collapsing("Paths", |ui| {
            let paths = [Tool::Line, Tool::Arrow, Tool::Ink];
            for t in &paths {
                let k = t.to_kind().unwrap_or(MarkupKind::Line);
                if tool_button(ui, active, *t, k.label()) {
                    *hint = format!("Using {}", k.label());
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
                let k = t.to_kind().unwrap_or(MarkupKind::MeasureLength);
                if tool_button(ui, active, *t, k.label()) {
                    *hint = format!("Measurement: {}", k.label());
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
