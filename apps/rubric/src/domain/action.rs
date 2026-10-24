//! Editor actions exposed by context menus (spec F-603).
//!
//! A context menu is a *view* of editor commands. The canvas and the
//! inspector render the same menu, so the command set, its labels and the
//! rules for when an entry is applicable live here rather than being
//! duplicated in the UI.

/// A user-invocable editor action offered by the canvas and inspector
/// context menus (spec F-603).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAction {
    Undo,
    Redo,
    DeleteSelection,
    ClearSelection,
    ZoomIn,
    ZoomOut,
    FitPage,
}

impl EditorAction {
    /// Every action, in menu order.
    pub const ALL: [EditorAction; 7] = [
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::DeleteSelection,
        EditorAction::ClearSelection,
        EditorAction::ZoomIn,
        EditorAction::ZoomOut,
        EditorAction::FitPage,
    ];

    /// Human-readable menu label.
    pub fn label(self) -> &'static str {
        match self {
            EditorAction::Undo => "Undo",
            EditorAction::Redo => "Redo",
            EditorAction::DeleteSelection => "Delete Selected",
            EditorAction::ClearSelection => "Clear Selection",
            EditorAction::ZoomIn => "Zoom In",
            EditorAction::ZoomOut => "Zoom Out",
            EditorAction::FitPage => "Fit Page",
        }
    }

    /// Whether the action does anything given the current selection size.
    ///
    /// Deleting or clearing with nothing selected is a no-op, so those two
    /// entries are disabled. The history and view actions are always offered:
    /// undo/redo simply do nothing on an empty history, and zoom always
    /// applies.
    pub fn is_enabled(self, selection_count: usize) -> bool {
        match self {
            EditorAction::DeleteSelection | EditorAction::ClearSelection => selection_count > 0,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_contains_every_variant_exactly_once() {
        assert_eq!(EditorAction::ALL.len(), 7);
        for (i, a) in EditorAction::ALL.iter().enumerate() {
            for b in EditorAction::ALL.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate action in ALL: {a:?}");
            }
        }
    }

    #[test]
    fn labels_are_non_empty_and_unique() {
        let mut seen = HashSet::new();
        for a in EditorAction::ALL {
            let label = a.label();
            assert!(!label.is_empty(), "empty label for {a:?}");
            assert!(seen.insert(label), "duplicate label {label:?}");
        }
    }

    #[test]
    fn selection_actions_require_a_selection() {
        for a in [EditorAction::DeleteSelection, EditorAction::ClearSelection] {
            assert!(
                !a.is_enabled(0),
                "{a:?} should be disabled with no selection"
            );
            assert!(a.is_enabled(1), "{a:?} should be enabled with a selection");
        }
    }

    #[test]
    fn history_and_view_actions_are_always_enabled() {
        let always = [
            EditorAction::Undo,
            EditorAction::Redo,
            EditorAction::ZoomIn,
            EditorAction::ZoomOut,
            EditorAction::FitPage,
        ];
        for a in always {
            assert!(
                a.is_enabled(0),
                "{a:?} should stay enabled with no selection"
            );
            assert!(
                a.is_enabled(3),
                "{a:?} should stay enabled with a selection"
            );
        }
    }
}
