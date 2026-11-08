//! Toolbar: tool groups and active tool selection.

use eframe::egui::{Align, Layout, Ui};

use crate::domain::{MarkupKind, Tool, ToolGroup, ToolSet};
use crate::ui::icon_button::{
    draw_arrow_icon, draw_calibrate_icon, draw_callout_icon, draw_cloud_icon, draw_ellipse_icon,
    draw_highlight_icon, draw_ink_icon, draw_length_icon, draw_line_icon, draw_measure_angle_icon,
    draw_measure_area_icon, draw_measure_count_icon, draw_measure_diameter_icon,
    draw_measure_perimeter_icon, draw_measure_radius_icon, draw_note_icon, draw_polygon_icon,
    draw_polyline_icon, draw_rectangle_icon, draw_select_icon, draw_squiggly_icon, draw_stamp_icon,
    draw_strikeout_icon, draw_text_icon, draw_underline_icon, icon_button,
};

/// Map a tool to its icon drawing function.
fn tool_icon(tool: Tool) -> fn(&eframe::egui::Painter, eframe::egui::Rect, eframe::egui::Color32) {
    match tool {
        Tool::Select => draw_select_icon,
        Tool::Highlight => draw_highlight_icon,
        Tool::Strikeout => draw_strikeout_icon,
        Tool::Underline => draw_underline_icon,
        Tool::Squiggly => draw_squiggly_icon,
        Tool::Text => draw_text_icon,
        Tool::Callout => draw_callout_icon,
        Tool::Note => draw_note_icon,
        Tool::Stamp => draw_stamp_icon,
        Tool::Rectangle => draw_rectangle_icon,
        Tool::Ellipse => draw_ellipse_icon,
        Tool::Cloud => draw_cloud_icon,
        Tool::Polygon => draw_polygon_icon,
        Tool::Polyline => draw_polyline_icon,
        Tool::Line => draw_line_icon,
        Tool::Arrow => draw_arrow_icon,
        Tool::Ink => draw_ink_icon,
        Tool::MeasureLength => draw_length_icon,
        Tool::MeasurePerimeter => draw_measure_perimeter_icon,
        Tool::MeasureArea => draw_measure_area_icon,
        Tool::MeasureAngle => draw_measure_angle_icon,
        Tool::MeasureDiameter => draw_measure_diameter_icon,
        Tool::MeasureRadius => draw_measure_radius_icon,
        Tool::MeasureCount => draw_measure_count_icon,
        Tool::Calibrate => draw_calibrate_icon,
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

/// Tool families used to order and visually group the toolbar strip.
/// Each family is a contiguous run of tools; a small gap separates families.
const TOOL_FAMILIES: &[(&str, &[Tool])] = &[
    (
        "Select",
        &[
            Tool::Select,
            Tool::Highlight,
            Tool::Strikeout,
            Tool::Underline,
            Tool::Squiggly,
        ],
    ),
    (
        "Review",
        &[Tool::Text, Tool::Callout, Tool::Note, Tool::Stamp],
    ),
    (
        "Shapes",
        &[
            Tool::Rectangle,
            Tool::Ellipse,
            Tool::Cloud,
            Tool::Polygon,
            Tool::Polyline,
        ],
    ),
    ("Paths", &[Tool::Line, Tool::Arrow, Tool::Ink]),
    (
        "Measure",
        &[
            Tool::MeasureLength,
            Tool::MeasurePerimeter,
            Tool::MeasureArea,
            Tool::MeasureAngle,
            Tool::MeasureDiameter,
            Tool::MeasureRadius,
            Tool::MeasureCount,
        ],
    ),
    ("Calibrate", &[Tool::Calibrate]),
];

pub(crate) fn draw_toolbar(ui: &mut Ui, active: &mut Tool, _status: &mut String) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Tools:");
        // Use TOOL_METADATA as the single source of truth for tool order/labels
        for meta in crate::domain::project::TOOL_METADATA.iter() {
            tool_button(ui, active, meta.tool, &meta.label);
            // Show shortcut if different from label
            if let Some(key) = meta.shortcut {
                // Shortcut already included in label per TOOL_METADATA convention
            }
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
                    *hint = format!("Using {}", t.label());
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
