use roost::events::{self, OnEvent, Event};
use roost::drawable::{Drawable};
use roost::{Context, Component};
use roost::layouts::{Enum, Stack};
use roost::emitters;

#[derive(Component)]
pub struct Button(Stack, Enum, #[skip] pub bool, #[skip] bool, #[skip] Box<dyn FnMut(&mut Context)>);

impl Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        is_disabled: bool,
        callback: impl FnMut(&mut Context) + 'static,
    ) -> emitters::Button<Self> {
        let start = if is_disabled {"disabled"} else {"default"};
        let mut items: Vec<(&str, Box<dyn Drawable>)> = Vec::new();
        items.push(("default", Box::new(default)));
        if let Some(h) = hover { items.push(("hover", Box::new(h))) }
        if let Some(p) = pressed { items.push(("pressed", Box::new(p))) }
        if let Some(d) = disabled { items.push(("disabled", Box::new(d))) }
        emitters::Button::new(Button(Stack::default(), Enum::new(items, start), is_disabled, false, Box::new(callback)))
    }
}

impl OnEvent for Button {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<events::TickEvent>().is_some() {
            match self.2 {
                true => self.1.display("disabled"),
                false if self.3 => self.1.display("hover"),
                false => self.1.display("default")
            }
        } else if let Some(event) = event.downcast_ref::<events::Button>() {
            if !self.2 {
                match event {
                    events::Button::Hover(true) => {
                        self.3 = true;
                        self.1.display("hover")
                    },
                    events::Button::Pressed(true) => {
                        self.3 = false;
                        self.1.display("pressed");
                        ctx.hardware.haptic();
                        (self.4)(ctx);
                    }
                    _ => {
                        self.3 = false;
                        self.1.display("default")
                    },
                }
            }
        }

        vec![event]
    }
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Button")
    }
}

