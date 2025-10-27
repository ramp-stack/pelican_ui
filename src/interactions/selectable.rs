use mustache::events::{self, OnEvent, Event};
use mustache::drawable::{Drawable};
use mustache::{Context, Component};
use mustache::layouts::{Enum, Stack};
use mustache::emitters;

#[derive(Component)]
pub struct Selectable(Stack, Enum, #[skip] Box<dyn FnMut(&mut Context)>);

impl Selectable {
    pub fn new(
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
        group_id: uuid::Uuid,
    ) -> emitters::Selectable<Self> {
        let start = if is_selected {"selected"} else {"default"};
        emitters::Selectable::new(Selectable(Stack::default(), Enum::new(vec![
            ("default", Box::new(default)),
            ("selected", Box::new(selected)),
        ], start), Box::new(on_click)), group_id)
    }
}

impl OnEvent for Selectable {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(events::Selectable(b)) = event.downcast_ref::<events::Selectable>() {
            match b {
                false => self.1.display("default"),
                true => {
                    self.1.display("selected");
                    ctx.hardware.haptic();
                    (self.2)(ctx);
                }
            }
        }
        Vec::new()
    }
}

impl std::fmt::Debug for Selectable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Selectable")
    }
}
