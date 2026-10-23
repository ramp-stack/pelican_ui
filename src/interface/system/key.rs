use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{DynEvent, Key, KeyboardEvent, KeyboardState, Modifiers};
use prism::events::{MouseEvent, MouseState, MouseButton, WindowEvent, Timer, OnChange, on_change};
use ptsd::interactions;
use ptsd::layout::{Stack, Offset, Size, Padding};

use std::time::{Duration, Instant};

use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon};

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct InputKey(Stack, interactions::Button, #[skip] Option<Repeat>);
impl InputKey {
    pub fn new(default: impl Drawable, pressed: impl Drawable, key: Key, repeat: bool) -> Self {
        let held = repeat.then(|| Repeat {
            started: Instant::now(), timer: Timer::default(), key: key.clone(),
        });
        Self(Stack::default(), interactions::Button::new(
            default, None::<Icon>, Some(pressed), None::<Icon>, None::<Icon>,
            move |ctx: &mut Context| Self::press(ctx, key.clone()), false,
        ), held)
    }

    pub fn press(ctx: &mut Context, key: Key) {
        ctx.emit(KeyboardEvent {key, state: KeyboardState::Pressed, modifiers: Modifiers::default()});
    }

    pub fn backspace(theme: &Theme, height: f32) -> Self {
        let face = |alpha| Backspace(
            Stack(Offset::Center, Offset::Center, Size::custom(|widths| (widths[1].0, 42.0)),
                Size::Static(height), Padding(3.0, 6.0, 3.0, 6.0)),
            Rectangle::new(Color::from_hex("ffffff", alpha), 4.0, None),
            Icon::new(theme, Icons::Backspace, Some(Color::white()), 36.0),
        );
        Self::new(face(40), face(57), Key::Backspace, true)
    }

    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(hold) = &mut self.2 {
            if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
                match mouse.state {
                    MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                        hold.started = Instant::now();
                        hold.timer.reset(Duration::from_millis(500));
                    }
                    MouseState::Released(_) => hold.timer.cancel(),
                    MouseState::Moved if mouse.position.is_none() => hold.timer.cancel(),
                    _ => {}
                }
            } else if matches!(event.downcast_ref::<WindowEvent>(), Some(WindowEvent::Focused(false))) {
                hold.timer.cancel();
            }
        }
        vec![event]
    }
}
#[on_change(async {
    match &mut self.2 {
        Some(repeat) => repeat.timer.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<()> for InputKey {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {
        let hold = self.2.as_mut().unwrap();
        let interval = if hold.started.elapsed() >= Duration::from_secs(2) {35} else {80};
        hold.timer.reset(Duration::from_millis(interval));
        Self::press(ctx, hold.key.clone());
    }
}

#[derive(Debug, Clone)]
struct Repeat {
    started: Instant,
    timer: Timer,
    key: Key,
}

#[derive(Component, Debug, Clone)]
struct Backspace(Stack, Rectangle, Icon);
