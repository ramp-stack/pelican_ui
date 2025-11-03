use roost::events::{self, OnEvent, Event, MouseState, MouseEvent};
use roost::drawable::{Drawable};
use roost::{Context, Component};
use roost::layouts::Stack;
use roost::emitters;

#[derive(Debug, Component)]
pub struct Scrollable<D: Drawable + 'static>(Stack, pub D, #[skip] (f32, f32));

impl<D: Drawable + 'static> Scrollable<D> {
    pub fn new(child: D) -> emitters::Scrollable<Self> {
        emitters::Scrollable::new(Scrollable(Stack::default(), child, (0.0, 0.0)))
    }
}

impl<D: Drawable + 'static> OnEvent for Scrollable<D> {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(MouseEvent{position: Some(position), state}) = event.downcast_ref::<events::MouseEvent>() {
            match state {
                MouseState::Pressed => {
                    self.2 = *position;
                    return Vec::new();
                },
                MouseState::Released => {
                    if (position.1 - self.2.1).abs() < 5.0 {
                        return vec![Box::new(MouseEvent{position: Some(*position), state: MouseState::Pressed}) as Box<dyn Event>];
                    }

                    return Vec::new();
                }
                _ => {}
            }
        } 

        vec![event]
    }
}

