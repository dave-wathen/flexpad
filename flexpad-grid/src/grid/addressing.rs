use std::ops::Range;

/// A [`RowCol`] represents the row and column address of a cell within a [`Grid`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowCol {
    pub row: u32,
    pub column: u32,
}

impl RowCol {
    /// A [`RowCol`] representing the top-left cell of a [`Grid`].
    pub const TOP_LEFT: RowCol = RowCol::new(0, 0);

    /// Create a new [`RowCol`].
    pub const fn new(row: u32, column: u32) -> Self {
        Self { row, column }
    }

    /// Returns the row range for this [`RowCol`]
    pub fn rows(&self) -> Range<u32> {
        (self.row)..(self.row + 1)
    }

    /// Returns the column range for this [`RowCol`]
    pub fn columns(&self) -> Range<u32> {
        (self.column)..(self.column + 1)
    }
}

impl std::ops::Add<(u32, u32)> for RowCol {
    type Output = Self;

    /// Adds a row and column offset to a [`RowCol`] producing a new [`RowCol`].
    fn add(self, addend: (u32, u32)) -> Self {
        Self {
            row: self.row + addend.0,
            column: self.column + addend.1,
        }
    }
}

impl std::fmt::Debug for RowCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(r:{}, c:{})", self.row, self.column)
    }
}

impl From<(u32, u32)> for RowCol {
    fn from(value: (u32, u32)) -> Self {
        Self {
            row: value.0,
            column: value.1,
        }
    }
}

/// A [`CellRange`] represents a contiguous block of cells in a [`Grid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellRange {
    pub start: RowCol,
    pub end: RowCol,
}

impl CellRange {
    pub fn new<RC1: Into<RowCol>, RC2: Into<RowCol>>(start: RC1, end: RC2) -> Self {
        let start = start.into();
        let end = end.into();

        debug_assert!(end.row >= start.row, "Start row cannot be after end row");
        debug_assert!(
            end.column >= start.column,
            "Start column cannot be after end column"
        );

        Self { start, end }
    }

    pub fn new_single<RC: Into<RowCol>>(rc: RC) -> Self {
        let rc = rc.into();
        Self::new(rc, rc)
    }

    /// Determines if there is an intersection between two [`CellRange`]s.
    pub fn intersects(&self, other: &CellRange) -> bool {
        let CellRange {
            start: rs1,
            end: re1,
        } = self;
        let CellRange {
            start: rs2,
            end: re2,
        } = other;
        (rs1.row <= re2.row && re1.row >= rs2.row)
            && (rs1.column <= re2.column && re1.column >= rs2.column)
    }

    /// Returns the row range for this [`CellRange`]
    pub fn rows(&self) -> Range<u32> {
        (self.start.row)..(self.end.row + 1)
    }

    /// Returns the column range for this [`CellRange`]
    pub fn columns(&self) -> Range<u32> {
        (self.start.column)..(self.end.column + 1)
    }

    /// Returns the number of [`RowCol`]s in this [`CellRange`]
    pub fn count(&self) -> usize {
        self.rows().count() * self.columns().count()
    }

    /// Returns an iterator of the individual [`RowCol]s in this [`CellRange`]
    pub fn iter(&self) -> impl Iterator<Item = RowCol> + '_ {
        let mut rw = self.start.row;
        let mut cl = self.start.column;

        std::iter::from_fn(move || {
            if rw <= self.end.row {
                let rc = RowCol::new(rw, cl);

                cl += 1;
                if cl > self.end.column {
                    rw += 1;
                    cl = self.start.column;
                }

                Some(rc)
            } else {
                None
            }
        })
    }
}

impl<RC: Into<RowCol>> From<RC> for CellRange {
    fn from(value: RC) -> Self {
        Self::new_single(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rowcol_adds_rows() {
        let base = RowCol::new(10, 20);
        let down = base + (5, 0);
        assert_eq!(15, down.row);
        assert_eq!(20, down.column);
    }

    #[test]
    fn rowcol_adds_columns() {
        let base = RowCol::new(10, 20);
        let across = base + (0, 20);
        assert_eq!(10, across.row);
        assert_eq!(40, across.column);
    }

    #[test]
    fn cellrange_intersects() {
        fn assert_intersects(r1: impl Into<CellRange>, r2: impl Into<CellRange>) {
            let r1 = r1.into();
            let r2 = r2.into();
            assert!(r1.intersects(&r2), "{r1:?} should intersect {r2:?}");
            assert!(r2.intersects(&r1), "{r2:?} should intersect {r1:?}");
        }
        fn assert_not_intersects(r1: impl Into<CellRange>, r2: impl Into<CellRange>) {
            let r1 = r1.into();
            let r2 = r2.into();
            assert!(!r1.intersects(&r2), "{r1:?} shouldn't intersect {r2:?}");
            assert!(!r2.intersects(&r1), "{r2:?} shouldn't intersect {r1:?}");
        }

        // single/single
        assert_intersects((10, 20), (10, 20));
        assert_not_intersects((10, 20), (10, 21));
        assert_not_intersects((10, 20), (9, 20));

        // range/single
        assert_intersects(CellRange::new((10, 20), (10, 25)), (10, 20));
        assert_intersects(CellRange::new((10, 20), (10, 25)), (10, 22));
        assert_intersects(CellRange::new((10, 20), (10, 25)), (10, 25));
        assert_intersects(CellRange::new((10, 20), (15, 20)), (10, 20));
        assert_intersects(CellRange::new((10, 20), (15, 20)), (12, 20));
        assert_intersects(CellRange::new((10, 20), (15, 20)), (15, 20));
        assert_intersects(CellRange::new((10, 20), (15, 25)), (12, 22));
        assert_not_intersects(CellRange::new((10, 20), (10, 25)), (10, 19));
        assert_not_intersects(CellRange::new((10, 20), (10, 25)), (10, 26));
        assert_not_intersects(CellRange::new((10, 20), (10, 25)), (9, 22));
        assert_not_intersects(CellRange::new((10, 20), (10, 25)), (11, 22));
        assert_not_intersects(CellRange::new((10, 20), (15, 20)), (9, 20));
        assert_not_intersects(CellRange::new((10, 20), (15, 20)), (16, 20));
        assert_not_intersects(CellRange::new((10, 20), (15, 20)), (12, 19));
        assert_not_intersects(CellRange::new((10, 20), (15, 20)), (12, 21));

        // range/range
        assert_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 20), (10, 25)),
        );
        assert_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 22), (10, 25)),
        );
        assert_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 18), (10, 25)),
        );
        assert_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 18), (10, 28)),
        );
        assert_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((10, 20), (15, 20)),
        );
        assert_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((12, 20), (17, 20)),
        );
        assert_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((8, 20), (13, 20)),
        );
        assert_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((8, 20), (18, 20)),
        );
        assert_intersects(
            CellRange::new((10, 20), (15, 25)),
            CellRange::new((8, 18), (18, 28)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 14), (10, 19)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((10, 26), (10, 31)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((9, 18), (9, 28)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (10, 25)),
            CellRange::new((11, 18), (11, 28)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((4, 20), (9, 20)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((16, 20), (21, 20)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((10, 19), (15, 19)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (15, 20)),
            CellRange::new((10, 21), (15, 21)),
        );
        assert_not_intersects(
            CellRange::new((10, 20), (15, 25)),
            CellRange::new((4, 14), (9, 19)),
        );
    }

    #[test]
    fn range_iteration_single() {
        let range = CellRange::new_single((1, 2));
        assert_eq!(1, range.count());

        let mut iter = range.iter();
        assert_eq!(Some(RowCol::new(1, 2)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn range_iteration_multple() {
        let range = CellRange::new((1, 2), (3, 4));
        assert_eq!(9, range.count());

        let mut iter = range.iter();
        assert_eq!(Some(RowCol::new(1, 2)), iter.next());
        assert_eq!(Some(RowCol::new(1, 3)), iter.next());
        assert_eq!(Some(RowCol::new(1, 4)), iter.next());
        assert_eq!(Some(RowCol::new(2, 2)), iter.next());
        assert_eq!(Some(RowCol::new(2, 3)), iter.next());
        assert_eq!(Some(RowCol::new(2, 4)), iter.next());
        assert_eq!(Some(RowCol::new(3, 2)), iter.next());
        assert_eq!(Some(RowCol::new(3, 3)), iter.next());
        assert_eq!(Some(RowCol::new(3, 4)), iter.next());
        assert_eq!(None, iter.next());
    }
}
