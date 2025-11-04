use roost_ui::events::{self, OnEvent, Event};
use roost_ui::drawable::{Drawable};
use roost_ui::{Context, Component};
use roost_ui::layouts::{Enum, Stack};
use roost_ui::emitters;


#[derive(Component, Debug)]
pub struct Selectable(Stack, emitters::Selectable<_Selectable>);
impl OnEvent for Selectable {}
impl Selectable {
    pub fn new(
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
        group_id: uuid::Uuid,
    ) -> Self {
        let selectable = _Selectable::new(default, selected, is_selected, on_click);
        Self(Stack::default(), emitters::Selectable::new(selectable, group_id))
    }
}

impl std::ops::Deref for Selectable {
    type Target = _Selectable;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Selectable {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component)]
pub struct _Selectable(Stack, Enum, #[skip] Box<dyn FnMut(&mut Context)>);

impl _Selectable {
    pub fn new(
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        let start = if is_selected {"selected"} else {"default"};
        _Selectable(Stack::default(), Enum::new(vec![
            ("default", Box::new(default)),
            ("selected", Box::new(selected)),
        ], start), Box::new(on_click))
    }
}

impl OnEvent for _Selectable {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(events::Selectable::Selected(b)) = event.downcast_ref::<events::Selectable>() {
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

impl std::fmt::Debug for _Selectable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Selectable")
    }
}
