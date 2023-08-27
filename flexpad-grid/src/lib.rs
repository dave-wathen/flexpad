type Renderer<Theme = iced::Theme> = iced::Renderer<Theme>;

mod borders;
mod grid;
mod sequence;

pub use borders::{Border, Borders};
pub use grid::addressing::{CellRange, RowCol};
pub use grid::cell::GridCell;
pub use grid::head::{ColumnHead, GridCorner, RowHead};
pub use grid::scroll::{self, GridScrollable, Viewport};
pub use grid::style::{self, Appearance, StyleSheet};
pub use grid::Grid;
pub use sequence::SumSeq;

// TODO scrollable
