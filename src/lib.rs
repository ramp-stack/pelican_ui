#![doc(html_logo_url = "https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/logo.png")]

//! # Pelican UI
//! 
//! Pelican UI is a composable interface framework built on top of **Roost**.  
//! It adds structure, theming, and reusable components for building modern Rust UIs.
//! 
//! ---
//! 
//! ## Overview
//! 
//! Pelican extends Roost’s [`Drawable`](roost_ui::drawable::Drawable) with:
//! - [`Shapes`](crate::components::shapes)
//! - [`Text`](crate::components::text)
//! - [`Components`](crate::components)
//! - [`Pages`](crate::pages)
//! 
//! ---
//! 
//! You can download the starter template [here](<https://github.com/EllaCouch20/ramp_template>).
//!
//! Checkout the [website](<http://ramp-stack.com/pelican_ui>) for additional information and our [Quick Start Guide](<http://ramp-stack.com/pelican_ui/getting_started>) for setting up your first app.

extern crate self as pelican_ui;

pub use roost_ui::*;

/// Interactions 
pub mod interactions;
pub mod components;
pub mod utils;
pub mod plugin;
pub mod theme;
pub mod pages;

use crate::components::interface::general::Interface;
use crate::theme::Theme;

pub struct PelicanUI<A: Application>(A);

pub trait Application {
    fn interface(ctx: &mut Context) -> Interface;

    fn theme(assets: &mut Assets) -> Theme {
        Theme::default(assets)
    }

    fn on_event(_interface: &mut Interface, _ctx: &mut Context, event: Box<dyn roost_ui::events::Event>) -> Vec<Box<dyn roost_ui::events::Event>> {vec![event]}
}

impl<A: Application> roost_ui::Application for PelicanUI<A> {
    async fn new(ctx: &mut Context) -> impl pelican_ui::drawable::Drawable {
        let on_event = |interface: &mut Interface, ctx: &mut Context, event: Box<dyn roost_ui::events::Event>| A::on_event(interface, ctx, event);
        let mut interface = A::interface(ctx);
        interface.3 = Some(Box::new(on_event));
        interface
    }

    fn plugins(ctx: &mut Context) -> Vec<Box<dyn pelican_ui::Plugin>> {
        ctx.assets.include_assets(pelican_ui::include_dir!("./resources"));
        let theme = A::theme(&mut ctx.assets);
        vec![Box::new(pelican_ui::plugin::PelicanUI::new(ctx, theme))]
    }
}


#[doc(hidden)]
pub mod __private {
    pub use roost_ui::start as roost_start;
    pub use pelican_ui::PelicanUI;
}

#[macro_export]
macro_rules! start {
    ($app:ty) => {
        pub use $crate::__private::*;

        roost_start!(PelicanUI<$app>);
    };
}