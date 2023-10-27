use iced::{
    alignment::Horizontal,
    event::Status,
    keyboard::{self, KeyCode, Modifiers},
    subscription, theme,
    widget::{button, horizontal_space, row, text},
    Event, Length, Subscription,
};
use rust_i18n::t;
use tracing::warn;

use super::{DIALOG_BUTTON_WIDTH, SPACE_M};

/// Actions for dialogs/forms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Ok,
    Cancel,
}

impl Action {
    fn key_subscription(&self) -> Subscription<Action> {
        match self {
            Action::Ok => subscription::events_with(|event, status| {
                check_event(
                    event,
                    status,
                    KeyCode::Enter,
                    Modifiers::empty(),
                    Action::Ok,
                )
            }),
            Action::Cancel => subscription::events_with(|event, status| {
                check_event(
                    event,
                    status,
                    KeyCode::Escape,
                    Modifiers::empty(),
                    Action::Cancel,
                )
            }),
        }
    }

    fn style(&self) -> theme::Button {
        match self {
            Action::Ok => theme::Button::Primary,
            Action::Cancel => theme::Button::Secondary,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Action::Ok)
    }

    pub fn is_cancel(&self) -> bool {
        matches!(self, Action::Cancel)
    }
}

fn check_event(
    event: Event,
    status: Status,
    key_code: KeyCode,
    modifiers: Modifiers,
    action: Action,
) -> Option<Action> {
    let sub_event = Event::Keyboard(keyboard::Event::KeyPressed {
        key_code,
        modifiers,
    });

    if status == Status::Ignored && event == sub_event {
        Some(action)
    } else {
        None
    }
}

/// A set of [`Action`]s
pub struct ActionSet {
    actions: Vec<Action>,
    names: Vec<String>,
}

impl ActionSet {
    /// Create an ['ActionSet'] of ['Action::Cancel'] and ['Action::Ok']
    pub fn cancel_ok() -> ActionSet {
        ActionSet {
            actions: vec![Action::Cancel, Action::Ok],
            names: vec![t!("Common.Cancel"), t!("Common.Ok")],
        }
    }

    /// Create an ['ActionSet'] of only ['Action::Ok']
    pub fn ok() -> ActionSet {
        ActionSet {
            actions: vec![Action::Ok],
            names: vec![t!("Common.Ok")],
        }
    }

    /// Create an ['ActionSet'] of only ['Action::Cancel']
    #[allow(dead_code)]
    pub fn cancel() -> ActionSet {
        ActionSet {
            actions: vec![Action::Cancel],
            names: vec![t!("Common.Cancel")],
        }
    }

    /// Change the text on the OK button
    pub fn ok_text(mut self, text: impl ToString) -> Self {
        match self.actions.iter().position(|a| *a == Action::Ok) {
            Some(idx) => self.names[idx] = text.to_string(),
            None => warn!("Trying to set text for OK when not included in the set"),
        }
        self
    }

    /// Change the text on the Cancel button
    #[allow(dead_code)]
    pub fn cancel_text(mut self, text: impl ToString) -> Self {
        match self.actions.iter().position(|a| *a == Action::Cancel) {
            Some(idx) => self.names[idx] = text.to_string(),
            None => warn!("Trying to set text for Cancel when not included in the set"),
        }
        self
    }

    /// Return a subscription to handle keystrokes for this [`ActionSet`]
    pub fn to_subscription(&self) -> Subscription<Action> {
        Subscription::batch(self.actions.iter().map(Action::key_subscription))
    }

    /// Convert the [`ButtonBar`] into an [`Element`]
    pub fn to_element<'a>(&self) -> iced::Element<'a, Action> {
        let mut buttons = row![];

        for (action, txt) in self.actions.iter().zip(self.names.iter()) {
            buttons = buttons.push(
                button(text(txt).horizontal_alignment(Horizontal::Center))
                    .width(DIALOG_BUTTON_WIDTH)
                    .style(action.style())
                    .on_press(*action),
            );
        }

        row![horizontal_space(Length::Fill), buttons.spacing(SPACE_M)]
            .width(Length::Fill)
            .into()
    }
}
