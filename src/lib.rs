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
}

impl<A: Application> roost_ui::Application for PelicanUI<A> {
    async fn new(ctx: &mut Context) -> impl pelican_ui::drawable::Drawable {
        A::interface(ctx)
    }

    fn plugins(ctx: &mut Context) -> Vec<Box<dyn pelican_ui::Plugin>> {
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