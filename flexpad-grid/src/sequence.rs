const COMPACT_SIZE: usize = 10;
type Repeat = (u32, f32);
type CompactArray = [Repeat; COMPACT_SIZE];

/// This represents a sequence of numbers (f32) for which it is possible to obtain the
/// sum of all values upto any element.  It also allows search by a value to determine
/// the element whose sum is the greatest value less than or equal to the given value.
///
/// This type is used to represent the column widths and row heights of a [`Grid`].
///
/// # Example usage
///
/// ```
/// # use flexpad_grid::SumSeq;
/// #
///
/// let mut seq = SumSeq::new();
/// seq.push(10.0);
/// seq.push_many(4, 15.0);
/// assert_eq!(5, seq.len());
/// assert_eq!(70.0, seq.sum());
/// ```
#[derive(Debug, Clone)]
// TODO Tree representation for larger sequences
// TODO Insert
// TODO Delete
pub struct SumSeq {
    data: Representation,
}

impl SumSeq {
    pub fn new() -> Self {
        Self {
            data: Representation::Compact([(0, 0.0); 10]),
        }
    }

    pub fn push(&mut self, value: f32) {
        self.push_many(1, value);
    }

    pub fn push_many(&mut self, repeat: u32, value: f32) {
        match self.data {
            Representation::Compact(ref mut values) => {
                for index in 0..COMPACT_SIZE {
                    if values[index].0 == 0 {
                        values[index] = (repeat, value);
                        return;
                    }
                    if values[index].1 == value
                        && index + 1 < COMPACT_SIZE
                        && values[index + 1].0 == 0
                    {
                        values[index].0 += repeat;
                        return;
                    }
                }
                todo!("Representation is full - switch to tree")
            }
            Representation::Tree => todo!(),
        }
    }

    /// Returns the number of terms in this [`Sequence`]
    pub fn len(&self) -> usize {
        match self.data {
            Representation::Compact(values) => values.iter().map(|(n, _)| *n as usize).sum(),
            Representation::Tree => todo!(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the sum of the terms in this [`Sequence`]
    pub fn sum(&self) -> f32 {
        match self.data {
            Representation::Compact(values) => values.iter().map(|(n, v)| (*n as f32) * v).sum(),
            Representation::Tree => todo!(),
        }
    }

    /// Returns the sum of the terms in this [`Sequence`] whose index is less than `end`.
    /// If `end >= len()` the sum of the whole sequence is returned
    pub fn sum_to(&self, end: usize) -> f32 {
        match self.data {
            Representation::Compact(values) => {
                let mut remaining = end;
                let mut sum = 0.0;
                for repeat in values {
                    if repeat.0 as usize <= remaining {
                        sum += (repeat.0 as f32) * repeat.1;
                        remaining -= repeat.0 as usize;
                    } else if remaining > 0 {
                        sum += (remaining as f32) * repeat.1;
                        remaining = 0;
                    }
                }
                sum
            }
            Representation::Tree => todo!(),
        }
    }

    /// Given a value this returns the index of the element whose sum it most closely represents.
    /// For example the sequence [10, 15, 20] can be mapped to the sums [10, 25, 45].  A given
    /// value isseacrched for in this space so 5 would equate to Some(0) and 20 to Some(1).
    ///
    /// If the search value exactly matches a value in the sum sequence it can be viewed as
    /// the last value in the search or as the first value of the subseqent.  In this circumstance
    /// the rounding determines which.  So 25 rounded Rounding::Down is Some(1) whilst 25
    /// rounded Rounding::Up is Some(2).
    ///
    /// If the search value is greater than the last term None is returned. None will also be returned
    /// if the sequence is empty or if the search value is 0.0 and rounding is Rounding::Down.
    pub fn index_of_sum(&self, sum: f32, rounding: Rounding) -> Option<usize> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        match self.data {
            Representation::Compact(values) => {
                let mut remaining = sum;
                let mut index: isize = -1;
                for repeat in values {
                    let repeat_sum = (repeat.0 as f32) * repeat.1;
                    if repeat_sum <= remaining {
                        index += repeat.0 as isize;
                        remaining -= repeat_sum;
                    } else if remaining > 0.0 {
                        let full_units = (remaining / repeat.1).floor();
                        remaining -= full_units * repeat.1;
                        index += full_units as isize;
                        break;
                    }
                }
                if remaining > 0.0 || rounding == Rounding::Up {
                    index += 1;
                }
                let index = index as usize;
                if index < len {
                    Some(index)
                } else {
                    None
                }
            }
            Representation::Tree => todo!(),
        }
    }

    /// Returns an iterator of the values in this [`SumSeq`]
    pub fn values(&self) -> impl Iterator<Item = f32> {
        match self.data {
            Representation::Compact(values) => Iter::Compact(CompactIter::new(values)),
            Representation::Tree => todo!(),
        }
    }
}

impl Default for SumSeq {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Rounding {
    #[default]
    Down,
    Up,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Representation {
    Compact(CompactArray),
    Tree,
}

enum Iter {
    Compact(CompactIter),
}

impl Iterator for Iter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Compact(ci) => ci.next(),
        }
    }
}

struct CompactIter {
    values: [Repeat; 10],
    index: usize,
    emitted: u32,
}

impl CompactIter {
    fn new(values: CompactArray) -> Self {
        Self {
            values,
            index: 0,
            emitted: 0,
        }
    }

    fn next(&mut self) -> Option<f32> {
        while self.index < COMPACT_SIZE && self.emitted >= self.values[self.index].0 {
            self.index += 1;
            self.emitted = 0;
        }
        if self.index >= COMPACT_SIZE {
            None
        } else {
            self.emitted += 1;
            Some(self.values[self.index].1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sequence() {
        let seq = SumSeq::new();
        assert_eq!(0, seq.len());
        assert_eq!(0.0, seq.sum());

        let mut iter = seq.values();
        assert_eq!(None, iter.next());
    }

    #[test]
    fn one_element() {
        let mut seq = SumSeq::new();
        seq.push(20.0);

        assert_eq!(1, seq.len());
        assert_eq!(20.0, seq.sum());
        assert_eq!(vec![20.0], seq.values().collect::<Vec<_>>());
    }

    #[test]
    fn repeated_element() {
        let mut seq = SumSeq::new();
        seq.push(20.0);
        seq.push(20.0);
        assert_compact(&seq, vec![(2, 20.0)]);

        assert_eq!(2, seq.len());
        assert_eq!(40.0, seq.sum());
        assert_eq!(vec![20.0, 20.0], seq.values().collect::<Vec<_>>());
    }

    #[test]
    fn differing_elements() {
        let mut seq = SumSeq::new();
        seq.push(20.0);
        seq.push(25.0);
        seq.push(10.0);
        seq.push(15.0);
        assert_compact(&seq, vec![(1, 20.0), (1, 25.0), (1, 10.0), (1, 15.0)]);

        assert_eq!(4, seq.len());
        assert_eq!(70.0, seq.sum());
        assert_eq!(
            vec![20.0, 25.0, 10.0, 15.0],
            seq.values().collect::<Vec<_>>()
        );
    }

    #[test]
    fn repeated_element_push_many() {
        let mut seq = SumSeq::new();
        seq.push_many(8, 20.0);
        assert_compact(&seq, vec![(8, 20.0)]);

        assert_eq!(8, seq.len());
        assert_eq!(160.0, seq.sum());
        assert_eq!(
            vec![20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0],
            seq.values().collect::<Vec<_>>()
        );
    }

    #[test]
    fn sums_to() {
        let mut seq = SumSeq::new();
        for v in 1..=5 {
            seq.push(v as f32);
        }
        assert_eq!(0.0, seq.sum_to(0));
        assert_eq!(1.0, seq.sum_to(1));
        assert_eq!(3.0, seq.sum_to(2));
        assert_eq!(6.0, seq.sum_to(3));
        assert_eq!(10.0, seq.sum_to(4));
        assert_eq!(15.0, seq.sum_to(5));
    }

    #[test]
    fn index_of_sums() {
        let mut seq = SumSeq::new();
        for v in 1..=5 {
            seq.push(v as f32);
        }
        assert_eq!(None, seq.index_of_sum(0.0, Rounding::Down));
        assert_eq!(Some(0), seq.index_of_sum(0.0, Rounding::Up));
        assert_eq!(Some(0), seq.index_of_sum(0.5, Rounding::Down));
        assert_eq!(Some(0), seq.index_of_sum(0.5, Rounding::Up));
        assert_eq!(Some(0), seq.index_of_sum(1.0, Rounding::Down));
        assert_eq!(Some(1), seq.index_of_sum(1.0, Rounding::Up));
        assert_eq!(Some(1), seq.index_of_sum(1.1, Rounding::Down));
        assert_eq!(Some(1), seq.index_of_sum(1.1, Rounding::Up));
        assert_eq!(Some(1), seq.index_of_sum(3.0, Rounding::Down));
        assert_eq!(Some(2), seq.index_of_sum(3.0, Rounding::Up));
        assert_eq!(Some(2), seq.index_of_sum(3.1, Rounding::Down));
        assert_eq!(Some(2), seq.index_of_sum(3.1, Rounding::Up));
        assert_eq!(Some(2), seq.index_of_sum(6.0, Rounding::Down));
        assert_eq!(Some(3), seq.index_of_sum(6.0, Rounding::Up));
        assert_eq!(Some(3), seq.index_of_sum(10.0, Rounding::Down));
        assert_eq!(Some(4), seq.index_of_sum(10.0, Rounding::Up));
        assert_eq!(Some(4), seq.index_of_sum(15.0, Rounding::Down));
        assert_eq!(None, seq.index_of_sum(15.0, Rounding::Up));
        assert_eq!(None, seq.index_of_sum(20.0, Rounding::Up));
    }

    fn assert_compact(seq: &SumSeq, expected: Vec<Repeat>) {
        assert!(expected.len() <= COMPACT_SIZE, "Expected is too large");
        let mut exp_array = [(0, 0.0); COMPACT_SIZE];
        expected
            .iter()
            .enumerate()
            .for_each(|(i, r)| exp_array[i] = *r);
        if let Representation::Compact(actual) = seq.data {
            assert_eq!(exp_array, actual);
        } else {
            panic!("Expected array representation");
        }
    }
}
