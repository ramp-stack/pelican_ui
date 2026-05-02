use prism::{drawables, Context, IS_MOBILE, IS_WEB, Request};
use prism::event::{self, Event, OnEvent, MouseEvent, MouseState, TickEvent};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::canvas::Align;
use prism::display::Bin;
use prism::layout::{Area, Column, Stack, Row, Padding, Offset, Size,  ScrollAnchor};

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::{Rectangle, TextInput, Profile};
use crate::components::text::{TextStyle, TextSize, ExpandableText};
use crate::components::button::{GhostIconButton, PrimaryButton, SecondaryButton};
use crate::components::avatar::{AvatarGroup, AvatarContent};
use crate::interface::system::MobileKeyboard;
use crate::interface::navigation::{RootInfo, Navigator};
use crate::interface::navigation::FlowContainer;

use ptsd::interfaces::{Body, Navigator as PTSDNavigator};
use ptsd::navigation::{NavigationEvent, AppPage};
use ptsd::utils::ValidationFn;
pub use ptsd::navigation::Pages;

use std::sync::{Arc, Mutex};

type OnEventFn = Box<dyn FnMut(&mut Box<dyn Drawable>, &mut Context, Box<dyn Event>) -> Vec<Box<dyn Event>>>;

/// The top-level interface of an app built with Pelican.
///
/// This interface automatically adapts to the platform.
#[derive(Component, Clone)]
pub struct Interface {
    layout: Stack,
    background: Rectangle,
    inner: ptsd::interfaces::Interface,
    // #[skip] pub on_event: Option<OnEventFn>
}

impl OnEvent for Interface {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // if IS_MOBILE && self.layout.4 == Padding(0.0, 0.0, 0.0, 0.0) {
        //     ctx.emit(Hardware::GetSafeArea);
        // }

        if let Some(InterfaceEvent::Disable(disable)) = event.downcast_ref::<InterfaceEvent>() {
            ctx.emit(event::Button::Disable(*disable));
        } //else if let Some(HardwareEvent::SafeArea(b, l, t, r)) = event.downcast_ref::<HardwareEvent>() {
        //     self.layout = Stack::new(Offset::default(), Offset::default(), Size::default(), Size::default(), Padding(*l, *t, *r, *b));
        //     println!("Setting padding to {:?}", self.layout.4);
        // }

        if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {*v = vec![0];}

        // let mut closure = self.on_event.take().expect("on_event missing");
        // let result = (closure)(self.inner(), ctx, event);
        // self.on_event = Some(closure);
        // result

        vec![event]
    }
}

impl std::fmt::Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Interface {
    pub fn new(ctx: &mut Context, theme: &Theme, mut roots: Vec<RootInfo>, _on_event: OnEventFn) -> Self {
        let pages: Vec<(String, Box<dyn AppPage>)> = roots.iter_mut().map(|r| (r.label.to_string(), r.page.take().unwrap() as Box<dyn AppPage>)).collect();
        let (b, l, t, r) = ctx.get_safe_area();
        Interface {
            layout: Stack::new(Offset::default(), Offset::default(), Size::default(), Size::default(), Padding(l, t, r, b)),
            background: Rectangle::new(theme.colors().get(ptsd::Background::Primary), 0.0, None),
            inner: match IS_WEB {
                true => { // web
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::web(theme, roots)) as Box<dyn PTSDNavigator>);
                    ptsd::interfaces::Interface::web(navigator, Screen::web(Pages::new(pages)))
                },
                false if IS_MOBILE => { // mobile
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::mobile(theme, roots)) as Box<dyn PTSDNavigator>);
                    ptsd::interfaces::Interface::mobile(navigator, Screen::mobile(Pages::new(pages)), MobileKeyboard::new(theme))
                },
                false => { // desktop
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::desktop(theme, roots)) as Box<dyn PTSDNavigator>);
                    ptsd::interfaces::Interface::desktop(navigator, Screen::desktop(theme, Pages::new(pages)))
                }
            },
            // on_event: Some(on_event),
        }
    }

    fn _inner(&mut self) -> &mut Box<dyn AppPage> {
        self.inner.pages().current()
    }
}

/// # Page
///
/// A Page is a UI container that holds [`Header`], [`Content`], and optional [`Bumper`] components.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/page.png"
///      alt="Page Example"
///      width="250">
#[derive(Debug, Component, Clone)]
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
            layout: Column::new(12.0, Offset::Center, Size::Fill, Padding(24.0, 0.0, 24.0, 0.0), None),
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
#[derive(Debug, Component, Clone)]
pub struct Content {
    layout: Stack,
    pub children: ContentChildren,
    #[skip] validation: Box<dyn ValidationFn>
}

impl Content {
    /// Creates a new `Content` component with a specified `Offset` (start, center, or end) and a list of `Box<dyn Drawable>` children.
    pub fn new(offset: Offset, children: Vec<Box<dyn Drawable>>, validation: Box<dyn ValidationFn>) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(375.0), 375.0));
        let anchor = if offset == Offset::End { ScrollAnchor::End } else { ScrollAnchor::Start };
        Content {
            layout: Stack::new(Offset::Center, offset, width, Size::Fill, Padding::default()),
            children: ContentChildren::new(children, anchor),
            validation,
        }
    }

    /// Find an item in the content. Will return the first instance of the type.
    ///
    /// ```rust
    /// let text = content.find::<Text>().expect("Could not find text in content");
    /// ```
    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.children.inner().iter_mut().find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let text_input = content.find_at::<TextInput>(0).expect("Could not find text input at first index in content");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.children.inner().get_mut(i).and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Remove an item from the content. Will remove the first instance of the type.
    ///
    /// ```rust
    /// let text = content.remove::<Text>().expect("Could not remove text from content");
    /// ```
    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) = self.children.inner().iter().position(|item| (**item).as_any().is::<T>()) {
            let boxed = self.children.inner().remove(pos);
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    pub fn children(&self) -> &Vec<Box<dyn Drawable>> {&self.children.1}
    pub fn children_mut(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.children.1}
}

impl OnEvent for Content {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let event = InterfaceEvent::Disable(!(self.validation)(ctx, self.children.inner().iter_mut().map(|c| c).collect()));
            ctx.emit(event);
        } else if let Some(AdjustScrollEvent::Vertical(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.children.column().adjust_scroll(*a);
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
            self.children.column().adjust_scroll(*y);
        }
        vec![event]
    }
}

#[derive(Debug, Component, Clone)]
pub struct ContentChildren(Column, Vec<Box<dyn Drawable>>);
impl OnEvent for ContentChildren {}
impl ContentChildren {
    pub fn new(children: Vec<Box<dyn Drawable>>, anchor: ScrollAnchor) -> Self {
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::default(), Some(anchor));
        // if anchor == ScrollAnchor::End { layout.set_scroll(f32::MAX); }
        ContentChildren(layout, children)
    }

    pub fn inner(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.1}
    pub fn column(&mut self) -> &mut Column {&mut self.0}
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
#[derive(Debug, Component, Clone)]
pub struct Header {
    layout: Row,
    pub left: HeaderIcon,
    pub center: Box<dyn Drawable>,
    pub right: HeaderIcon
}

impl OnEvent for Header {}

impl Header {
    /// A `Header` preset used for home pages.
    pub fn home(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        Self::_new(theme, title, None, icon, TextSize::H3)
    }

    /// A `Header` preset used for in-flow pages.
    pub fn stack(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        let closure = |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Pop);
        Self::_new(theme, title, Some((Icons::Left, Box::new(closure))), icon, TextSize::H4)
    }

    /// A `Header` preset used for end-of-flow pages.
    pub fn stack_end(theme: &Theme, title: &str) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Reset);
        Self::_new(theme, title, Some((Icons::Close, Box::new(closure))), None, TextSize::H4)
    }

    pub fn messaging(ctx: &mut Context, theme: &Theme, profiles: Vec<Profile>, exact_len: usize, info: Box<dyn FlowContainer>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| (0..exact_len).for_each(|_| ctx.emit(NavigationEvent::Pop));
        let l_icon = HeaderIcon::new(theme, Icons::Left, closure);
        let r_icon = HeaderIcon::new(theme, Icons::Info, Box::new(move |ctx: &mut Context, _theme: &Theme| {
            ctx.emit(NavigationEvent::Push(Some(info.clone()), vec![]))
        })); // this needs to navigate to info page

        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        Header {
            layout,
            left: l_icon,
            center: Box::new(MessageHeader::new(theme, profiles)),
            right: r_icon
        }
    }

    fn _new(
        theme: &Theme,
        title: &str,
        l_icon: Option<(Icons, Box<dyn Callback>)>,
        r_icon: Option<(Icons, Box<dyn Callback>)>,
        size: TextSize,
    ) -> Self {
        let text = ExpandableText::new(theme, &title, size, TextStyle::Heading, Align::Center, Some(1));

        let l_icon = l_icon.map(|(n, c)| HeaderIcon::new(theme, n, c)).unwrap_or_default();
        let r_icon = r_icon.map(|(n, c)| HeaderIcon::new(theme, n, c)).unwrap_or_default();

        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        Header {
            layout,
            left: l_icon,
            center: Box::new(text),
            right: r_icon
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageHeader(Column, AvatarGroup, ExpandableText);
impl OnEvent for MessageHeader {}
impl MessageHeader {
    pub fn new(theme: &Theme, profiles: Vec<Profile>) -> Self {
        let title = match profiles.len() > 1 {
            true => "Group Message".to_string(),
            false => match profiles.get(0) {
                Some(first) => first.name.to_string(),
                None => "New Message".to_string(),
            },
        };
        let text = ExpandableText::new(theme, &title, TextSize::H4, TextStyle::Heading, Align::Center, Some(1));
        let avatars = profiles.iter().map(|p| p.pfp.clone()).collect::<Vec<_>>();
        MessageHeader(Column::center(8.0), AvatarGroup::new(theme, avatars), text)
    }
}

/// # Header Icon
/// 
/// Optionally contains an icon, otherwise just reserves the space.
/// These are only to be used in [`Header`] components.
#[derive(Debug, Component, Clone)]
pub struct HeaderIcon {
    layout: Stack,
    pub icon: Option<GhostIconButton>
}
impl OnEvent for HeaderIcon {}
impl Default for HeaderIcon {fn default() -> Self {Self::none()}}

impl HeaderIcon {
    pub fn new(theme: &Theme, icon: Icons, closure: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon{layout, icon: Some(GhostIconButton::new(theme, icon, closure))}
    }

    pub fn none() -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        HeaderIcon {layout, icon: None}
    }
}

#[derive(Component, Debug, Clone)]
pub struct Bumper {layout: Stack, background: Rectangle, content: BumperContent}
impl OnEvent for Bumper {}

impl Bumper {
    /// A `Bumper` preset used for home pages.
    pub fn home(theme: &Theme, first: Option<(String, Box<dyn Callback>)>, second: Option<(String, Box<dyn Callback>)>) -> Self {
        let mut content: Vec<Box<dyn Drawable>> = vec![];
        if let Some((l, c)) = first {content.push(Box::new(PrimaryButton::new(theme, &l, c))); }
        if let Some((l, c)) = second { content.push(Box::new(PrimaryButton::new(theme, &l, c))); }
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    /// A `Bumper` preset used for in-flow pages.
    pub fn stack(
        theme: &Theme,
        label: Option<&str>, 
        on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static, 
        secondary: Option<(String, Box<dyn Callback>)>, 
    ) -> Self {
        let mut content = drawables![PrimaryButton::new(theme, label.unwrap_or("Continue"), Box::new(on_click))];
        if let Some((l, c)) = secondary { content.push(Box::new(SecondaryButton::large(theme, &l, c))); }
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    /// A `Bumper` preset used for end-of-flow pages.
    pub fn stack_end(theme: &Theme, exact_pages: Option<usize>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| match exact_pages {
            Some(num) => (0..num).for_each(|_| ctx.emit(NavigationEvent::Pop)),
            None => ctx.emit(NavigationEvent::Reset),
        };
        
        let content = SecondaryButton::large(theme, "Done", Box::new(closure));
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(vec![Box::new(content)]) }
    }

    pub fn input(theme: &Theme, placeholder: &str, on_submit: impl FnMut(&mut Context, &mut String) + 'static) -> Self {
        let content = TextInput::new(theme, None, None, Some(placeholder), None, Some((Icons::Send, Arc::new(Mutex::new(on_submit)))));
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(vec![Box::new(content)]) }
    }

    fn layout(theme: &Theme) -> (Stack, Rectangle) {
        let background = Rectangle::new(theme.colors().get(ptsd::Background::Primary), 0.0, None);
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(375.0), 375.0));
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let layout = Stack(Offset::Center, Offset::Start, width, height, Padding::default());
        
        (layout, background)
    }

    pub fn on_click(&mut self) -> Vec<&mut Box<dyn ptsd::utils::Callback>> {
        self.content.children.iter_mut().filter_map(|child| {
            child.downcast_mut::<PrimaryButton>().map(|b| b.1.on_click())
        }).collect::<Vec<_>>()
    }

    pub fn default(theme: &Theme) -> Self {
        Self::home(theme, 
            Some(("Press me".to_string(), Box::new(|_: &mut Context, _: &Theme| println!("Pressed....")))), 
            Some(("No Press me".to_string(), Box::new(|_: &mut Context, _: &Theme| println!("Pressed....")))), 
        )
    }
}

#[derive(Debug, Component, Clone)]
pub struct BumperContent { layout: Row, children: Vec<Box<dyn Drawable>> }
impl OnEvent for BumperContent {}
impl BumperContent {
    fn new(children: Vec<Box<dyn Drawable>>) -> Self {
        let padding = if children.is_empty() {Padding::default()} else {Padding(0.0, 16.0, 0.0, 16.0)};
        BumperContent{ layout: Row::new(16.0, Offset::Center, Size::Fit, padding), children }
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

/// Adjust the scroll value of a [`Scroll`] layout.
#[derive(Debug, Clone)]
pub enum InterfaceEvent {
    Disable(bool)
}

impl Event for InterfaceEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Component, Clone)]
pub struct Screen(Stack, Pages, Option<Bin<Stack, Rectangle>>);

impl OnEvent for Screen {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {*v = vec![1];}
        vec![event]
    }
}

impl Screen {
    pub fn desktop(theme: &Theme, pages: Pages) -> Self {
        let color = theme.colors().get(ptsd::Outline::Secondary);
        let line_layout = Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default());
        let border = Bin(line_layout, Rectangle::new(color, 0.0, None));

        Screen(Stack::default(), pages, Some(border))
    }

    pub fn mobile(pages: Pages) -> Self {
        Screen(Stack::default(), pages, None)
    }

    pub fn web(pages: Pages) -> Self {
        Screen(Stack::default(), pages, None)
    }
}

impl Body for Screen {
    fn pages(&mut self) -> &mut Pages {
        &mut self.1
    }
}
