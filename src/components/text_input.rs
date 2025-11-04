use roost_ui::events::{OnEvent, TickEvent, Event, self};
use roost_ui::drawable::{Align, Color};
use roost_ui::{Context, Component};
use roost_ui::layouts::{Padding, Column, Offset, Size, EitherOr, Opt, Row, Bin, Stack};

use crate::interactions;
use crate::components::{Rectangle, ExpandableText, Text, TextSize, TextStyle, TextEditor};
use crate::components::button::SecondaryIconButton;
use crate::plugin::PelicanUI;

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
    layout: Column,
    label: Option<Text>,
    pub inner: interactions::InputField,
    hint: EitherOr<Option<ExpandableText>, ExpandableText>,
    #[skip] pub error: Option<String>,
}

type InputCallback = Box<dyn FnMut(&mut Context, &mut String)>;

impl TextInput {
    pub fn new(
        ctx: &mut Context,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: Option<&str>,
        help_text: Option<&str>,
        icon_button: Option<(&str, InputCallback)>,
    ) -> Self {
        let background = |bg: Color, o: Color| Rectangle::new(bg, 8.0, Some((1.0, o)));
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;

        let input_field = interactions::InputField::new(
            background(Color::TRANSPARENT, colors.outline.secondary),
            background(Color::TRANSPARENT, colors.outline.primary),
            Some(background(colors.background.secondary, colors.outline.secondary)),
            Some(background(Color::TRANSPARENT, colors.status.danger)),
            _InputContent::new(ctx, value, placeholder, icon_button),
            48.0,
        );

        let error = ExpandableText::new(ctx, "", TextSize::Sm, TextStyle::Error, Align::Left, None); 
        let help = help_text.map(|t| ExpandableText::new(ctx, t, TextSize::Sm, TextStyle::Secondary, Align::Left, None));

        TextInput { 
            layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            label: label.map(|text| Text::new(ctx, text, TextSize::H5, TextStyle::Heading, Align::Left, None)),
            inner: input_field, 
            hint: EitherOr::new(help, error),
            error: None
        }
    }

    pub fn value(&mut self) -> String {
        self.inner.2.as_any().downcast_ref::<_InputContent>().unwrap().value.to_string()
    }  
}

impl OnEvent for TextInput { 
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> { 
        if event.as_any().downcast_ref::<TickEvent>().is_some() { 
            self.hint.display_left(self.error.is_some()); 
            self.inner.error(self.error.is_some());
            if let Some(e) = &self.error { 
                self.hint.right().0.spans[0] = e.to_string(); 
            } 
        } 
        vec![event] 
    } 
}


#[derive(Component)]
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
        ctx: &mut Context,
        value: Option<&str>,
        placeholder: Option<&str>,
        button: Option<(&str, InputCallback)>,
    ) -> Self {
        let (button, on_submit) = button.map(|(icon, cb)| {
            let btn = SecondaryIconButton::medium(ctx, icon, |ctx: &mut Context| ctx.trigger_event(TextInputEvent::Submit));
            (Some(btn), Some(cb))
        }).unwrap_or((None, None));
        
        let default = TextEditor::new(ctx, value.unwrap_or_default(), TextSize::Md, TextStyle::Primary, Align::Left); 
        let empty = ExpandableText::new(ctx, placeholder.unwrap_or("Enter text..."), TextSize::Md, TextStyle::Secondary, Align::Left, None);
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
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> { 
        if let Some(events::TextInput::Focused(x)) = event.downcast_ref::<events::TextInput>() {
            self.is_focused = *x;
        } else if event.downcast_ref::<TickEvent>().is_some() {
            self.value = self.default.inner().inner().1.0.spans[0].clone();

            self.default.display(self.is_focused);
            self.empty.display(!self.is_focused);
            self.default.inner().inner().display_cursor(self.is_focused);

            if !self.is_focused {
                self.default.display(!self.value.is_empty());
                self.empty.display(self.value.is_empty());
            }
        } else if let Some(TextInputEvent::Submit) = event.downcast_ref::<TextInputEvent>() { 
            if let Some(on_submit) = &mut self.on_submit {
                (on_submit)(ctx, &mut self.value);
            }
        }
        vec![event]
    }
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Submit,
}

impl Event for TextInputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
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
