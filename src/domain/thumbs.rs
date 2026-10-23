//! Page-thumbnail grid geometry for the pages sidebar (spec F-404).
//!
//! The sidebar is pure layout: how many cells fit per row, and how tall a cell
//! must be. Keeping it in the domain means the packing rules are testable
//! without a display.

/// Grid geometry for the page-thumbnail sidebar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThumbLayout {
    /// Number of cells that fit on one row. Always at least 1.
    pub per_row: usize,
    /// Number of rows needed for `count` pages (0 when there are no pages).
    pub rows: usize,
    /// Width of one cell, in points. Wide panels add columns rather than
    /// stretching cells, so thumbnails stay a predictable size.
    pub cell_w: f64,
    /// Height of one cell, in points (thumbnail box plus caption).
    pub cell_h: f64,
    /// Height reserved for the caption under each thumbnail.
    pub caption_h: f64,
}

/// Compute a thumbnail layout that fits `count` pages into `width`.
///
/// `min_cell` is the smallest useful cell width and `gap` the gutter between
/// cells. When `width` cannot hold one cell, the cell is clamped to `min_cell`
/// so the sidebar still renders something usable instead of collapsing to zero.
pub fn thumb_layout(width: f64, count: usize, min_cell: f64, gap: f64) -> ThumbLayout {
    let gap = gap.max(0.0);
    let min_cell = min_cell.max(1.0);
    let usable = width.max(min_cell);

    // Largest column count whose minimum footprint still fits.
    let per_row = ((usable + gap) / (min_cell + gap)).floor().max(1.0) as usize;
    // Leftover width is shared out, so the gap stays exact.
    let cell_w = (usable - gap * (per_row - 1) as f64) / per_row as f64;
    let caption_h = 16.0;
    let rows = if count == 0 {
        0
    } else {
        count.div_ceil(per_row)
    };

    ThumbLayout {
        per_row,
        rows,
        cell_w,
        cell_h: cell_w * 0.75 + caption_h,
        caption_h,
    }
}

/// Width needed for `columns` cells of at least `min_cell` with `gap` gutters.
/// Used by the tests (and readable in the UI) as the "one more column would
/// not fit" oracle.
pub fn columns_needed(columns: usize, min_cell: f64, gap: f64) -> f64 {
    let columns = columns.max(1) as f64;
    columns * min_cell.max(1.0) + (columns - 1.0) * gap.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_at_least_one_column_even_in_a_tiny_panel() {
        for w in [0.0, 1.0, 40.0, 96.0] {
            let l = thumb_layout(w, 5, 96.0, 6.0);
            assert_eq!(l.per_row, 1, "width {w} must still fit one column");
            assert!(l.cell_w > 0.0);
        }
    }

    #[test]
    fn one_more_column_would_not_fit() {
        for w in [100.0, 200.0, 320.0, 640.0, 1024.0] {
            let l = thumb_layout(w, 12, 96.0, 6.0);
            assert!(
                columns_needed(l.per_row + 1, 96.0, 6.0) > w,
                "width {w} fits {}+1 columns of 96pt + 6pt gaps",
                l.per_row
            );
            // The chosen columns do fit.
            assert!(
                columns_needed(l.per_row, 96.0, 6.0) <= w.max(96.0),
                "chosen {}/9 columns do not fit width {w}",
                l.per_row
            );
        }
    }

    #[test]
    fn columns_never_shrink_as_the_panel_widens() {
        let mut last = 0;
        for w in (0..1400).step_by(7) {
            let l = thumb_layout(w as f64, 1, 96.0, 6.0);
            assert!(l.per_row >= last, "columns must not shrink at width {w}");
            last = l.per_row;
        }
    }

    #[test]
    fn rows_grow_only_as_far_as_the_count_needs() {
        let l = thumb_layout(210.0, 5, 96.0, 6.0);
        assert_eq!(l.per_row, 2);
        assert_eq!(l.rows, 3, "5 pages over 2 columns = 3 rows");

        let exact = thumb_layout(210.0, 4, 96.0, 6.0);
        assert_eq!(exact.rows, 2);
        assert_eq!(thumb_layout(210.0, 1, 96.0, 6.0).rows, 1);
        assert_eq!(thumb_layout(210.0, 0, 96.0, 6.0).rows, 0);
    }

    #[test]
    fn cells_reserve_room_for_the_caption() {
        let l = thumb_layout(200.0, 1, 96.0, 6.0);
        assert_eq!(l.caption_h, 16.0);
        assert_eq!(l.cell_h, l.cell_w * 0.75 + l.caption_h);
        // Thumbnail box keeps roughly a page aspect ratio.
        assert!((l.cell_w * 0.75 - (l.cell_h - l.caption_h)).abs() < 1e-6);
    }

    #[test]
    fn gap_is_never_negative_and_width_is_shared_exactly() {
        let l = thumb_layout(300.0, 9, 90.0, 0.0);
        assert_eq!(l.per_row, 3);
        assert_eq!(l.cell_w, 100.0);
        assert_eq!(l.rows, 3);
        // Total occupied width equals the panel width with no gap left over.
        assert_eq!(l.cell_w * l.per_row as f64, 300.0);
    }
}
