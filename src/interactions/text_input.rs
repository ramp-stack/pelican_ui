use roost::events::{self, OnEvent, Event};
use roost::drawable::{Drawable};
use roost::{Context, Component};
use roost::layouts::{Enum, Stack, Size, Offset, Padding};
use roost::emitters;


#[derive(Debug, Component)]
pub struct InputField(Stack, Enum, Box<dyn Drawable>, #[skip] pub bool);

impl InputField {
    pub fn new(
        default: impl Drawable + 'static,
        focus: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        error: Option<impl Drawable + 'static>,
        content: impl Drawable + 'static,
        height: f32,
    ) -> emitters::TextInput<Self> {
        let height = Size::custom(move |h: Vec<(f32, f32)>| (h[1].0.max(height), h[1].1.max(height)));
        let layout = Stack(Offset::Start, Offset::Start, Size::Fit, height, Padding::default());

        let mut items: Vec<(&str, Box<dyn Drawable>)> = Vec::new();
        items.push(("default", Box::new(default)));
        items.push(("focus", Box::new(focus)));
        if let Some(h) = hover { items.push(("hover", Box::new(h))) }
        if let Some(e) = error { items.push(("error", Box::new(e))) }

        emitters::TextInput::new(InputField(layout, Enum::new(items, "default"), Box::new(content), false))
    }
}

impl OnEvent for InputField {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(e) = event.downcast_ref::<events::TextInput>() {
            match e {
                events::TextInput::Hover(true) => self.1.display("hover"),
                events::TextInput::Focused(true) => {
                    ctx.hardware.haptic();
                    self.1.display("focus");
                },
                _ => self.1.display(if self.3 {"error"} else {"default"}),
            }
        }
        
        vec![event]
    }
}