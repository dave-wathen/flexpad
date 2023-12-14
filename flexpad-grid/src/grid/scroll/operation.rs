use crate::{scroll::state::GridScrollableState, RowCol, Viewport};
use iced::{
    advanced::widget::{self, operation::Outcome},
    Rectangle,
};
use tracing::debug;

pub fn scroll_to_cell(target: widget::Id, cell: RowCol) -> impl widget::Operation<Viewport> {
    struct ScrollToCell {
        target: widget::Id,
        cell: RowCol,
        viewport: Option<Viewport>,
    }

    impl widget::Operation<Viewport> for ScrollToCell {
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
                state.scroll_to_column(self.cell.column);
                state.scroll_to_row(self.cell.row);
                self.viewport = Some(state.viewport());
            }
        }

        fn finish(&self) -> Outcome<Viewport> {
            match self.viewport {
                Some(viewport) => {
                    debug!(target: "flexpad_grid", result=%viewport, "operation"="ScrollToCell", "Operation");
                    Outcome::Some(viewport)
                }
                None => {
                    debug!(target: "flexpad_grid", result="Not found", "operation"="ScrollToCell", "Operation");
                    Outcome::None
                }
            }
        }
    }

    ScrollToCell {
        target,
        cell,
        viewport: None,
    }
}

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
            match self.viewport {
                Some(viewport) => {
                    debug!(target: "flexpad_grid", result=%viewport, "operation"="EnsureCellVisible", "Operation");
                    Outcome::Some(viewport)
                }
                None => {
                    debug!(target: "flexpad_grid", result="Not found", "operation"="EnsureCellVisible", "Operation");
                    Outcome::None
                }
            }
        }
    }

    EnsureCellVisible {
        target,
        cell,
        viewport: None,
    }
}

pub fn get_viewport(target: widget::Id) -> impl widget::Operation<Viewport> {
    struct GetViewport {
        target: widget::Id,
        viewport: Option<Viewport>,
    }

    impl widget::Operation<Viewport> for GetViewport {
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
                    .downcast_ref::<GridScrollableState>()
                    .expect("Downcast widget state");
                self.viewport = Some(state.viewport());
            }
        }

        fn finish(&self) -> Outcome<Viewport> {
            match self.viewport {
                Some(viewport) => {
                    debug!(target: "flexpad_grid", result=%viewport, "operation"="GetViewport", "Operation");
                    Outcome::Some(viewport)
                }
                None => {
                    debug!(target: "flexpad_grid", result="Not found", "operation"="GetViewport", "Operation");
                    Outcome::None
                }
            }
        }
    }

    GetViewport {
        target,
        viewport: None,
    }
}
