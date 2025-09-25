use mustache::events::{OnEvent, MouseState, MouseEvent, Event};
use mustache::drawable::{Drawable, Component, Align};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::components::{Rectangle, TextStyle, Text};
use crate::events::{TextInputSelect, AdjustScrollEvent};
use crate::layout::{Column, Stack, Row, Padding, Offset, Size, Scroll, ScrollAnchor};
use crate::components::avatar::AvatarContent;
use crate::components::button::{IconButton, Button};
use crate::components::TextInput;
use crate::utils::ElementID;
use crate::pages::AppPage;
use std::fmt::Debug;

use crate::components::interface::{
    desktop::DesktopInterface,
    mobile::MobileInterface,
    web::WebInterface,
};

pub type NavigateInfo = (&'static str, String, Option<AvatarContent>, Option<Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>>);
pub type PageBuilder = Option<Vec<Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>>>;

/// The top-level interface of an app built with Pelican.
///
/// This interface automatically adapts to the platform:
/// - On desktop, it uses [`DesktopInterface`].
/// - On web, it uses [`WebInterface`].
/// - On mobile, it uses [`MobileInterface`].
///
/// The background color is taken from `ctx.theme.colors.background.primary` by default.
/// You can customize it by setting ctx.theme to a customized [`Theme`] object.
///
/// # Required
/// - A `Box<dyn AppPage>` to serve as the starting page.
///
/// # Optional
/// - A navigation bar, which requires:
///   - The index of the starting page.
///   - Two vectors of [`NavigateInfo`], which define top and bottom sections of the navigator on desktop.
///     On web and mobile, these vectors are combined with no visual separation.
/// - A vector of socials for web, as tuples `(icon, URL)` representing the social icon and its link.
#[derive(Debug, Component)]
pub struct Interface (Stack, Option<Rectangle>, Option<MobileInterface>, Option<DesktopInterface>, Option<WebInterface>);
impl OnEvent for Interface {}
impl Interface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        navigation: Option<(usize, Vec<NavigateInfo>, Vec<NavigateInfo>)>,
        socials: Option<Vec<(&'static str, String)>>
    ) -> Self {
        let color = ctx.theme.colors.background.primary;

        let (mobile, desktop, web) = match crate::config::IS_WEB {
            true => (None, None, Some(WebInterface::new(ctx, start_page, navigation, socials))),
            false if crate::config::IS_MOBILE => (Some(MobileInterface::new(ctx, start_page, navigation)), None, None),
            false => (None, Some(DesktopInterface::new(ctx, start_page, navigation)), None),
        };

        Interface(Stack::default(), Some(Rectangle::new(color, 0.0, None)), mobile, desktop, web)
    }

    // //move background to pages
    // pub fn new_with_background(
    //     ctx: &mut Context, 
    //     image: resources::Image,
    //     start_page: Box<dyn AppPage>,
    //     navigation: Option<(usize, Vec<NavigateInfo>, Vec<NavigateInfo>)>,
    //     socials: Option<Vec<(&'static str, String)>>
    // ) -> Self {
    //     let background = ExpandableImage::new(image, None);
    //     let (mobile, desktop, web) = match crate::config::IS_WEB {
    //         true => (None, None, Some(WebInterface::new(ctx, start_page, navigation, socials))),
    //         false if crate::config::IS_MOBILE => (Some(MobileInterface::new(ctx, start_page, navigation)), None, None),
    //         false => (None, Some(DesktopInterface::new(ctx, start_page, navigation)), None),
    //     };
    //     Interface(Stack::default(), None, Some(background), mobile, desktop, web)
    // }

    /// Returns the DesktopInterface if on desktop
    pub fn desktop(&mut self) -> &mut Option<DesktopInterface> { &mut self.3 }
    /// Returns the MobileInterface if on mobile
    pub fn mobile(&mut self) -> &mut Option<MobileInterface> { &mut self.2 }
    /// Returns the WebInterface if on web
    pub fn web(&mut self) -> &mut Option<WebInterface> { &mut self.4 }
    // pub fn navigation(&mut self) -> (Option<&mut Option<MobileNavigator>>, Option<&mut Option<DesktopNavigator>>) {
    //     (self.desktop().as_mut().map(|d| &mut d.navigator()), self.mobile().as_mut().map(|m| &mut m.navigator()))
    // }
}  

/// # Page
///
/// A Page is a UI container that holds optional [`Header`], [`Content`], and optional [`Bumper`] components.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/page.png"
///      alt="Page Example"
///      width="250">

#[derive(Debug, Component)]
pub struct Page(Column, Option<Header>, Content, Option<Bumper>);
impl OnEvent for Page {}

impl Page {
    /// Creates a new [`Page`] from an optional [`Header`], [`Content`], and optional [`Bumper`]
    pub fn new(header: Option<Header>, content: Content, bumper: Option<Bumper>) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, f32::MAX));
        Page(
            Column::new(12.0, Offset::Center, width, Padding::default()),
            header,
            content,
            bumper,
        )
    }

    /// Returns the header if it exists.
    pub fn header(&mut self) -> &mut Option<Header> {&mut self.1}
    /// Returns the content.
    pub fn content(&mut self) -> &mut Content {&mut self.2}
    /// Returns the bumper if it exists.
    pub fn bumper(&mut self) -> &mut Option<Bumper> {&mut self.3}
}

/// # Content
///
/// The main portion of a page, placed between an optional [`Header`] and an optional [`Bumper`]
/// 
/// Contents are vertical scrollables and can contain unlimited children.
/// Content components can only be used inside [`Page`] components.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/content.png"
///      alt="Content Example"
///      width="250">
///
/// ```rust
/// let text_size = ctx.theme.fonts.size.lg;
/// let text = Text::new(ctx, "Set up a name, description, and team before starting your project.", TextStyle::Primary, text_size, Align::Center);
/// let content = Content::new(ctx, Offset::Center, vec![Box::new(text)]);
/// ```
#[derive(Debug, Component)]
pub struct Content (Scroll, ContentChildren);

impl Content {
    /// Creates a new `Content` component with a specified `Offset` (start, center, or end) and a list of `Box<dyn Drawable>` children.
    pub fn new(ctx: &mut Context, offset: Offset, content: Vec<Box<dyn Drawable>>) -> Self {
        let max = ctx.theme.layout.content_max;
        let padding = ctx.theme.layout.content_padding;
        let max = if crate::config::IS_WEB {1200.0} else {max};
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(max), max));
        let height = Size::custom(move |_: Vec<(f32, f32)>|(0.0, f32::MAX));
        let anchor = if offset == Offset::End { ScrollAnchor::End } else { ScrollAnchor::Start };
        let layout = Scroll::new(Offset::Center, offset, width, height, Padding::default(), anchor);
        // if offset == Offset::End { layout.set_scroll(f32::MAX); }
        Content(layout, ContentChildren::new(content, padding)) 
    }

    /// Find an item in the content. Will return the first instance of the type.
    ///
    /// ```rust
    /// let text = content.find::<Text>().expect("Could not find text in content");
    /// ```
    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.items().iter_mut().find_map(|item| item.as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let text_input = content.find_at::<TextInput>(0).expect("Could not find text input at first index in content");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.items().get_mut(i)?.as_any_mut().downcast_mut::<T>()
    }

    /// Remove an item from the content. Will remove the first instance of the type.
    ///
    /// ```rust
    /// let text = content.remove::<Text>().expect("Could not remove text from content");
    /// ```
    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) = self.items().iter().position(|item| item.as_any().is::<T>()) {
            let boxed = self.items().remove(pos);
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    /// Returns all the items in the content
    pub fn items(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.1.1}
    /// Returns the offset of the items.
    pub fn offset(&mut self) -> &mut Offset {self.0.offset()}
}

impl OnEvent for Content {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustScrollEvent::Vertical(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.0.adjust_scroll(*a);
        } else if let Some(TextInputSelect(id)) = event.downcast_ref::<TextInputSelect>() {
            if crate::config::IS_MOBILE {
                let mut total_height = 0.0;
                for item in self.items().iter_mut() {
                    match item.as_any_mut().downcast_mut::<TextInput>() {
                        Some(input) if input.get_id() == *id => {
                            self.0.set_scroll(total_height);
                            break;
                        }
                        _ => {
                            let size = item.request_size(ctx);
                            total_height += size.max_height();
                        }
                    }
                }
            }
        } else if let Some(MouseEvent { state: MouseState::Scroll(_, y), position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            self.0.adjust_scroll(*y);
        }
        true
    }
}

#[derive(Debug, Component)]
struct ContentChildren (Column, Vec<Box<dyn Drawable>>);
impl OnEvent for ContentChildren {}

impl ContentChildren {
    pub fn new(content: Vec<Box<dyn Drawable>>, padding: f32) -> Self {
        ContentChildren(Column::new(24.0, Offset::Center, Size::Fit, Padding::new(padding)), content)
    }
}

/// # Header
///
/// The top section of a page that displays the page title 
/// and may include supporting elements like navigation, 
/// search, or action buttons, helping users understand where 
/// they are and what they can do.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/header.png"
///      alt="Header Example"
///      width="250">
///
/// Header components can only be used inside [`Page`] components.
#[derive(Debug, Component)]
pub struct Header(Row, HeaderIcon, HeaderContent, HeaderIcon);
impl OnEvent for Header {}

impl Header {
    /// Creates `Header` a new  component from a left and right [`HeaderIcon`] and a centered [`HeaderContent`].
    pub fn new(left: HeaderIcon, content: HeaderContent, right: HeaderIcon) -> Self {
        Header(
            Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0)),
            left, content, right,
        )
    }

    /// A `Header` preset used for home pages.
    ///
    /// ```rust
    /// let header = Header::home(ctx, "My Account", None);
    /// ```
    pub fn home(ctx: &mut Context, title: &str, icon: Option<IconButton>) -> Self {
        Header(
            Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0)),
            HeaderIcon::new(None), 
            HeaderContent::home(ctx, title),
            HeaderIcon::new(icon)
        )
    }

    /// A `Header` preset used for in-flow pages.
    ///
    /// ```rust
    /// let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| println!("Go Back!"));
    /// let header = Header::stack(ctx, Some(back), "Select role", None);
    /// ```
    pub fn stack(
        ctx: &mut Context, 
        left: Option<IconButton>, 
        title: &str, 
        right: Option<IconButton>
    ) -> Self {
        Header(
            Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0)),
            HeaderIcon::new(left), 
            HeaderContent::stack(ctx, title), 
            HeaderIcon::new(right)
        )
    }

    pub fn content(&mut self) -> &mut HeaderContent {&mut self.2}
}

/// # Header Content
///
/// Middle portion of a header containing a column of the important
/// content like Text and an optionally a [`Box<dyn Drawable>`].
/// 
/// This is only to be used inside [`Header`] component.
#[derive(Debug, Component)]
pub struct HeaderContent(Column, Option<Box<dyn Drawable>>, Text);
impl OnEvent for HeaderContent {}

impl HeaderContent {
    /// Creates a new [`HeaderContent`] from an optional [`Box<dyn Drawable>`] and
    /// [`Text`] component.
    pub fn new(content: Option<Box<dyn Drawable>>, text: Text) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, f32::MAX));
        HeaderContent(
            Column::new(10.0, Offset::Center, width, Padding::default()), 
            content, text
        )
    }

    /// HeaderContent preset containing only H3 text.
    pub fn home(ctx: &mut Context, title: &str) -> Self {
        let text_size = ctx.theme.fonts.size.h3;
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, f32::MAX));
        HeaderContent(
            Column::new(10.0, Offset::Center, width, Padding::default()), 
            None,
            Text::new(ctx, title, TextStyle::Heading, text_size, Align::Left),
        )
    }

    /// HeaderContent preset containing only H4 text.
    pub fn stack(ctx: &mut Context, title: &str) -> Self {
        let text_size = ctx.theme.fonts.size.h4;
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, f32::MAX));
        HeaderContent(
            Column::new(10.0, Offset::Center, width, Padding::default()),  
            None,
            Text::new(ctx, title, TextStyle::Heading, text_size, Align::Left),
        )
    }

    /// Returns the content of the `HeaderContent`
    pub fn content(&mut self) -> &mut Option<Box<dyn Drawable>> {&mut self.1}
}

/// # Header Icon
/// 
/// Optionally contains an icon, otherwise just reserves the space.
/// These are only to be used in [`Header`] components.
#[derive(Debug, Component)]
pub struct HeaderIcon(Stack, Option<IconButton>);
impl OnEvent for HeaderIcon {}

impl HeaderIcon {
    pub fn new(icon: Option<IconButton>) -> Self {
        HeaderIcon(
            Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default()),
            icon
        )
    }
}

/// # Bumper
///
/// A fixed container at the bottom of the screen, 
/// usually holding key actions like buttons or text inputs, 
/// ensuring important interactions stay accessible without scrolling.
///
/// Bumper components can only be used inside [`Page`] components.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/bumper.png"
///      alt="Bumper Example"
///      width="450">
///
///```rust
/// let button = Button::primary(ctx, "Continue");
/// let bumper = Bumper::single_button(ctx, button);
///```
#[derive(Debug, Component)]
pub struct Bumper (Stack, Rectangle, BumperContent);
impl OnEvent for Bumper {}

impl Bumper {
    /// Creates a new `Bumper` from a vector of boxed [`Drawables`](Drawable)
    pub fn new(ctx: &mut Context, content: Vec<Box<dyn Drawable>>) -> Self {
        let background = ctx.theme.colors.background.primary;
        let max = ctx.theme.layout.bumper_max;
        let max = if crate::config::IS_WEB {1200.0} else {max};
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(max), max));
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let layout = Stack(Offset::Center, Offset::Start, width, height, Padding::default());
        Bumper(layout, Rectangle::new(background, 0.0, None), BumperContent::new(content))
    }

    /// Creates a `Bumper` from two buttons.
    pub fn double_button(ctx: &mut Context, a: Button, b: Button) -> Self {
        Self::new(ctx, vec![Box::new(a), Box::new(b)])
    }

    /// Creates a `Bumper` from a single button.
    pub fn single_button(ctx: &mut Context, a: Button) -> Self {
        Self::new(ctx, vec![Box::new(a)])
    }

    /// Creates a `Bumper` from a text input.
    pub fn input(ctx: &mut Context, input: TextInput) -> Self {
        Self::new(ctx, vec![Box::new(input)])
    }

    /// Returns the items in the `Bumper`.
    pub fn items(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        &mut self.2.1
    }

    /// Find an item in the bumper. Will return the first instance of the type.
    ///
    /// ```rust
    /// let button = bumper.find::<Button>().expect("Could not find button in bumper");
    /// ```
    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.items().iter_mut().find_map(|item| item.as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let button = bumper.find_at::<Button>(0).expect("Could not find button at the first index in the bumper");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.items().get_mut(i)?.as_any_mut().downcast_mut::<T>()
    }
}

#[derive(Debug, Component)]
struct BumperContent (Row, Vec<Box<dyn Drawable>>);

impl BumperContent {
    fn new(content: Vec<Box<dyn Drawable>>) -> Self {
        BumperContent(Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0)), content)
    }
}

impl OnEvent for BumperContent {}

/// Button wrapper used for navigators.
#[derive(Debug, Component)]
pub struct NavigationButton(Stack, Option<Button>, Option<IconButton>, #[skip] ElementID);
impl OnEvent for NavigationButton {}
impl NavigationButton {
    pub fn new(id: ElementID, button: Option<Button>, icon_button: Option<IconButton>) -> Self {
        NavigationButton(Stack::default(), button, icon_button, id)
    }

    /// Returns the id of the `NavigationButton`
    pub fn id(&self) -> ElementID {
        self.3
    }

    /// Returns the inner `Button` component if it exists.
    pub fn button(&mut self) -> &mut Option<Button> {
        &mut self.1
    }

    /// Returns the inner `IconButton` component if it exists.
    pub fn icon_button(&mut self) -> &mut Option<IconButton> {
        &mut self.2
    }
}
