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

use mustache::{Context, drawable::Drawable, events::Event};

/// This trait is used to define pages in the application.
/// 
/// Every page must implement this trait. 
///
/// Every page must implement [`Debug`] and [`Component`].
///
///
/// # Navigation
/// **'navigate'** is called to navigate away from this page.
///
/// The `index` parameter is the index that was triggered. Match on the index to navigate to
/// the desired page. The returned value must be an `Ok` variant with a boxed `dyn AppPage`.
///
/// If the index is not an expected value, return `Err(self)` and the user will be navigated
/// to an error page where `self` acts as the **"go back"** button.
///
/// ```rust
/// fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) {
///     match index {
///         0 => Ok(Box::new(Home::new(ctx))),
///         1 => Ok(Box::new(Settings::new(ctx))),
///         2 => Ok(Box::new(Search::new(ctx))),
///         _ => Err(self),
///     }
/// }
/// ```
///
/// # Navigation Example
/// This is an example of button triggering a [`NavigateEvent`].
/// According to the example above, this will send the user to the settings page.
///
/// ```rust
/// let button = Button::primary(ctx, "Continue", |ctx: &mut Context| {
///     ctx.trigger_event(NavigateEvent(1));
/// })
/// ```
///
/// # Navigator Bar
///
/// When creating an [`Interface`], you can optionally pass in navigatable pages to the navigation bar.
///
/// The navigation bar is only optional on mobile. On web and desktop, if a navigator was passed into the interface,
/// it will always be shown.
pub trait AppPage: Drawable + std::fmt::Debug + 'static {
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) 
        -> Result<Box<dyn AppPage>, Box<dyn AppPage>>;

    /// Returns whether a navigation bar is visible (mobile specific).
    fn has_nav(&self) -> bool {true}
}

/// Event used to navigate between pages of the app.
#[derive(Debug, Clone)]
pub struct NavigateEvent(pub usize);

impl Event for NavigateEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}