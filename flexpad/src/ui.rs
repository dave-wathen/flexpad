use crate::version::Version;
use iced::widget::{self, button, column, container, horizontal_rule, image, row, text};
use iced::{
    alignment, font, theme, window, Alignment, Application, Command, Element, Font, Length,
    Settings, Subscription, Theme,
};
use rust_i18n::t;
use tracing::debug;

use self::workpad::{WorkpadMessage, WorkpadUI};
use crate::model::workpad::WorkpadMaster;

mod action;
mod images;
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
    FontLoaded(Result<(), font::Error>),
    OpenBlankWorkpad,
    OpenStarterWorkpad,
    WorkpadMsg(WorkpadMessage),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message::")?;
        match self {
            Self::FontLoaded(result) => write!(f, "FontLoaded({result:?})"),
            Self::OpenBlankWorkpad => write!(f, "OpenBlankWokpad"),
            Self::OpenStarterWorkpad => write!(f, "OpenStarterWokpad"),
            Self::WorkpadMsg(msg) => msg.fmt(f),
        }
    }
}

const SPACE_S: f32 = 5.0;
const SPACE_M: f32 = SPACE_S * 2.0;
const SPACE_L: f32 = SPACE_S * 4.0;
// const SPACE_XL: u16 = SPACE_S * 8;

const TEXT_SIZE_LABEL: f32 = 12.0;
const TEXT_SIZE_INPUT: f32 = 16.0;
const TEXT_SIZE_ERROR: f32 = 14.0;

// TODO Can we avoid a constant width via layouts
const DIALOG_BUTTON_WIDTH: f32 = 100.0;

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
            _ => t!("Product"),
        }
    }

    #[tracing::instrument(target = "flexpad", skip_all)]
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match self.state {
            State::Loading => {
                if let Message::FontLoaded(result) = message {
                    debug!(target: "flexpad", %message);
                    match result {
                        Ok(_) => self.state = State::FrontScreen,
                        Err(err) => panic!("{err:?}"),
                    }
                }
                Command::none()
            }
            State::FrontScreen => match message {
                Message::OpenBlankWorkpad => {
                    debug!(target: "flexpad", %message);
                    let workpad = WorkpadMaster::new_blank();
                    self.state = State::Workpad(WorkpadUI::new(workpad));
                    Command::none()
                }
                Message::OpenStarterWorkpad => {
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
        match self.state {
            State::Loading => container(
                text(t!("Common.Loading"))
                    .style(TextStyle::Default)
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

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self.state {
            State::Loading => Subscription::none(),
            State::FrontScreen => Subscription::none(),
            State::Workpad(ref pad) => pad.subscription().map(Message::WorkpadMsg),
        }
    }
}

impl Flexpad {
    fn front_screen_view(&self) -> iced::Element<'_, Message> {
        let image_button = |img, title, msg| {
            column![
                button(image(img).width(48).height(48))
                    .on_press(msg)
                    .style(theme::Button::Text),
                text(title).size(12)
            ]
            .align_items(Alignment::Center)
        };

        column![
            image(images::app()).width(200).height(200),
            text(self.version.description()).size(12),
            horizontal_rule(3),
            text(t!("Workpads.Create"))
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
            row![
                image_button(
                    images::workpad_no_sheets(),
                    t!("Workpads.Blank"),
                    Message::OpenBlankWorkpad
                ),
                image_button(
                    images::workpad_and_sheets(),
                    t!("Workpads.Starter"),
                    Message::OpenStarterWorkpad
                )
            ]
            .spacing(SPACE_M)
            .width(Length::Fill),
            horizontal_rule(3),
            text(t!("Workpads.Reopen"))
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

fn input_label<'a, Message>(label: impl ToString) -> Element<'a, Message> {
    iced::widget::text(label)
        .size(TEXT_SIZE_LABEL)
        .style(TextStyle::Label)
        .into()
}

fn labeled_element<'a, Message>(
    label: impl ToString,
    elem: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    column![input_label(label), elem.into()]
        .spacing(SPACE_S)
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
                .style(TextStyle::Error),
        )
        .height(SPACE_L)
        .into(),
        None => iced::widget::vertical_space(SPACE_L).into(),
    };

    let input_style = match error {
        Some(_) => TextInputStyle::Error,
        None => TextInputStyle::Default,
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

enum TextStyle {
    Default,
    Label,
    Error,
}

impl From<TextStyle> for theme::Text {
    fn from(value: TextStyle) -> Self {
        // TODO Theme - there's no Custom for theme::Text!
        let palette = Theme::Light.extended_palette();
        let color = match value {
            TextStyle::Default => palette.primary.base.text,
            TextStyle::Label => palette.primary.weak.text,
            TextStyle::Error => palette.danger.base.color,
        };
        theme::Text::Color(color)
    }
}

enum TextInputStyle {
    Default,
    Error,
}

impl From<TextInputStyle> for theme::TextInput {
    fn from(value: TextInputStyle) -> Self {
        theme::TextInput::Custom(Box::new(value))
    }
}

impl widget::text_input::StyleSheet for TextInputStyle
where
    iced::Theme: widget::text_input::StyleSheet<Style = theme::TextInput>,
{
    type Style = iced::Theme;

    fn active(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.active(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.strong.color,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn focused(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.focused(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.strong.color,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn disabled(&self, theme: &Self::Style) -> widget::text_input::Appearance {
        let dflt = theme.disabled(&theme::TextInput::Default);

        if let Self::Error = self {
            let palette = theme.extended_palette();

            widget::text_input::Appearance {
                border_color: palette.danger.weak.color,
                //icon_color: palette.background.weak.text,
                ..dflt
            }
        } else {
            dflt
        }
    }

    fn placeholder_color(&self, theme: &Self::Style) -> iced::Color {
        theme.placeholder_color(&theme::TextInput::Default)
    }

    fn value_color(&self, theme: &Self::Style) -> iced::Color {
        theme.value_color(&theme::TextInput::Default)
    }

    fn disabled_color(&self, theme: &Self::Style) -> iced::Color {
        theme.disabled_color(&theme::TextInput::Default)
    }

    fn selection_color(&self, theme: &Self::Style) -> iced::Color {
        theme.selection_color(&theme::TextInput::Default)
    }
}
