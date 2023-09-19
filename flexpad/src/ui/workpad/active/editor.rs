use iced::widget::text_input::Value;

use crate::ui::workpad::{Move, WorkpadMessage};

use super::cursor::Cursor;

pub struct Editor {
    value: Value,
    mode: Mode,
    edit_value: Value,
    cursor: Cursor,
}

#[allow(dead_code)]
impl Editor {
    pub fn new(value: &str) -> Editor {
        Editor {
            value: Value::new(value),
            mode: Default::default(),
            edit_value: Value::new(""),
            cursor: Default::default(),
        }
    }

    pub fn is_editing(&self) -> bool {
        match self.mode {
            Mode::NotEditing => false,
            Mode::EditingWithTerminalNavigation => true,
            Mode::EditingWithInternalNavigation => true,
        }
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn move_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    pub fn toggle_edit_mode(&mut self) {
        match self.mode {
            Mode::NotEditing => {
                self.edit_value = self.value.clone();
                self.cursor = Cursor::default();
                self.cursor.move_to(self.edit_value.len());
                self.mode = Mode::EditingWithInternalNavigation
            }
            Mode::EditingWithTerminalNavigation => self.mode = Mode::EditingWithInternalNavigation,
            Mode::EditingWithInternalNavigation => self.mode = Mode::EditingWithTerminalNavigation,
        }
    }

    pub fn end_editing(&mut self) -> String {
        let result = self.edit_value.to_string();
        self.mode = Mode::NotEditing;
        result
    }

    pub fn abandon_editing(&mut self) {
        self.mode = Mode::NotEditing;
    }

    pub fn value(&self) -> &Value {
        match self.mode {
            Mode::NotEditing => &self.value,
            Mode::EditingWithTerminalNavigation => &self.edit_value,
            Mode::EditingWithInternalNavigation => &self.edit_value,
        }
    }

    pub fn contents(&self) -> String {
        self.value().to_string()
    }

    pub fn navigate(&mut self, mve: Move) -> Option<WorkpadMessage> {
        match self.mode {
            Mode::NotEditing => Some(WorkpadMessage::ActiveCellMove(mve)),
            Mode::EditingWithTerminalNavigation => {
                let value = self.end_editing();
                Some(WorkpadMessage::Multi(vec![
                    WorkpadMessage::ActiveCellNewValue(value),
                    WorkpadMessage::ActiveCellMove(mve),
                ]))
            }
            Mode::EditingWithInternalNavigation => {
                match mve {
                    Move::Left => self.cursor.move_left(&self.edit_value),
                    Move::Right => self.cursor.move_right(&self.edit_value),
                    Move::JumpLeft => self.cursor.move_left_by_words(&self.edit_value),
                    Move::JumpRight => self.cursor.move_right_by_words(&self.edit_value),
                    _ => {}
                }
                None
            }
        }
    }

    pub fn left(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::Left)
    }

    pub fn right(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::Right)
    }

    pub fn up(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::Up)
    }

    pub fn down(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::Down)
    }

    pub fn jump_left(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::JumpLeft)
    }

    pub fn jump_right(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::JumpRight)
    }

    pub fn jump_up(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::JumpUp)
    }

    pub fn jump_down(&mut self) -> Option<WorkpadMessage> {
        self.navigate(Move::JumpDown)
    }

    pub fn enter(&mut self) -> Option<WorkpadMessage> {
        match self.mode {
            Mode::NotEditing => self.navigate(Move::Down),
            Mode::EditingWithTerminalNavigation => self.navigate(Move::Down),
            Mode::EditingWithInternalNavigation => {
                Some(WorkpadMessage::ActiveCellNewValue(self.end_editing()))
            }
        }
    }

    pub fn back_enter(&mut self) -> Option<WorkpadMessage> {
        match self.mode {
            Mode::NotEditing => self.navigate(Move::Up),
            Mode::EditingWithTerminalNavigation => self.navigate(Move::Up),
            Mode::EditingWithInternalNavigation => {
                Some(WorkpadMessage::ActiveCellNewValue(self.end_editing()))
            }
        }
    }

    pub fn tab(&mut self) -> Option<WorkpadMessage> {
        match self.mode {
            Mode::NotEditing => self.navigate(Move::Right),
            Mode::EditingWithTerminalNavigation => self.navigate(Move::Right),
            Mode::EditingWithInternalNavigation => {
                Some(WorkpadMessage::ActiveCellNewValue(self.end_editing()))
            }
        }
    }

    pub fn back_tab(&mut self) -> Option<WorkpadMessage> {
        match self.mode {
            Mode::NotEditing => self.navigate(Move::Left),
            Mode::EditingWithTerminalNavigation => self.navigate(Move::Left),
            Mode::EditingWithInternalNavigation => {
                Some(WorkpadMessage::ActiveCellNewValue(self.end_editing()))
            }
        }
    }

    pub fn cursor_state(&self) -> super::cursor::State {
        self.cursor.state(&self.edit_value)
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.cursor.selection(&self.edit_value)
    }

    pub fn select_to(&mut self, position: usize) {
        let start = self.cursor.start(&self.edit_value);
        self.cursor.select_range(start, position);
    }

    pub fn select_word_at(&mut self, position: usize) {
        let start = self.edit_value.previous_start_of_word(position);
        let end = self.edit_value.next_end_of_word(position);
        self.cursor.select_range(start, end);
    }

    pub fn select_left(&mut self) {
        self.cursor.select_left(&self.edit_value);
    }

    pub fn select_right(&mut self) {
        self.cursor.select_right(&self.edit_value);
    }

    pub fn select_left_by_words(&mut self) {
        self.cursor.select_left_by_words(&self.edit_value);
    }

    pub fn select_right_by_words(&mut self) {
        self.cursor.select_right_by_words(&self.edit_value);
    }

    pub fn select_all(&mut self) {
        self.cursor.select_all(&self.edit_value);
    }

    pub fn insert(&mut self, character: char) {
        if !self.is_editing() {
            self.edit_value = Value::new("");
            self.cursor = Cursor::default();
            self.mode = Mode::EditingWithTerminalNavigation
        }

        if let Some((left, right)) = self.selection() {
            self.cursor.move_left(&self.edit_value);
            self.edit_value.remove_many(left, right);
        }

        self.edit_value
            .insert(self.cursor.end(&self.edit_value), character);
        self.cursor.move_right(&self.edit_value);
    }

    pub fn paste(&mut self, content: Value) {
        let selection = self.selection();
        let length = content.len();
        if let Some((left, right)) = selection {
            self.cursor.move_left(&self.edit_value);
            self.edit_value.remove_many(left, right);
        }

        self.edit_value
            .insert_many(self.cursor.end(&self.edit_value), content);

        self.cursor.move_right_by_amount(&self.edit_value, length);
    }

    pub fn jump_backspace(&mut self) {
        if self.selection().is_none() {
            self.select_left_by_words();
        }
        self.backspace();
    }

    pub fn backspace(&mut self) {
        match self.selection() {
            Some((start, end)) => {
                self.cursor.move_left(&self.edit_value);
                self.edit_value.remove_many(start, end);
            }
            None => {
                let start = self.cursor.start(&self.edit_value);

                if start > 0 {
                    self.cursor.move_left(&self.edit_value);
                    self.edit_value.remove(start - 1);
                }
            }
        }
    }

    pub fn delete(&mut self) {
        match self.selection() {
            Some(_) => {
                self.backspace();
            }
            None => {
                let end = self.cursor.end(&self.edit_value);

                if end < self.edit_value.len() {
                    self.edit_value.remove(end);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    NotEditing,
    EditingWithTerminalNavigation,
    EditingWithInternalNavigation,
}
