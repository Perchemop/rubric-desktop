//! Pure page-navigation helpers used by the top bar, the Pages menu, and the
//! keyboard shortcuts.  Keeping the step logic in the domain makes the
//! previous/next ordering, bounds checking, and disabled state testable
//! headlessly and guarantees all callers agree on the semantics.

use serde::{Deserialize, Serialize};

/// Navigation direction for page steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageStep {
    /// Move to the previous page (decrement index).
    Previous,
    /// Move to the next page (increment index).
    Next,
}

/// Compute the target page index for a navigation step.
///
/// Returns `Some(target_index)` when the step is valid within the document
/// bounds, or `None` when the step would leave the document (first page for
/// `Previous`, last page for `Next`, or empty document).  The function is
/// pure and testable, so the top bar, menu and keyboard shortcuts all share
/// the same ordering semantics.
///
/// # Bounds
///
/// - `page_count == 0` → always `None`.
/// - `current >= page_count` is treated as an invalid state and produces
///   `None` – callers should clamp `current` before calling.
/// - `Previous` is valid only when `current > 0`.
/// - `Next` is valid only when `current + 1 < page_count`.
///
/// # Examples
///
/// ```
/// use crate::domain::nav::{PageStep, page_nav_target};
/// assert_eq!(page_nav_target(0, 5, PageStep::Previous), None);
/// assert_eq!(page_nav_target(0, 5, PageStep::Next), Some(1));
/// assert_eq!(page_nav_target(4, 5, PageStep::Next), None);
/// ```
pub fn page_nav_target(current: usize, page_count: usize, step: PageStep) -> Option<usize> {
    if page_count == 0 {
        return None;
    }
    if current >= page_count {
        return None;
    }

    match step {
        PageStep::Previous => current.checked_sub(1),
        PageStep::Next => {
            let next = current + 1;
            if next < page_count {
                Some(next)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_nav_target_empty_document() {
        assert_eq!(page_nav_target(0, 0, PageStep::Previous), None);
        assert_eq!(page_nav_target(0, 0, PageStep::Next), None);
        // Out-of-range current also returns None
        assert_eq!(page_nav_target(5, 0, PageStep::Previous), None);
    }

    #[test]
    fn page_nav_target_single_page() {
        assert_eq!(page_nav_target(0, 1, PageStep::Previous), None);
        assert_eq!(page_nav_target(0, 1, PageStep::Next), None);
    }

    #[test]
    fn page_nav_target_first_page() {
        assert_eq!(page_nav_target(0, 5, PageStep::Previous), None);
        assert_eq!(page_nav_target(0, 5, PageStep::Next), Some(1));
    }

    #[test]
    fn page_nav_target_middle_page() {
        assert_eq!(page_nav_target(2, 5, PageStep::Previous), Some(1));
        assert_eq!(page_nav_target(2, 5, PageStep::Next), Some(3));
    }

    #[test]
    fn page_nav_target_last_page() {
        assert_eq!(page_nav_target(4, 5, PageStep::Previous), Some(3));
        assert_eq!(page_nav_target(4, 5, PageStep::Next), None);
    }

    #[test]
    fn page_nav_target_invalid_current() {
        // Current equal to count – treated as invalid
        assert_eq!(page_nav_target(5, 5, PageStep::Previous), None);
        assert_eq!(page_nav_target(5, 5, PageStep::Next), None);
        // Current past the end – also invalid
        assert_eq!(page_nav_target(10, 5, PageStep::Previous), None);
        assert_eq!(page_nav_target(10, 5, PageStep::Next), None);
    }

    #[test]
    fn page_step_serialization_roundtrip() {
        let cases = [PageStep::Previous, PageStep::Next];
        for step in cases {
            let s = serde_json::to_string(&step).unwrap();
            let de: PageStep = serde_json::from_str(&s).unwrap();
            assert_eq!(de, step);
        }
    }
}
