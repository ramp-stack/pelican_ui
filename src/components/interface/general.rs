use mustache::events::{OnEvent, MouseState, MouseEvent, Event};
use mustache::drawable::{Drawable, Component, Align};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{drawables, Context, Component};

use crate::components::{Rectangle, TextStyle, Text};
use crate::events::{TextInputSelect, AdjustScrollEvent};
use crate::layout::{Column, Stack, Row, Padding, Offset, Size, Scroll, ScrollAnchor};
use crate::components::avatar::{AvatarContent, Avatar};
use crate::components::interactions::{ButtonState, self};
use crate::components::button::{ButtonSize, ButtonWidth};
use crate::components::button::GhostIconButton;
use crate::components::button::IconButton;
use crate::components::button::Button;
use crate::components::{TextInput, Icon};
use crate::events::NavigateEvent;
use crate::utils::ElementID;
use crate::pages::AppPage;
use crate::plugin::PelicanUI;
use crate::Callback;
use crate::components::interface::{
    desktop::DesktopInterface,
    mobile::MobileInterface,
    web::WebInterface,
};

pub struct NavigateInfo {
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) get_page: Option<PageBuilder>
}

impl NavigateInfo {
    pub fn icon(icon: &'static str, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        NavigateInfo {
            icon,
            label: label.to_string(),
            avatar: None,
            get_page: Some(Box::new(get_page))
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        NavigateInfo {
            icon: "profile",
            label: label.to_string(),
            avatar: Some(avatar),
            get_page: Some(Box::new(get_page))
        }
    }
}

pub type PageBuilder = Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>;

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
        navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>,
    ) -> Self {
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;

        let (mobile, desktop, web) = match crate::config::IS_WEB {
            true => (None, None, Some(WebInterface::new(ctx, start_page, navigation, None))),
            false if crate::config::IS_MOBILE => (Some(MobileInterface::new(ctx, start_page, navigation)), None, None),
            false => (None, Some(DesktopInterface::new(ctx, start_page, navigation)), None),
        };

        Interface(Stack::default(), Some(Rectangle::new(color, 0.0, None)), mobile, desktop, web)
    }

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
        let layout = ctx.get::<PelicanUI>().get().0.theme().layout.clone();
        let max = if crate::config::IS_WEB {1200.0} else {layout.content_max};
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(max), max));
        let height = Size::custom(move |_: Vec<(f32, f32)>|(0.0, f32::MAX));
        let anchor = if offset == Offset::End { ScrollAnchor::End } else { ScrollAnchor::Start };
        let scroll = Scroll::new(Offset::Center, offset, width, height, Padding::default(), anchor);
        // if offset == Offset::End { layout.set_scroll(f32::MAX); }
        Content(scroll, ContentChildren::new(content, layout.content_padding)) 
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
                        Some(input) if input.element_id == *id => {
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
pub struct Header(Row, HeaderIcon, Box<dyn Drawable>, HeaderIcon);
impl OnEvent for Header {}

impl Header {
    /// A `Header` preset used for home pages.
    ///
    /// ```rust
    /// let header = Header::home(ctx, "My Account", None);
    /// ```
    pub fn home(ctx: &mut Context, title: &str, icon: Option<(&'static str, usize)>) -> Self {
        let icon = icon.map(|(i,u)| {
            let c = move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(u));
            HeaderIcon::new(ctx, i, c)
        }).unwrap_or_default();
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.h3;
        let title = Text::new(ctx, title, size, TextStyle::Heading, Align::Left, None);
        Header::_new(HeaderIcon::none(), title, icon)
    }

    /// A `Header` preset used for in-flow pages.
    ///
    /// ```rust
    /// let header = Header::stack(ctx, 0, "Select role");
    /// ```
    pub fn stack(ctx: &mut Context, back_index: usize, title: &str) -> Self {
        let back = HeaderIcon::new(ctx, "left", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(back_index)));
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.h4;
        let title = Text::new(ctx, title, size, TextStyle::Heading, Align::Left, None);
        Header::_new(back, title, HeaderIcon::none())
    }

    /// A `Header` preset used for end-of-flow pages.
    ///
    /// ```rust
    /// let header = Header::stack_end(ctx, 0, "Select role");
    /// ```
    pub fn stack_end(ctx: &mut Context, next_index: usize, title: &str) -> Self {
        let back = HeaderIcon::new(ctx, "close", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(next_index)));
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.h4;
        let title = Text::new(ctx, title, size, TextStyle::Heading, Align::Left, None);
        Header::_new(back, title, HeaderIcon::none())
    }

    fn _new(left: HeaderIcon, content: impl Drawable, right: HeaderIcon) -> Self {
        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0));
        Header(layout, left, Box::new(content), right)
    }
}

/// # Header Icon
/// 
/// Optionally contains an icon, otherwise just reserves the space.
/// These are only to be used in [`Header`] components.
#[derive(Debug, Component)]
pub struct HeaderIcon(Stack, Option<GhostIconButton>);
impl OnEvent for HeaderIcon {}
impl Default for HeaderIcon {fn default() -> Self {Self::none()}}

impl HeaderIcon {
    pub fn new(ctx: &mut Context, icon: &'static str, closure: impl FnMut(&mut Context) + 'static) -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon(layout, Some(GhostIconButton::new(ctx, icon, closure)))
    }

    pub fn none() -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon(layout, None)
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
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let max = ctx.get::<PelicanUI>().get().0.theme().layout.bumper_max;
        let max = if crate::config::IS_WEB {1200.0} else {max};
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(max), max));
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let layout = Stack(Offset::Center, Offset::Start, width, height, Padding::default());
        Bumper(layout, Rectangle::new(background, 0.0, None), BumperContent::new(content))
    }

    /// Returns the items in the `Bumper`.
    pub fn items(&mut self) -> &mut Vec<Box<dyn Drawable>> { &mut self.2.1 }

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
impl OnEvent for BumperContent {}

impl BumperContent {
    fn new(content: Vec<Box<dyn Drawable>>) -> Self {
        BumperContent(Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0)), content)
    }
}


/// Button wrapper used for navigators.
#[derive(Debug, Component)]
pub struct NavigationButton(Stack, NavigatorGhostButton, #[skip] ElementID);
impl OnEvent for NavigationButton {}
impl NavigationButton {
    pub fn new(id: ElementID, button: NavigatorGhostButton) -> Self {
        NavigationButton(Stack::default(), button, id)
    }

    /// Returns the id of the `NavigationButton`
    pub fn id(&self) -> ElementID { self.2 }

    /// Returns the inner [`NavigatorGhostButton`] component.
    pub fn inner(&mut self) -> &mut NavigatorGhostButton { &mut self.1 }
}

#[derive(Debug, Component)]
pub struct NavigatorGhostButton(Stack, interactions::Button);
impl OnEvent for NavigatorGhostButton {}
impl NavigatorGhostButton {
    pub fn desktop_icon(ctx: &mut Context, icon: &'static str, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool) -> NavigatorGhostButton {
        let state = if is_selected {ButtonState::Selected} else {ButtonState::Default};
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font(ctx);
            let icon_size = ButtonSize::Large.icon();
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let icon = Icon::new(ctx, icon, colors.label, icon_size);
            Button::new(drawables![icon, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorGhostButton(Stack::default(), interactions::Button::new(Box::new(on_click), default, hover, pressed, selected, disabled, state))
    }

    pub fn desktop_avatar(ctx: &mut Context, avatar: AvatarContent, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool) -> NavigatorGhostButton {
        let state = if is_selected {ButtonState::Selected} else {ButtonState::Default};
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font(ctx);
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let avatar = Avatar::new(ctx, avatar.clone(), None, false, ButtonSize::Large.icon(), None);
            Button::new(drawables![avatar, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorGhostButton(Stack::default(), interactions::Button::new(Box::new(on_click), default, hover, pressed, selected, disabled, state))
    }

    pub fn mobile(ctx: &mut Context, icon: &'static str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool) -> NavigatorGhostButton {
        let state = if is_selected {ButtonState::Selected} else {ButtonState::Default};
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.disabled, colors.hover, colors.pressed, colors.default, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|colors| {
            IconButton::new(ctx, icon, false, ButtonSize::Large, colors.background, colors.outline, colors.label)
        });
        NavigatorGhostButton(Stack::default(), interactions::Button::new(Box::new(on_click), default, hover, pressed, selected, disabled, state))
    }

    pub fn inner(&mut self) -> &mut interactions::Button {&mut self.1}
}