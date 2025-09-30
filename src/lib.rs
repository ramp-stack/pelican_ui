#![doc(html_logo_url = "https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/logo.png")]

//! Pelican UI Standard provides a wide range of components, layouts, elements, utilities, and pages for building beautiful, consistently designed applications. You can download the starter template [here](<https://github.com/EllaCouch20/ramp_template>).
//!
//! Checkout the [website](<http://ramp-stack.com/pelican_ui>) for additional information, our [Quick Start Guide](<http://ramp-stack.com/pelican_ui/getting_started>) for setting up your first app, and interact with the [community](<https://discord.gg/cTRaRbUZ>) if you have any questions!
//!
//! You must add the Pelican plugin to your application's plugins. 
//!
//! At its core, Pelican UI Standard revolves around **components**, which are composed from layouts and elements. Every structure implementing the [`Component`](mustache::Component) trait must meet a few requirements:
//! - Its first element must implement [`Layout`](mustache::layout::Layout), ensuring correct management of positioning, sizing, and nested layouts.
//! - It must implement [`OnEvent`](mustache::events::OnEvent) and derive [`Debug`].
//!
//! Components are built from **elements**, the lowest-level primitives such as [`Text`], [`AspectRatioImage`], and [`Circle`], and can use different **layouts** like [`Column`], [`Row`], and [`Stack`] to arrange them.
//! Components are also often built from combining components.
//!
//! Pelican UI Standard includes multiple [`Events`](mustache::events::Event) used and triggered by its components, as well as configuration variables for platform detection such as [`IS_MOBILE`] and [`IS_WEB`]. Additional utilities like [`Timestamp`] and [`ElementID`] are also provided.
//!
//! Beyond individual components, Pelican UI Standard ships with a few ready-to-use **pages** built entirely from its own system, including [`PelicanHome`], [`Error`], and [`Splash`]. These can be used directly or serve as references when creating custom pages.
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

pub mod events;
// pub use events::{
//     NavigateEvent,
//     KeyboardActiveEvent,
//     ClearActiveInput,
//     SetActiveInput,
//     TextInputSelect,
//     ListItemSelect,
//     NavigatorSelect,
//     NavigatorEvent,
//     SearchEvent,
//     InputEditedEvent,
//     AdjustScrollEvent,
//     QRCodeScannedEvent,
//     AttachmentEvent,
// };

mod config;
pub use config::{IS_MOBILE, IS_WEB};

pub mod layout;
// pub use layout::{
//     Offset, 
//     Size, 
//     Padding, 
//     Column, 
//     Row, 
//     Wrap, 
//     Scroll, 
//     Stack, 
//     ScrollAnchor, 
//     Bin, 
//     Opt, 
//     EitherOr,
//     UniformExpand
// };

// mod elements;
// pub use elements::{
//     Text, 
//     ExpandableText, 
//     TextStyle, 
//     BulletedText,
//     ExpandableImage, 
//     EncodedImage,
//     AspectRatioImage, 
//     Icon, 
//     Circle, 
//     Rectangle,
//     RoundedRectangle, 
//     OutlinedRectangle,
//     Outline,
// };

pub mod components;
// pub use components::{
//     Button,
//     ButtonStyle,
//     ButtonSize,
//     ButtonState,
//     ButtonWidth,
//     IconButton,
//     QuickActions,
//     Alert,
//     Avatar,
//     AvatarContent,
//     AvatarIconStyle,
//     DataItem,
//     ListItem,
//     ListItemGroup,
//     ListItemSelector,
//     TextInput,
//     Searchbar,
//     QRCode,
//     QRCodeScanner,
//     Slider,
//     Interface,
//     Page,
//     Header,
//     Bumper,
//     Content,
//     HeaderIcon,
//     HeaderContent
// };

mod utils;
pub use utils::{
    // Timestamp, 
    // ElementID,
    Callback
};

pub mod plugin;
pub mod theme;

mod pages;
// pub use pages::{
//     AppPage, 
//     Error, 
//     Splash, 
//     PelicanHome
// };

// mod themes;
// pub use themes::{
//     PelicanColorThemes
// };

/*
How to create an app with Ramp
How to create a page with Ramp
How to create a component with Ramp
How to create a layout with Ramp
How to create a plugin with Ramp
How to create a messaging app with Ramp
How to create a bitcoin wallet with Ramp
How to create a AIR profile with Ramp
*/
/*
// ! ```rust
// ! #[derive(Debug, Component)]
// ! pub struct InfoCard(Column, Text, Text);
// ! impl OnEvent for InfoCard {}
// !
// ! impl InfoCard {
// !     pub fn new(ctx: &mut Context, title: &str, description: &str) -> Self {
// !         let font_size = ctx.theme.fonts.size;
// !
// !         InfoCard(
// !             Column::new(8.0, Offset::Center, Size::Fit, Padding::new(16.0)),
// !             Text::new(ctx, title, TextStyle::Header, font_size.h3, Align::Left),
// !             Text::new(ctx, description, TextStyle::Primary, font_size.md, Align::Left)
// !         )
// !     }
// ! }
// ! ```\
*/