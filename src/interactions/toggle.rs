use roost_ui::events::{self, OnEvent, Event};
use roost_ui::drawable::{Drawable};
use roost_ui::{Context, Component};
use roost_ui::layouts::{Enum, Stack};
use roost_ui::emitters;


#[derive(Component, Debug)]
pub struct Toggle(Stack, emitters::Button<_Toggle>);
impl OnEvent for Toggle {}
impl Toggle {
    pub fn new(
        on: impl Drawable + 'static,
        off: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + 'static,
    ) -> Self {
        let toggle = _Toggle::new(on, off, is_selected, on_click);
        Self(Stack::default(), emitters::Button::new(toggle))
    }
}

impl std::ops::Deref for Toggle {
    type Target = _Toggle;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Toggle {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component)]
pub struct _Toggle(Stack, Enum, #[skip] bool, #[skip] ToggleCallback);

impl _Toggle {
    pub fn new(
        on: impl Drawable + 'static,
        off: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + 'static,
    ) -> Self {
        let start = if is_selected {"on"} else {"off"};
        _Toggle(Stack::default(), 
            Enum::new(vec![("on", Box::new(on)), ("off", Box::new(off))], start), 
            !is_selected, Box::new(on_click)
        )
    }
}

impl OnEvent for _Toggle {
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

impl std::fmt::Debug for _Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Toggle")
    }
}


type ToggleCallback = Box<dyn FnMut(&mut Context, bool)>;