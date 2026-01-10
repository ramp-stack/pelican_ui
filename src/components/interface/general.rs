use prism::{drawables, Context, IS_MOBILE, IS_WEB, Request};
use prism::event::{Event, OnEvent, MouseEvent, MouseState, TickEvent};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::canvas::Align;
use prism::layout::{Area, Column, Stack, Row, Padding, Offset, Size,  ScrollAnchor};
use prism::display::Bin;

use crate::Theme;
use crate::components::{Rectangle};
use crate::components::text::{TextStyle, TextSize, ExpandableText};
use crate::components::button::{GhostIconButton, PrimaryButton, SecondaryButton};
use crate::components::interface::navigation::{NavigationEvent, RootInfo};
use crate::components::interface::interfaces;

use crate::utils::ValidationFn;
use crate::utils::Callback;

type OnEventFn = Box<dyn FnMut(&mut Context, Box<dyn Event>) -> Vec<Box<dyn Event>>>;

/// The top-level interface of an app built with Pelican.
///
/// This interface automatically adapts to the platform.
#[derive(Component)]
pub struct Interface {
    layout: Stack,
    background: Rectangle,
    inner: interfaces::Interface,
    #[skip] pub on_event: OnEventFn
}

impl OnEvent for Interface {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        (self.on_event)(ctx, event)
    }
}

impl std::fmt::Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Interface {
    pub fn new(ctx: &mut Context, navigation: Vec<RootInfo>, on_event: Box<dyn FnMut(&mut Context, Box<dyn Event>) -> Vec<Box<dyn Event>>>) -> Self {
        let color = ctx.state.get_or_default::<Theme>().colors.background.primary;
        Interface {
            layout: Stack::default(),
            background: Rectangle::new(color, 0.0, None),
            inner: match IS_WEB {
                true => interfaces::Interface::web(ctx, navigation),
                false if IS_MOBILE => interfaces::Interface::mobile(ctx, navigation),
                false => interfaces::Interface::desktop(ctx, navigation),
            },
            on_event,
        }
    }
}

/// # Page
///
/// A Page is a UI container that holds [`Header`], [`Content`], and optional [`Bumper`] components.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/page.png"
///      alt="Page Example"
///      width="250">
#[derive(Debug, Component)]
pub struct Page {
    layout: Column,
    pub header: Header,
    pub content: Content,
    pub bumper: Option<Bumper>
}

impl OnEvent for Page {}

impl Page {
    /// Creates a new [`Page`] from an optional [`Header`], [`Content`], and optional [`Bumper`]
    pub fn new(header: Header, content: Content, bumper: Option<Bumper>) -> Self {
        Page {
            layout: Column::new(12.0, Offset::Center, Size::Fill, Padding::default(), None),
            header,
            content,
            bumper,
        }
    }
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
/// let text = Text::new(ctx, "Set up a name, description, and team before starting your project.", TextStyle::Primary, text_size, Align::Center);
/// let content = Content::new(ctx, Offset::Center, vec![Box::new(text)]);
/// ```
#[derive(Debug, Component)]
pub struct Content {
    layout: Column,
    pub children: Vec<Box<dyn Drawable>>
}

impl Content {
    /// Creates a new `Content` component with a specified `Offset` (start, center, or end) and a list of `Box<dyn Drawable>` children.
    pub fn new(offset: Offset, children: Vec<Box<dyn Drawable>>) -> Self {
        println!("PAGE OFFSET {:?}", offset);
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(375.0), 375.0));
        let anchor = if offset == Offset::End { ScrollAnchor::End } else { ScrollAnchor::Start };
        // if offset == Offset::End { layout.set_scroll(f32::MAX); }
        Content {
            layout: Column::new(24.0, offset, width, Padding::new(16.0), Some(anchor)),
            children,
        }
    }

    /// Find an item in the content. Will return the first instance of the type.
    ///
    /// ```rust
    /// let text = content.find::<Text>().expect("Could not find text in content");
    /// ```
    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.children.iter_mut().find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let text_input = content.find_at::<TextInput>(0).expect("Could not find text input at first index in content");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.children.get_mut(i).and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Remove an item from the content. Will remove the first instance of the type.
    ///
    /// ```rust
    /// let text = content.remove::<Text>().expect("Could not remove text from content");
    /// ```
    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) = self.children.iter().position(|item| (**item).as_any().is::<T>()) {
            let boxed = self.children.remove(pos);
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }
}

impl OnEvent for Content {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(AdjustScrollEvent::Vertical(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.layout.adjust_scroll(*a);
        // } else if let Some(events::InputField::Select(id, true)) = event.downcast_ref::<events::InputField>() {
        //     if roost_ui::IS_MOBILE {
        //         let mut total_height = 0.0;
        //         for item in self.items().iter_mut() {
        //             match item.as_any_mut().downcast_mut::<TextInput>() {
        //                 Some(input) if input.inner.5 == *id => {
        //                     self.0.set_scroll(total_height);
        //                     break;
        //                 }
        //                 _ => {
        //                     let size = item.request_size(ctx);
        //                     total_height += size.max_height();
        //                 }
        //             }
        //         }
        //     }
        } else if let Some(MouseEvent { state: MouseState::Scroll(_, y), position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            self.layout.adjust_scroll(*y);
        }
        vec![event]
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
pub struct Header {
    layout: Row,
    pub left: HeaderIcon,
    pub center: Box<dyn Drawable>,
    pub right: HeaderIcon
}

impl OnEvent for Header {}

impl Header {
    /// A `Header` preset used for home pages.
    pub fn home(ctx: &mut Context, title: &str, icon: Option<(String, Callback)>) -> Self {
        Self::_new(ctx, title, None, icon, TextSize::H3)
    }

    /// A `Header` preset used for in-flow pages.
    pub fn stack(ctx: &mut Context, title: &str, icon: Option<(String, Callback)>) -> Self {
        let closure = |ctx: &mut Context| ctx.send(Request::Event(Box::new(NavigationEvent::Pop)));
        Self::_new(ctx, title, Some(("left".to_string(), Box::new(closure))), icon, TextSize::H4)
    }

    /// A `Header` preset used for end-of-flow pages.
    pub fn stack_end(ctx: &mut Context, title: &str) -> Self {
        let closure = move |ctx: &mut Context| ctx.send(Request::Event(Box::new(NavigationEvent::Reset)));
        Self::_new(ctx, title, Some(("close".to_string(), Box::new(closure))), None, TextSize::H4)
    }

    fn _new(
        ctx: &mut Context,
        title: &str,
        l_icon: Option<(String, Callback)>,
        r_icon: Option<(String, Callback)>,
        size: TextSize,
    ) -> Self {
        let clean: String = title.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
        let title = clean[..1].to_uppercase() + &clean[1..].to_lowercase();
        let text = ExpandableText::new(ctx, &title, size, TextStyle::Heading, Align::Center, Some(1));

        let l_icon = l_icon.map(|(n, c)| HeaderIcon::new(ctx, &n, c)).unwrap_or_default();
        let r_icon = r_icon.map(|(n, c)| HeaderIcon::new(ctx, &n, c)).unwrap_or_default();

        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0));
        Header {
            layout,
            left: l_icon,
            center: Box::new(text),
            right: r_icon
        }
    }
}

/// # Header Icon
/// 
/// Optionally contains an icon, otherwise just reserves the space.
/// These are only to be used in [`Header`] components.
#[derive(Debug, Component)]
pub struct HeaderIcon {
    layout: Stack,
    pub icon: Option<GhostIconButton>
}
impl OnEvent for HeaderIcon {}
impl Default for HeaderIcon {fn default() -> Self {Self::none()}}

impl HeaderIcon {
    pub fn new(ctx: &mut Context, icon: &str, closure: impl FnMut(&mut Context) + 'static) -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon{layout, icon: Some(GhostIconButton::new(ctx, icon, closure))}
    }

    pub fn none() -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon {layout, icon: None}
    }
}

#[derive(Component, Debug)]
pub struct Bumper {layout: Stack, background: Rectangle, content: BumperContent}
impl OnEvent for Bumper {}

impl Bumper {
    /// A `Bumper` preset used for home pages.
    pub fn home(ctx: &mut Context, first: (String, Callback), second: Option<(String, Callback)>, validation: Option<Box<dyn ValidationFn>>) -> Self {
        let mut content = drawables![PrimaryButton::new(ctx, &first.0, Box::new(first.1), validation.clone())];
        if let Some((l, c)) = second { content.push(Box::new(PrimaryButton::new(ctx, &l, c, validation))); }
        let (layout, background) = Self::layout(ctx);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    /// A `Bumper` preset used for in-flow pages.
    pub fn stack(
        ctx: &mut Context, 
        label: Option<&str>, 
        on_click: impl FnMut(&mut Context) + 'static, 
        secondary: Option<(String, Callback)>, 
        validation: Option<Box<dyn ValidationFn>>
    ) -> Self {
        let mut content = drawables![PrimaryButton::new(ctx, label.unwrap_or("Continue"), Box::new(on_click), validation.clone())];
        if let Some((l, c)) = secondary { content.push(Box::new(SecondaryButton::large(ctx, &l, c, validation))); }
        let (layout, background) = Self::layout(ctx);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    /// A `Bumper` preset used for end-of-flow pages.
    pub fn stack_end(ctx: &mut Context, exact_pages: Option<usize>) -> Self {
        let closure = move |ctx: &mut Context| match exact_pages {
            Some(num) => (0..num).for_each(|_| ctx.send(Request::Event(Box::new(NavigationEvent::Pop)))),
            None => ctx.send(Request::Event(Box::new(NavigationEvent::Reset)))
        };
        
        let content = SecondaryButton::large(ctx, "Done", Box::new(closure), None);
        let (layout, background) = Self::layout(ctx);
        Bumper { layout, background, content: BumperContent::new(vec![Box::new(content)]) }
    }

    fn layout(ctx: &mut Context) -> (Stack, Rectangle) {
        let background = Rectangle::new(ctx.state.get_or_default::<Theme>().colors.background.primary, 0.0, None);
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(375.0), 375.0));
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let layout = Stack(Offset::Center, Offset::Start, width, height, Padding::default());
        
        (layout, background)
    }

    pub fn default(ctx: &mut Context) -> Self {
        Self::home(ctx, 
            ("Press me".to_string(), Box::new(|_: &mut Context| println!("Pressed...."))), 
            Some(("No Press me".to_string(), Box::new(|_: &mut Context| println!("Pressed....")))), 
            None
        )
    }
}

#[derive(Debug, Component)]
pub struct BumperContent { layout: Row, children: Vec<Box<dyn Drawable>> }
impl OnEvent for BumperContent {}
impl BumperContent {
    fn new(children: Vec<Box<dyn Drawable>>) -> Self {
        BumperContent{ layout: Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0)), children }
    }
}

/// Adjust the scroll value of a [`Scroll`] layout.
#[derive(Debug, Clone)]
pub enum AdjustScrollEvent {
    Vertical(f32),
    Horizontal(f32),
}

impl Event for AdjustScrollEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}
