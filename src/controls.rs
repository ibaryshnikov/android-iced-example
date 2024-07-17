use iced_wgpu::Renderer;
use iced_widget::{
    button, column, container, horizontal_space, pick_list, row, slider, text, text_editor,
    text_input, vertical_space, PickList, Slider, Space,
};
use iced_winit::core::{Alignment, Color, Element, Length, Theme};
use iced_winit::runtime::{Command, Program};
use iced_winit::winit::event_loop::EventLoopProxy;

use crate::UserEvent;

const EXAMPLES: [Example; 3] = [Example::Integration, Example::Counter, Example::TextEditor];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Example {
    Integration,
    Counter,
    TextEditor,
}

impl std::fmt::Display for Example {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Controls {
    background_color: Color,
    input: String,
    value: i32,
    selected_example: Example,
    editor: text_editor::Content<Renderer>,
    proxy: EventLoopProxy<UserEvent>,
}

#[derive(Debug, Clone)]
pub enum Message {
    RedChanged(f32),
    GreenChanged(f32),
    BlueChanged(f32),
    InputChanged(String),
    EditorAction(text_editor::Action),
    ExampleSelected(Example),
    Inc,
    Dec,
}

impl Controls {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Controls {
        Controls {
            background_color: Color::BLACK,
            input: String::default(),
            value: 0,
            selected_example: Example::Integration,
            editor: text_editor::Content::new(),
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
            Message::Inc => self.value += 1,
            Message::Dec => self.value -= 1,
            Message::ExampleSelected(example) => self.selected_example = example,
            Message::InputChanged(value) => self.input = value,
            Message::RedChanged(r) => self.background_color.r = r,
            Message::GreenChanged(g) => self.background_color.g = g,
            Message::BlueChanged(b) => self.background_color.b = b,
            Message::EditorAction(action) => match action {
                text_editor::Action::Focus => {
                    // it's possible to call java::call_instance_method("showKeyboard")
                    // right here, but needed something to show the usage of user events
                    let _ = self.proxy.send_event(UserEvent::ShowKeyboard);
                }
                text_editor::Action::Blur => {
                    let _ = self.proxy.send_event(UserEvent::HideKeyboard);
                }
                other => self.editor.perform(other),
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        match self.selected_example {
            Example::Integration => self.integration(),
            Example::Counter => self.counter(),
            Example::TextEditor => self.text_editor(),
        }
    }
}

fn color_slider<'a>(value: f32, f: impl Fn(f32) -> Message + 'a) -> Slider<'a, f32, Message> {
    slider(0.0..=1.0, value, f).step(0.01)
}

impl Controls {
    fn examples(&self) -> PickList<Example, &[Example], Example, Message> {
        pick_list(
            &EXAMPLES[..],
            Some(self.selected_example),
            Message::ExampleSelected,
        )
    }
    fn integration(&self) -> Element<Message, Theme, Renderer> {
        let sliders = row![
            color_slider(self.background_color.r, Message::RedChanged),
            color_slider(self.background_color.g, Message::GreenChanged),
            color_slider(self.background_color.b, Message::BlueChanged),
        ]
        .width(Length::Fill)
        .spacing(20);

        container(
            column![
                Space::with_height(20),
                self.examples(),
                vertical_space(),
                row![
                    text!("{:?}", self.background_color).size(14),
                    horizontal_space(),
                ],
                text_input("Placeholder", &self.input).on_input(Message::InputChanged),
                sliders,
                Space::with_height(20),
            ]
            .align_items(Alignment::Center)
            .spacing(10),
        )
        .padding(10)
        .into()
    }

    fn counter(&self) -> Element<Message, Theme, Renderer> {
        container(
            column![
                Space::with_height(30),
                self.examples(),
                vertical_space(),
                button("Increment").on_press(Message::Inc),
                text!("{}", self.value).size(50),
                button("Decrement").on_press(Message::Dec),
                vertical_space(),
                Space::with_height(100),
            ]
            .align_items(Alignment::Center)
            .spacing(10),
        )
        .center(Length::Fill)
        .style(add_background)
        .into()
    }

    fn text_editor(&self) -> Element<Message, Theme, Renderer> {
        container(
            column![
                Space::with_height(30),
                self.examples(),
                vertical_space(),
                text_editor::<Message, Theme, Renderer>(&self.editor)
                    .height(400)
                    .on_action(Message::EditorAction),
                vertical_space(),
            ]
            .align_items(Alignment::Center),
        )
        .padding(10)
        .center(Length::Fill)
        .style(add_background)
        .into()
    }
}

fn add_background(theme: &Theme) -> container::Style {
    theme.palette().background.into()
}
