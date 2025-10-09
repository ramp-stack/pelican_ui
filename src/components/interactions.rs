use mustache::events::{MouseState, MouseEvent, OnEvent, Event, TickEvent, KeyboardState, KeyboardEvent};
use mustache::drawable::{Drawable, Color};
use mustache::{Context, Component};

use std::sync::mpsc;

// use crate::components::avatar::{Avatar, AvatarContent};
use crate::utils::{Callback, ElementID};
use crate::layout::{Stack, Bin, Opt, Offset, Size, Row, Padding, EitherOr};
use crate::components::{ExpandableText, TextEditor, Rectangle};
use crate::plugin::PelicanUI;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum ButtonState {Default, Disabled, Selected, Pressed, Hover}

#[derive(Component)]
pub struct Button {
    layout: Stack,
    default: Opt<Box<dyn Drawable>>,
    hover: Opt<Box<dyn Drawable>>,
    pressed: Opt<Box<dyn Drawable>>,
    selected: Opt<Box<dyn Drawable>>,
    disabled: Opt<Box<dyn Drawable>>,
    #[skip] state: ButtonState,
    #[skip] on_click: Callback,
}

impl Button{
    pub fn new(
        on_click: Callback,
        default: impl Drawable + 'static,
        hover: impl Drawable + 'static,
        pressed: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        disabled: impl Drawable + 'static,
        state: ButtonState
    ) -> Self {
        Button {
            layout: Stack::default(),
            on_click,
            default: Opt::new(Box::new(default), state == ButtonState::Default),
            disabled: Opt::new(Box::new(disabled), state == ButtonState::Disabled),
            selected: Opt::new(Box::new(selected), state == ButtonState::Selected),
            pressed: Opt::new(Box::new(pressed), state == ButtonState::Pressed),
            hover: Opt::new(Box::new(hover), state == ButtonState::Hover),
            state,
        }
    }

    pub fn selected(&mut self, is_selected: bool) {
        self.state = if is_selected {ButtonState::Selected} else {ButtonState::Default};
    }

    pub fn disabled(&mut self, is_disabled: bool) {
        self.state = if is_disabled {ButtonState::Disabled} else {ButtonState::Default};
    }
}

impl OnEvent for Button {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() {
            self.default.display(self.state == ButtonState::Default);
            self.hover.display(self.state == ButtonState::Hover);
            self.pressed.display(self.state == ButtonState::Pressed);
            self.selected.display(self.state == ButtonState::Selected);
            self.disabled.display(self.state == ButtonState::Disabled);
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            let state = match self.state {
                ButtonState::Default if event.position.is_some() => {
                    match event.state {
                        MouseState::Pressed => {
                            Some(ButtonState::Pressed)
                        },
                        MouseState::Moved | MouseState::Scroll(..) => Some(if mustache::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
                        _ => None
                    }
                },
                ButtonState::Pressed => {
                    match event.state {
                        MouseState::Released if event.position.is_some() => Some(if mustache::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
                        MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
                        _ => None
                    }
                },
                ButtonState::Hover => {
                    match event.state {
                        MouseState::Pressed if event.position.is_some() => Some(ButtonState::Pressed),
                        MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
                        _ => None
                    }
                }
                _ => None
            };

            if let Some(state) = state { self.state = state; }
            if let MouseEvent { state: MouseState::Pressed, position: Some(_) } = event {
                if matches!(self.state, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
                    ctx.hardware.haptic();
                    (self.on_click)(ctx);
                }
            }
        }
        false
    }
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Button")
    }
}


#[derive(Component)]
pub struct Selectable {
    layout: Stack,
    default: Opt<Box<dyn Drawable>>,
    selected: Opt<Box<dyn Drawable>>,
    #[skip] is_selected: bool,
    #[skip] on_click: Callback,
    #[skip] id: ElementID,
}

impl std::fmt::Debug for Selectable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Selectable")
    }
}

impl Selectable {
    pub fn new(
        on_click: impl FnMut(&mut Context) + 'static,
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
    ) -> Self {
        Selectable {
            layout: Stack::default(),
            on_click: Box::new(on_click),
            default: Opt::new(Box::new(default), !is_selected),
            selected: Opt::new(Box::new(selected), is_selected),
            is_selected,
            id: ElementID::new(),
        }
    }

    pub fn selected(&mut self, is_selected: bool) { self.is_selected = is_selected; }
}

impl OnEvent for Selectable {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() {
            self.default.display(!self.is_selected);
            self.selected.display(self.is_selected);
        } else if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            (self.on_click)(ctx);
            ctx.trigger_event(SelectableEvent(self.id))
        } else if let Some(SelectableEvent(id)) = event.downcast_ref::<SelectableEvent>() {
            self.is_selected = *id == self.id;
        }
        false
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SelectableEvent(pub ElementID);
impl Event for SelectableEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

// start: f32, 
// on_release: impl FnMut(&mut Context, f32) + 'static, 
// background: impl Drawable + 'static, 
// foreground: impl Drawable + 'static,
// moveable: impl Drawable + 'static,

type SliderClosure = Box<dyn FnMut(&mut Context, f32)>;

impl std::fmt::Debug for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slider")
    }
}

#[derive(Component)]
pub struct Slider {
    layout: Stack,
    pub background: Bin<Stack, Rectangle>,
    pub foreground: Bin<Stack, Rectangle>,
    pub knob: SliderKnob,
    #[skip] value: f32,
    #[skip] closure: SliderClosure,
    #[skip] dragging: bool,
}

impl Slider {
    pub fn new(
        ctx: &mut Context, 
        start: f32, 
        background: Color,
        knob_size: (f32, f32),
        knob: impl Drawable + 'static,
        on_release: impl FnMut(&mut Context, f32) + 'static
    ) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(300.0), f32::MAX));
        let track = Stack(Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default());
        let fill = Stack(Offset::Start, Offset::Start, Size::Static(30.0), Size::Static(6.0), Padding::default());
        let layout = Stack(Offset::Start, Offset::Center, Size::Fit, Size::Fit, Padding::default());
        let brand = ctx.get::<PelicanUI>().get().0.theme().colors.brand;

        Slider {
            layout,
            background: Bin(track, Rectangle::new(background, 3.0, None)),
            foreground: Bin(fill, Rectangle::new(brand, 3.0, None)),
            knob: SliderKnob::new(knob_size, knob),
            value: start, 
            closure: Box::new(on_release),
            dragging: false,
        }
    }

    pub fn set_value(&mut self) {
        self.value = self.value.clamp(0.0, 1.0);
        let track_width = self.foreground.inner().size().0;
        self.knob.adjust_position(self.value * track_width, track_width);
    }


    fn set_knob_pixel(&mut self, px: f32, track_width: f32) {
        let clamped = px.clamp(0.0, track_width);
        self.knob.adjust_position(clamped, track_width);
        self.foreground.layout().2 = Size::Static(clamped);
    }
}

// impl Drawable for Box<dyn SliderBackground> {
//     fn request_size(&self, ctx: &mut Context) -> RequestBranch {Drawable::request_size(&**self, ctx)}

//     fn name(&self) -> String {Drawable::name(&**self)}
//     fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> { self }
//     fn as_any(&self) -> &dyn std::any::Any { self }
//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
// }


impl OnEvent for Slider {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        let width = self.background.inner().size().0;
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            self.set_value();
        } else if let Some(MouseEvent { state: MouseState::Pressed, position: Some((x, _)) }) = event.downcast_ref::<MouseEvent>() {
            self.dragging = true;

            if width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.value = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p = self.value;
                (self.closure)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Scroll(..), position: Some((x, _))}) = event.downcast_ref::<MouseEvent>() {
            if self.dragging && width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.value = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p = self.value;
                (self.closure)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Moved, position: Some((x, _)) }) = event.downcast_ref::<MouseEvent>() {
            if self.dragging && width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.value = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p = self.value;
                (self.closure)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Released, .. }) = event.downcast_ref::<MouseEvent>() {
            if self.dragging {
                self.dragging = false;
                let p = self.value;
                (self.closure)(ctx, p);
            }
        } else if event.downcast_ref::<TickEvent>().is_some() && width > 0.0 {
            self.set_knob_pixel(self.value * width, width);
        }

        true
    }
}

#[derive(Debug, Component)]
pub struct SliderKnob(Stack, Box<dyn Drawable>);
impl OnEvent for SliderKnob {}

impl SliderKnob {
    pub fn new(size: (f32, f32), knob: impl Drawable + 'static) -> Self {
        let layout = Stack(Offset::Start, Offset::Start, Size::Static(size.0), Size::Static(size.1), Padding::default());
        SliderKnob(layout, Box::new(knob))
    }

    pub fn adjust_position(&mut self, x: f32, track_width: f32) {
        let clamped_x = x.clamp(9.0, track_width);
        self.0.0 = Offset::Static(clamped_x - 9.0);
    }
}

#[derive(Debug, Component)]
pub struct InputField {
    _layout: Stack,
    _default: Opt<Box<dyn Drawable>>,
    _hover: Opt<Box<dyn Drawable>>,
    _focus: Opt<Box<dyn Drawable>>,
    _error: Opt<Box<dyn Drawable>>,
    _content: InputContent,
    #[skip] _state: InputState,
    #[skip] pub id: ElementID,
    #[skip] pub error: Option<String>,
}

impl InputField {
    pub fn new(
        default: impl Drawable + 'static,
        hover: impl Drawable + 'static,
        focus: impl Drawable + 'static,
        error: impl Drawable + 'static,
        content: InputContent,
    ) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| (heights[4].0.max(48.0), heights[4].1.max(48.0)));
        InputField {
            _layout: Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()),
            _default: Opt::new(Box::new(default), true),
            _hover: Opt::new(Box::new(hover), false),
            _focus: Opt::new(Box::new(focus), false),
            _error: Opt::new(Box::new(error), false),
            _content: content,
            _state: InputState::Default,
            id: ElementID::new(),
            error: None,
        }
    }

    pub fn focus(&mut self, is_focused: bool) {
        self._state = if is_focused {InputState::Focus} else {InputState::Default};
    }
}

impl OnEvent for InputField {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            match (self._state, self.error.is_some()) {
                (InputState::Default, true) => self._state = InputState::Error,
                (InputState::Error, false) => self._state = InputState::Default,
                _ => {}
            };

            self._content.state = self._state;
            self._content.error = self.error.is_some();

            self._default.display(self._state == InputState::Default);
            self._hover.display(self._state == InputState::Hover);
            self._focus.display(self._state == InputState::Focus);
            self._error.display(self._state == InputState::Error);
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self._state = match self._state {
                InputState::Default => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
                            ctx.hardware.haptic();
                            ctx.trigger_event(TextInputEvent::TextInputSelect(self.id));
                            ctx.trigger_event(TextInputEvent::ShowKeyboard(false)); 
                            Some(InputState::Focus)
                        },
                        MouseEvent{state: MouseState::Moved, position: Some(_)} => Some(InputState::Hover),
                        _ => None
                    }
                },
                InputState::Hover => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
                            ctx.trigger_event(TextInputEvent::TextInputSelect(self.id));
                            Some(InputState::Focus)
                        },
                        MouseEvent{state: MouseState::Moved, position: None} if self.error.is_some() => Some(InputState::Error),
                        MouseEvent{state: MouseState::Moved, position: None} => Some(InputState::Default),
                        _ => None
                    }
                },
                InputState::Focus => {
                    match event {
                        MouseEvent{state: MouseState::Pressed, position: None} if self.error.is_some() && !mustache::IS_MOBILE => Some(InputState::Error),
                        MouseEvent{state: MouseState::Pressed, position: None} if !mustache::IS_MOBILE => Some(InputState::Default),
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
            }.unwrap_or(self._state);
        } else if let Some(input_event) = event.downcast_ref::<TextInputEvent>() {
            match input_event {
                TextInputEvent::TextInputSelect(id) => match *id == self.id { 
                    true => self._state = InputState::Focus,
                    false if self.error.is_some() => self._state = InputState::Error,
                    false  => self._state = InputState::Default,
                },
                TextInputEvent::ShowKeyboard(false) if self._state == InputState::Focus => {
                    self._state = if self.error.is_some() {InputState::Error} else {InputState::Default};
                }
                _ => {}
            }
        }
        true
    }
}

pub type SubmitCallback = Box<dyn FnMut(&mut Context, &mut String)>;

#[derive(Component)]
pub struct InputContent {
    layout: Row,
    default: Opt<EitherOr<ExpandableText, ExpandableText>>,
    focus: Opt<TextEditor>,
    button: Option<Bin<Stack, Button>>,
    #[skip] state: InputState,
    #[skip] on_submit: Option<(mpsc::Receiver<u8>, SubmitCallback)>,
    #[skip] error: bool,
    #[skip] value: String,
}

impl InputContent {
    pub fn new(
        editor: TextEditor,
        default: ExpandableText,
        placeholder: ExpandableText,
        button: Option<(Button, mpsc::Receiver<u8>, SubmitCallback)>,
    ) -> Self {
        let value = editor.1.0.spans[0].to_string();
        let (button, on_submit) = button.map(|(b, r, c)| (Some(b), Some((r, c)))).unwrap_or((None, None));
        let bin_layout = Stack(Offset::default(), Offset::End, Size::Fit, Size::Fit, Padding(-8.0, -8.0, -8.0, -8.0));
        let button = button.map(|b| Bin(bin_layout, b));
        InputContent {
            layout: Row::new(0.0, Offset::Start, Size::Fill, Padding(16.0, 14.0, 16.0, 14.0)),
            default: Opt::new(EitherOr::new(default, placeholder), true),
            focus: Opt::new(editor, false),
            button,
            state: InputState::Default,
            on_submit,
            error: false,
            value,
        }
    }
}

impl std::fmt::Debug for InputContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InputContent")
    }
}

impl OnEvent for InputContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            self.focus.display(self.state == InputState::Focus);
            self.default.display(self.state != InputState::Focus);

            if let Some((receiver, on_submit)) = &mut self.on_submit {
                if receiver.try_recv().is_ok() {
                    on_submit(ctx, &mut self.value)
                }
            }

            if self.state != InputState::Focus {
                self.default.inner().display_left(!self.value.is_empty());
            }

            self.default.inner().left().0.spans[0] = self.value.clone();
            self.value = self.focus.inner().1.0.spans[0].clone();
        } else if let Some(input_event) = event.downcast_ref::<TextInputEvent>() {
            match input_event {
                TextInputEvent::ClearActiveInput => self.focus.inner().1.0.spans[0] = String::new(),
                TextInputEvent::SetActiveInput(s) => self.focus.inner().1.0.spans[0] = s.to_string(),
                _ => {}
            }
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref() {
            if self.state == InputState::Focus {
                self.focus.inner().apply_edit(ctx, key);
                ctx.trigger_event(TextInputEvent::InputEditedEvent);
            }
        } 
        true
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum InputState {
    Default,
    Hover,
    Focus,
    Error
}

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    /// Event used to bring up or hide the keyboard.
    ShowKeyboard(bool),
    /// Clears the contents of the active text input.
    ClearActiveInput,
    /// Sets the contents of the active [`TextInput`] with the provided `String`
    SetActiveInput(String),
    /// Selects the [`TextInput`] with the given [`ElementID`] and deselects all other items.
    TextInputSelect(ElementID),
    /// Event trigger by [`TextInput`] when contents are edited. 
    InputEditedEvent
}

impl Event for TextInputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}