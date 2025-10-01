use mustache::events::{MouseState, MouseEvent, OnEvent, Event, TickEvent};
use mustache::drawable::{Drawable};
use mustache::{Context, Component};

// use crate::components::avatar::{Avatar, AvatarContent};
use crate::utils::{Callback, ElementID};
use crate::layout::{Stack, Opt};

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