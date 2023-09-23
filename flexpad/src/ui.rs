use crate::version::Version;
use iced::widget::{button, column, horizontal_rule, image, row, text};
use iced::{alignment, theme, window, Alignment, Application, Command, Length, Settings, Theme};

use self::workpad::{WorkpadMessage, WorkpadUI};
use crate::model::workpad::WorkpadMaster;

mod images;
mod workpad;

pub(crate) fn run() -> iced::Result {
    let settings = Settings::default();
    Flexpad::run(settings)
}

#[derive(Default)]
enum State {
    #[default]
    FrontScreen,
    Workpad(WorkpadUI),
}

pub struct Flexpad {
    version: Version,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenNewWokpad,
    WorkpadMessage(WorkpadMessage),
}

impl Application for Flexpad {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Flexpad, Command<Message>) {
        (
            Self {
                version: Default::default(),
                state: Default::default(),
            },
            window::maximize(true),
        )
    }

    fn title(&self) -> String {
        match self.state {
            State::FrontScreen => "Flexpad".to_owned(),
            State::Workpad(ref pad) => pad.title(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match self.state {
            State::FrontScreen => match message {
                Message::OpenNewWokpad => {
                    let workpad = WorkpadMaster::new();
                    self.state = State::Workpad(WorkpadUI::new(workpad));
                    Command::none()
                }
                _ => Command::none(),
            },
            State::Workpad(ref mut pad) => match message {
                Message::WorkpadMessage(WorkpadMessage::PadClose) => {
                    self.state = State::FrontScreen;
                    Command::none()
                }
                Message::WorkpadMessage(msg) => pad.update(msg).map(Message::WorkpadMessage),
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        match self.state {
            State::FrontScreen => self.front_screen_view(),
            State::Workpad(ref pad) => pad.view().map(Message::WorkpadMessage),
        }
    }
}

impl Flexpad {
    fn front_screen_view(&self) -> iced::Element<'_, Message> {
        column![
            image(images::app()).width(200).height(200),
            text(self.version.description()).size(12),
            horizontal_rule(3),
            text("Create New ...")
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
            row![column![
                button(image(images::workpad()))
                    .on_press(Message::OpenNewWokpad)
                    .style(theme::Button::Text),
                text("Workpad").size(12)
            ]
            .align_items(Alignment::Center)]
            .width(Length::Fill),
            horizontal_rule(3),
            text("Reopen ...")
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
        ]
        .width(Length::Fill)
        .spacing(5)
        .padding(50)
        .align_items(Alignment::Center)
        .into()
    }
}
