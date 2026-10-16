use prism::event::{OnEvent, TickEvent, Event, self};
use prism::canvas::Align;
use prism::drawable::{Component, SizedTree};
use prism::Context;
use prism::layout::{Padding, Column, Offset, Size, Row, Stack};
use prism::display::{EitherOr, Opt, Bin};

use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextSize, TextStyle, TextEditor, ExpandableText};
use crate::components::Rectangle;
use crate::components::button::SecondaryIconButton;
use crate::components::QRCodeScannedEvent;
use crate::interface::general::InterfaceEvent;
use crate::interface::system::{KeyboardPages, MobileKeyboardEvent};

crate::closure_trait!(SubmitFn, FnMut(&mut Context, &mut String));
crate::closure_trait!(ValidFn, FnMut(&mut Context, &mut String) -> bool);

type SubmitButton = (Icons, Box<dyn SubmitFn>, Box<dyn ValidFn>);

#[derive(Debug, Component, Clone)]
pub struct TextInput {
    layout: Column,
    label: Option<Text>,
    pub inner: interactions::InputField,
    hint: EitherOr<Option<ExpandableText>, ExpandableText>,
    #[skip] pub error: Option<String>,
    #[skip] pub is_focused: bool,
}

impl TextInput {
    pub fn new(
        theme: &Theme,
        value: Option<&str>,
        label: Option<&str>,
        placeholder: Option<&str>,
        mut help_text: Option<&str>,
        icon_button: Option<SubmitButton>,
        keyboard_pages: KeyboardPages,
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

        if let Some(h) = help_text && h.is_empty() {help_text = None};
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

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, None, Some("First name"), None, None, None, KeyboardPages::default())
    }

    pub fn value(&self) -> String {
        self.inner.2.as_any().downcast_ref::<_InputContent>().unwrap().value.to_string()
    }

    pub fn has_focus(&self) -> bool {
        self.inner.2.as_any().downcast_ref::<_InputContent>().is_some_and(|content| content.is_focused)
    }

    pub fn insert(&mut self, text: &str) {
        if let Some(content) = (*self.inner.2).as_any_mut().downcast_mut::<_InputContent>() {
            content.default.inner().inner().insert(text);
            content.value = content.default.inner().inner().1.0.spans[0].clone();
        }
    }

    pub fn delete_back(&mut self) {
        if let Some(content) = (*self.inner.2).as_any_mut().downcast_mut::<_InputContent>() {
            content.default.inner().inner().delete_back();
            content.value = content.default.inner().inner().1.0.spans[0].clone();
        }
    }

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
        if event.downcast_ref::<TickEvent>().is_some() {
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
    #[skip] id: uuid::Uuid,
    #[skip] keyboard_pages: KeyboardPages,
    #[skip] pub value: String,
    #[skip] on_submit: Option<Box<dyn SubmitFn>>,
    #[skip] is_focused: bool,
    #[skip] is_valid: Option<Box<dyn ValidFn>>,
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
        button: Option<SubmitButton>,
        keyboard_pages: KeyboardPages
    ) -> Self {
        let id = uuid::Uuid::new_v4();
        let (button, on_submit, is_valid) = button.map(|(icon, on_submit, is_valid)| {
            let button = SecondaryIconButton::medium(theme, icon, move |ctx: &mut Context, _: &Theme| ctx.emit(TextInputEvent::Submit(id)));
            (Some(button), Some(on_submit), Some(is_valid))
        }).unwrap_or((None, None, None));

        let default = TextEditor::new(theme, value.unwrap_or_default(), TextSize::Md, TextStyle::Primary, Align::Left);
        let empty = ExpandableText::new(theme, placeholder.unwrap_or("Enter text..."), TextSize::Md, TextStyle::Secondary, Align::Left, None);
        _InputContent {
            layout: Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 8.0, 8.0, 8.0)),
            default: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 8.0, 16.0, 8.0)), default), false),
            empty: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 8.0, 16.0, 8.0)), empty), true),
            button,
            id,
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
}

impl OnEvent for _InputContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(mouse) = event.downcast_ref::<event::MouseEvent>() {
            let editor = sized.1.first().and_then(|(o1, opt)| {
                let (o2, bin) = opt.1.first()?;
                let (o3, editor) = bin.1.first()?;
                Some(((o1.0 + o2.0 + o3.0, o1.1 + o2.1 + o3.1), editor.0))
            });
            if let Some((origin, size)) = editor {
                let local = mouse.position.map(|p| (p.0 - origin.0, p.1 - origin.1));
                let editor = self.default.inner().inner();
                match mouse.state {
                    event::MouseState::Pressed(event::MouseButton::Left) => {
                        if let Some(p) = local && p.0 >= -origin.0 && p.0 <= size.0 {
                            editor.pointer_pressed(p);
                        }
                    }
                    event::MouseState::Moved => editor.pointer_moved(local, None),
                    event::MouseState::Scroll(dx, dy) if crate::IS_MOBILE => editor.pointer_moved(local, Some((-dx, -dy))),
                    event::MouseState::Released(_) => editor.pointer_released(),
                    _ => {}
                }
            }
            return vec![event];
        }

        if !crate::IS_MOBILE
            && self.is_focused
            && self.on_submit.is_some()
            && let Some(event::KeyboardEvent{key: event::Key::Enter, state: event::KeyboardState::Pressed, ..}) = event.downcast_ref::<event::KeyboardEvent>()
        {
            ctx.emit(TextInputEvent::Submit(self.id));
            return vec![];
        } else if let Some(TextInputEvent::Set(data)) = event.downcast_ref::<TextInputEvent>() {
            self.set_value(data);
        } else if let Some(QRCodeScannedEvent(data)) = event.downcast_ref::<QRCodeScannedEvent>() {
            self.set_value(data);
        } else if let Some(event::TextInput::Focused(x)) = event.downcast_ref::<event::TextInput>() {
            self.is_focused = *x;
            if crate::IS_MOBILE {ctx.emit(InterfaceEvent::FocusTextInput(*x));}
            if crate::IS_MOBILE && *x {ctx.emit(MobileKeyboardEvent::Pages(self.keyboard_pages));}
        } else if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(is_valid) = &mut self.is_valid && let Some(button) = &mut self.button {
                button.1.disable(is_valid(ctx, &mut self.value));
            }

            self.value = self.default.inner().inner().1.0.spans[0].clone();

            self.default.display(self.is_focused);
            self.empty.display(!self.is_focused);
            self.default.inner().inner().display_cursor(self.is_focused);

            if !self.is_focused {
                self.default.display(!self.value.is_empty());
                self.empty.display(self.value.is_empty());
            }
        } else if let Some(TextInputEvent::Submit(id)) = event.downcast_ref::<TextInputEvent>()
            && *id == self.id
            && let Some(on_submit) = &mut self.on_submit
        {
            on_submit(ctx, &mut self.value);
            self.default.inner().inner().set_text("");
            self.value.clear();
        }
        vec![event]
    }
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Submit(uuid::Uuid),
    Edited(String, String),
    Set(String),
}

prism::impl_event_all_children!(TextInputEvent);
