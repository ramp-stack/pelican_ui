use prism::events::{Event, DynEvent, OnEvent, on_event, Change, ChangeFuture, self as event};
use prism::drawable::text::{Align, Text as BasicText};
use prism::drawable::{Component, Drawable};
use prism::Context;
use ptsd::layout::{Padding, Column, Offset, Size, Row, Stack};
use ptsd::display::{EitherOr, Opt, Bin};
use ptsd::interactions;
use ptsd::Validate;
use uuid::Uuid;

use std::ops::{Deref, DerefMut};

use crate::theme::{Theme, Color, Icons};
use crate::components::text::{Text, TextSize, TextStyle, TextEditor, ExpandableText};
use crate::components::Rectangle;
use crate::components::button::SecondaryIconButton;
use crate::interface::general::{InterfaceEvent, FormEvent};

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Focus {id: Uuid},
    Submit {id: Uuid},
    Changed {id: Uuid, value: String},
    Set {id: Uuid, value: String},
    Refresh {id: Uuid},
}

impl Event for TextInputEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

crate::closure_trait!(SubmitFn, FnMut(&mut Context, &mut String));
pub trait ValidFn: for<'a> Validate<&'a mut String, bool> {
    fn clone_box(&self) -> Box<dyn ValidFn>;
}
impl<V: for<'a> Validate<&'a mut String, bool> + Clone> ValidFn for V {
    fn clone_box(&self) -> Box<dyn ValidFn> {Box::new(self.clone())}
}
impl Clone for Box<dyn ValidFn> {fn clone(&self) -> Self {self.as_ref().clone_box()}}
impl PartialEq for dyn ValidFn {fn eq(&self, _: &Self) -> bool {true}}
impl std::fmt::Debug for dyn ValidFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("ValidFn")}
}

#[derive(Debug, Component, Clone)]
pub struct TextInput {
    layout: Column,
    label: Option<Text>,
    pub inner: interactions::InputField,
    hint: EitherOr<Option<ExpandableText>, ExpandableText>,
    #[skip] error: Option<String>,
}

impl TextInput {
    pub fn new(
        theme: &Theme, id: Uuid, value: Option<&str>, label: Option<&str>,
        placeholder: Option<&str>, mut help_text: Option<&str>,
        icon_button: Option<SubmitButton>,
    ) -> Self {
        let background = |bg: Color, o: Color| Rectangle::new(bg, 8.0, Some((1.0, o)));
        let colors = theme.colors();
        let input_field = interactions::InputField::new(
            background(Color::transparent(), colors.get(ptsd::Outline::Secondary)),
            background(Color::transparent(), colors.get(ptsd::Outline::Primary)),
            Some(background(colors.get(ptsd::Background::Secondary), colors.get(ptsd::Outline::Secondary))),
            Some(background(Color::transparent(), colors.get(ptsd::Status::Danger))),
            _InputContent::new(theme, id, value, placeholder, icon_button),
            48.0,
        );

        if let Some(h) = help_text && h.is_empty() {help_text = None};
        let error = ExpandableText::new(theme, "", TextSize::Sm, TextStyle::Error, Align::Left, None);
        let help = help_text.map(|text| {
            ExpandableText::new(theme, text, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        });

        TextInput {
            layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            label: label.map(|l| Text::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None)),
            inner: input_field,
            hint: EitherOr::new(help, error),
            error: None,
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, Uuid::new_v4(), None, Some("First name"), None, None, None)
    }

    pub fn id(&self) -> Uuid {self.content().default.inner_ref().id()}
    pub fn value(&self) -> String {self.content().default.inner_ref().value().into_owned()}
    pub fn has_focus(&self) -> bool {self.content().is_focused}
    pub fn refresh(&self, ctx: &mut Context) {ctx.emit(TextInputEvent::Refresh {id: self.id()});}

    pub fn insert(&mut self, ctx: &mut Context, text: &str) {
        self.content_mut().default.inner().inner().insert(ctx, text);
    }
    pub fn delete_back(&mut self, ctx: &mut Context) {
        self.content_mut().default.inner().inner().delete_back(ctx);
    }
    pub fn set_value(&mut self, ctx: &mut Context, value: &str) {
        self.content_mut().default.inner().inner().set_text(ctx, value);
    }

    fn content(&self) -> &_InputContent {(*self.inner.2).as_any().downcast_ref().unwrap()}
    fn content_mut(&mut self) -> &mut _InputContent {(*self.inner.2).as_any_mut().downcast_mut().unwrap()}

    pub fn error(&mut self, error: Result<(), String>) {
        let error = error.err().filter(|error| !error.is_empty());
        if self.error == error {return}
        self.inner.error(error.is_some());
        self.hint.display_left(error.is_none());
        if let Some(error) = &error {self.hint.right().0.spans[0].text.clone_from(error);}
        self.error = error;
    }
}

impl Deref for TextInput {
    type Target = BasicText;
    fn deref(&self) -> &Self::Target {self.content().default.inner_ref().1.deref()}
}

impl DerefMut for TextInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.content_mut().default.inner().inner().deref_mut()
    }
}

#[on_event]
impl OnEvent<TextInputEvent> for TextInput {
    fn on(&mut self, ctx: &mut Context, event: TextInputEvent) -> Vec<Box<dyn DynEvent>> {
        if let TextInputEvent::Focus {id} = &event && *id == self.id() {
            self.inner.focus(ctx);
            return vec![];
        }
        prism::events![event]
    }
}

type SubmitButton = (Icons, Box<dyn SubmitFn>, Box<dyn ValidFn>);

#[derive(Component, Clone)]
#[on_dyn_event]
#[changed]
struct _InputContent {
    layout: Row,
    default: Opt<Bin<Stack, TextEditor>>,
    empty: Opt<Bin<Stack, ExpandableText>>,
    button: Option<SecondaryIconButton>,
    #[skip] on_submit: Option<Box<dyn SubmitFn>>,
    #[skip] is_focused: bool,
    #[skip] is_valid: Option<Box<dyn ValidFn>>,
    #[skip] form: Option<Uuid>,
    #[skip] initial: bool,
}

impl std::fmt::Debug for _InputContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_InputContent")
    }
}

impl _InputContent {
    pub fn new(
        theme: &Theme, id: Uuid, value: Option<&str>, placeholder: Option<&str>,
        button: Option<SubmitButton>,
    ) -> Self {
        let trailing = if button.is_some() {0.0} else {8.0};
        let (button, on_submit, is_valid) = button.map(|(icon, on_submit, is_valid)| {
            let layout = Stack(
                Offset::Center, Offset::End, Size::Fit, Size::Fit, Padding(0.0, 8.0, 8.0, 8.0),
            );
            let button = SecondaryIconButton::medium(
                theme, icon, layout,
                move |ctx: &mut Context, _: &Theme| ctx.emit(TextInputEvent::Submit {id}),
            );
            (Some(button), Some(on_submit), Some(is_valid))
        }).unwrap_or((None, None, None));

        let default = TextEditor::new(
            theme, id, value.unwrap_or_default(), TextSize::Md, TextStyle::Primary, Align::Left,
        );
        let empty = ExpandableText::new(
            theme, placeholder.unwrap_or("Enter text..."),
            TextSize::Md, TextStyle::Secondary, Align::Left, None,
        );
        _InputContent {
            layout: Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 0.0, trailing, 0.0)),
            default: Opt::new(Bin(
                Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 16.0, 16.0, 16.0)),
                default,
            ), value.is_some_and(|value| !value.is_empty())),
            empty: Opt::new(Bin(
                Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 16.0, 16.0, 16.0)),
                empty,
            ), value.is_none_or(str::is_empty)),
            button,
            on_submit,
            is_focused: false,
            is_valid,
            form: None, initial: true,
        }
    }

    fn refresh(&mut self, ctx: &mut Context) {
        if let Some(is_valid) = &mut self.is_valid && let Some(button) = &mut self.button {
            let mut value = self.default.inner_ref().value().into_owned();
            button.1.disable(is_valid.validate(ctx, &mut value));
        }
        let empty = self.default.inner_ref().value().is_empty();
        self.default.display(self.is_focused || !empty);
        self.empty.display(!self.is_focused && empty);
        self.default.inner().inner().display_cursor(self.is_focused);
    }
}

impl _InputContent {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let id = self.default.inner().inner().id();
        if let Some(FormEvent::Bind(form)) = event.downcast_ref::<FormEvent>() {
            self.form = Some(*form);
            return vec![];
        } else if !ctx.window.touch_controls
            && self.is_focused
            && self.on_submit.is_some()
            && let Some(event::KeyboardEvent {
                key: event::Key::Enter, state: event::KeyboardState::Pressed, ..
            }) = event.downcast_ref::<event::KeyboardEvent>() {
            ctx.emit(TextInputEvent::Submit {id});
            return vec![];
        } else if let Some(TextInputEvent::Set {id: target, value}) = event.downcast_ref::<TextInputEvent>()
            && *target == id
        {
            self.default.inner().inner().set_text(ctx, value);
            return vec![];
        } else if let Some(TextInputEvent::Changed {id: source, ..}) = event.downcast_ref::<TextInputEvent>()
            && *source == id
        {
            self.refresh(ctx);
            if let Some(form) = self.form {ctx.emit(FormEvent::Refresh(form));}
        } else if let Some(TextInputEvent::Refresh {id: target}) = event.downcast_ref::<TextInputEvent>()
            && *target == id {
            self.refresh(ctx);
        } else if let Some(ptsd::events::TextInput::Focused(x)) =
            event.downcast_ref::<ptsd::events::TextInput>() {
            self.is_focused = *x;
            self.refresh(ctx);
            if ctx.window.touch_controls {ctx.emit(InterfaceEvent::FocusTextInput(*x));}
        } else if let Some(TextInputEvent::Submit {id: target}) = event.downcast_ref::<TextInputEvent>()
            && *target == id
            && let Some(on_submit) = &mut self.on_submit
        {
            let mut value = self.default.inner_ref().value().into_owned();
            on_submit(ctx, &mut value);
            self.default.inner().inner().set_text(ctx, "");
            return vec![];
        }
        vec![event]
    }
}

impl _InputContent {
    fn changed(&mut self) -> ChangeFuture<'_> {
        let editor = self.default.inner().inner();
        let mut changes: Vec<ChangeFuture<'_>> = vec![Box::pin(async move {
            Drawable::changed(editor).await.map::<Self>(|this| this.default.inner().inner())
        })];
        if let Some(button) = &mut self.button {
            changes.push(Box::pin(async move {
                Drawable::changed(button).await.map::<Self>(|this| this.button.as_mut().unwrap())
            }));
        }
        let initial = &mut self.initial;
        let valid = &mut self.is_valid;
        changes.push(Box::pin(async move {
            if !std::mem::take(initial) {
                match valid {
                    Some(valid) => if valid.change().await.is_none() {std::future::pending::<()>().await;},
                    None => std::future::pending().await,
                }
            }
            Change::new(|drawable, ctx| drawable.downcast_mut::<Self>().unwrap().refresh(ctx))
        }));
        Box::pin(Change::select(changes))
    }
}
