use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::layout::{Offset, Stack};
use crate::components::interface::general::{Page, Content, Header, Bumper};
use crate::components::button::Button;
use crate::components::{TextStyle, Text, AspectRatioImage};
use crate::events::NavigateEvent;

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


/// Error page that will be shown when the user is navigated to an invalid index.
///
/// This page typically appears when `AppPage::navigate` returns `Err(self)`.
/// The `self` page acts as a **"go back"** button, allowing the user to return to the previous page.
#[derive(Debug, Component)]
pub struct Error(Stack, Page, #[skip] Box<dyn AppPage>);
impl OnEvent for Error {}

impl AppPage for Error {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Ok(self.2) }
}

impl Error {
    pub fn new(ctx: &mut Context, error: &str, home: Box<dyn AppPage>) -> Self {
        let theme = &ctx.theme;
        let illustration = theme.brand.illustrations.get("error").unwrap();
        let font_size = theme.fonts.size;
        let illustration = AspectRatioImage::new(illustration, (300.0, 300.0));
        let title = Text::new(ctx, "Something went wrong.", TextStyle::Heading, font_size.h4, Align::Left);
        let text = Text::new(ctx, error, TextStyle::Primary, font_size.md, Align::Center);
        let content = Content::new(ctx, Offset::Center, vec![Box::new(illustration), Box::new(title), Box::new(text)]);
        let button = Button::primary(ctx, "Go Back", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let bumper = Bumper::single_button(ctx, button);
        let header = Header::stack(ctx, None, "", None);
        Error(Stack::default(), Page::new(Some(header), content, Some(bumper)), home)
    }
}

/// Splash page shown when the app first launches.
#[derive(Debug, Component)]
pub struct Splash(Stack, Page);
impl OnEvent for Splash {}
impl AppPage for Splash {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Ok(self) }
}

impl Splash {
    pub fn new(ctx: &mut Context) -> Self {
        let wordmark = ctx.theme.brand.wordmark.clone();
        let content = Content::new(ctx, Offset::Center, vec![Box::new(AspectRatioImage::new(wordmark, (162.0, 34.5)))]);

        Splash(Stack::default(), Page::new(None, content, None))
    }
}

/// Example landing page for Pelican UI.
///
/// `PelicanHome` demonstrates how to create a basic page with a logo, heading, 
/// and tagline. It is intended as a template or reference for building other pages.
#[derive(Debug, Component)]
pub struct PelicanHome(Stack, Page);
impl OnEvent for PelicanHome {}

impl AppPage for PelicanHome {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Err(self) }
}

impl PelicanHome {
    pub fn new(ctx: &mut Context) -> Self {
        let theme = &ctx.theme;
        let logo = theme.brand.logomark.clone();
        let font_size = theme.fonts.size;
        let illustration = AspectRatioImage::new(logo, (150.0, 150.0));
        let title = Text::new(ctx, "Welcome to Pelican UI", TextStyle::Heading, font_size.h4, Align::Center);
        let text = Text::new(ctx, "featherlight ui for heavy ideas", TextStyle::Primary, font_size.md, Align::Center);
        let content = Content::new(ctx, Offset::Center, vec![Box::new(illustration), Box::new(title), Box::new(text)]);
        PelicanHome(Stack::default(), Page::new(None, content, None))
    }
}