//! Inspector: properties, comments, measurement display and settings.

use uuid::Uuid;

use eframe::egui::{self, ComboBox, Ui};

use crate::domain::{
    style::Color,
    units::{MeasureUnit, MAX_PRECISION},
    BatesCorner, LayerError, MarkupKind, Project, ReviewStatus, Scale, StampPreset,
    MAX_BATES_DIGITS, MAX_BATES_START, MIN_BATES_DIGITS,
};
use crate::ui::app::{InspectorMode, UiState};

fn color32_to_domain(c: egui::Color32) -> Color {
    let [r, g, b, a] = c.to_array();
    Color { r, g, b, a }
}

fn domain_to_color32(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Whether two scales describe the same mapping, within float tolerance.
fn same_scale(a: Scale, b: Scale) -> bool {
    a.unit == b.unit && (a.units_per_point - b.units_per_point).abs() < 1e-9
}

pub(crate) fn draw_inspector(
    ui: &mut Ui,
    project: &mut Project,
    ui_state: &mut UiState,
    comment_text: &mut String,
) -> bool {
    let mut committed = false;

    ui.heading("Inspector");
    ui.separator();

    // Segmented inspector tabs (P1-7): switch which panel section is primary.
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                ui_state.inspector_mode == InspectorMode::Selection,
                "Selection",
            )
            .on_hover_text("Markup properties")
            .clicked()
        {
            ui_state.inspector_mode = InspectorMode::Selection;
        }
        if ui
            .selectable_label(
                ui_state.inspector_mode == InspectorMode::Properties,
                "Properties",
            )
            .on_hover_text("Stroke, width, layer")
            .clicked()
        {
            ui_state.inspector_mode = InspectorMode::Properties;
        }
        if ui
            .selectable_label(
                ui_state.inspector_mode == InspectorMode::Comments,
                "Comments",
            )
            .on_hover_text("Threaded comments")
            .clicked()
        {
            ui_state.inspector_mode = InspectorMode::Comments;
        }
        if ui
            .selectable_label(
                ui_state.inspector_mode == InspectorMode::Measurements,
                "Measure",
            )
            .on_hover_text("Measurement readout")
            .clicked()
        {
            ui_state.inspector_mode = InspectorMode::Measurements;
        }
        if ui
            .selectable_label(
                ui_state.inspector_mode == InspectorMode::Settings,
                "Settings",
            )
            .on_hover_text("Project & document settings")
            .clicked()
        {
            ui_state.inspector_mode = InspectorMode::Settings;
        }
    });
    ui.separator();

    // Status legend (spec F-305): one row per status with swatch + label + count.
    ui.collapsing("Status legend", |ui| {
        let mut counts = [0usize; ReviewStatus::ALL.len()];
        if let Some(doc) = project.active() {
            for m in &doc.markups {
                for (i, s) in ReviewStatus::ALL.iter().enumerate() {
                    if m.status == *s {
                        counts[i] += 1;
                    }
                }
            }
        }
        for (i, s) in ReviewStatus::ALL.iter().enumerate() {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 5.0, domain_to_color32(s.color()));
                ui.label(s.label());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(counts[i].to_string());
                });
            });
        }
    });

    if project.active().is_none() {
        ui.small("No active document.");
        return false;
    }

    // Layers panel (compact section). Always available.
    ui.collapsing("Layers", |ui| {
        if ui.button("Add layer").clicked() {
            if let Some(doc) = project.active_mut() {
                doc.add_layer("");
                committed = true;
                ui_state.status = "Layer added".to_string();
            }
        }
        let layer_ids: Vec<Uuid> = project
            .active()
            .map(|d| d.layers.iter().map(|l| l.id).collect())
            .unwrap_or_default();
        let default_id = project
            .active()
            .map(|d| d.default_layer_id())
            .unwrap_or_default();
        for id in layer_ids {
            let layer = project.active().and_then(|d| d.layer(id).cloned());
            let Some(layer) = layer else {
                continue;
            };
            let is_default = id == default_id;
            ui.horizontal(|ui| {
                let visible = layer.visible;
                let eye = if visible { "👁" } else { "·" };
                let eye_tip = if visible { "Hide layer" } else { "Show layer" };
                if ui.button(eye).on_hover_text(eye_tip).clicked() {
                    if let Some(doc) = project.active_mut() {
                        if doc.set_layer_visible(id, !visible) {
                            committed = true;
                            ui_state.status = if !visible {
                                format!("Layer '{}' visible", layer.name)
                            } else {
                                format!("Layer '{}' hidden", layer.name)
                            };
                        }
                    }
                }
                let locked = layer.locked;
                let padlock = if locked { "🔒" } else { "🔓" };
                let padlock_tip = if locked { "Unlock layer" } else { "Lock layer" };
                if ui.button(padlock).on_hover_text(padlock_tip).clicked() {
                    if let Some(doc) = project.active_mut() {
                        if doc.set_layer_locked(id, !locked) {
                            committed = true;
                            ui_state.status = if !locked {
                                format!("Layer '{}' locked", layer.name)
                            } else {
                                format!("Layer '{}' unlocked", layer.name)
                            };
                        }
                    }
                }
                // Editable layer name (rename): edit a local buffer, commit on
                // lost focus (Enter included), never more than one snapshot.
                let mut name = ui_state
                    .layer_name_drafts
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| layer.name.clone());
                let name_resp = ui
                    .add(egui::TextEdit::singleline(&mut name).desired_width(120.0))
                    .on_hover_text("Layer name");
                if name_resp.changed() {
                    ui_state.layer_name_drafts.insert(id, name.clone());
                }
                if name_resp.lost_focus() {
                    if let Some(doc) = project.active_mut() {
                        if doc.rename_layer(id, &name) {
                            ui_state.layer_name_drafts.remove(&id);
                            committed = true;
                            ui_state.status = format!("Layer renamed to '{}'", name.trim());
                        } else {
                            ui_state.status = "Layer name cannot be empty".to_string();
                        }
                    }
                }
                // Reorder buttons
                if is_default {
                    ui.small("(default)");
                } else {
                    if ui
                        .button("▲")
                        .on_hover_text("Move layer up (towards the front)")
                        .clicked()
                    {
                        if let Some(doc) = project.active_mut() {
                            if doc.move_layer_up(id) {
                                committed = true;
                                ui_state.status = format!("Layer '{}' moved up", layer.name);
                            }
                        }
                    }
                    if ui
                        .button("▼")
                        .on_hover_text("Move layer down (towards the back)")
                        .clicked()
                    {
                        if let Some(doc) = project.active_mut() {
                            if doc.move_layer_down(id) {
                                committed = true;
                                ui_state.status = format!("Layer '{}' moved down", layer.name);
                            }
                        }
                    }
                }
                if !is_default && ui.button("Delete").clicked() {
                    if let Some(doc) = project.active_mut() {
                        match doc.remove_layer(id) {
                            Ok(()) => {
                                committed = true;
                                ui_state.status = "Layer removed".to_string();
                            }
                            Err(LayerError::NotEmpty) => {
                                ui_state.status =
                                    "Layer not empty — move its markups first".to_string();
                            }
                            Err(LayerError::DefaultLayer) => {
                                ui_state.status = "Cannot remove default layer".to_string();
                            }
                            Err(LayerError::NotFound) => {}
                        }
                    }
                }
            });
        }
    });

    // Page / scale settings (always reachable, even with a selection).
    ui.collapsing("Scale", |ui| {
        let page_idx = if let Some(doc) = project.active() {
            let count = doc.page_count();
            if count == 0 {
                0
            } else {
                ui_state.current_page.min(count - 1)
            }
        } else {
            ui_state.current_page
        };
        let before = project
            .active()
            .and_then(|d| d.pages.get(page_idx).copied())
            .map(|p| p.scale)
            .unwrap_or(Scale::PAPER);
        let mut scale = before;

        // Re-sync custom-ratio inputs when the page changes.
        let page_changed = ui_state.last_scale_page != Some(page_idx);
        if page_changed {
            ui_state.last_scale_page = Some(page_idx);
            ui_state.scale_real_units = before.units_per_point * 72.0;
            ui_state.scale_paper_inches = 1.0;
        }

        ui.label("Presets");
        ui.horizontal_wrapped(|ui| {
            let presets: [(&str, Scale); 4] = [
                ("1:1", Scale::PAPER),
                (
                    "1 in = 1 ft",
                    Scale::from_ratio(MeasureUnit::Foot, 1.0, 72.0),
                ),
                (
                    "1 in = 10 ft",
                    Scale::from_ratio(MeasureUnit::Foot, 10.0, 72.0),
                ),
                (
                    "1 in = 50 ft",
                    Scale::from_ratio(MeasureUnit::Foot, 50.0, 72.0),
                ),
            ];
            for (label, sc) in presets.iter() {
                if ui.button(*label).clicked() && !same_scale(*sc, before) {
                    scale = *sc;
                    if let Some(doc) = project.active_mut() {
                        if let Some(page) = doc.pages.get_mut(page_idx) {
                            page.scale = scale;
                        }
                        let s = doc.pages.get(page_idx).map(|p| p.scale).unwrap_or(scale);
                        for m in doc.markups.iter_mut().filter(|m| m.page == page_idx) {
                            m.recompute_measurement(s);
                        }
                        doc.touch();
                    }
                    committed = true;
                    ui_state.status = format!("Scale → {}", scale.describe());
                }
            }
        });

        ui.separator();
        ui.label("Unit");
        let mut unit_changed = false;
        ComboBox::from_label("Display unit")
            .selected_text(scale.unit.name())
            .show_ui(ui, |ui| {
                for u in MeasureUnit::ALL.iter() {
                    if ui.selectable_value(&mut scale.unit, *u, u.name()).clicked() {
                        unit_changed = true;
                    }
                }
            });
        if unit_changed && scale.unit != before.unit {
            let new_unit = scale.unit;
            scale = before.with_display_unit(new_unit);
            if let Some(doc) = project.active_mut() {
                if let Some(page) = doc.pages.get_mut(page_idx) {
                    page.scale = scale;
                }
                let s = doc.pages.get(page_idx).map(|p| p.scale).unwrap_or(scale);
                for m in doc.markups.iter_mut().filter(|m| m.page == page_idx) {
                    m.recompute_measurement(s);
                }
                doc.touch();
            }
            committed = true;
            ui_state.status = format!("Unit → {}", scale.unit.name());
        }

        ui.separator();
        ui.label("Precision (decimal places)");
        let precision_before = project.settings.precision;
        let mut precision = precision_before;
        let precision_resp = ui.add(
            egui::DragValue::new(&mut precision)
                .speed(1.0)
                .clamp_range(0..=MAX_PRECISION),
        );
        if precision_resp.changed() {
            project.settings.set_precision(precision);
        }
        if project.settings.precision != precision_before
            && (precision_resp.drag_stopped() || precision_resp.lost_focus())
        {
            project.touch();
            committed = true;
            ui_state.status = format!("Precision → {}", project.settings.precision);
        }

        ui.separator();
        ui.label("Custom ratio: paper inches = real units");
        ui.horizontal(|ui| {
            ui.label("1 in paper =");
            let _ = ui.add(
                egui::DragValue::new(&mut ui_state.scale_real_units)
                    .speed(0.1)
                    .clamp_range(0.001..=1_000_000.0),
            );
            ui.label(scale.unit.suffix());
            ui.label("over");
            let _ = ui.add(
                egui::DragValue::new(&mut ui_state.scale_paper_inches)
                    .speed(0.1)
                    .clamp_range(0.01..=100.0),
            );
            ui.label("in paper");
        });
        if ui.button("Apply custom").clicked() {
            if ui_state.scale_real_units <= 0.0 || ui_state.scale_paper_inches <= 0.0 {
                ui_state.status = "Scale values must be positive".to_string();
            } else {
                let paper_pts = (ui_state.scale_paper_inches * 72.0).max(1.0);
                let new_scale = Scale::from_ratio(scale.unit, ui_state.scale_real_units, paper_pts);
                if !same_scale(new_scale, before) {
                    if let Some(doc) = project.active_mut() {
                        if let Some(page) = doc.pages.get_mut(page_idx) {
                            page.scale = new_scale;
                        }
                        let s = doc
                            .pages
                            .get(page_idx)
                            .map(|p| p.scale)
                            .unwrap_or(new_scale);
                        for m in doc.markups.iter_mut().filter(|m| m.page == page_idx) {
                            m.recompute_measurement(s);
                        }
                        doc.touch();
                    }
                    committed = true;
                    ui_state.status = format!("Scale → {}", new_scale.describe());
                } else {
                    ui_state.status = "Scale unchanged".to_string();
                }
            }
        }
    });

    let selected: Vec<Uuid> = ui_state.selected.iter().copied().collect();
    let multi = selected.len() > 1;

    if selected.is_empty() {
        ui.label("No selection");
        ui.small("Click a markup on the canvas to edit it.");
    } else {
        // Work on a clone of the first selected markup; write it back at the end.
        let first = selected[0];
        let Some(mut m) = project.active().and_then(|d| d.find_markup(first)).cloned() else {
            ui.small("Selected markup no longer exists.");
            // Skip property editing; Project/Document sections render below.
            // Jump to them by leaving this block.
            // (Nothing else to do here.)
            // We still need to avoid running the property code, so return early
            // but the common sections below would be skipped. To keep them
            // reachable, duplicate minimal: actually fall through by not returning.
            // Rust requires the binding, so handle via a flag instead.
            ui.separator();
            // Render common sections then return.
            ui.collapsing("Project", |ui| {
                let mut name = project.name.clone();
                let resp = ui.text_edit_singleline(&mut name);
                if resp.changed() {
                    project.rename(name);
                    if resp.lost_focus() {
                        committed = true;
                    }
                }
                ui.label("Author");
                let mut author = project.settings.author.clone();
                let resp_a = ui.text_edit_singleline(&mut author);
                if resp_a.changed() {
                    project.settings.author = author;
                    ui_state.status = "Author updated".to_string();
                    if resp_a.lost_focus() {
                        committed = true;
                    }
                }
            });
            ui.collapsing("Document", |ui| {
                if let Some(doc) = project.active_mut() {
                    let mut title = doc.title.clone();
                    let resp = ui.text_edit_singleline(&mut title);
                    if resp.changed() {
                        doc.title = title;
                        doc.touch();
                        if resp.lost_focus() {
                            committed = true;
                        }
                    }
                    ui.label(format!("Pages: {}", doc.page_count()));
                    ui.label(format!("Markups: {}", doc.markups.len()));
                }
            });
            if ui_state.active_tool.to_kind() == Some(MarkupKind::Stamp) {
                ui.separator();
                ui.label("Stamp preset (tool)");
                ComboBox::from_label("Preset")
                    .selected_text(ui_state.active_stamp.label())
                    .show_ui(ui, |ui| {
                        for preset in StampPreset::ALL.iter() {
                            if ui
                                .selectable_value(
                                    &mut ui_state.active_stamp,
                                    *preset,
                                    preset.label(),
                                )
                                .clicked()
                            {
                                ui_state.status = format!("Stamp tool → {}", preset.label());
                            }
                        }
                    });
            }
            return committed;
        };

        let editable = project
            .active()
            .map(|d| d.is_markup_editable(&m))
            .unwrap_or(true);

        if multi {
            ui.label(format!("{} selected", selected.len()));

            // Batch status change for multi-select (v0.4-02).
            ui.horizontal(|ui| {
                ui.label("Batch Status");
                ComboBox::from_id_salt("batch_status")
                    .selected_text("Change status…")
                    .show_ui(ui, |ui| {
                        for s in ReviewStatus::ALL.iter() {
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(12.0, 12.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    rect.center(),
                                    5.0,
                                    domain_to_color32(s.color()),
                                );
                                if ui.selectable_label(false, s.label()).clicked() {
                                    for id in &selected {
                                        let doc = project.active_mut();
                                        if let Some(doc) = doc {
                                            // Check editability first (immutable borrow)
                                            let editable = doc
                                                .find_markup(*id)
                                                .map(|m| doc.is_markup_editable(m))
                                                .unwrap_or(false);
                                            if editable {
                                                if let Some(markup) = doc.find_markup_mut(*id) {
                                                    markup.status = *s;
                                                    markup.touch();
                                                }
                                            }
                                        }
                                    }
                                    if let Some(doc) = project.active_mut() {
                                        doc.touch();
                                    }
                                    ui_state.status = format!(
                                        "Batch status → {} for {} markup(s)",
                                        s.label(),
                                        selected.len()
                                    );
                                    committed = true;
                                }
                            });
                        }
                    });
            });
        }

        let mut edited = false;

        egui::CollapsingHeader::new("Properties")
            .default_open(true)
            .show(ui, |ui| {
                if ui_state.inspector_mode != InspectorMode::Properties {
                    ui.small("Click the Properties tab to edit stroke, width, layer, status.");
                    return;
                }

                ui.label(format!("ID: {}", m.id));
                ui.label(format!("Kind: {}", m.kind.label()));
                ui.label(format!("Page: {}", m.page));

                ui.add_enabled_ui(editable, |ui| {
                    if !multi {
                        ui.horizontal(|ui| {
                            ui.label("Subject");
                            let mut subj = m.subject.clone();
                            let resp = ui.text_edit_singleline(&mut subj);
                            if resp.changed() {
                                m.subject = subj;
                                edited = true;
                                if resp.lost_focus() {
                                    committed = true;
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Author");
                            let mut auth = m.author.clone();
                            let resp = ui.text_edit_singleline(&mut auth);
                            if resp.changed() {
                                m.author = auth;
                                edited = true;
                                if resp.lost_focus() {
                                    committed = true;
                                }
                            }
                        });

                        ComboBox::from_label("Status")
                            .selected_text(
                                egui::RichText::new(m.status.label())
                                    .color(domain_to_color32(m.status.color())),
                            )
                            .show_ui(ui, |ui| {
                                for s in ReviewStatus::ALL.iter() {
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(12.0, 12.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(
                                            rect.center(),
                                            5.0,
                                            domain_to_color32(s.color()),
                                        );
                                        if ui.selectable_label(m.status == *s, s.label()).clicked()
                                            && m.status != *s
                                        {
                                            m.status = *s;
                                            ui_state.status = format!("Status → {}", s.label());
                                            edited = true;
                                            committed = true;
                                        }
                                    });
                                }
                            });

                        // Stamp preset (spec F-111): only for stamp markups.
                        if editable && m.kind == MarkupKind::Stamp {
                            ui.label("Preset");
                            for preset in StampPreset::ALL.iter() {
                                let clicked = ui
                                    .selectable_label(m.stamp == Some(*preset), preset.label())
                                    .clicked();
                                if clicked && m.stamp != Some(*preset) {
                                    m.stamp = Some(*preset);
                                    m.style = m.style.with_stroke(preset.color());
                                    edited = true;
                                    committed = true;
                                    ui_state.status = format!("Stamp → {}", preset.label());
                                }
                            }
                        }

                        // Layer assignment.
                        let default_id = project
                            .active()
                            .map(|d| d.default_layer_id())
                            .unwrap_or_default();
                        let layers: Vec<(Uuid, String)> = project
                            .active()
                            .map(|d| d.layers.iter().map(|l| (l.id, l.name.clone())).collect())
                            .unwrap_or_default();
                        let current_id = m.layer_id.unwrap_or(default_id);
                        let current_name = layers
                            .iter()
                            .find(|(id, _)| *id == current_id)
                            .map(|(_, n)| n.clone())
                            .unwrap_or_else(|| "Layer 1".to_string());
                        ComboBox::from_label("Layer")
                            .selected_text(current_name)
                            .show_ui(ui, |ui| {
                                for (id, name) in &layers {
                                    if ui.selectable_label(current_id == *id, name).clicked() {
                                        m.layer_id =
                                            if *id == default_id { None } else { Some(*id) };
                                        edited = true;
                                        committed = true;
                                        ui_state.status = format!("Layer → {}", name);
                                    }
                                }
                            });
                    }

                    ui.label("Stroke colour");
                    let mut sc = domain_to_color32(m.style.stroke);
                    let colour_changed = ui
                        .color_edit_button_srgba(&mut sc)
                        .on_hover_text("Stroke colour")
                        .changed();
                    if colour_changed {
                        let c = color32_to_domain(sc);
                        m.style.stroke = c;
                        // Keep the fill's own translucency (UI audit P1-7):
                        // adopt the new hue but preserve the fill's alpha —
                        // recolouring the stroke used to erase the fill tint.
                        m.style.fill = Color {
                            r: c.r,
                            g: c.g,
                            b: c.b,
                            a: m.style.fill.a,
                        };
                        edited = true;
                        ui_state.colour_pending = Some(sc);
                    }
                    // Commit when the value stops changing (popup closed / no
                    // change this frame), exactly one snapshot.
                    let pending_commit = ui_state.colour_pending.is_some() && !colour_changed;
                    if pending_commit {
                        ui_state.colour_pending = None;
                        committed = true;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Stroke width");
                        let resp = ui.add(egui::Slider::new(&mut m.style.width, 0.25..=10.0));
                        if resp.changed() {
                            edited = true;
                            if resp.drag_stopped() || resp.lost_focus() {
                                committed = true;
                            }
                        }
                    });
                });

                if !editable {
                    ui.small("Locked layer — edits disabled");
                }

                if m.kind.is_measure() {
                    if ui_state.inspector_mode != InspectorMode::Measurements {
                        ui.small("Click the Measure tab to view the measurement readout.");
                    } else {
                        let scale = project
                            .active()
                            .and_then(|d| d.pages.get(m.page).copied())
                            .map(|p| p.scale)
                            .unwrap_or(Scale::PAPER);
                        if let Some(meas) = m.measurement {
                            ui.separator();
                            ui.label("Measurement");
                            ui.small(format!(
                                "Value: {}",
                                meas.label_with(scale.unit, project.settings.precision)
                            ));
                            ui.small(format!("Points: {:.2} pt", meas.value_points));
                            ui.small(format!(
                                "Scale: {:.6} {}/pt",
                                scale.units_per_point,
                                scale.unit.suffix()
                            ));
                        } else {
                            ui.small("Measurement not computed");
                        }
                    }
                }
            });

        ui.collapsing("Comments", |ui| {
            if ui_state.inspector_mode != InspectorMode::Comments {
                ui.small("Click the Comments tab to view and add threaded comments.");
            } else if m.comments.is_empty() {
                ui.small("No comments yet.");
            } else {
                for c in &m.comments {
                    ui.label(format!("{} · {}", c.author, c.created.format("%H:%M")));
                    ui.small(&c.text);
                    ui.separator();
                }
            }
            ui.add_enabled_ui(editable, |ui| {
                ui.text_edit_multiline(comment_text)
                    .on_hover_text("Comment text");
                let add_clicked = ui.button("Add Comment").clicked();
                if add_clicked {
                    let author = project.settings.author.clone();
                    if m.add_comment(author, comment_text.clone()).is_some() {
                        comment_text.clear();
                        ui_state.status = "Comment added".to_string();
                        edited = true;
                        committed = true;
                    }
                }
            });
        });

        if edited && editable {
            m.touch();
            if let Some(doc) = project.active_mut() {
                if multi {
                    doc.apply_style_to(&selected, m.style);
                } else if let Some(slot) = doc.markups.iter().position(|x| x.id == m.id) {
                    doc.markups[slot] = m;
                    doc.touch();
                }
            }
        }
    }

    // Project / Document editors and tool stamp-preset picker: shown under
    // Settings tab, or when nothing is selected.
    if ui_state.inspector_mode == InspectorMode::Settings || selected.is_empty() {
        ui.collapsing("Project", |ui| {
            let mut name = project.name.clone();
            let resp = ui.text_edit_singleline(&mut name);
            if resp.changed() {
                project.rename(name);
                if resp.lost_focus() {
                    committed = true;
                }
            }
            ui.label("Author");
            let mut author = project.settings.author.clone();
            let resp_a = ui.text_edit_singleline(&mut author);
            if resp_a.changed() {
                project.settings.author = author;
                ui_state.status = "Author updated".to_string();
                if resp_a.lost_focus() {
                    committed = true;
                }
            }
        });

        ui.collapsing("Document", |ui| {
            if let Some(doc) = project.active_mut() {
                let mut title = doc.title.clone();
                let resp = ui.text_edit_singleline(&mut title);
                if resp.changed() {
                    doc.title = title;
                    doc.touch();
                    if resp.lost_focus() {
                        committed = true;
                    }
                }
                ui.label(format!("Pages: {}", doc.page_count()));
                ui.label(format!("Markups: {}", doc.markups.len()));
            }
        });

        // Bates numbering (spec F-801–F-803): the configuration UI. The stamp
        // is a non-destructive overlay applied at export/print time; this
        // section only edits the persisted configuration and previews the
        // sequence.
        let bates_pages = project.active().map(|d| d.page_count()).unwrap_or(0);
        ui.collapsing("Bates numbering", |ui| {
            let bates = &mut project.settings.bates;
            // Undo discipline (UI audit P1-4): discrete controls (checkbox,
            // corner picker) commit immediately; continuous ones (text
            // fields, drag values) apply live so the First/Last previews
            // track, but commit exactly ONE snapshot when the interaction
            // ends. Typing a prefix or dragging Start used to push a
            // snapshot per frame.
            let mut committed_now = ui
                .checkbox(&mut bates.enabled, "Enable Bates numbering")
                .changed();
            if committed_now {
                ui_state.bates_dirty = false;
            }
            let mut interaction_ended = false;
            ui.add_enabled_ui(bates.enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Prefix");
                    let resp = ui.text_edit_singleline(&mut bates.prefix);
                    if resp.changed() {
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.lost_focus();
                });
                ui.horizontal(|ui| {
                    ui.label("Suffix");
                    let resp = ui.text_edit_singleline(&mut bates.suffix);
                    if resp.changed() {
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.lost_focus();
                });
                ui.horizontal(|ui| {
                    ui.label("Start");
                    let mut start = bates.start;
                    let resp =
                        ui.add(egui::DragValue::new(&mut start).clamp_range(0..=MAX_BATES_START));
                    if resp.changed() {
                        bates.start = start;
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.drag_stopped() || resp.lost_focus();
                });
                ui.horizontal(|ui| {
                    ui.label("Digits");
                    let mut digits = bates.digits;
                    let resp = ui.add(
                        egui::DragValue::new(&mut digits)
                            .clamp_range(MIN_BATES_DIGITS..=MAX_BATES_DIGITS),
                    );
                    if resp.changed() {
                        bates.set_digits(digits);
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.drag_stopped() || resp.lost_focus();
                });
                ComboBox::from_id_salt("bates_corner")
                    .selected_text(bates.corner.label())
                    .show_ui(ui, |ui| {
                        for corner in BatesCorner::ALL {
                            if ui
                                .selectable_value(&mut bates.corner, corner, corner.label())
                                .changed()
                            {
                                committed_now = true;
                                ui_state.bates_dirty = false;
                            }
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Margin (pt)");
                    let resp =
                        ui.add(egui::DragValue::new(&mut bates.margin).clamp_range(0.0..=200.0));
                    if resp.changed() {
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.drag_stopped() || resp.lost_focus();
                });
                ui.horizontal(|ui| {
                    ui.label("Font size (pt)");
                    let resp =
                        ui.add(egui::DragValue::new(&mut bates.font_size).clamp_range(4.0..=72.0));
                    if resp.changed() {
                        ui_state.bates_dirty = true;
                    }
                    interaction_ended |= resp.drag_stopped() || resp.lost_focus();
                });
                ui.horizontal(|ui| {
                    ui.label("Pages to stamp");
                    ui.small(format!(
                        "{} of {}",
                        bates.applied_count(bates_pages),
                        bates_pages
                    ));
                });
                if let Some(first) = bates.stamp_for_page(bates.range.from) {
                    ui.small(format!("First: {first}"));
                }
                if let Some(last) = bates.last_stamp(bates_pages) {
                    ui.small(format!("Last: {last}"));
                }
            });
            if committed_now || (interaction_ended && ui_state.bates_dirty) {
                committed = true;
                ui_state.bates_dirty = false;
            }
        });
    }

    // Compact stamp-preset selector for the active Stamp tool (UI state only).
    if ui_state.active_tool.to_kind() == Some(MarkupKind::Stamp) {
        ui.separator();
        ui.label("Stamp preset (tool)");
        ComboBox::from_label("Preset")
            .selected_text(ui_state.active_stamp.label())
            .show_ui(ui, |ui| {
                for preset in StampPreset::ALL.iter() {
                    if ui
                        .selectable_value(&mut ui_state.active_stamp, *preset, preset.label())
                        .clicked()
                    {
                        ui_state.status = format!("Stamp tool → {}", preset.label());
                    }
                }
            });
    }

    committed
}
