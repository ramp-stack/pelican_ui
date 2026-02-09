// use crate::components::interface::Interface;

use prism::drawable::{Drawable, Component};
use prism::event::OnEvent;
use prism::layout::Stack;

use std::hash::Hash;
use std::fmt::Debug;

pub mod components;

pub mod interface;
use interface::general::Interface;

pub mod theme;
use theme::Theme;

pub use prism::*;

extern crate self as pelican_ui;

pub struct PelicanUI;

impl PelicanUI {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(interface: impl FnOnce(&Theme) -> Interface) -> Interface {
        (interface)(&Theme::default())
    }
}

#[derive(Debug, Component)]
pub struct Listener<D: Drawable, T: Hash + Debug + Clone + 'static>(Stack, prism::Listener<D, T>);
impl<D: Drawable, T: Hash + Debug + Clone + 'static> OnEvent for Listener<D, T> {}
impl<D: Drawable, T: Hash + Debug + Clone + 'static> Listener<D, T> {
    pub fn new(ctx: &mut Context, theme: &Theme, inner: D, updated_on: impl Fn(&mut Context, &Theme, &mut D, T) + 'static) -> Self {
        let theme = theme.clone();
        let updated_on = move |ctx: &mut Context, inner: &mut D, other: T| (updated_on)(ctx, &theme.clone(), inner, other);
        Listener(Stack::default(), prism::Listener::new(ctx, inner, updated_on))
    }
}