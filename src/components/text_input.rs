use mustache::events::{OnEvent, TickEvent, MouseState, MouseEvent, Event, KeyboardState, KeyboardEvent};
use mustache::drawable::{Drawable, Component, Align, Color};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::components::{Rectangle, ExpandableText, Text, TextStyle, TextEditor};
use crate::events::{InputEditedEvent, KeyboardActiveEvent, SetActiveInput, TextInputSelect, ClearActiveInput};
use crate::layout::{EitherOr, Padding, Column, Stack, Offset, Size, Row, Bin};
use crate::components::button::SecondaryIconButton;
use crate::utils::ElementID;
use crate::plugin::PelicanUI;

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
///     Some("Species Name"),
///     "Enter the full species name",
///     Some("Example: Northern Cardinal"),
///     None,
/// );
/// ```
#[derive(Debug, Component)]
pub struct TextInput {
    _layout: Column,
    _label: Option<Text>,
    _input: InputField,
    _hint: Option<ExpandableText>,
    _error:  Option<Text>,
    #[skip] pub error: Option<String>,
    #[skip] pub hint: Option<String>,
    #[skip] pub value: String,
    #[skip] pub element_id: ElementID,
}

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
    ) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let element_id = ElementID::new();

        TextInput {
            _layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            _label: label.map(|text| Text::new(ctx, text, size.h5, TextStyle::Heading, Align::Left, None)),
            _input: InputField::new(ctx, value, placeholder, icon_button, element_id),
            _hint: help_text.map(|t| ExpandableText::new(ctx, t, size.sm, TextStyle::Secondary, Align::Left, None)),
            _error: None,
            hint: help_text.map(|t| t.to_string()),
            error: None,
            value: value.map(|v| v.to_string()).unwrap_or_default(),
            element_id,
        }
    }

    pub fn sync_input_value(&mut self, actual_value: &str) -> bool {
        let changed = self.value != actual_value;
        if *self._input.state() != InputState::Focus && !changed {
            self.value = actual_value.to_string();
        }
        changed
    }
}

impl OnEvent for TextInput {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.as_any().downcast_ref::<TickEvent>().is_some() {
            let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.sm;

            if let Some(e) = &self.error {
                self._error = Some(Text::new(ctx, e, size, TextStyle::Error, Align::Left, None));
                self._hint = None;
                self._hint = None;
            }

            if let Some(h) = &self.hint {
                self._hint = Some(ExpandableText::new(ctx, h, size, TextStyle::Secondary, Align::Left, None));
                self._error = None;
                self.error = None;
            }
            
            *self._input.input() = self.value.clone();
        }
        false
    }
}

#[derive(Debug, Component)]
struct InputField(Stack, Rectangle, InputContent, #[skip] InputState, #[skip] bool, #[skip] ElementID);

impl InputField {
    pub fn new(
        ctx: &mut Context,
        value: Option<&str>,
        placeholder: &str,
        icon_button: Option<(&'static str, impl FnMut(&mut Context, &mut String) + 'static)>,
        element_id: ElementID
    ) -> Self {
        let (background, outline) = InputState::Default.get_color(ctx);
        let content = InputContent::new(ctx, value, placeholder, icon_button);
        let background = Rectangle::new(background, 8.0, Some((1.0, outline)));
        let height = Size::custom(|heights: Vec<(f32, f32)>| (heights[1].0.max(48.0), heights[1].1.max(48.0)));

        InputField(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()), 
            background, content, InputState::Default, false, element_id,
        )
    }

    pub fn input(&mut self) -> &mut String { &mut self.2.text().text().spans[0].text }
    pub fn state(&mut self) -> &mut InputState {&mut self.3}
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
            *self.input() = String::new();
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
                            ctx.trigger_event(KeyboardActiveEvent(Some(false))); 
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
    Row, Bin<Stack, EitherOr<TextEditor, ExpandableText>>, Option<SecondaryIconButton>,
    #[skip] bool, #[skip] Option<(Receiver<u8>, SubmitCallback)>
);

impl InputContent {
    fn new(ctx: &mut Context, value: Option<&str>, placeholder: &str, icon_button: Option<(&'static str, impl FnMut(&mut Context, &mut String) + 'static)>) -> Self {
        let (icon_button, callback) = icon_button.map(|(icon, on_click)| {
            let (sender, receiver) = mpsc::channel();
            let icon_button = SecondaryIconButton::new(ctx, icon, move |_| {sender.send(0).unwrap();});
            let callback = (receiver, Box::new(on_click) as SubmitCallback);
            (Some(icon_button), Some(callback))
        }).unwrap_or((None, None));

        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.md;
        let bin_layout = Stack(Offset::default(), Offset::End, Size::Fill, Size::Fit, Padding::new(8.0));
        let text_editor = TextEditor::new(ctx, value.unwrap_or(""), size, TextStyle::Primary, Align::Left);
        let placeholder = ExpandableText::new(ctx, placeholder, size, TextStyle::Secondary, Align::Left, None);
        let bin = Bin(bin_layout, EitherOr::new(text_editor, placeholder));
        let layout = Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 8.0, 8.0, 8.0));
        InputContent(layout, bin, icon_button, false, callback)
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
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;
        match self {
            InputState::Default => (Color::TRANSPARENT, colors.outline.secondary),
            InputState::Hover => (colors.background.secondary, colors.outline.secondary),
            InputState::Focus => (Color::TRANSPARENT, colors.outline.primary),
            InputState::Error => (Color::TRANSPARENT, colors.status.danger)
        }
    }
}

// /// # Searchbar
// /// 
// /// Searchbar component
// #[derive(Debug, Component)]
// pub struct Searchbar(Stack, TextInput);
// impl Searchbar {
//     pub fn new(input: TextInput) -> Self {
//         Searchbar(Stack::default(), input)
//     }
// }

// impl OnEvent for Searchbar {
//     fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
//         if event.downcast_ref::<InputEditedEvent>().is_some() && self.1.2.3 == InputState::Focus {
//             ctx.trigger_event(SearchEvent(self.1.value().clone()))
//         }
//         true
//     }
// }