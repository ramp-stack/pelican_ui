use prism::event::{OnEvent, TickEvent, Event, self};
use prism::canvas::Align;
use prism::drawable::{Component, SizedTree, Drawable};
use prism::{Context};
use prism::layout::{Padding, Column, Offset, Size, Row, Stack, Area};
use prism::display::{EitherOr, Opt, Bin};

use ptsd::interactions;

use std::sync::{Arc, Mutex};

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextSize, TextStyle, TextEditor, ExpandableText};
use crate::components::Rectangle;
use crate::components::button::SecondaryIconButton;
use crate::components::QRCodeScannedEvent;
use crate::interface::system::KeyboardPages;

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
    #[skip] pub is_focused: bool,
}

type InputCallback = Arc<Mutex<dyn FnMut(&mut Context, &mut String) + 'static>>;
type ValidTextCallback = Arc<Mutex<dyn FnMut(&mut Context, &mut String) -> bool + 'static>>;

impl TextInput {
    pub fn new(
        theme: &Theme,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: Option<&str>,
        mut help_text: Option<&str>,
        icon_button: Option<(Icons, InputCallback, ValidTextCallback)>,
        keyboard_pages: KeyboardPages,
        // on_edit: impl FnMut(&mut Context, &mut String) + 'static,
    ) -> Self {
        let background = |bg: Color, o: Color| Rectangle::new(bg, 8.0, Some((1.0, o)));
        let colors = theme.colors();
        let input_field = interactions::InputField::new(
            background(Color::TRANSPARENT, colors.get(ptsd::Outline::Secondary)),
            background(Color::TRANSPARENT, colors.get(ptsd::Outline::Primary)),
            Some(background(colors.get(ptsd::Background::Secondary), colors.get(ptsd::Outline::Secondary))),
            Some(background(Color::TRANSPARENT, colors.get(ptsd::Status::Danger))),
            _InputContent::new(theme, value, placeholder, icon_button, keyboard_pages),
            48.0,
        );

        if let Some(h) = help_text {if h.is_empty() {help_text = None}};
        let error = ExpandableText::new(theme, "", TextSize::Sm, TextStyle::Error, Align::Left, None); 
        let help = help_text.map(|t| ExpandableText::new(theme, t, TextSize::Sm, TextStyle::Secondary, Align::Left, None));

        TextInput { 
            layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            label: label.map(|l| Text::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None)),
            inner: input_field, 
            hint: EitherOr::new(help, error),
            error: None,
            is_focused: false,
        }
    }
    
    pub fn default(theme: &Theme, ) -> Self {
        Self::new(theme, None, Some("First name"), None, None, None, KeyboardPages::default())
    }

    pub fn value(&self) -> String {
        self.inner.2.as_any().downcast_ref::<_InputContent>().unwrap().value.to_string()
    }

    /// Replaces the text in this input and moves the cursor to the end.
    pub fn set_value(&mut self, value: &str) {
        if let Some(content) = (*self.inner.2).as_any_mut().downcast_mut::<_InputContent>() {
            content.set_value(value);
        }
    }
    
    pub fn error(&mut self, error: Result<(), String>) {
        self.inner.error(matches!(error, Err(ref e) if !e.is_empty()));
        match error {
            Ok(_) => self.error = None,
            Err(e) => self.error = (!e.is_empty()).then_some(e),
        }
    }
}

impl OnEvent for TextInput { 
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.as_any().downcast_ref::<TickEvent>().is_some() { 
            self.is_focused = self.inner.is_focused();
            self.hint.display_left(self.error.is_none()); 
            if let Some(e) = &self.error && !e.is_empty() {
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
    #[skip] keyboard_pages: KeyboardPages,
    #[skip] pub value: String,
    #[skip] on_submit: Option<InputCallback>,
    #[skip] is_focused: bool,
    #[skip] is_valid: Option<ValidTextCallback>,
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
        button: Option<(Icons, InputCallback, ValidTextCallback)>,
        keyboard_pages: KeyboardPages
    ) -> Self {
        let (button, on_submit, is_valid) = button.map(|(icon, cb, vtc)| {
            let btn = SecondaryIconButton::medium(theme, icon, |ctx: &mut Context, _: &Theme| ctx.emit(TextInputEvent::Submit));
            (Some(btn), Some(cb), Some(vtc))
        }).unwrap_or((None, None, None));
        
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
            is_valid,
            keyboard_pages,
        }
    }

    fn set_value(&mut self, value: &str) {
        self.default.inner().inner().set_text(value);
        self.value = value.to_string();
    }

    /// Where the editor sits inside this row, and its size: row → Opt → Bin → TextEditor.
    /// `None` while the editor is hidden (placeholder showing).
    fn editor_area(sized: &SizedTree) -> Option<((f32, f32), (f32, f32))> {
        let (o1, opt) = sized.1.first()?;
        let (o2, bin) = opt.1.first()?;
        let (o3, editor) = bin.1.first()?;
        Some(((o1.0 + o2.0 + o3.0, o1.1 + o2.1 + o3.1), editor.0))
    }

    /// Forwards pointer input to the editor in its own coordinates, so presses
    /// anywhere in the field place the cursor and drags scrub it.
    fn forward_pointer(&mut self, sized: &SizedTree, mouse: &event::MouseEvent) {
        let Some((origin, size)) = Self::editor_area(sized) else { return };
        let local = mouse.position.map(|p| (p.0 - origin.0, p.1 - origin.1));
        let editor = self.default.inner().inner();

        match mouse.state {
            // Presses on the button (right of the editor) are left to the button.
            event::MouseState::Pressed(event::MouseButton::Left) => {
                if let Some(p) = local && p.0 >= -origin.0 && p.0 <= size.0 {
                    editor.pointer_pressed(p);
                }
            }
            event::MouseState::Moved => editor.pointer_moved(local, None),
            // Touch drags arrive as scroll deltas (negated finger movement); they keep
            // coming even after the finger leaves the field, so tracking continues.
            event::MouseState::Scroll(dx, dy) if crate::IS_MOBILE => editor.pointer_moved(local, Some((-dx, -dy))),
            event::MouseState::Released(_) => editor.pointer_released(),
            _ => {}
        }
    }
}

impl OnEvent for _InputContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // The editor's hit area is only its text line, so cursor placement and
        // scrubbing are handled here, where presses anywhere in the field arrive.
        if let Some(mouse) = event.downcast_ref::<event::MouseEvent>() {
            self.forward_pointer(sized, mouse);
            return vec![event];
        }

        // Inside a `Content`, these two events are routed to a single input (see
        // `Content::on_event`). They only arrive here unrouted for inputs outside
        // a `Content`, such as the bumper's message input.
        if !crate::IS_MOBILE
            && self.is_focused
            && self.on_submit.is_some()
            && let Some(event::KeyboardEvent{key: event::Key::Enter, state: event::KeyboardState::Pressed, ..}) = event.downcast_ref::<event::KeyboardEvent>()
        {
            ctx.emit(TextInputEvent::Submit);
            return vec![];
        } else if let Some(TextInputEvent::Set(data)) = event.downcast_ref::<TextInputEvent>() {
            self.set_value(data);
        // } else if let Some(HardwareEvent::Clipboard(data)) = event.downcast_ref::<HardwareEvent>() {
        //     self.default.inner().inner().1.0.spans[0] = data.to_string();
        } else if let Some(QRCodeScannedEvent(data)) = event.downcast_ref::<QRCodeScannedEvent>() {
            self.set_value(data);
        } else if let Some(event::TextInput::Focused(x)) = event.downcast_ref::<event::TextInput>() {
            self.is_focused = *x;
            if crate::IS_MOBILE {ctx.emit(crate::interface::general::InterfaceEvent::FocusTextInput(*x));}
            if crate::IS_MOBILE && *x {ctx.emit(self.keyboard_pages);}
            // println!("FOCUSED {:?}", self.is_focused);
        } else if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(is_valid) = &mut self.is_valid 
            && let Ok(mut cb) = is_valid.lock() {
                let result = (cb)(ctx, &mut self.value);
                self.button.as_mut().unwrap().1.disable(result);
            }
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
            self.default.inner().inner().set_text("");
            self.value.clear();
        }
        vec![event]
    }
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Submit,
    Edited(String, String),
    Set(String),
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