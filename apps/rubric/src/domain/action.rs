//! Editor actions exposed by context menus (spec F-603).
//!
//! A context menu is a *view* of editor commands. The canvas and the
//! inspector render the same menu, so the command set, its labels and the
//! rules for when an entry is applicable live here rather than being
//! duplicated in the UI.
use crate::domain::markup::ReviewStatus;

/// A user-invocable editor action offered by the canvas and inspector
/// context menus (spec F-603).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAction {
    Undo,
    Redo,
    DeleteSelection,
    ClearSelection,
    CopyProperties,
    PasteProperties,
    BringForward,
    SendBackward,
    BringToFront,
    SendToBack,
    ZoomIn,
    ZoomOut,
    FitPage,
    BatchStatus(ReviewStatus),
}

impl EditorAction {
    /// Every action, in menu order.
    pub const ALL: [EditorAction; 14] = [
        EditorAction::Undo,
        EditorAction::Redo,
        EditorAction::DeleteSelection,
        EditorAction::ClearSelection,
        EditorAction::CopyProperties,
        EditorAction::PasteProperties,
        EditorAction::BringForward,
        EditorAction::SendBackward,
        EditorAction::BringToFront,
        EditorAction::SendToBack,
        EditorAction::ZoomIn,
        EditorAction::ZoomOut,
        EditorAction::FitPage,
        EditorAction::BatchStatus(ReviewStatus::Open),
    ];

    /// Human-readable menu label.
    pub fn label(self) -> &'static str {
        match self {
            EditorAction::Undo => "Undo",
            EditorAction::Redo => "Redo",
            EditorAction::DeleteSelection => "Delete Selected",
            EditorAction::ClearSelection => "Clear Selection",
            EditorAction::CopyProperties => "Copy Properties",
            EditorAction::PasteProperties => "Paste Properties",
            EditorAction::BringForward => "Bring Forward",
            EditorAction::SendBackward => "Send Backward",
            EditorAction::BringToFront => "Bring to Front",
            EditorAction::SendToBack => "Send to Back",
            EditorAction::ZoomIn => "Zoom In",
            EditorAction::ZoomOut => "Zoom Out",
            EditorAction::FitPage => "Fit Page",
            EditorAction::BatchStatus(status) => status.label(),
        }
    }

    /// Whether the action does anything given the current selection and
    /// whether a style has been copied.
    ///
    /// Deleting, clearing or pasting with nothing selected is a no-op, so those
    /// entries are disabled. Copying and z-order need exactly one selected
    /// markup. The history and view actions are always offered: undo/redo
    /// simply do nothing on an empty history, and zoom always applies.
    pub fn is_enabled(self, selection_count: usize, has_style_clipboard: bool) -> bool {
        match self {
            EditorAction::DeleteSelection | EditorAction::ClearSelection => selection_count > 0,
            EditorAction::CopyProperties
            | EditorAction::BringForward
            | EditorAction::SendBackward
            | EditorAction::BringToFront
            | EditorAction::SendToBack => selection_count == 1,
            EditorAction::PasteProperties => selection_count > 0 && has_style_clipboard,
            EditorAction::BatchStatus(_) => selection_count > 0,
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
        assert_eq!(EditorAction::ALL.len(), 13);
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
                !a.is_enabled(0, false),
                "{a:?} should be disabled with no selection"
            );
            assert!(
                a.is_enabled(1, false),
                "{a:?} should be enabled with a selection"
            );
        }
    }

    #[test]
    fn copy_and_z_order_require_exactly_one_selection() {
        let single = [
            EditorAction::CopyProperties,
            EditorAction::BringForward,
            EditorAction::SendBackward,
            EditorAction::BringToFront,
            EditorAction::SendToBack,
        ];
        for a in single {
            assert!(!a.is_enabled(0, true), "{a:?} needs a selection");
            assert!(a.is_enabled(1, true), "{a:?} needs one selection");
            assert!(!a.is_enabled(3, true), "{a:?} needs exactly one");
        }
    }

    #[test]
    fn paste_requires_a_selection_and_a_copied_style() {
        let paste = EditorAction::PasteProperties;
        assert!(!paste.is_enabled(0, false));
        assert!(!paste.is_enabled(1, false), "nothing has been copied");
        assert!(!paste.is_enabled(0, true), "nothing is selected");
        assert!(paste.is_enabled(1, true));
        assert!(paste.is_enabled(3, true));
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
                a.is_enabled(0, false),
                "{a:?} should stay enabled with no selection"
            );
            assert!(
                a.is_enabled(3, true),
                "{a:?} should stay enabled with a selection"
            );
        }
    }
}
