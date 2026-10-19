//! Inspector: properties, comments, measurement display and settings.

use uuid::Uuid;

use eframe::egui::{self, ComboBox, Ui};

use crate::domain::{
    style::Color,
    units::{MeasureUnit, MAX_PRECISION},
    Project, ReviewStatus, Scale,
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

    if project.active().is_none() {
        ui.small("No active document.");
        return false;
    }

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
            // Determine active page index (first page).
            let page_idx = 0usize;
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

        return committed;
    }

    // Work on a clone of the first selected markup; write it back at the end.
    let first = selected[0];
    let Some(mut m) = project.active().and_then(|d| d.find_markup(first)).cloned() else {
        ui.small("Selected markup no longer exists.");
        return committed;
    };

    let mut edited = false;

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
            .selected_text(m.status.label())
            .show_ui(ui, |ui| {
                for s in ReviewStatus::ALL.iter() {
                    if ui.selectable_value(&mut m.status, *s, s.label()).clicked() {
                        ui_state.status = format!("Status → {}", s.label());
                        edited = true;
                    }
                }
            });

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
        ui.text_edit_multiline(comment_text);
        if ui.button("Add Comment").clicked() && !comment_text.is_empty() {
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

    // Write the markup back only when this frame actually edited it, otherwise
    // merely selecting a markup would churn `modified` on every frame.
    if edited {
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
