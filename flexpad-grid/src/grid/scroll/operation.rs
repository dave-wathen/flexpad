use crate::{scroll::state::GridScrollableState, RowCol, Viewport};
use iced::{
    advanced::widget::{self, operation::Outcome},
    Rectangle,
};

pub fn ensure_cell_visible(target: widget::Id, cell: RowCol) -> impl widget::Operation<Viewport> {
    struct EnsureCellVisible {
        target: widget::Id,
        cell: RowCol,
        viewport: Option<Viewport>,
    }

    impl widget::Operation<Viewport> for EnsureCellVisible {
        fn container(
            &mut self,
            _id: Option<&widget::Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn widget::Operation<Viewport>),
        ) {
            operate_on_children(self)
        }

        fn custom(&mut self, state: &mut dyn std::any::Any, id: Option<&widget::Id>) {
            if id == Some(&self.target) {
                let state = state
                    .downcast_mut::<GridScrollableState>()
                    .expect("Downcast widget state");
                state.ensure_column_visible(self.cell.column);
                state.ensure_row_visible(self.cell.row);
                self.viewport = Some(state.viewport());
            }
        }

        fn finish(&self) -> Outcome<Viewport> {
            Outcome::Some(self.viewport.expect("Viewport missing"))
        }
    }

    EnsureCellVisible {
        target,
        cell,
        viewport: None,
    }
}
