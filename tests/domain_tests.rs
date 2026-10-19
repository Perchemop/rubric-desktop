//! Domain tests: geometry, measurements, hit testing, serialization round-trips.

use rubric::domain::{
    geom::{angle_degrees, path_length, polygon_area},
    style::palette,
    units::MeasureUnit,
    Document, Markup, MarkupGeometry, MarkupKind, Page, Project, Pt, Rect, ReviewStatus, Scale,
};

fn foot_scale() -> Scale {
    Scale::new(MeasureUnit::Foot, 1.0 / 72.0)
}

#[test]
fn document_lifecycle() {
    let mut doc = Document::empty("Spec Sheet");
    assert_eq!(doc.page_count(), 1);
    doc.add_page(Page::letter(1));
    assert_eq!(doc.page_count(), 2);
    assert_eq!(doc.primary_page().unwrap().width, 595.28); // A4 first; but letter added
    doc.remove_page(0);
    assert_eq!(doc.page_count(), 1);
    assert_eq!(doc.primary_page().unwrap().width, 612.0);
}

#[test]
fn markup_serialization_roundtrip() {
    let geom = MarkupGeometry::Segment {
        a: Pt::new(0.0, 0.0),
        b: Pt::new(100.0, 0.0),
    };
    let m = Markup::new(0, MarkupKind::MeasureLength, geom, "reviewer");
    let json = serde_json::to_string(&m).expect("json");
    let m2: Markup = serde_json::from_str(&json).expect("from_json");
    assert_eq!(m.kind, m2.kind);
    assert_eq!(m.page, m2.page);
    assert_eq!(m.bounds(), m2.bounds());
}

#[test]
fn geometry_calculations() {
    let pts = vec![
        Pt::new(0.0, 0.0),
        Pt::new(72.0, 0.0),
        Pt::new(72.0, 72.0),
        Pt::new(0.0, 72.0),
    ];
    assert!((path_length(&pts) - 216.0).abs() < 1e-4);
    assert!((polygon_area(&pts) - 5184.0).abs() < 1e-4);
    let ang = angle_degrees(Pt::new(0.0, 0.0), Pt::new(1.0, 0.0), Pt::new(1.0, 1.0));
    assert!((ang - 90.0).abs() < 1e-3);
}

#[test]
fn measure_length_and_radius() {
    let mut mlen = Markup::new(
        0,
        MarkupKind::MeasureLength,
        MarkupGeometry::Segment {
            a: Pt::new(0.0, 0.0),
            b: Pt::new(144.0, 0.0),
        },
        "t",
    );
    mlen.recompute_measurement(foot_scale());
    // 144 pt = 2 ft
    assert!((mlen.measurement.unwrap().value - 2.0).abs() < 1e-6);
    let mut mr = Markup::new(
        0,
        MarkupKind::MeasureRadius,
        MarkupGeometry::Segment {
            a: Pt::new(0.0, 0.0),
            b: Pt::new(144.0, 0.0),
        },
        "t",
    );
    mr.recompute_measurement(foot_scale());
    assert!((mr.measurement.unwrap().value - 1.0).abs() < 1e-6); // half
}

#[test]
fn review_status_and_style_defaults() {
    let m = Markup::new(
        0,
        MarkupKind::Highlight,
        MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 10.0, 10.0)),
        "a",
    );
    assert_eq!(m.status, ReviewStatus::None);
    assert_eq!(m.style.stroke, palette::YELLOW);
}

#[test]
fn project_save_load_roundtrip() {
    let mut proj = Project::new("Test Project");
    let mut doc = Document::empty("Doc1");
    let m = Markup::new(
        0,
        MarkupKind::Rectangle,
        MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 50.0, 40.0)),
        "a",
    );
    doc.add_markup(m);
    proj.add_document(doc);
    let json = serde_json::to_string_pretty(&proj).expect("ser");
    let proj2: Project = serde_json::from_str(&json).expect("de");
    assert_eq!(proj2.name, "Test Project");
    assert_eq!(proj2.documents.len(), 2);
    assert_eq!(proj2.active_document, 1);
}

fn rect_markup() -> Markup {
    Markup::new(
        0,
        MarkupKind::Rectangle,
        MarkupGeometry::Bounds(Rect::new(0.0, 0.0, 50.0, 40.0)),
        "a",
    )
}

#[test]
fn legacy_project_without_layer_data_loads_default_layer() {
    let mut proj = Project::new("Legacy");
    let mut doc = Document::empty("Doc");
    doc.add_markup(rect_markup());
    proj.add_document(doc);

    let mut value = serde_json::to_value(&proj).expect("json");
    let docs = value
        .get_mut("documents")
        .and_then(serde_json::Value::as_array_mut)
        .expect("documents");
    for doc in docs {
        let obj = doc.as_object_mut().expect("document");
        obj.remove("layers");
        if let Some(markups) = obj
            .get_mut("markups")
            .and_then(serde_json::Value::as_array_mut)
        {
            for markup in markups {
                if let Some(markup) = markup.as_object_mut() {
                    markup.remove("layer_id");
                }
            }
        }
    }

    let loaded: Project = serde_json::from_value(value).expect("deserialize legacy");
    let active = loaded.active().expect("active document");
    assert_eq!(active.layers.len(), 1);
    let default = active.default_layer_id();
    assert_eq!(active.markups.len(), 1);
    assert_eq!(active.markups[0].layer_id, None);
    assert_eq!(active.layer_id_for(&active.markups[0]), Some(default));
    assert_eq!(active.markups_on_layer(default).len(), 1);
}

#[test]
fn project_layers_and_membership_roundtrip() {
    let mut proj = Project::new("Layered");
    let layer = {
        let doc = proj.active_mut().expect("active document");
        let id = doc.add_layer("Markups");
        doc.set_layer_locked(id, true);
        doc.add_markup(rect_markup().with_layer(id));
        id
    };

    let json = serde_json::to_string_pretty(&proj).expect("ser");
    let back: Project = serde_json::from_str(&json).expect("de");
    let doc = back.active().expect("active document");
    assert_eq!(doc.layers.len(), 2);
    assert_eq!(doc.layer(layer).map(|l| l.locked), Some(true));
    assert_eq!(doc.markups_on_layer(layer).len(), 1);
    assert!(
        doc.is_markup_visible(&doc.markups[0]),
        "locked stays visible"
    );
    assert!(!doc.is_markup_editable(&doc.markups[0]));
}
