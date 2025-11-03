use roost::events::{self, OnEvent, Event};
use roost::drawable::{Drawable};
use roost::{Context, Component};
use roost::layouts::{Enum, Stack};
use roost::emitters;

type ToggleCallback = Box<dyn FnMut(&mut Context, bool)>;

#[derive(Component)]
pub struct Toggle(Stack, Enum, #[skip] bool, #[skip] ToggleCallback);

impl Toggle {
    pub fn new(
        on: impl Drawable + 'static,
        off: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + 'static,
    ) -> emitters::Button<Self> {
        let start = if is_selected {"on"} else {"off"};
        emitters::Button::new(Toggle(Stack::default(), 
            Enum::new(vec![("on", Box::new(on)), ("off", Box::new(off))], start), 
            !is_selected, Box::new(on_click)
        ))
    }
}

impl OnEvent for Toggle {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(events::Button::Pressed(true)) = event.downcast_ref::<events::Button>() {
            self.2 = !self.2;
            ctx.hardware.haptic();
            (self.3)(ctx, !self.2);
            match self.2 {
                false => self.1.display("on"),
                true => self.1.display("off"),
            }
        }
        Vec::new()
    }
}

impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Toggle")
    }
}
