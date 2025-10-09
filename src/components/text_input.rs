use mustache::events::{OnEvent, TickEvent, Event};
use mustache::drawable::{Align, Color};
use mustache::{Context, Component};

use crate::components::interactions::{SubmitCallback, self};
use crate::components::{Rectangle, ExpandableText, Text, TextStyle, TextEditor};
use crate::layout::{Padding, Column, Stack, Offset, Size};
use crate::components::button::SecondaryIconButton;
use crate::utils::ElementID;
use crate::plugin::PelicanUI;

use std::sync::mpsc;

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
}

type TextInputButton = (&'static str, Box<dyn FnMut(&mut Context, &mut String)>);

impl TextInput {
    pub fn new(
        ctx: &mut Context,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: &str,
        help_text: Option<&str>,
        icon_button: Option<TextInputButton>,
    ) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;

        TextInput {
            _layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            _label: label.map(|text| Text::new(ctx, text, size.h5, TextStyle::Heading, Align::Left, None)),
            _input: InputField::new(ctx, value, placeholder,icon_button),
            _hint: help_text.map(|t| ExpandableText::new(ctx, t, size.sm, TextStyle::Secondary, Align::Left, None)),
            _error: None,
            hint: help_text.map(|t| t.to_string()),
            error: None,
        }
    }

    pub fn id(&self) -> &ElementID {&self._input.1.id}

    // pub fn sync_input_value(&mut self, actual_value: &str) -> bool {
    //     let changed = self.value != actual_value;
    //     if *self._input.inner().state != InputState::Focus && !changed {
    //         self.value = actual_value.to_string();
    //     }
    //     changed
    // }
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
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct InputField(Stack, interactions::InputField);
impl OnEvent for InputField {}
impl InputField {
    pub fn new(ctx: &mut Context, value: Option<&str>, placeholder: &str, icon_button: Option<TextInputButton>) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.md;
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;

        let icon_button = icon_button.map(|(icon, on_click)| {
            let (sender, receiver) = mpsc::channel();
            let icon_button = SecondaryIconButton::new(ctx, icon, move |_| {sender.send(0).unwrap();});
            (icon_button.1, receiver, Box::new(on_click) as SubmitCallback)
        });

        println!("Value is {:?}, placeholder is {:?}", value, placeholder);

        let content = interactions::InputContent::new(
            TextEditor::new(ctx, value.unwrap_or(""), size, TextStyle::Primary, Align::Left),
            ExpandableText::new(ctx, value.unwrap_or(""), size, TextStyle::Primary, Align::Left, None),
            ExpandableText::new(ctx, placeholder, size, TextStyle::Secondary, Align::Left, None),
            icon_button,
        );

        let field = interactions::InputField::new(
            InputBackground::new(Color::TRANSPARENT, colors.outline.secondary),
            InputBackground::new(colors.background.secondary, colors.outline.secondary),
            InputBackground::new(Color::TRANSPARENT, colors.outline.primary),
            InputBackground::new(Color::TRANSPARENT, colors.status.danger),
            content,
        );

        InputField(Stack::default(), field)
    }

    // pub fn inner(&mut self) -> &mut interactions::InputField {&mut self.1}
}


struct InputBackground;
impl InputBackground {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(background: Color, outline: Color) -> Rectangle {
        Rectangle::new(background, 8.0, Some((1.0, outline)))
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