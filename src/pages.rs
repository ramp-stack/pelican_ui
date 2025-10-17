use mustache::events::OnEvent;
use mustache::drawable::{Align};
use mustache::{drawables, Context, Component};

use crate::layout::{Offset, Stack};
use crate::components::interface::general::{Page, Content, Header, Bumper};
use crate::components::{TextStyle, Text, AspectRatioImage};
use crate::components::button::PrimaryButton;
use crate::plugin::PelicanUI;
use crate::components::interface::navigation::AppPage;
use crate::components::interface::navigation::NavigateEvent;

/// Error page that will be shown when the user is navigated to an invalid index.
///
/// This page typically appears when `AppPage::navigate` returns `Err(self)`.
/// The `self` page acts as a **"go back"** button, allowing the user to return to the previous page.
#[derive(Debug, Component)]
pub struct Error(Stack, Page, #[skip] Box<dyn AppPage>);
impl OnEvent for Error {}

impl AppPage for Error {
    fn mobile_navigator(&self) -> bool { false }
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Ok(self.2) }
}

impl Error {
    pub fn new(ctx: &mut Context, error: &str, home: Box<dyn AppPage>) -> Self {
        let error_illustration = ctx.get::<PelicanUI>().get().0.theme().brand.error.clone();
        let font_size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let illustration = AspectRatioImage::new(error_illustration, (300.0, 300.0));
        let title = Text::new(ctx, "Something went wrong.", font_size.h4, TextStyle::Heading, Align::Left, None);
        let text = Text::new(ctx, error, font_size.md, TextStyle::Primary, Align::Center, None);
        let content = Content::new(ctx, Offset::Center, drawables![illustration, title, text]);
        let button = PrimaryButton::new(ctx, "Go Back", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)), false);
        let bumper = Bumper::new(ctx, drawables![button]);
        let header = Header::home(ctx, "", None);
        Error(Stack::default(), Page::new(Some(header), content, Some(bumper)), home)
    }
}

// /// Splash page shown when the app first launches.
// #[derive(Debug, Component)]
// pub struct Splash(Stack, Page);
// impl OnEvent for Splash {}
// impl AppPage for Splash {
//     fn has_nav(&self) -> bool { false }
//     fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Ok(self) }
// }

// impl Splash {
//     pub fn new(ctx: &mut Context) -> Self {
//         let wordmark = ctx.theme.brand.wordmark.clone();
//         let content = Content::new(ctx, Offset::Center, vec![Box::new(AspectRatioImage::new(wordmark, (162.0, 34.5)))]);

//         Splash(Stack::default(), Page::new(None, content, None))
//     }
// }

/// Example landing page for Pelican UI.
///
/// `PelicanHome` demonstrates how to create a basic page with a logo, heading, 
/// and tagline. It is intended as a template or reference for building other pages.
#[derive(Debug, Component)]
pub struct PelicanHome(Stack, Page);
impl OnEvent for PelicanHome {}

impl AppPage for PelicanHome {
    fn mobile_navigator(&self) -> bool { false }
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { Err(self) }
}

impl PelicanHome {
    pub fn new(ctx: &mut Context) -> Self {
        let logo = ctx.get::<PelicanUI>().get().0.theme().brand.logo.clone();
        let font_size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let illustration = AspectRatioImage::new(logo, (150.0, 150.0));
        let title = Text::new(ctx, "Welcome to Pelican UI", font_size.h4, TextStyle::Heading, Align::Center, None);
        let text = Text::new(ctx, "featherlight ui for heavy ideas", font_size.md, TextStyle::Primary, Align::Center, None);
        let content = Content::new(ctx, Offset::Center, vec![Box::new(illustration), Box::new(title), Box::new(text)]);
        PelicanHome(Stack::default(), Page::new(None, content, None))
    }
}