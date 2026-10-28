//! Inspector: properties, comments, measurement display and settings.

use uuid::Uuid;

use eframe::egui::{self, ComboBox, Ui};

use crate::domain::{
    style::Color,
    units::{MeasureUnit, MAX_PRECISION},
    LayerError, MarkupKind, Project, ReviewStatus, Scale, StampPreset,
};
use crate::ui::app::UiState;

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
            let layer = project.active().and_then(|d| d.layer(id)).cloned();
            let Some(layer) = layer else {
                continue;
            };
            let is_default = id == default_id;
            ui.horizontal(|ui| {
                let visible = layer.visible;
                let eye = if visible { "👁" } else { "·" };
                if ui.button(eye).clicked() {
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
                if ui.button(padlock).clicked() {
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
                // Editable layer name (rename)
                let mut name = layer.name.clone();
                let name_resp = ui.add(egui::TextEdit::singleline(&mut name).desired_width(120.0));
                if name_resp.changed() {
                    if let Some(doc) = project.active_mut() {
                        if doc.rename_layer(id, &name) {
                            ui_state.status = format!("Layer renamed to '{}'", name);
                        }
                    }
                }
                if name_resp.lost_focus() {
                    committed = true;
                }
                // Reorder buttons
                if is_default {
                    ui.small("(default)");
                    ui.enabled(false).button("▲");
                    ui.enabled(false).button("▼");
                } else {
                    if ui.button("▲").clicked() {
                        if let Some(doc) = project.active_mut() {
                            if doc.move_layer_up(id) {
                                committed = true;
                                ui_state.status = format!("Layer '{}' moved up", layer.name);
                            }
                        }
                    }
                    if ui.button("▼").clicked() {
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

    let selected: Vec<Uuid> = ui_state.selected.iter().copied().collect();

    if selected.is_empty() {
        ui.label("No selection");
        ui.small("Click a markup on the canvas to edit it.");
        ui.separator();

        ui.collapsing("Project", |ui| {
            let mut name = project.name.clone();
            if ui.text_edit_singleline(&mut name).changed() {
                project.rename(name);
            }
            ui.label("Author");
            if ui
                .text_edit_singleline(&mut project.settings.author)
                .changed()
            {
                ui_state.status = "Author updated".to_string();
            }
        });

        ui.collapsing("Document", |ui| {
            if let Some(doc) = project.active_mut() {
                let mut title = doc.title.clone();
                if ui.text_edit_singleline(&mut title).changed() {
                    doc.title = title;
                    doc.touch();
                }
                ui.label(format!("Pages: {}", doc.page_count()));
                ui.label(format!("Markups: {}", doc.markups.len()));
            }
        });

        // Page / scale settings.
        ui.collapsing("Scale", |ui| {
            // The page currently displayed on the canvas, clamped to the
            // active document's page count.
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
            // Read current scale, keeping a copy so we can tell whether a
            // preset / unit / custom click actually changed anything.
            let before = project
                .active()
                .and_then(|d| d.pages.get(page_idx).copied())
                .map(|p| p.scale)
                .unwrap_or(Scale::PAPER);
            let mut scale = before;

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
                    // Re-clicking the active preset is a no-op: no snapshot.
                    if ui.button(*label).clicked() && !same_scale(*sc, before) {
                        scale = *sc;
                        // apply
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
            // Re-selecting the active unit is a no-op: no snapshot, no rewrite.
            if unit_changed && scale.unit != before.unit {
                // Convert the ratio so the physical length is preserved when the
                // displayed unit changes (10 ft must not silently become 10 m).
                let new_unit = scale.unit;
                scale = scale.with_display_unit(new_unit);
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
                    .range(0..=MAX_PRECISION),
            );
            if precision_resp.changed() {
                project.settings.set_precision(precision);
            }
            // Commit once per drag/entry: snapshot only on release / focus loss.
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
                let _ = ui.add(egui::DragValue::new(&mut ui_state.scale_real_units).speed(0.1));
                ui.label(scale.unit.suffix());
                ui.label("over");
                let _ = ui.add(
                    egui::DragValue::new(&mut ui_state.scale_paper_inches)
                        .speed(0.1)
                        .range(0.01..=100.0),
                );
                ui.label("in paper");
            });
            if ui.button("Apply custom").clicked() {
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
                }
            }
        });

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

        return committed;
    }

    // Work on a clone of the first selected markup; write it back at the end.
    let first = selected[0];
    let Some(mut m) = project.active().and_then(|d| d.find_markup(first)).cloned() else {
        ui.small("Selected markup no longer exists.");
        return committed;
    };

    let mut edited = false;
    let editable = project
        .active()
        .map(|d| d.is_markup_editable(&m))
        .unwrap_or(true);

    ui.collapsing("Properties", |ui| {
        ui.label(format!("ID: {}", m.id));
        ui.label(format!("Kind: {}", m.kind.label()));
        ui.label(format!("Page: {}", m.page));

        ui.horizontal(|ui| {
            ui.label("Subject");
            let mut subj = m.subject.clone();
            if ui.text_edit_singleline(&mut subj).changed() {
                m.subject = subj;
                edited = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Author");
            let mut auth = m.author.clone();
            if ui.text_edit_singleline(&mut auth).changed() {
                m.author = auth;
                edited = true;
            }
        });

        ComboBox::from_label("Status")
            .selected_text(
                egui::RichText::new(m.status.label()).color(domain_to_color32(m.status.color())),
            )
            .show_ui(ui, |ui| {
                for s in ReviewStatus::ALL.iter() {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            rect.center(),
                            5.0,
                            domain_to_color32(s.color()),
                        );
                        if ui.selectable_label(m.status == *s, s.label()).clicked() {
                            m.status = *s;
                            ui_state.status = format!("Status → {}", s.label());
                            edited = true;
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
        if editable {
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
                            m.layer_id = if *id == default_id { None } else { Some(*id) };
                            edited = true;
                            committed = true;
                            ui_state.status = format!("Layer → {}", name);
                        }
                    }
                });
        } else {
            ui.small("Locked layer — edits disabled");
        }

        ui.label("Stroke colour");
        let mut sc = domain_to_color32(m.style.stroke);
        if ui.color_edit_button_srgba(&mut sc).changed() {
            let c = color32_to_domain(sc);
            m.style.stroke = c;
            m.style.fill = c;
            edited = true;
        }
        ui.horizontal(|ui| {
            ui.label("Stroke width");
            if ui
                .add(egui::Slider::new(&mut m.style.width, 0.25..=10.0))
                .changed()
            {
                edited = true;
            }
        });

        if m.kind.is_measure() {
            // Current page scale.
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
    });

    ui.collapsing("Comments", |ui| {
        if m.comments.is_empty() {
            ui.small("No comments yet.");
        } else {
            for c in &m.comments {
                ui.label(format!("{} · {}", c.author, c.created.format("%H:%M:%S")));
                ui.small(&c.text);
                ui.separator();
            }
        }
        // Disabled while the markup is locked: a click would otherwise consume
        // the typed text and the write-back guard would drop it, losing it
        // with no feedback. The draft text is only cleared once added.
        ui.add_enabled_ui(editable, |ui| {
            ui.text_edit_multiline(comment_text);
            let added = ui.button("Add Comment").clicked() && !comment_text.is_empty();
            if added {
                m.comments.push(crate::domain::Comment::new(
                    project.settings.author.clone(),
                    comment_text.clone(),
                ));
                comment_text.clear();
                ui_state.status = "Comment added".to_string();
                edited = true;
                // Adding a comment is a committed action: snapshot it once.
                committed = true;
            }
        });
    });

    // Write the markup back only when this frame actually edited it, otherwise
    // merely selecting a markup would churn `modified` on every frame.
    // Locked markups (per-markup or via a locked layer) are never written back.
    if edited && editable {
        m.touch();
        if let Some(doc) = project.active_mut() {
            if let Some(slot) = doc.markups.iter().position(|x| x.id == m.id) {
                doc.markups[slot] = m;
                doc.touch();
            }
        }
    }

    committed
}
