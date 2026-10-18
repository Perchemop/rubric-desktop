//! Inspector: properties, comments, measurement display and settings.

use uuid::Uuid;

use eframe::egui::{self, ComboBox, Ui};

use crate::domain::{style::Color, units::MeasureUnit, Project, ReviewStatus, Scale};

fn color32_to_domain(c: egui::Color32) -> Color {
    let [r, g, b, a] = c.to_array();
    Color { r, g, b, a }
}

fn domain_to_color32(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

pub(crate) fn draw_inspector(
    ui: &mut Ui,
    project: &mut Project,
    selected: &mut std::collections::HashSet<Uuid>,
    comment_text: &mut String,
    mut scale: Scale,
    status: &mut String,
) {
    ui.heading("Inspector");
    ui.separator();

    if project.active().is_none() {
        ui.small("No active document.");
        return;
    }

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
            if ui.text_edit_singleline(&mut project.settings.author).changed() {
                *status = "Author updated".to_string();
            }
            ComboBox::from_label("Unit")
                .selected_text(scale.unit.name())
                .show_ui(ui, |ui| {
                    for u in MeasureUnit::ALL.iter() {
                        ui.selectable_value(&mut scale.unit, *u, u.name());
                    }
                });
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

        // Apply a changed unit to the active page's scale.
        if let Some(doc) = project.active_mut() {
            if let Some(page) = doc.pages.first_mut() {
                if page.scale.unit != scale.unit {
                    page.scale.unit = scale.unit;
                    doc.touch();
                }
            }
        }
        return;
    }

    // Work on a clone of the first selected markup; write it back at the end.
    let first = *selected.iter().next().unwrap();
    let Some(mut m) = project.active().and_then(|d| d.find_markup(first)).cloned() else {
        ui.small("Selected markup no longer exists.");
        return;
    };

    ui.collapsing("Properties", |ui| {
        ui.label(format!("ID: {}", m.id));
        ui.label(format!("Kind: {}", m.kind.label()));
        ui.label(format!("Page: {}", m.page));

        ui.horizontal(|ui| {
            ui.label("Subject");
            let mut subj = m.subject.clone();
            if ui.text_edit_singleline(&mut subj).changed() {
                m.subject = subj;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Author");
            let mut auth = m.author.clone();
            if ui.text_edit_singleline(&mut auth).changed() {
                m.author = auth;
            }
        });

        ComboBox::from_label("Status")
            .selected_text(m.status.label())
            .show_ui(ui, |ui| {
                for s in ReviewStatus::ALL.iter() {
                    if ui.selectable_value(&mut m.status, *s, s.label()).clicked() {
                        *status = format!("Status → {}", s.label());
                    }
                }
            });

        ui.label("Stroke colour");
        let mut sc = domain_to_color32(m.style.stroke);
        if ui.color_edit_button_srgba(&mut sc).changed() {
            let c = color32_to_domain(sc);
            m.style.stroke = c;
            m.style.fill = c;
        }
        ui.horizontal(|ui| {
            ui.label("Stroke width");
            ui.add(egui::Slider::new(&mut m.style.width, 0.25..=10.0));
        });

        if m.kind.is_measure() {
            if let Some(meas) = m.measurement {
                ui.separator();
                ui.label("Measurement");
                ui.small(format!("Value: {}", meas.label(scale.unit)));
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
            *status = "Comment added".to_string();
        }
    });

    // Write the edited markup back.
    m.touch();
    if let Some(doc) = project.active_mut() {
        if let Some(slot) = doc.markups.iter().position(|x| x.id == m.id) {
            doc.markups[slot] = m;
            doc.touch();
        }
    }
}
