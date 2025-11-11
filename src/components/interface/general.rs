use roost_ui::{emitters, drawables, Component, Context, IS_MOBILE, IS_WEB};
use roost_ui::events::{Event, OnEvent, MouseEvent, MouseState};
use roost_ui::drawable::{Drawable, Align};
use roost_ui::layouts::{AdjustScrollEvent, Column, Stack, Row, Padding, Offset, Size, Scroll, ScrollAnchor, ScrollDirection};

use crate::components::{Rectangle, TextStyle, TextSize, ExpandableText};
use crate::components::button::{GhostIconButton, PrimaryButton, SecondaryButton};
use crate::components::interface::navigation::{NavigationEvent, RootInfo};
use crate::components::interface::interfaces;

use crate::utils::Callback;
use crate::plugin::PelicanUI;

/// The top-level interface of an app built with Pelican.
///
/// This interface automatically adapts to the platform.
#[derive(Component, Debug)]
pub struct Interface(Stack, Rectangle, interfaces::Interface);
impl OnEvent for Interface {}

impl Interface {
    pub fn new(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let interface: interfaces::Interface = match IS_WEB {
            true => interfaces::Interface::web(ctx, navigation),
            false if IS_MOBILE => interfaces::Interface::mobile(ctx, navigation),
            false => interfaces::Interface::desktop(ctx, navigation),
        };

        Interface(Stack::default(), Rectangle::new(color, 0.0, None), interface)
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
pub struct Page(Column, Header, Content, Option<Bumper>);
impl OnEvent for Page {}

impl Page {
    /// Creates a new [`Page`] from an optional [`Header`], [`Content`], and optional [`Bumper`]
    pub fn new(header: Header, content: Content, bumper: Option<Bumper>) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, f32::MAX));
        Page(
            Column::new(12.0, Offset::Center, width, Padding::default()),
            header,
            content,
            bumper,
        )
    }

    /// Returns the header if it exists.
    pub fn header(&mut self) -> &mut Header {&mut self.1}
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
/// let text = Text::new(ctx, "Set up a name, description, and team before starting your project.", TextStyle::Primary, text_size, Align::Center);
/// let content = Content::new(ctx, Offset::Center, vec![Box::new(text)]);
/// ```
#[derive(Debug, Component)]
pub struct Content (Scroll, emitters::Scrollable<ContentChildren>);

impl Content {
    /// Creates a new `Content` component with a specified `Offset` (start, center, or end) and a list of `Box<dyn Drawable>` children.
    pub fn new(ctx: &mut Context, offset: Offset, content: Vec<Box<dyn Drawable>>) -> Self {
        let layout = ctx.get::<PelicanUI>().get().0.theme().layout.clone();
        let max = if roost_ui::IS_WEB {1200.0} else {layout.content_max};
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(max), max));
        let height = Size::custom(move |_: Vec<(f32, f32)>|(0.0, f32::MAX));
        let anchor = if offset == Offset::End { ScrollAnchor::End } else { ScrollAnchor::Start };
        let scroll = Scroll::new(Offset::Center, offset, width, height, Padding::default(), anchor, ScrollDirection::Vertical);
        // if offset == Offset::End { layout.set_scroll(f32::MAX); }
        Content(scroll, emitters::Scrollable::new(ContentChildren::new(content, layout.content_padding))) 
    }

    /// Find an item in the content. Will return the first instance of the type.
    ///
    /// ```rust
    /// let text = content.find::<Text>().expect("Could not find text in content");
    /// ```
    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.items().iter_mut().find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let text_input = content.find_at::<TextInput>(0).expect("Could not find text input at first index in content");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.items().get_mut(i).and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Remove an item from the content. Will remove the first instance of the type.
    ///
    /// ```rust
    /// let text = content.remove::<Text>().expect("Could not remove text from content");
    /// ```
    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) = self.items().iter().position(|item| (**item).as_any().is::<T>()) {
            let boxed = self.items().remove(pos);
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    /// Returns all the items in the content
    pub fn items(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.1.1.inner.1}
    /// Returns the offset of the items.
    pub fn offset(&mut self) -> &mut Offset {self.0.offset()}
}

impl OnEvent for Content {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(AdjustScrollEvent::Vertical(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.0.adjust_scroll(*a);
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
            self.0.adjust_scroll(*y);
        }
        vec![event]
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
    /// let header = Header::home(ctx, "Explore", Some(("search", 1)))
    /// ```
    pub fn home(ctx: &mut Context, title: &str, icon: Option<(&'static str, Callback)>) -> Self {
        Self::_new(ctx, title, None, icon, TextSize::H3)
    }

    /// A `Header` preset used for in-flow pages.
    /// This header contains a back button that always navigates to index 0.
    ///
    /// ```rust
    /// let header = Header::stack(ctx, "Select role");
    /// ```
    pub fn stack(ctx: &mut Context, title: &str) -> Self {
        let closure = |ctx: &mut Context| ctx.trigger_event(NavigationEvent::Pop);
        Self::_new(ctx, title, Some(("left", Box::new(closure))), None, TextSize::H4)
    }

    /// A `Header` preset used for end-of-flow pages.
    /// This header contains a close button that always pops back the number of times provided.
    ///
    /// ```rust
    /// let header = Header::stack_end(ctx, "Select role", 4);
    /// ```
    pub fn stack_end(ctx: &mut Context, title: &str, len: usize) -> Self {
        let closure = move |ctx: &mut Context| {
            for _ in 0..len {
                ctx.trigger_event(NavigationEvent::Pop);
            }
        };
        Self::_new(ctx, title, Some(("close", Box::new(closure))), None, TextSize::H4)
    }


    fn _new(
        ctx: &mut Context,
        title: &str,
        l_icon: Option<(&'static str, Callback)>,
        r_icon: Option<(&'static str, Callback)>,
        size: TextSize,
    ) -> Self {
        let clean: String = title.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
        let title = clean[..1].to_uppercase() + &clean[1..].to_lowercase();
        let text = ExpandableText::new(ctx, &title, size, TextStyle::Heading, Align::Center, Some(1));

        let l_icon = l_icon.map(|(n, c)| HeaderIcon::new(ctx, n, c)).unwrap_or_default();
        let r_icon = r_icon.map(|(n, c)| HeaderIcon::new(ctx, n, c)).unwrap_or_default();

        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(24.0, 16.0, 24.0, 16.0));
        Header(layout, l_icon, Box::new(text), r_icon)
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
    /// A `Bumper` preset used for home pages.
    ///
    /// ```rust
    /// let bumper = Header::home(ctx, "New Message", None); // navigates to 1
    /// let bumper = Header::home(ctx, "Receive", Some("Send")) // navigates to 1 and 2
    /// ```
    pub fn home(ctx: &mut Context, first: (&str, impl FnMut(&mut Context) + 'static), second: Option<(&str, Callback)>) -> Self {
        let first = PrimaryButton::new(ctx, first.0, Box::new(first.1), false);

        let second = second.map(|(label, on_click)| {
            PrimaryButton::new(ctx, label, on_click, false)
        });

        Self::new(ctx, drawables![first, second])
    }

    /// A `Bumper` preset used for in-flow pages.
    /// This bumper contains a button. If no label is provided, it will default to "Continue".
    ///
    /// ```rust
    /// let bumper = Bumper::stack(ctx, false);
    /// ```
    pub fn stack(ctx: &mut Context, label: Option<&str>, is_disabled: bool, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let button = PrimaryButton::new(ctx, label.unwrap_or("Continue"), Box::new(on_click), is_disabled);
        Self::new(ctx, drawables![button])
    }

    /// A `Bumper` preset used for end-of-flow pages.
    /// This bumper contains a button labeled "Done".
    ///
    /// ```rust
    /// let bumper = Bumper::stack_end(ctx);
    /// ```
    pub fn stack_end(ctx: &mut Context, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let button = SecondaryButton::large(ctx, "Done", Box::new(on_click));
        Self::new(ctx, drawables![button])
    }

    /// Creates a new `Bumper` from a vector of boxed [`Drawables`](Drawable)
    pub fn new(ctx: &mut Context, content: Vec<Box<dyn Drawable>>) -> Self {
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let max = ctx.get::<PelicanUI>().get().0.theme().layout.bumper_max;
        let max = if roost_ui::IS_WEB {1200.0} else {max};
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
        self.items().iter_mut().find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    /// Find an item in the bumper at a specific index.
    ///
    /// ```rust
    /// let button = bumper.find_at::<Button>(0).expect("Could not find button at the first index in the bumper");
    /// ```
    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.items().get_mut(i).and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
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
