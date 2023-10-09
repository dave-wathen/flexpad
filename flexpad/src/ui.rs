use crate::version::Version;
use iced::widget::{button, column, container, horizontal_rule, image, row, text};
use iced::{
    alignment, font, theme, window, Alignment, Application, Command, Length, Settings, Theme,
};
use tracing::info;

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
    Loading,
    FrontScreen,
    Workpad(WorkpadUI),
}

pub struct Flexpad {
    version: Version,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),
    OpenNewWokpad,
    WorkpadMsg(WorkpadMessage),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::FontLoaded(result) => write!(f, "Message::WorkpadMessage({result:?})"),
            Message::OpenNewWokpad => write!(f, "Message::OpenNewWokpad"),
            Message::WorkpadMsg(msg) => write!(f, "Message::WorkpadMessage({msg})"),
        }
    }
}

const SPACE_S: u16 = 5;
const SPACE_M: u16 = SPACE_S * 2;
// const SPACE_L: u16 = SPACE_S * 3;
// const SPACE_XL: u16 = SPACE_S * 4;

// TODO Can we avoid a constant width via layouts
const DIALOG_BUTTON_WIDTH: u16 = 100;

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
            Command::batch(vec![
                font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(Message::FontLoaded),
                window::maximize(true),
            ]),
        )
    }

    fn title(&self) -> String {
        match self.state {
            State::Workpad(ref pad) => pad.title(),
            _ => "Flexpad".to_owned(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        info!(
            %message,
            "Flexpad update"
        );
        match self.state {
            State::Loading => {
                if let Message::FontLoaded(result) = message {
                    match result {
                        Ok(_) => self.state = State::FrontScreen,
                        Err(err) => panic!("{err:?}"),
                    }
                }
                Command::none()
            }
            State::FrontScreen => match message {
                Message::OpenNewWokpad => {
                    let workpad = WorkpadMaster::new();
                    self.state = State::Workpad(WorkpadUI::new(workpad));
                    Command::none()
                }
                _ => Command::none(),
            },
            State::Workpad(ref mut pad) => match message {
                Message::WorkpadMsg(WorkpadMessage::PadClose) => {
                    self.state = State::FrontScreen;
                    Command::none()
                }
                Message::WorkpadMsg(msg) => pad.update(msg).map(Message::WorkpadMsg),
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        match self.state {
            State::Loading => container(
                text("Loading...")
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .size(50),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .center_x()
            .into(),
            State::FrontScreen => self.front_screen_view(),
            State::Workpad(ref pad) => pad.view().map(Message::WorkpadMsg),
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
        .spacing(SPACE_S)
        .padding(50)
        .align_items(Alignment::Center)
        .into()
    }
}
