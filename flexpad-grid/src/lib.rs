type Renderer<Theme = iced::Theme> = iced::Renderer<Theme>;

mod borders;
mod grid;
mod lengths;

pub use borders::{Border, Borders};
pub use grid::addressing::{CellRange, RowCol};
pub use grid::cell::GridCell;
pub use grid::head::{ColumnHead, GridCorner, RowHead};
pub use grid::scroll::GridScrollable;
pub use grid::style::{self, Appearance, StyleSheet};
pub use grid::Grid;
pub use lengths::Lengths;

// TODO scrollable