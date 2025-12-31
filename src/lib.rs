use prism::{Context, drawable::Drawable};
use crate::components::interface::Interface;

pub mod components;
pub mod interactions;
pub mod utils;

pub mod theme;
use theme::Theme;

extern crate self as pelican_ui;

pub struct PelicanUI;

impl PelicanUI {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, 
        interface: impl FnOnce(&mut Context) -> Interface, 
        on_tick: impl FnMut(&mut Context) + 'static
    ) -> Interface {
        ctx.state.insert(Theme::default());
        let mut interface = (interface)(ctx);
        interface.on_tick = Some(Box::new(on_tick));
        interface
    }
}