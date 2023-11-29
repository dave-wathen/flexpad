use std::time::Instant;

use crate::version::Version;
use iced::widget::{self, button, column, container, horizontal_space, row, text, Button, Row};
use iced::{
    alignment, keyboard, theme, window, Application, Command, Element, Event, Font, Length,
    Settings, Subscription, Theme,
};
use rust_i18n::t;
use tracing::debug;

use self::workpad::{WorkpadMessage, WorkpadUI};
use crate::model::workpad::WorkpadMaster;

mod dialog;
mod front_screen;
mod images;
mod key;
mod loading;
mod menu;
mod modal;
mod style;
mod workpad;

pub(crate) fn run() -> iced::Result {
    let settings = Settings::default();
    Flexpad::run(settings)
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Loading,
    FrontScreen,
    Workpad(WorkpadUI),
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Loading => write!(f, "Loading"),
            State::FrontScreen => write!(f, "FrontScreen"),
            State::Workpad(_) => write!(f, "Workpad"),
        }
    }
}

// TODO Focus management currently missing from iced - not easy to fake up in the meantime

#[derive(Debug)]
pub struct Flexpad {
    version: Version,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<(), String>),
    NewBlankWorkpad,
    NewStarterWorkpad,
    WorkpadMsg(WorkpadMessage),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message::")?;
        match self {
            Self::Loaded(result) => write!(f, "FontLoaded({result:?})"),
            Self::NewBlankWorkpad => write!(f, "OpenBlankWokpad"),
            Self::NewStarterWorkpad => write!(f, "OpenStarterWokpad"),
            Self::WorkpadMsg(msg) => msg.fmt(f),
        }
    }
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
            Command::batch(vec![
                iced::Command::perform(load(), Message::Loaded), //font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(Message::FontLoaded),
                window::maximize(true),
            ]),
        )
    }

    fn title(&self) -> String {
        match self.state {
            State::Workpad(ref pad) => pad.title(),
            _ => t!("Product"),
        }
    }

    #[tracing::instrument(target = "flexpad", skip_all)]
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match self.state {
            State::Loading => {
                debug!(target: "flexpad", %message);
                if let Message::Loaded(result) = message {
                    match result {
                        Ok(_) => self.state = State::FrontScreen,
                        Err(err) => panic!("{err:?}"),
                    }
                }
                Command::none()
            }
            State::FrontScreen => match message {
                Message::NewBlankWorkpad => {
                    debug!(target: "flexpad", %message);
                    let workpad = WorkpadMaster::new_blank();
                    self.state = State::Workpad(WorkpadUI::new(workpad));
                    Command::none()
                }
                Message::NewStarterWorkpad => {
                    debug!(target: "flexpad", %message);
                    let workpad = WorkpadMaster::new_starter();
                    self.state = State::Workpad(WorkpadUI::new(workpad));
                    Command::none()
                }
                _ => Command::none(),
            },
            State::Workpad(ref mut pad) => match message {
                Message::WorkpadMsg(msg) => match msg {
                    WorkpadMessage::PadClose => {
                        debug!(target: "flexpad", message=%msg);
                        self.state = State::FrontScreen;
                        Command::none()
                    }
                    _ => pad.update(msg).map(Message::WorkpadMsg),
                },
                _ => Command::none(),
            },
        }
    }

    #[tracing::instrument(skip_all)]
    fn view(&self) -> iced::Element<'_, Self::Message> {
        debug!(target: "flexpad", state=%self.state, "View");

        let body = match self.state {
            State::Loading => loading::view(),
            State::FrontScreen => front_screen::view(&self.version),
            State::Workpad(ref pad) => pad.view().map(Message::WorkpadMsg),
        };

        let paths: Vec<menu::Path<Message>> = match self.state {
            State::Loading => loading::menu_paths(),
            State::FrontScreen => front_screen::menu_paths(),
            State::Workpad(ref pad) => pad
                .menu_paths()
                .into_iter()
                .map(|p| p.map(Message::WorkpadMsg))
                .collect(),
        };

        crate::ui::menu::MenuedContent::new(paths, body).into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self.state {
            State::Loading => Subscription::none(),
            State::FrontScreen => Subscription::none(),
            State::Workpad(ref pad) => pad.subscription().map(Message::WorkpadMsg),
        }
    }
}

async fn load() -> Result<(), String> {
    let t0 = Instant::now();
    while Instant::now().duration_since(t0).as_secs() < 3 {}
    Ok(())
}

const SPACE_S: f32 = 5.0;
const SPACE_M: f32 = SPACE_S * 2.0;
const SPACE_L: f32 = SPACE_S * 4.0;
// const SPACE_XL: u16 = SPACE_S * 8;

const TEXT_SIZE_DIALOG_TITLE: f32 = 16.0;
const TEXT_SIZE_LABEL: f32 = 12.0;
const TEXT_SIZE_INPUT: f32 = 16.0;
const TEXT_SIZE_ERROR: f32 = 14.0;

const DIALOG_BUTTON_WIDTH: f32 = 100.0;

fn dialog_title<'a, Message>(
    title: impl ToString,
    style: style::DialogStyle,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(title).size(TEXT_SIZE_DIALOG_TITLE).style(style)).into()
}

fn dialog_button<'a, Message>(
    label: impl ToString,
    style: style::DialogButtonStyle,
) -> Button<'a, Message>
where
    Message: 'a,
{
    button(text(label).horizontal_alignment(alignment::Horizontal::Center))
        .width(DIALOG_BUTTON_WIDTH)
        .style(theme::Button::Custom(Box::new(style)))
}

fn button_bar<'a, Message, Renderer>() -> ButtonBar<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    ButtonBar {
        row: row![horizontal_space(Length::Fill)].spacing(SPACE_M),
    }
}

pub struct ButtonBar<'a, Message, Renderer> {
    row: Row<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> ButtonBar<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: iced::widget::button::StyleSheet,
{
    pub fn push(self, button: Button<'a, Message, Renderer>) -> Self {
        let Self { row } = self;
        Self {
            row: row.push(button),
        }
    }
}

impl<'a, Message, Renderer> From<ButtonBar<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(value: ButtonBar<'a, Message, Renderer>) -> Self {
        value.row.into()
    }
}

fn input_label<'a, Message>(label: impl ToString) -> Element<'a, Message> {
    iced::widget::text(label)
        .size(TEXT_SIZE_LABEL)
        .style(style::TextStyle::Label)
        .into()
}

fn text_input<'a, Message, F>(
    label: impl ToString,
    placeholder: impl ToString,
    value: &str,
    on_input: F,
    error: Option<&String>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    F: 'a + Fn(String) -> Message,
{
    let below: Element<'a, Message> = match error {
        Some(msg) => widget::container(
            widget::text(msg)
                .size(TEXT_SIZE_ERROR)
                .style(style::TextStyle::Error),
        )
        .height(SPACE_L)
        .into(),
        None => iced::widget::vertical_space(SPACE_L).into(),
    };

    let input_style = match error {
        Some(_) => style::TextInputStyle::Error,
        None => style::TextInputStyle::Default,
    };

    let icon = match error {
        Some(_) => widget::text_input::Icon {
            font: Font::default(),
            code_point: '\u{2757}',
            size: Some(TEXT_SIZE_INPUT),
            spacing: SPACE_M,
            side: widget::text_input::Side::Right,
        },
        None => widget::text_input::Icon {
            font: Font::default(),
            code_point: '\u{2713}',
            size: Some(TEXT_SIZE_INPUT),
            spacing: SPACE_M,
            side: widget::text_input::Side::Right,
        },
    };

    column![
        input_label(label),
        widget::vertical_space(SPACE_S),
        iced::widget::text_input(&placeholder.to_string(), value)
            .size(TEXT_SIZE_INPUT)
            .icon(icon)
            .style(input_style)
            .on_input(on_input),
        below
    ]
    .into()
}

const ESCAPE: Event = Event::Keyboard(keyboard::Event::KeyPressed {
    key_code: keyboard::KeyCode::Escape,
    modifiers: keyboard::Modifiers::empty(),
});
const ENTER: Event = Event::Keyboard(keyboard::Event::KeyPressed {
    key_code: keyboard::KeyCode::Enter,
    modifiers: keyboard::Modifiers::empty(),
});

fn handle_ok_key<Message>(event: &Event, on_ok: Message) -> Option<Message> {
    if *event == ENTER {
        Some(on_ok)
    } else {
        None
    }
}

fn handle_cancel_key<Message>(event: &Event, on_cancel: Message) -> Option<Message> {
    if *event == ESCAPE {
        Some(on_cancel)
    } else {
        None
    }
}

fn handle_ok_and_cancel_keys<Message>(
    event: &Event,
    on_ok: Message,
    on_cancel: Message,
) -> Option<Message> {
    handle_ok_key(event, on_ok).or_else(|| handle_cancel_key(event, on_cancel))
}
