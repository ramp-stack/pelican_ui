use prism::drawable::{Drawable, Component};
use prism::event::OnEvent;
use prism::layout::Stack;

use std::hash::Hash;
use std::fmt::Debug;

pub mod components;
pub mod interface;

pub mod theme;
use theme::Theme;

pub use prism::*;
pub use ptsd::{colors, utils, navigation};

pub use image;

extern crate self as pelican_ui;

// pub struct PelicanUI;

// impl PelicanUI {
//     #[allow(clippy::new_ret_no_self)]
//     pub fn new(interface: impl FnOnce(&Theme) -> Interface, theme) -> Interface {
//         (interface)(&Theme::default())
//     }
// }

#[derive(Debug, Component, Clone)]
pub struct Listener<D: Drawable + Clone, T: Hash + Debug + Clone + 'static>(Stack, prism::Listener<D, T>);
impl<D: Drawable + Clone, T: Hash + Debug + Clone + 'static> OnEvent for Listener<D, T> {}
impl<D: Drawable + Clone, T: Hash + Debug + Clone + 'static> Listener<D, T> {
    pub fn new(ctx: &mut Context, theme: &Theme, inner: D, updated_on: impl Fn(&mut Context, &Theme, &mut D, T) + Send + Sync + 'static) -> Self {
        let theme = theme.clone();
        let updated_on = move |ctx: &mut Context, inner: &mut D, other: T| (updated_on)(ctx, &theme, inner, other);
        Listener(Stack::default(), prism::Listener::new(ctx, inner, updated_on))
    }
}

pub trait Callback: FnMut(&mut Context, &Theme) + 'static {
    fn clone_box(&self) -> Box<dyn Callback>;
}

impl PartialEq for dyn Callback{fn eq(&self, _: &Self) -> bool {true}}

impl<F> Callback for F where F: FnMut(&mut Context, &Theme) + Clone + 'static {
    fn clone_box(&self) -> Box<dyn Callback> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Callback> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl std::fmt::Debug for dyn Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clonable Closure")
    }
}
