use std::ops::Range;

/// This represents the heights of rows in the grid and also the widths of columns.
/// Conversions allow Lengths to be coded using primitives and tuples of primitives:
///
/// ```
/// # use flexpad_grid::Lengths;
/// #
/// let lengths: Lengths = Lengths::from(20.0); // A single length of 20.0
/// assert_eq!(20.0, lengths.sum());
/// let lengths: Lengths = Lengths::from((3, 20.0)); // 3 lengths of 20.0
/// assert_eq!(60.0, lengths.sum());
/// let lengths: Lengths = Lengths::from((10, (20.0, 25.0))); // 20 lengths alternating between 20.0 and 25.0
/// assert_eq!(450.0, lengths.sum());
/// ```
///
/// Repeats can be used to represent rows/columns that follow a regular pattern.
/// In its simplest form this allows a grid where all rows and columns are the same
/// size to be represented:
///
/// ```ignore
/// # use flexpad_grid::Grid;
/// #
/// // A Grid of 50 rows each 30.0 high and 10 columns each 100.0 wide
/// Grid::new((50, 30.0), (10, 100.0));
/// ```
#[derive(Debug, Clone)]
pub enum Lengths {
    Single(f32),
    Sequence(Vec<Lengths>),
    Repeat(u32, Box<Lengths>),
}

impl Lengths {
    /// Returns the number of lengths represented by this [`Lengths`]
    pub fn count(&self) -> usize {
        match self {
            Lengths::Single(_) => 1,
            Lengths::Sequence(seq) => seq.iter().map(Lengths::count).sum(),
            Lengths::Repeat(n, lens) => *n as usize * lens.count(),
        }
    }

    /// Returns the total length represented by this [`Lengths`]
    pub fn sum(&self) -> f32 {
        match self {
            Lengths::Single(len) => *len,
            Lengths::Sequence(seq) => seq.iter().map(Lengths::sum).sum(),
            Lengths::Repeat(n, lens) => *n as f32 * lens.sum(),
        }
    }

    /// Returns an iterator of the individual lengths represented by this [`Lengths`]
    pub fn lengths<'a>(&'a self) -> Box<dyn Iterator<Item = f32> + 'a> {
        match self {
            Lengths::Single(len) => Box::new(std::iter::once(*len)),
            Lengths::Sequence(seq) => Box::new(seq.iter().flat_map(|l| l.lengths())),
            Lengths::Repeat(n, lens) => match **lens {
                Lengths::Single(len) => Box::new(std::iter::repeat(len).take(*n as usize)),
                _ => Box::new((0..*n).flat_map(|_| lens.lengths())),
            },
        }
    }

    /// calculate the start and end of a subrange if the lengths
    pub fn span(&self, range: Range<u32>) -> (f32, f32) {
        let from_inc = range.start as usize;
        let to_excl = range.end as usize;
        let mut start = 0.0;
        let mut end = 0.0;
        for (i, len) in self.lengths().enumerate().take(to_excl) {
            if i < from_inc {
                start += len;
            }
            end += len;
        }
        (start, end)
    }
}

impl From<f32> for Lengths {
    fn from(value: f32) -> Self {
        Lengths::Single(value)
    }
}

impl<L0, L1> From<(L0, L1)> for Lengths
where
    L0: Into<Lengths>,
    L1: Into<Lengths>,
{
    fn from(value: (L0, L1)) -> Self {
        Lengths::Sequence(vec![value.0.into(), value.1.into()])
    }
}

impl<L0, L1, L2> From<(L0, L1, L2)> for Lengths
where
    L0: Into<Lengths>,
    L1: Into<Lengths>,
    L2: Into<Lengths>,
{
    fn from(value: (L0, L1, L2)) -> Self {
        Lengths::Sequence(vec![value.0.into(), value.1.into(), value.2.into()])
    }
}

impl<L0, L1, L2, L3> From<(L0, L1, L2, L3)> for Lengths
where
    L0: Into<Lengths>,
    L1: Into<Lengths>,
    L2: Into<Lengths>,
    L3: Into<Lengths>,
{
    fn from(value: (L0, L1, L2, L3)) -> Self {
        Lengths::Sequence(vec![
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        ])
    }
}

impl<L> From<(u32, L)> for Lengths
where
    L: Into<Lengths>,
{
    fn from(value: (u32, L)) -> Self {
        Lengths::Repeat(value.0, Box::new(value.1.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_iteration_one() {
        let lengths = Lengths::from(10.0);
        assert_eq!(1, lengths.count());

        let mut iter = lengths.lengths();
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn lengths_iteration_repeat_one() {
        let lengths = Lengths::from((4, 10.0));
        assert_eq!(4, lengths.count());

        let mut iter = lengths.lengths();
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn lengths_iteration_sequence() {
        let lengths = Lengths::from((10.0, 20.0, 30.0));
        assert_eq!(3, lengths.count());

        let mut iter = lengths.lengths();
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(20.0), iter.next());
        assert_eq!(Some(30.0), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn lengths_iteration_repeat_sequence() {
        let lengths = Lengths::from((2, (10.0, 20.0, 30.0)));
        assert_eq!(6, lengths.count());

        let mut iter = lengths.lengths();
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(20.0), iter.next());
        assert_eq!(Some(30.0), iter.next());
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(20.0), iter.next());
        assert_eq!(Some(30.0), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn lengths_iteration_repeat_complex() {
        let lengths = Lengths::from((10.0, (2, (20.0, 30.0, (3, (40.0, 50.0)))), 60.0));
        assert_eq!(18, lengths.count());

        let mut iter = lengths.lengths();
        assert_eq!(Some(10.0), iter.next());
        assert_eq!(Some(20.0), iter.next());
        assert_eq!(Some(30.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(20.0), iter.next());
        assert_eq!(Some(30.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(40.0), iter.next());
        assert_eq!(Some(50.0), iter.next());
        assert_eq!(Some(60.0), iter.next());
        assert_eq!(None, iter.next());
    }
}
