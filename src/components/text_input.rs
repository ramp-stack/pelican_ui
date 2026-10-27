use prism::events::{
    Event, DynEvent, OnEvent, on_event, OnChange, on_change, ListenCallback, self as event,
};
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
use crate::components::text::{Text, TextSize, TextStyle, TextEditor};
use crate::components::Rectangle;
use crate::components::button::SecondaryIconButton;
use crate::interface::general::{InterfaceEvent, FormEvent};

type SubmitButton = (Icons, Box<dyn SubmitFn>, Box<dyn ValidFn>);

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
    hint: EitherOr<Option<Text>, Text>,
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
        let error = Text::new(theme, "", TextSize::Sm, TextStyle::Error, Align::Left, None);
        let help = help_text.map(|text| {
            Text::new(theme, text, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        });

        TextInput {
            layout: Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            label: label.map(|l| {
                Text::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None).fit()
            }),
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
        if let Some(error) = &error {self.hint.right().spans[0].text.clone_from(error);}
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

#[derive(Component, Clone)]
struct _InputContent {
    layout: Row,
    default: Opt<Bin<Stack, TextEditor>>,
    empty: Opt<Bin<Stack, Text>>,
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
            let layout = Stack::new(
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
        let empty = Text::new(
            theme, placeholder.unwrap_or("Enter text..."),
            TextSize::Md, TextStyle::Secondary, Align::Left, None,
        );
        _InputContent {
            layout: Row::new(0.0, Offset::End, Size::Fit, Padding(16.0, 0.0, trailing, 0.0)),
            default: Opt::new(Bin(
                Stack::new(
                    Offset::Start, Offset::Start, Size::Fit, Size::Fit,
                    Padding(0.0, 16.0, 16.0, 16.0),
                ),
                default,
            ), value.is_some_and(|value| !value.is_empty())),
            empty: Opt::new(Bin(
                Stack::new(
                    Offset::Start, Offset::Start, Size::Fit, Size::Fit,
                    Padding(0.0, 16.0, 16.0, 16.0),
                ),
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
        self.initial = false;
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

#[on_event]
impl OnEvent<FormEvent> for _InputContent {
    fn on(&mut self, _ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Bind(form) = event {
            self.form = Some(form);
            return vec![];
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<event::KeyboardEvent> for _InputContent {
    fn on(&mut self, ctx: &mut Context, event: event::KeyboardEvent) -> Vec<Box<dyn DynEvent>> {
        if !ctx.window.is_mobile && self.is_focused && self.on_submit.is_some()
            && matches!(event, event::KeyboardEvent {
                key: event::Key::Enter, state: event::KeyboardState::Pressed, ..
            }) {
            let id = self.default.inner().inner().id();
            ctx.emit(TextInputEvent::Submit {id});
            return vec![];
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<TextInputEvent> for _InputContent {
    fn on(&mut self, ctx: &mut Context, event: TextInputEvent) -> Vec<Box<dyn DynEvent>> {
        let id = self.default.inner().inner().id();
        match &event {
            TextInputEvent::Set {id: target, value} if *target == id => {
                self.default.inner().inner().set_text(ctx, value);
                return vec![];
            },
            TextInputEvent::Changed {id: source, ..} if *source == id => {
                self.refresh(ctx);
                if let Some(form) = self.form {ctx.emit(FormEvent::Refresh(form));}
            },
            TextInputEvent::Refresh {id: target} if *target == id => self.refresh(ctx),
            TextInputEvent::Submit {id: target} if *target == id => {
                if let Some(on_submit) = &mut self.on_submit {
                    let mut value = self.default.inner_ref().value().into_owned();
                    on_submit(ctx, &mut value);
                    self.default.inner().inner().set_text(ctx, "");
                    return vec![];
                }
            },
            _ => {},
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<ptsd::events::TextInput> for _InputContent {
    fn on(&mut self, ctx: &mut Context, event: ptsd::events::TextInput) -> Vec<Box<dyn DynEvent>> {
        if let ptsd::events::TextInput::Focused(focused) = event {
            self.is_focused = focused;
            self.refresh(ctx);
            if ctx.window.is_mobile {ctx.emit(InterfaceEvent::FocusTextInput(focused));}
        }
        prism::events![event]
    }
}

#[on_change(Drawable::changed(self.default.inner().inner()))]
impl OnChange<ListenCallback> for _InputContent {
    fn on_change(&mut self, ctx: &mut Context, callback: ListenCallback) {
        callback(self.default.inner().inner(), ctx);
    }
}

#[on_change(async {
    match &mut self.is_valid {
        Some(valid) => if valid.change().await.is_none() {std::future::pending::<()>().await;},
        None => std::future::pending().await,
    }
})]
impl OnChange<()> for _InputContent {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {self.refresh(ctx);}
}

#[on_change(async {
    if !self.initial {std::future::pending::<()>().await;}
})]
impl OnChange<()> for _InputContent {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {self.refresh(ctx);}
}
