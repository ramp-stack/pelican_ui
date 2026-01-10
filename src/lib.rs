use crate::components::interface::Interface;

pub mod components;
pub mod interactions;
pub mod utils;

pub mod theme;
use theme::Theme;

pub use prism::*;

extern crate self as pelican_ui;

pub struct PelicanUI;

impl PelicanUI {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, interface: impl FnOnce(&mut Context) -> Interface) -> Interface {
        ctx.state.insert(Theme::default());
        (interface)(ctx)
    }
}