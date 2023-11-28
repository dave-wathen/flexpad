use std::cell::RefCell;

#[derive(Debug)]
struct MenuState {
    menu_profile: Vec<bool>,
    active: Option<usize>,
    focused: bool,
}

#[derive(Debug)]
// The states of all menus below the menu bar indexed by their depth:
//   0 is only the menu bar is visible,
//   1 the menu bar and menus immediately below it are visible,
//   2 the menu bar, menus immediately below it, and their immediate submenus are visible,
//   etc.
pub(super) struct MenuStates {
    states: RefCell<Vec<MenuState>>,
}

impl MenuStates {
    pub(super) fn new() -> Self {
        Self {
            states: RefCell::new(vec![]),
        }
    }

    pub(super) fn depth(&self) -> usize {
        self.states.borrow().len()
    }

    pub(super) fn clear_after(&self, depth: usize) {
        let mut states = self.states.borrow_mut();
        if depth == 0 {
            states.clear();
        } else {
            states.truncate(depth);
        }
    }

    pub(super) fn push(&self, menu_profile: Vec<bool>) {
        let mut states = self.states.borrow_mut();
        states.push(MenuState {
            menu_profile,
            active: None,
            focused: false,
        });
    }

    pub(super) fn focused(&self) -> usize {
        self.states
            .borrow()
            .iter()
            .position(|s| s.focused)
            .map(|i| i + 1)
            .unwrap_or(0)
    }

    pub(super) fn focus_on(&self, to: usize) {
        let focused = self.focused();
        let depth = self.depth();
        debug_assert!(
            to <= depth,
            "Invalid state for focus_on(): to: {}, depth: {}",
            to,
            depth
        );

        if focused != to {
            let mut states = self.states.borrow_mut();
            if focused > 0 {
                let menu_state = states.get_mut(focused - 1).unwrap();
                menu_state.focused = false;
            }
            let menu_state = states.get_mut(to - 1).unwrap();
            menu_state.focused = true;
        }
    }

    pub(super) fn focus_next(&self, focus_first: bool) {
        let focused = self.focused();
        let depth = self.depth();
        debug_assert!(
            focused < depth,
            "Invalid state for focus_next(): focused: {}, depth: {}",
            focused,
            depth
        );

        let mut states = self.states.borrow_mut();
        if focused > 0 {
            let menu_state = states.get_mut(focused - 1).unwrap();
            menu_state.focused = false;
        }
        let menu_state = states.get_mut(focused).unwrap();
        menu_state.focused = true;
        menu_state.active = if focus_first {
            menu_state.menu_profile.iter().position(|x| *x)
        } else {
            None
        };
    }

    pub(super) fn focus_previous(&self) {
        let focused = self.focused();
        debug_assert!(
            focused > 1,
            "Invalid state for focus_previous(): focused: {}, depth: {}",
            focused,
            self.depth()
        );

        let mut states = self.states.borrow_mut();
        let menu_state = states.get_mut(focused - 2).unwrap();
        menu_state.focused = true;
        states.truncate(focused - 1);
    }

    pub(super) fn move_up(&self) {
        let focused = self.focused();
        debug_assert!(
            focused > 0,
            "Should not call move_up when no menus visible with focus",
        );

        let mut states = self.states.borrow_mut();
        let menu_state = states.get_mut(focused - 1).unwrap();
        if let Some(mut active) = menu_state.active {
            let len = menu_state.menu_profile.len();
            active = (active + len - 1) % len;
            while !menu_state.menu_profile[active] {
                active = (active + len - 1) % len;
            }
            menu_state.active = Some(active);
        }
    }

    pub(super) fn move_down(&self) {
        let focused = self.focused();
        debug_assert!(
            focused > 0,
            "Should not call move_down when no menus visible with focus",
        );

        let mut states = self.states.borrow_mut();
        let menu_state = states.get_mut(focused - 1).unwrap();
        if let Some(mut active) = menu_state.active {
            let len = menu_state.menu_profile.len();
            active = (active + 1) % len;
            while !menu_state.menu_profile[active] {
                active = (active + 1) % len;
            }
            menu_state.active = Some(active);
        }
    }

    pub(super) fn move_to(&self, index: usize) {
        let focused = self.focused();
        debug_assert!(
            focused > 0,
            "Should not call move_down when no menus visible with focus",
        );

        let mut states = self.states.borrow_mut();
        let menu_state = states.get_mut(focused - 1).unwrap();
        menu_state.active = Some(index);
    }

    pub(super) fn clear_active(&self) {
        let focused = self.focused();
        debug_assert!(
            focused > 0,
            "Should not call move_down when no menus visible with focus",
        );

        let mut states = self.states.borrow_mut();
        let menu_state = states.get_mut(focused - 1).unwrap();
        menu_state.active = None;
    }

    pub(super) fn selected(&self, depth: usize) -> Option<usize> {
        let states = self.states.borrow();

        debug_assert!(
            depth <= states.len(),
            "Invalid depth: {} of {}",
            depth,
            states.len()
        );
        if depth == 0 || depth > states.len() {
            None
        } else {
            let menu_state = states.get(depth - 1).unwrap();
            menu_state.active
        }
    }
}
