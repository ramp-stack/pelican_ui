#![doc(html_logo_url = "https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/logo.png")]

//! Pelican is a UI design system built on top of [Mustache](<http://ramp-stack.com/mustache>) which provides a wide range of components, layouts, elements, utilities, and pages for building beautiful, consistently designed applications. You can download the starter template [here](<https://github.com/EllaCouch20/ramp_template>).
//!
//! Checkout the [website](<http://ramp-stack.com/pelican_ui>) for additional information and our [Quick Start Guide](<http://ramp-stack.com/pelican_ui/getting_started>) for setting up your first app.
//!
//! You must add the Pelican plugin to your application's plugins. 
//!
//! At its core, Pelican revolves around **components**, which are composed from layouts and elements. Every structure implementing the [`Component`](mustache::Component) trait must meet a few requirements:
//! - Its first element must implement [`Layout`](mustache::layout::Layout), ensuring correct management of positioning, sizing, and nested layouts.
//! - It must implement [`OnEvent`](mustache::events::OnEvent).
//!
//! ### App Page Example
//!
//! ```rust
//! #[derive(Debug, Component)]
//! pub struct FirstScreen(Stack, Page);
//! impl OnEvent for FirstScreen {}
//!
//! impl AppPage for FirstScreen {
//!     fn has_nav(&self) -> bool { false }
//!     fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Err(self) }
//! }
//!
//! impl FirstScreen {
//!     pub fn new(ctx: &mut Context) -> Self {
//!         let color = ctx.theme.colors.text.heading;
//!         let icon = Icon::new(ctx, "pelican_ui", color, 128.0);
//!
//!         let font_size = ctx.theme.fonts.size;
//!         let text = Text::new(ctx, "Hello World!", TextStyle::Heading, font_size.h2, Align::Center);
//!         let subtext = ExpandableText::new(ctx, "First project loaded successfully.", TextStyle::Primary, font_size.md, Align::Center, None);
//!
//!         let content = Content::new(ctx, Offset::Center, vec![Box::new(icon), Box::new(text), Box::new(subtext)]);
//!
//!         let header = Header::home(ctx, "My Screen", None);
//!
//!         FirstScreen(Stack::default(), Page::new(Some(header), content, None))
//!     }
//! }
//!```
//!

pub mod layout;
pub mod components;
pub mod utils;
pub mod plugin;
pub mod theme;
pub mod pages;
