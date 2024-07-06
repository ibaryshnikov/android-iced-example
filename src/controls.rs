use iced_wgpu::Renderer;
use iced_widget::{
    button, column, container, pick_list, row, slider, text, text_editor, text_input,
};
use iced_winit::core::{alignment, Alignment, Color, Element, Length, Theme};
use iced_winit::runtime::{Command, Program};
use iced_winit::winit::event_loop::EventLoopProxy;

use crate::UserEvent;

pub struct Controls {
    background_color: Color,
    input: String,
    // value: i32,
    // selected_language: Option<Language>,
    // editor: text_editor::Content<Renderer>,
    proxy: EventLoopProxy<UserEvent>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    InputChanged(String),
    // EditorAction(text_editor::Action),
    // LanguageSelected(Language),
    // Inc,
    // Dec,
}

impl Controls {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Controls {
        Controls {
            background_color: Color::BLACK,
            input: String::default(),
            // value: 0,
            // selected_language: None,
            // editor: text_editor::Content::new(),
            proxy,
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}

impl Program for Controls {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // Message::Inc => self.value += 1,
            // Message::Dec => self.value -= 1,
            // Message::LanguageSelected(language) => self.selected_language = Some(language),
            Message::InputChanged(value) => self.input = value,
            Message::BackgroundColorChanged(color) => self.background_color = color,
            // Message::EditorAction(action) => match action {
            //     text_editor::Action::Focus => {
            //         let _ = self.proxy.send_event(UserEvent::ShowKeyboard);
            //     }
            //     text_editor::Action::Blur => {
            //         let _ = self.proxy.send_event(UserEvent::HideKeyboard);
            //     }
            //     other => self.editor.perform(other),
            // },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        // let pick_list = pick_list(
        //     &Language::ALL[..],
        //     self.selected_language,
        //     Message::LanguageSelected,
        // )
        // .placeholder("Choose a language...");

        let background_color = self.background_color;

        let sliders = row![
            slider(0.0..=1.0, background_color.r, move |r| {
                Message::BackgroundColorChanged(Color {
                    r,
                    ..background_color
                })
            })
            .step(0.01),
            slider(0.0..=1.0, background_color.g, move |g| {
                Message::BackgroundColorChanged(Color {
                    g,
                    ..background_color
                })
            })
            .step(0.01),
            slider(0.0..=1.0, background_color.b, move |b| {
                Message::BackgroundColorChanged(Color {
                    b,
                    ..background_color
                })
            })
            .step(0.01),
        ]
        .width(500)
        .spacing(20);

        container(
            column![
                // button("Increment").on_press(Message::Inc),
                // text!("{}", self.value).size(50),
                // button("Decrement").on_press(Message::Dec),
                // pick_list,
                text!("{background_color:?}").size(14),
                text_input("Placeholder", &self.input).on_input(Message::InputChanged),
                sliders,
                // text_editor::<Message, Theme, Renderer>(&self.editor)
                //     .height(Length::Fixed(400.0))
                //     .on_action(Message::EditorAction),
            ]
            // .align_items(Alignment::Center)
            .spacing(10),
        )
        // .center(Length::Fill)
        // .style(|theme: &Theme| theme.palette().background.into())
        .padding(10)
        .height(Length::Fill)
        .align_y(alignment::Vertical::Bottom)
        .into()
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// pub enum Language {
//     #[default]
//     Rust,
//     Elm,
//     Ruby,
//     Haskell,
//     C,
//     Javascript,
//     Other,
// }
//
// impl Language {
//     const ALL: [Language; 7] = [
//         Language::C,
//         Language::Elm,
//         Language::Ruby,
//         Language::Haskell,
//         Language::Rust,
//         Language::Javascript,
//         Language::Other,
//     ];
// }
//
// impl std::fmt::Display for Language {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 Language::Rust => "Rust",
//                 Language::Elm => "Elm",
//                 Language::Ruby => "Ruby",
//                 Language::Haskell => "Haskell",
//                 Language::C => "C",
//                 Language::Javascript => "Javascript",
//                 Language::Other => "Some other language",
//             }
//         )
//     }
// }
