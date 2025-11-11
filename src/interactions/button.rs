use roost_ui::events::{self, OnEvent, Event};
use roost_ui::drawable::{Drawable};
use roost_ui::{Context, Component};
use roost_ui::layouts::{Enum, Stack};
use roost_ui::emitters;

#[derive(Component, Debug)]
pub struct Button(Stack, emitters::Button<_Button>);
impl OnEvent for Button {}
impl Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        is_disabled: bool,
        callback: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let button = _Button::new(default, hover, pressed, disabled, is_disabled, callback);
        Self(Stack::default(), emitters::Button::new(button))
    }
}

impl std::ops::Deref for Button {
    type Target = _Button;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component)]
pub struct _Button(Stack, Enum, #[skip] bool, #[skip] Box<dyn FnMut(&mut Context)>);

impl _Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        is_disabled: bool,
        callback: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let start = if is_disabled {"disabled"} else {"default"};
        let mut items: Vec<(String, Box<dyn Drawable>)> = Vec::new();
        items.push(("default".to_string(), Box::new(default)));
        if let Some(h) = hover { items.push(("hover".to_string(), Box::new(h))) }
        if let Some(p) = pressed { items.push(("pressed".to_string(), Box::new(p))) }
        if let Some(d) = disabled { items.push(("disabled".to_string(), Box::new(d))) }
        _Button(Stack::default(), Enum::new(items, start.to_string()), is_disabled, Box::new(callback))
    }

    pub fn disable(&mut self, disable: bool) {
        self.2 = disable;

        match self.2 {
            true => self.1.display("disabled"),
            false => self.1.display("default")
        }
    }
}

impl OnEvent for _Button {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<events::Button>() {
            if !self.2 {
                match event {
                    events::Button::Hover(true) => self.1.display("hover"),
                    events::Button::Pressed(true) => {
                        self.1.display("pressed");
                        ctx.hardware.haptic();
                        (self.3)(ctx);
                    }
                    _ => self.1.display("default"),
                }
            }
        }

        vec![event]
    }
}

impl std::fmt::Debug for _Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Button")
    }
}

