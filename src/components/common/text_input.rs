use pelican_ui::events::{OnEvent, TickEvent, MouseState, MouseEvent, Event, KeyboardState, KeyboardEvent};
use pelican_ui::drawable::{Drawable, Component, Align, Color};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::components::{Rectangle, ExpandableText, Text, TextStyle, TextEditor};
use crate::components::button::IconButton;
use crate::events::{SearchEvent, InputEditedEvent, KeyboardActiveEvent, SetActiveInput, TextInputSelect, ClearActiveInput};
use crate::layout::{EitherOr, Padding, Column, Stack, Offset, Size, Row, Bin};
use crate::utils::ElementID;

use std::sync::mpsc::{self, Receiver};

/// ## Text Input
///
/// A text input field with optional label, placeholder, help text, and an icon button.  
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/text_input.png"
///      alt="Text Input Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let input = TextInput::new(
///     ctx,
///     None,
///     Some("Bird Name"),
///     "Enter bird name",
///     Some("The name will be used in your bird list"),
///     None,
/// );
/// ```  
#[derive(Debug, Component)]
pub struct TextInput(Column, Option<Text>, InputField, Option<ExpandableText>, Option<Text>);

impl TextInput {
    #[allow(clippy::type_complexity)]
    pub const NO_ICON: Option<(&str, fn(&mut Context, &mut String))> = None::<(&'static str, fn(&mut Context, &mut String))>;

    pub fn new(
        ctx: &mut Context,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: &str,
        help_text: Option<&str>,
        icon_button: Option<(&'static str, impl FnMut(&mut Context, &mut String) + 'static)>,
        keyboard_actions: bool,
    ) -> Self {
        let font_size = ctx.theme.fonts.size;

        TextInput(
            Column::new(16.0, Offset::Start, Size::fill(), Padding::default()),
            label.map(|text| Text::new(ctx, text, TextStyle::Heading, font_size.h5, Align::Left)),
            InputField::new(ctx, value, placeholder, icon_button, keyboard_actions),
            help_text.map(|t| ExpandableText::new(ctx, t, TextStyle::Secondary, font_size.sm, Align::Left, None)),
            None
        )
    }

    pub fn set_error(&mut self, ctx: &mut Context, error: &str) {
        let font_size = ctx.theme.fonts.size.sm;
        self.4 = Some(Text::new(ctx, error, TextStyle::Error, font_size, Align::Left));
        self.3 = None;
    }

    pub fn set_help(&mut self, ctx: &mut Context, help: &str) {
        let font_size = ctx.theme.fonts.size.sm;
        self.3 = Some(ExpandableText::new(ctx, help, TextStyle::Secondary, font_size, Align::Left, None));
        self.4 = None;
    }

    pub fn error(&mut self) -> &mut bool {
        self.2.error()
    }

    pub fn value(&mut self) -> &mut String {
        self.2.input()
    }

    pub fn sync_input_value(&mut self, actual_value: &str) -> bool {
        let current = self.value().to_string();
        let changed = current != actual_value;
        if *self.status() != InputState::Focus && !changed {
            *self.value() = actual_value.to_string();
        }
        changed
    }

    pub fn get_id(&self) -> ElementID { self.2.5 }
    pub fn status(&mut self) -> &mut InputState {self.2.status()}
}

impl OnEvent for TextInput {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            *self.2.error() = self.4.is_some();
        }
        true
    }
}

#[derive(Debug, Component)]
struct InputField(Stack, Rectangle, InputContent, #[skip] InputState, #[skip] bool, #[skip] ElementID, #[skip] bool);

impl InputField {
    pub fn new(
        ctx: &mut Context,
        value: Option<&str>,
        placeholder: &str,
        icon_button: Option<(&'static str, impl FnMut(&mut Context, &mut String) + 'static)>,
        keyboard_actions: bool,
    ) -> Self {
        let (background, outline) = InputState::Default.get_color(ctx);
        let content = InputContent::new(ctx, value, placeholder, icon_button);
        let background = Rectangle::new(background, 8.0, Some((1.0, outline)));
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, widths[0].1));            
        let height = Size::custom(|heights: Vec<(f32, f32)>| (heights[1].0.max(48.0), heights[1].1.max(48.0)));

        InputField(
            Stack(Offset::Start, Offset::Start, width, height, Padding::default()), 
            background, content, InputState::Default, false, ElementID::new(), keyboard_actions,
        )
    }

    pub fn error(&mut self) -> &mut bool { &mut self.4 }
    pub fn input(&mut self) -> &mut String { &mut self.2.text().text().spans[0].text }
    pub fn status(&mut self) -> &mut InputState {&mut self.3}
}

impl OnEvent for InputField {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            self.2.text().display_cursor(self.3 == InputState::Focus);
            self.3 = match self.3 {
                InputState::Default if self.4 => Some(InputState::Error),
                InputState::Error if !self.4 => Some(InputState::Default),
                _ => None
            }.unwrap_or(self.3);

            let (background, outline) = self.3.get_color(ctx);
            *self.1.background() = background;
            if let Some(c) = self.1.outline() { *c = outline; }
            *self.2.focus() = self.3 == InputState::Focus;
        } else if let Some(ClearActiveInput) = event.downcast_ref::<ClearActiveInput>() {
            // self.3 = if *self.error() { InputState::Error } else { InputState::Default };
        } else if let Some(SetActiveInput(s)) = event.downcast_ref::<SetActiveInput>() {
            *self.input() = s.to_string();
        } else if let Some(TextInputSelect(id)) = event.downcast_ref::<TextInputSelect>() {
            if *id != self.5 && self.3 == InputState::Focus {
                if self.4 { self.3 = InputState::Error } else { self.3 = InputState::Default }
            }
        } else if let Some(KeyboardActiveEvent(keyboard)) = event.downcast_ref::<KeyboardActiveEvent>() {
            if keyboard.is_none() && self.3 == InputState::Focus {
                if self.4 { self.3 = InputState::Error } else { self.3 = InputState::Default }
            }
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.3 = match self.3 {
                InputState::Default => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
                            ctx.hardware.haptic();
                            ctx.trigger_event(TextInputSelect(self.5));
                            ctx.trigger_event(KeyboardActiveEvent(Some(self.6))); 
                            Some(InputState::Focus)
                        },
                        MouseEvent{state: MouseState::Moved, position: Some(_)} => Some(InputState::Hover),
                        _ => None
                    }
                },
                InputState::Hover => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
                            ctx.trigger_event(TextInputSelect(self.5));
                            Some(InputState::Focus)
                        },
                        MouseEvent{state: MouseState::Moved, position: None} if self.4 => Some(InputState::Error),
                        MouseEvent{state: MouseState::Moved, position: None} => Some(InputState::Default),
                        _ => None
                    }
                },
                InputState::Focus => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: None} if self.4 && !crate::config::IS_MOBILE => Some(InputState::Error),
                        MouseEvent{state: MouseState::Pressed, position: None} if !crate::config::IS_MOBILE => Some(InputState::Default),
                        _ => None
                    }
                },
                InputState::Error => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: Some(_)} => Some(InputState::Focus),
                        MouseEvent{state: MouseState::Moved, position: Some(_)} => Some(InputState::Hover),
                        _ => None
                    }
                }
            }.unwrap_or(self.3);
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref() {
            if self.3 == InputState::Focus {
                self.2.text().apply_edit(ctx, key);
            }
            ctx.trigger_event(InputEditedEvent);
        }
        true
    }
}

pub type SubmitCallback = Box<dyn FnMut(&mut Context, &mut String)>;

#[derive(Component)]
struct InputContent(
    Row, Bin<Stack, EitherOr<TextEditor, ExpandableText>>, Option<IconButton>,
    #[skip] bool, #[skip] Option<(Receiver<u8>, SubmitCallback)>
);

impl InputContent {
    fn new(
        ctx: &mut Context,
        value: Option<&str>,
        placeholder: &str,
        icon_button: Option<(&'static str, impl FnMut(&mut Context, &mut String) + 'static)>,
    ) -> Self {
        let font_size = ctx.theme.fonts.size.md;
        let (icon_button, callback) = icon_button.map(|(icon, on_click)| {
            let (sender, receiver) = mpsc::channel();
            (
                Some(IconButton::input(ctx, icon, move |_| {sender.send(0).unwrap();})),
                Some((receiver, Box::new(on_click) as SubmitCallback)),
            )
        }).unwrap_or((None, None));

        InputContent(
            Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 8.0, 8.0, 8.0)),
            Bin(
                Stack(Offset::default(), Offset::End, Size::fill(), Size::Fit, Padding(8.0, 8.0, 8.0, 8.0)),
                EitherOr::new(
                    TextEditor::new(ctx, value.unwrap_or(""), TextStyle::Primary, font_size, Align::Left),
                    ExpandableText::new(ctx, placeholder, TextStyle::Secondary, font_size, Align::Left, None)
                )
            ),
            icon_button,
            false,
            callback,
        )
    }

    fn text(&mut self) -> &mut TextEditor { self.1.inner().left() }
    fn focus(&mut self) -> &mut bool {&mut self.3}
}

impl OnEvent for InputContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some((receiver, on_submit)) = self.4.as_mut() {
                if receiver.try_recv().is_ok() {
                    on_submit(ctx, &mut self.1.inner().left().text().spans[0].text)
                }
            }

            let input = !self.1.inner().left().text().spans[0].text.is_empty();
            self.1.inner().display_left(input || self.3)
        } else if let Some(ClearActiveInput) = event.downcast_ref::<ClearActiveInput>() {
            self.1.inner().left().text().spans[0].text = String::new();
            // self.1.inner().display_left(false);
        }
        true
    }
}

impl std::fmt::Debug for InputContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InputContent(...)")
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum InputState {
    Default,
    Hover,
    Focus,
    Error
}

impl InputState {
    fn get_color(&self, ctx: &mut Context) -> (Color, Color) { // background, outline
        let colors = &ctx.theme.colors;
        match self {
            InputState::Default => (colors.shades.transparent, colors.outline.secondary),
            InputState::Hover => (colors.background.secondary, colors.outline.secondary),
            InputState::Focus => (colors.shades.transparent, colors.outline.primary),
            InputState::Error => (colors.shades.transparent, colors.status.danger)
        }
    }
}

/// # Searchbar
/// 
/// Searchbar component
#[derive(Debug, Component)]
pub struct Searchbar(Stack, TextInput);
impl Searchbar {
    pub fn new(input: TextInput) -> Self {
        Searchbar(Stack::default(), input)
    }
}

impl OnEvent for Searchbar {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<InputEditedEvent>().is_some() && self.1.2.3 == InputState::Focus {
            ctx.trigger_event(SearchEvent(self.1.value().clone()))
        }
        true
    }
}