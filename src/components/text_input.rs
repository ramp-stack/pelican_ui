use prism::event::{OnEvent, TickEvent, Event, self};
use prism::canvas::Align;
use prism::drawable::{Component, SizedTree};
use prism::{Context, Request};
use prism::layout::{Padding, Column, Offset, Size, Row, Stack, Area};
use prism::display::{EitherOr, Opt, Bin};

use ptsd::interactions;

use std::sync::{Arc, Mutex};

use crate::theme::{Theme, Color};

use crate::components::text::{Text, TextSize, TextStyle, TextEditor, ExpandableText};
use crate::components::Rectangle;
use crate::components::button::SecondaryIconButton;

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
#[derive(Debug, Component, Clone)]
pub struct TextInput {
    layout: Column,
    label: Option<Text>,
    pub inner: interactions::InputField,
    hint: EitherOr<Option<ExpandableText>, ExpandableText>,
    #[skip] pub error: Option<String>,
}

type InputCallback = Arc<Mutex<dyn FnMut(&mut Context, &mut String) + 'static>>;

impl TextInput {
    pub fn new(
        theme: &Theme,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: Option<&str>,
        help_text: Option<&str>,
        icon_button: Option<(&str, InputCallback)>,
        // on_edit: impl FnMut(&mut Context, &mut String) + 'static,
    ) -> Self {
        let background = |bg: Color, o: Color| Rectangle::new(bg, 8.0, Some((1.0, o)));
        let colors = theme.colors();
        let input_field = interactions::InputField::new(
            background(Color::TRANSPARENT, colors.get(ptsd::Outline::Secondary)),
            background(Color::TRANSPARENT, colors.get(ptsd::Outline::Primary)),
            Some(background(colors.get(ptsd::Outline::Primary), colors.get(ptsd::Outline::Secondary))),
            Some(background(Color::TRANSPARENT, colors.get(ptsd::Status::Danger))),
            _InputContent::new(theme, value, placeholder, icon_button),
            48.0,
        );

        let error = ExpandableText::new(theme, "", TextSize::Sm, TextStyle::Error, Align::Left, None); 
        let help = help_text.map(|t| ExpandableText::new(theme, t, TextSize::Sm, TextStyle::Secondary, Align::Left, None));

        TextInput { 
            layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            label: label.map(|l| Text::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None)),
            inner: input_field, 
            hint: EitherOr::new(help, error),
            error: None
        }
    }
    
    pub fn default(theme: &Theme, ) -> Self {
        Self::new(theme, None, Some("First name"), None, None, None)
    }

    pub fn value(&self) -> String {
        self.inner.2.as_any().downcast_ref::<_InputContent>().unwrap().value.to_string()
    }  
}

impl OnEvent for TextInput { 
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> { 
        if event.as_any().downcast_ref::<TickEvent>().is_some() { 
            self.hint.display_left(self.error.is_some()); 
            if let Some(e) = &self.error { 
                self.hint.right().0.spans[0] = e.to_string(); 
            } 
        } 
        vec![event] 
    } 
}

#[derive(Component, Clone)]
struct _InputContent {
    layout: Row,
    default: Opt<Bin<Stack, TextEditor>>,
    empty: Opt<Bin<Stack, ExpandableText>>,
    button: Option<SecondaryIconButton>,
    #[skip] pub value: String,
    #[skip] on_submit: Option<InputCallback>,
    #[skip] is_focused: bool,
}

impl std::fmt::Debug for _InputContent { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "_InputContent") 
    } 
}

impl _InputContent {
    pub fn new(
        theme: &Theme,
        value: Option<&str>,
        placeholder: Option<&str>,
        button: Option<(&str, InputCallback)>,
    ) -> Self {
        let (button, on_submit) = button.map(|(icon, cb)| {
            let btn = SecondaryIconButton::medium(theme, icon, |ctx: &mut Context, _: &Theme| ctx.send(Request::Event(Box::new(TextInputEvent::Submit))));
            (Some(btn), Some(cb))
        }).unwrap_or((None, None));
        
        let default = TextEditor::new(theme, value.unwrap_or_default(), TextSize::Md, TextStyle::Primary, Align::Left); 
        let empty = ExpandableText::new(theme, placeholder.unwrap_or("Enter text..."), TextSize::Md, TextStyle::Secondary, Align::Left, None);
        _InputContent { 
            layout: Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 8.0, 8.0, 8.0)), 
            default: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 8.0, 16.0, 8.0)), default), false), 
            empty: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 8.0, 16.0, 8.0)), empty), true), 
            button,
            value: value.unwrap_or_default().to_string(), 
            on_submit,
            is_focused: false,
        }
    }
}

impl OnEvent for _InputContent { 
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> { 
        if let Some(event::TextInput::Focused(x)) = event.downcast_ref::<event::TextInput>() {
            self.is_focused = *x;
            // println!("FOCUSED {:?}", self.is_focused);
        } else if event.downcast_ref::<TickEvent>().is_some() {
            // if let Some(i) = ctx.state.get::<TextInputState>() {
            //     if i.0 == self.state_name {
            //         self.default.inner().inner().1.0.spans[0] = i.1.to_string();
            //     }
            // }

            self.value = self.default.inner().inner().1.0.spans[0].clone();

            self.default.display(self.is_focused);
            self.empty.display(!self.is_focused);
            self.default.inner().inner().display_cursor(self.is_focused);

            if !self.is_focused {
                self.default.display(!self.value.is_empty());
                self.empty.display(self.value.is_empty());
            }
        } else if let Some(TextInputEvent::Submit) = event.downcast_ref::<TextInputEvent>() 
        && let Some(on_submit) = &mut self.on_submit 
        && let Ok(mut cb) = on_submit.lock() {
            (cb)(ctx, &mut self.value);
        }
        vec![event]
    }
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Submit,
    Edited(String, String),
}

impl Event for TextInputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
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
//     fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> bool {
//         if event.downcast_ref::<InputEditedEvent>().is_some() && self.1.2.3 == InputState::Focus {
//             ctx.trigger_event(SearchEvent(self.1.value().clone()))
//         }
//         true
//     }
// }