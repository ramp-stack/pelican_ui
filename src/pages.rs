use roost::events::OnEvent;
use roost::drawable::{Align};
use roost::{drawables, Context, Component};

use roost::layouts::{Offset, Stack};
use crate::components::interface::general::{Page, Content, Header, Bumper};
use crate::components::{TextStyle, Text, TextSize, AspectRatioImage};
use crate::components::button::PrimaryButton;
use crate::plugin::PelicanUI;
use crate::components::interface::navigation::AppPage;
use crate::components::interface::navigation::NavigationEvent;

/// Error page that will be shown when the user is navigated to an invalid index.
///
/// This page typically appears when `AppPage::navigate` returns `Err(self)`.
/// The `self` page acts as a **"go back"** button, allowing the user to return to the previous page.
#[derive(Debug, Component)]
pub struct Error(Stack, Page);
impl OnEvent for Error {}
impl AppPage for Error {}

impl Error {
    pub fn new(ctx: &mut Context, error: String) -> Self {
        let error_illustration = ctx.get::<PelicanUI>().get().0.theme().brand.error.clone();
        let illustration = AspectRatioImage::new(error_illustration, (300.0, 300.0));
        let title = Text::new(ctx, "Something went wrong.", TextSize::H4, TextStyle::Heading, Align::Left, None);
        let text = Text::new(ctx, &error, TextSize::Md, TextStyle::Primary, Align::Center, None);
        let content = Content::new(ctx, Offset::Center, drawables![illustration, title, text]);
        let button = PrimaryButton::new(ctx, "Go Back", move |ctx: &mut Context| ctx.trigger_event(NavigationEvent::Pop(1)), false);
        let bumper = Bumper::new(ctx, drawables![button]);
        let header = Header::home(ctx, "Error", None);
        Error(Stack::default(), Page::new(header, content, Some(bumper)))
    }
}

/// Example landing page for Pelican UI.
///
/// `PelicanHome` demonstrates how to create a basic page with a logo, heading, 
/// and tagline. It is intended as a template or reference for building other pages.
#[derive(Debug, Component)]
pub struct PelicanHome(Stack, Page);
impl OnEvent for PelicanHome {}
impl AppPage for PelicanHome {}

impl PelicanHome {
    pub fn new(ctx: &mut Context) -> Self {
        let logo = ctx.get::<PelicanUI>().get().0.theme().brand.logo.clone();
        let illustration = AspectRatioImage::new(logo, (150.0, 150.0));
        let title = Text::new(ctx, "Welcome to Pelican UI", TextSize::H4, TextStyle::Heading, Align::Center, None);
        let text = Text::new(ctx, "featherlight ui for heavy ideas", TextSize::Md, TextStyle::Primary, Align::Center, None);
        let content = Content::new(ctx, Offset::Center, vec![Box::new(illustration), Box::new(title), Box::new(text)]);
        let header = Header::home(ctx, "Pelican UI", None);
        PelicanHome(Stack::default(), Page::new(header, content, None))
    }
}