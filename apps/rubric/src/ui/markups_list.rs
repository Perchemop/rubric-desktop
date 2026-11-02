//! Markups List panel: filter, sort, group and select markups.
//!
//! The panel is a thin view over the pure-domain [`MarkupList`] model. It never
//! re-implements filtering, sorting or grouping; it only translates the UI
//! controls into domain values and renders the resulting rows.

use std::collections::{BTreeMap, HashMap, HashSet};

use eframe::egui::{self, ComboBox, RichText, Ui};
use uuid::Uuid;

use crate::domain::units::{MeasureKind, MeasureUnit, Scale};
use crate::domain::{
    MarkupGroupKey, MarkupKind, MarkupList, MarkupListFilter, MarkupListSort, Project,
    ReviewStatus, NONE_LABEL,
};
use crate::ui::app::{MarkupLayerFilter, UiState};

/// A request from the list to the app: select `id` and show its page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkupListAction {
    pub(crate) id: Uuid,
    pub(crate) page: usize,
}

/// Translate the panel's UI controls into the domain filter.
///
/// A blank search and `MarkupLayerFilter::All` leave the corresponding domain
/// fields unset. The default layer is represented by an empty layer set plus
/// `include_default_layer`, matching the domain contract for markups whose
/// `layer_id` is `None`.
pub(crate) fn filter_from_state(state: &UiState) -> MarkupListFilter {
    let mut filter = MarkupListFilter::default();
    let search = state.markup_search.trim();
    if !search.is_empty() {
        filter.search = Some(search.to_string());
    }
    if let Some(status) = state.markup_status {
        filter.statuses = Some([status].into_iter().collect());
    }
    if let Some(author) = &state.markup_author {
        filter.authors = Some([author.clone()].into_iter().collect());
    }
    if let Some(subject) = &state.markup_subject {
        filter.subjects = Some([subject.clone()].into_iter().collect());
    }
    if let Some(kind) = state.markup_kind {
        filter.kinds = Some([kind].into_iter().collect());
    }
    if let Some(page) = state.markup_page {
        filter.pages = Some([page].into_iter().collect());
    }
    match state.markup_layer {
        MarkupLayerFilter::All => {}
        MarkupLayerFilter::DefaultLayer => {
            filter.layers = Some(HashSet::new());
            filter.include_default_layer = true;
        }
        MarkupLayerFilter::Layer(id) => {
            filter.layers = Some([id].into_iter().collect());
        }
    }
    filter
}

/// Case-insensitive, sorted dedupe of non-empty values. The first spelling of
/// a value wins, so "Alice" is preferred over "alice".
fn unique_sorted<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for value in values {
        let value = value.as_ref().trim();
        if value.is_empty() {
            continue;
        }
        map.entry(value.to_lowercase())
            .or_insert_with(|| value.to_string());
    }
    map.into_values().collect()
}

fn status_color(status: ReviewStatus) -> egui::Color32 {
    let c = status.color();
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Status colour for *text*: the bright swatch on the dark chrome (where it
/// already reads), and the darkened [`ReviewStatus::text_color`] on the light
/// chrome, where the swatch fails WCAG AA (the amber is only ≈ 1.9:1 on white).
fn status_text_color(status: ReviewStatus, dark: bool) -> egui::Color32 {
    let c = if dark {
        status.color()
    } else {
        status.text_color()
    };
    egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Keep a table cell from stretching the panel on a long value.
fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut out: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Draw the Markups List panel. Returns a selection request when a row is
/// clicked; the app owns selection/page changes and their snapshot policy.
pub(crate) fn draw_markups_list(
    ui: &mut Ui,
    project: &mut Project,
    state: &mut UiState,
) -> (Option<MarkupListAction>, bool) {
    let mut status_changed = false;

    let Some(doc) = project.active() else {
        ui.heading("Markups List");
        ui.separator();
        ui.label("No active document.");
        return (None, false);
    };

    let layer_names: HashMap<Uuid, String> =
        doc.layers.iter().map(|l| (l.id, l.name.clone())).collect();
    let layer_name = |id: Uuid| {
        layer_names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| id.to_string())
    };

    let all = MarkupList::from_document(doc);
    let authors = unique_sorted(all.rows().iter().map(|r| r.author.as_str()));
    let subjects = unique_sorted(all.rows().iter().map(|r| r.subject.as_str()));
    let default_layer_name = doc
        .default_layer()
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "Default".to_string());

    ui.horizontal(|ui| {
        ui.heading("Markups List");
        let plural = if all.len() == 1 { "" } else { "s" };
        ui.label(format!("{} markup{plural}", all.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Clear filters").clicked() {
                state.markup_search.clear();
                state.markup_status = None;
                state.markup_author = None;
                state.markup_subject = None;
                state.markup_kind = None;
                state.markup_page = None;
                state.markup_layer = MarkupLayerFilter::All;
                state.status = "Markup filters cleared".to_string();
            }
        });
    });
    ui.separator();

    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.markup_search)
                .hint_text("Search author, subject, summary")
                .desired_width(170.0),
        );

        ComboBox::from_id_salt("markup_status_filter")
            .selected_text(
                state
                    .markup_status
                    .map(|s| s.label())
                    .unwrap_or("All statuses"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_status, None, "All statuses");
                for status in ReviewStatus::ALL {
                    ui.selectable_value(&mut state.markup_status, Some(status), status.label());
                }
            });

        ComboBox::from_id_salt("markup_author_filter")
            .selected_text(state.markup_author.as_deref().unwrap_or("All authors"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_author, None, "All authors");
                for author in &authors {
                    ui.selectable_value(
                        &mut state.markup_author,
                        Some(author.clone()),
                        author.as_str(),
                    );
                }
            });

        ComboBox::from_id_salt("markup_subject_filter")
            .selected_text(state.markup_subject.as_deref().unwrap_or("All subjects"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_subject, None, "All subjects");
                for subject in &subjects {
                    ui.selectable_value(
                        &mut state.markup_subject,
                        Some(subject.clone()),
                        subject.as_str(),
                    );
                }
            });

        ComboBox::from_id_salt("markup_kind_filter")
            .selected_text(state.markup_kind.map(|k| k.label()).unwrap_or("All kinds"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_kind, None, "All kinds");
                for kind in MarkupKind::ALL {
                    ui.selectable_value(&mut state.markup_kind, Some(kind), kind.label());
                }
            });

        ComboBox::from_id_salt("markup_page_filter")
            .selected_text(match state.markup_page {
                Some(p) => format!("Page {}", p + 1),
                None => "All pages".to_string(),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_page, None, "All pages");
                for page in 0..doc.pages.len() {
                    ui.selectable_value(
                        &mut state.markup_page,
                        Some(page),
                        format!("Page {}", page + 1),
                    );
                }
            });

        ComboBox::from_id_salt("markup_layer_filter")
            .selected_text(match state.markup_layer {
                MarkupLayerFilter::All => "All layers".to_string(),
                MarkupLayerFilter::DefaultLayer => {
                    format!("{default_layer_name} (default)")
                }
                MarkupLayerFilter::Layer(id) => layer_name(id),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.markup_layer,
                    MarkupLayerFilter::All,
                    "All layers",
                );
                ui.selectable_value(
                    &mut state.markup_layer,
                    MarkupLayerFilter::DefaultLayer,
                    format!("{default_layer_name} (default)"),
                );
                for layer in doc.layers.iter().skip(1) {
                    ui.selectable_value(
                        &mut state.markup_layer,
                        MarkupLayerFilter::Layer(layer.id),
                        layer.name.as_str(),
                    );
                }
            });

        ComboBox::from_id_salt("markup_sort_filter")
            .selected_text(state.markup_sort.label())
            .show_ui(ui, |ui| {
                for sort in MarkupListSort::ALL {
                    ui.selectable_value(&mut state.markup_sort, sort, sort.label());
                }
            });

        ComboBox::from_id_salt("markup_group_filter")
            .selected_text(state.markup_group.map(|key| key.label()).unwrap_or("None"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.markup_group, None, "None");
                for key in MarkupGroupKey::ALL {
                    ui.selectable_value(&mut state.markup_group, Some(key), key.label());
                }
            });
    });
    ui.separator();

    let filter = filter_from_state(state);
    let mut filtered = all.filter(&filter);
    filtered.sort(state.markup_sort);

    ui.horizontal(|ui| {
        let plural = if filtered.len() == 1 { "" } else { "s" };
        ui.small(format!(
            "Showing {} markup{plural} of {}",
            filtered.len(),
            all.len()
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Export CSV…").clicked() {
                export_csv(project, &filtered, state);
            }
        });
    });

    // Batch status for multi-select (v0.4-02).
    let selected_count = state.selected.len();
    if selected_count > 1 {
        ui.horizontal(|ui| {
            ui.label(format!("{} selected:", selected_count));
            ComboBox::from_id_salt("batch_status_list")
                .selected_text("Change status…")
                .show_ui(ui, |ui| {
                    for s in ReviewStatus::ALL.iter() {
                        ui.horizontal(|ui| {
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                            ui.painter()
                                .circle_filled(rect.center(), 5.0, status_color(*s));
                            if ui.selectable_label(false, s.label()).clicked() {
                                for id in &state.selected {
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
                                status_changed = true;
                                state.status = format!(
                                    "Batch status → {} for {} markup(s)",
                                    s.label(),
                                    selected_count
                                );
                            }
                        });
                    }
                });
        });
        ui.separator();
    }

    if filtered.is_empty() {
        if all.is_empty() {
            ui.label("No markups yet. Draw a markup on the canvas to see it here.");
        } else {
            ui.label("No markups match the current filters.");
        }
        return (None, status_changed);
    }

    let list_action = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_list(
                ui,
                &filtered,
                project,
                state,
                &layer_name,
                &default_layer_name,
                &mut status_changed,
            )
        })
        .inner;

    // Measurement totals footer over the FILTERED view.
    render_totals_footer(ui, project, &filtered);

    (list_action, status_changed)
}

fn render_list(
    ui: &mut Ui,
    list: &MarkupList,
    project: &mut Project,
    state: &mut UiState,
    layer_name: &dyn Fn(Uuid) -> String,
    default_layer_name: &str,
    status_changed: &mut bool,
) -> Option<MarkupListAction> {
    match state.markup_group {
        Some(key) => {
            for (label, group) in list.group(key, layer_name) {
                // Keep the domain label as the widget identity; only the
                // visible text changes for the default layer.
                let shown = if key == MarkupGroupKey::Layer && label == NONE_LABEL {
                    format!("{default_layer_name} (default)")
                } else {
                    label.clone()
                };
                let mut action = None;
                egui::CollapsingHeader::new(format!("{} ({})", shown, group.len()))
                    .id_salt(format!("markup-group-{label}"))
                    .default_open(true)
                    .show(ui, |ui| {
                        action = render_rows(ui, &group, project, state, &label, status_changed);
                    });
                if action.is_some() {
                    return action;
                }
            }
            None
        }
        None => render_rows(ui, list, project, state, "all", status_changed),
    }
}

fn render_rows(
    ui: &mut Ui,
    list: &MarkupList,
    project: &mut Project,
    state: &mut UiState,
    id_salt: &str,
    status_changed: &mut bool,
) -> Option<MarkupListAction> {
    let mut action = None;
    let dark = ui.visuals().dark_mode;
    egui::Grid::new(format!("markups-grid-{id_salt}"))
        .num_columns(6)
        .striped(true)
        .spacing(egui::vec2(12.0, 3.0))
        .show(ui, |ui| {
            for header in ["Page", "Type", "Author", "Subject", "Summary", "Status"] {
                ui.label(RichText::new(header).strong());
            }
            ui.end_row();

            for row in list.rows() {
                let selected = state.selected.contains(&row.id);
                let mut clicked = false;
                clicked |= ui
                    .selectable_label(selected, (row.page + 1).to_string())
                    .on_hover_text(format!("Page {}", row.page + 1))
                    .clicked();
                clicked |= ui
                    .selectable_label(selected, row.kind.label())
                    .on_hover_text(row.kind.label())
                    .clicked();
                clicked |= ui
                    .selectable_label(selected, truncate(row.author.as_str(), 24))
                    .on_hover_text(row.author.as_str())
                    .clicked();
                clicked |= ui
                    .selectable_label(selected, truncate(row.subject.as_str(), 24))
                    .on_hover_text(row.subject.as_str())
                    .clicked();
                clicked |= ui
                    .selectable_label(selected, truncate(row.summary.as_str(), 40))
                    .on_hover_text(row.summary.as_str())
                    .clicked();

                // Inline status editor. Locked (or otherwise non-editable)
                // markups show a dimmed, read-only label.
                let editable = project
                    .active()
                    .and_then(|d| d.find_markup(row.id))
                    .map(|m| {
                        project
                            .active()
                            .map(|d| d.is_markup_editable(m))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if editable {
                    let mut new_status = row.status;
                    ComboBox::from_id_salt(format!("markup-row-status-{}", row.id))
                        .selected_text(row.status.label())
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
                                        status_color(*s),
                                    );
                                    if ui.selectable_label(row.status == *s, s.label()).clicked() {
                                        new_status = *s;
                                    }
                                });
                            }
                        });
                    if new_status != row.status {
                        if let Some(doc) = project.active_mut() {
                            doc.apply_status_to(&[row.id], new_status);
                        }
                        *status_changed = true;
                        state.status = format!("Status → {} for 1 markup", new_status.label());
                    }
                } else {
                    ui.label(
                        RichText::new(row.status.label())
                            .color(status_text_color(row.status, dark))
                            .weak(),
                    );
                }
                ui.end_row();

                if clicked {
                    action = Some(MarkupListAction {
                        id: row.id,
                        page: row.page,
                    });
                }
            }
        });
    action
}

fn render_totals_footer(ui: &mut Ui, project: &Project, filtered: &MarkupList) {
    ui.separator();
    let Some(doc) = project.active() else {
        return;
    };
    let mut by_kind: BTreeMap<&'static str, (MeasureKind, f64, MeasureUnit)> = BTreeMap::new();
    let mut total_measurements = 0usize;
    let mut count_markers = 0usize;
    for row in filtered.rows() {
        let Some(markup) = doc.markups.iter().find(|m| m.id == row.id) else {
            continue;
        };
        let Some(meas) = &markup.measurement else {
            continue;
        };
        total_measurements += 1;
        if markup.kind == MarkupKind::MeasureCount {
            count_markers += 1;
        }
        let scale = doc
            .pages
            .get(row.page)
            .map(|p| p.scale)
            .unwrap_or(Scale::PAPER);
        let converted = match meas.kind {
            MeasureKind::Angle | MeasureKind::Count => meas.value_points,
            MeasureKind::Area => meas.value_points * scale.units_per_point * scale.units_per_point,
            _ => scale.points_to_units(meas.value_points),
        };
        let entry = by_kind
            .entry(meas.kind.label())
            .or_insert((meas.kind, 0.0, scale.unit));
        entry.1 += converted;
    }
    ui.horizontal_wrapped(|ui| {
        for (kind, sum, unit) in by_kind.values() {
            let label = match kind {
                MeasureKind::Length => "Total length".to_string(),
                MeasureKind::Area => "Total area".to_string(),
                MeasureKind::Count => format!("Count: {} markers", *sum as usize),
                _ => format!("Total {}", kind.label()),
            };
            if matches!(kind, MeasureKind::Count) {
                ui.label(label.to_string());
            } else {
                let text = crate::domain::units::format_value_precise(
                    *kind,
                    *unit,
                    *sum,
                    project.settings.precision,
                );
                ui.label(format!("{label}: {text}"));
            }
        }
        if count_markers > 0 && !by_kind.contains_key(MeasureKind::Count.label()) {
            ui.label(format!("Count: {} markers", count_markers));
        }
        ui.label(format!("Measurements: {}", total_measurements));
    });
}

fn export_csv(project: &Project, filtered: &MarkupList, state: &mut UiState) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("markups.csv")
        .save_file()
    else {
        return;
    };
    let Some(doc) = project.active() else {
        state.status = "No active document.".to_string();
        return;
    };
    let mut out = String::new();
    out.push_str("Page,Kind,Author,Subject,Status,Measurement,Comments\n");
    for row in filtered.rows() {
        let markup = doc.markups.iter().find(|m| m.id == row.id);
        let measurement = markup
            .and_then(|m| m.measurement.as_ref())
            .map(|me| {
                let scale = doc
                    .pages
                    .get(row.page)
                    .map(|p| p.scale)
                    .unwrap_or(Scale::PAPER);
                me.label_with(scale.unit, project.settings.precision)
            })
            .unwrap_or_default();
        let comments = markup
            .map(|m| {
                m.comments
                    .iter()
                    .map(|c| c.text.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();
        let fields = [
            (row.page + 1).to_string(),
            row.kind.label().to_string(),
            row.author.clone(),
            row.subject.clone(),
            row.status.label().to_string(),
            measurement,
            comments,
        ];
        let line: Vec<String> = fields.iter().map(csv_escape).collect();
        out.push_str(&line.join(","));
        out.push('\n');
    }
    match std::fs::write(&path, out) {
        Ok(_) => state.status = format!("Exported {} row(s) to {}", filtered.len(), path.display()),
        Err(e) => state.status = format!("CSV export failed: {e}"),
    }
}

fn csv_escape(value: &String) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_from_state_translates_every_control() {
        let mut state = UiState::default();
        state.markup_search = "  roof  ".to_string();
        state.markup_status = Some(ReviewStatus::Approved);
        state.markup_author = Some("Alice".to_string());
        state.markup_subject = Some("Roof".to_string());

        let filter = filter_from_state(&state);
        assert_eq!(filter.search.as_deref(), Some("roof"));
        assert!(filter
            .statuses
            .as_ref()
            .is_some_and(|s| s.contains(&ReviewStatus::Approved)));
        assert!(filter.authors.as_ref().is_some_and(|s| s.contains("Alice")));
        assert!(filter.subjects.as_ref().is_some_and(|s| s.contains("Roof")));
        assert!(filter.layers.is_none());
        assert!(!filter.include_default_layer);
    }

    #[test]
    fn filter_from_state_maps_default_and_named_layers() {
        let mut state = UiState::default();
        state.markup_layer = MarkupLayerFilter::DefaultLayer;
        let filter = filter_from_state(&state);
        assert!(filter.layers.as_ref().is_some_and(|s| s.is_empty()));
        assert!(filter.include_default_layer);

        let id = Uuid::new_v4();
        state.markup_layer = MarkupLayerFilter::Layer(id);
        let filter = filter_from_state(&state);
        assert!(filter.layers.as_ref().is_some_and(|s| s.contains(&id)));
        assert!(!filter.include_default_layer);
    }

    #[test]
    fn blank_search_and_all_layers_are_unset() {
        let state = UiState::default();
        let filter = filter_from_state(&state);
        assert!(filter.search.is_none());
        assert!(filter.statuses.is_none());
        assert!(filter.authors.is_none());
        assert!(filter.subjects.is_none());
        assert!(filter.layers.is_none());
        assert!(!filter.include_default_layer);
    }

    #[test]
    fn unique_sorted_dedupes_case_insensitively_and_drops_blanks() {
        let values = unique_sorted(["Alice", "alice", "", "  ", "Bob"]);
        assert_eq!(values, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn truncate_preserves_short_values_and_marks_long_ones() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghij", 5), "abcd…");
    }
}
