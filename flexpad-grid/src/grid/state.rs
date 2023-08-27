use std::rc::Rc;

use crate::SumSeq;

/// The local state of a [`Grid`].
pub struct GridState {
    pub row_heights: Rc<SumSeq>,
    pub column_widths: Rc<SumSeq>,
}

impl GridState {
    pub(crate) fn new(row_heights: Rc<SumSeq>, column_widths: Rc<SumSeq>) -> Self {
        Self {
            row_heights,
            column_widths,
        }
    }
}
