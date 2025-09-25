use mustache::events::{MouseState, MouseEvent, OnEvent, Event};
use mustache::drawable::{Drawable, Component};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

// use crate::components::avatar::{Avatar, AvatarContent};
use crate::Callback;
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
}

impl OnEvent for Button {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            let state = match self.state {
                ButtonState::Default if event.position.is_some() => {
                    match event.state {
                        MouseState::Pressed => {
                            Some(ButtonState::Pressed)
                        },
                        MouseState::Moved | MouseState::Scroll(..) => Some(if crate::config::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
                        _ => None
                    }
                },
                ButtonState::Pressed => {
                    match event.state {
                        MouseState::Released if event.position.is_some() => Some(if crate::config::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
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
        }

        self.default.display(self.state == ButtonState::Default);
        self.hover.display(self.state == ButtonState::Hover);
        self.pressed.display(self.state == ButtonState::Pressed);
        self.selected.display(self.state == ButtonState::Selected);
        self.disabled.display(self.state == ButtonState::Disabled);

        if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            if matches!(self.state, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
                ctx.hardware.haptic();
                (self.on_click)(ctx);
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
