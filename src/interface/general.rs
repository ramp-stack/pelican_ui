use prism::Context;
use prism::events::{DynEvent, TickEvent};
use prism::drawable::{Drawable, Component};
use prism::drawable::text::Align;
use ptsd::display::Bin;
use ptsd::layout::{Column, Stack, Row, Padding, Offset, Size};
use ptsd::emitters::Scrollable;

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::{Rectangle, TextInput, TextInputEvent, QRCodeScannedEvent, Profile};
use crate::components::text::{TextStyle, TextSize, ExpandableText};
use crate::components::button::{GhostIconButton, PrimaryButton, SecondaryButton};
use crate::components::avatar::AvatarGroup;
use crate::interface::system::{MobileKeyboard, MobileKeyboardEvent};
use crate::interface::message_bar::MessageBar;
use crate::interface::navigation::{RootInfo, Navigator};
use crate::components::Alert;

use ptsd::interfaces::{Body, Navigator as PTSDNavigator};
use ptsd::navigation::{NavigationEvent, AppPage};
use ptsd::utils::ValidationFn;
pub use ptsd::navigation::Pages;

crate::closure_trait!(OnEventFn, FnMut(&mut Context, Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>>);

#[derive(Component, Debug, Clone)]
#[on_dyn_event]
pub struct Interface {
    layout: Stack,
    background: Rectangle,
    inner: ptsd::interfaces::Interface,
    #[skip] pub on_event: Box<dyn OnEventFn>
}

impl Interface {
    fn on_event(&mut self, ctx: &mut Context, mut event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {

        if let Some(InterfaceEvent::Disable(disable)) = event.downcast_ref::<InterfaceEvent>() {
            ctx.emit(ptsd::events::Button::Disable(*disable));
        }

        if let ptsd::interfaces::Interface::Mobile{keyboard, ..} = &mut self.inner {
            let showing = keyboard.is_showing();
            if let Some(keyboard) = keyboard.inner().downcast_mut::<MobileKeyboard>() && !keyboard.on_interface_event(ctx, &*event, showing) {
                return vec![];
            }
        }

        if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {*v = vec![0];}

        (self.on_event)(ctx, event)
    }
}

impl Interface {
    pub fn new(ctx: &mut Context, theme: &Theme, mut roots: Vec<RootInfo>, on_event: Box<dyn OnEventFn>) -> Self {
        let pages: Vec<(String, Box<dyn AppPage>)> = roots.iter_mut().map(|r| (r.label.to_string(), r.page.take().unwrap() as Box<dyn AppPage>)).collect();
        Interface {
            layout: Stack::default(),
            background: Rectangle::new(theme.colors().get(ptsd::Background::Primary), 0.0, None),
            inner: match cfg!(target_arch = "wasm32") {
                true => {
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::web(theme, roots)) as Box<dyn PTSDNavigator>);
                    ptsd::interfaces::Interface::web(navigator, Screen::web(Pages::new(pages)))
                },
                false if cfg!(any(target_os = "ios", target_os = "android")) => {
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::mobile(theme, roots)) as Box<dyn PTSDNavigator>);
                    let keyboard = MobileKeyboard::new(ctx, theme);
                    ptsd::interfaces::Interface::mobile(navigator, Screen::mobile(Pages::new(pages)), keyboard)
                },
                false => {
                    let navigator = (pages.len() > 1).then_some(Box::new(Navigator::desktop(theme, roots)) as Box<dyn PTSDNavigator>);
                    ptsd::interfaces::Interface::desktop(navigator, Screen::desktop(theme, Pages::new(pages)))
                }
            },
            on_event,
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct Page {
    layout: Column,
    pub header: Header,
    pub content: Content,
    pub bumper: Option<Bumper>,
}


impl Page {
    pub fn new(header: Header, content: Content, bumper: Option<Bumper>) -> Self {
        Page {
            layout: Column::new(12.0, Offset::Center, Size::Fill, Padding(24.0, 0.0, 24.0, 0.0), None),
            header,
            content,
            bumper,
        }
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Content {
    layout: Stack,
    pub children: Scrollable<ContentChildren>,
    #[skip] validation: Box<dyn ValidationFn>,
    #[skip] last_input: Option<usize>,
}

impl Content {
    pub fn new(offset: Offset, children: Vec<Box<dyn Drawable>>, validation: Box<dyn ValidationFn>) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0.min(375.0), 375.0));
        let children = if offset == Offset::End { Scrollable::end(ContentChildren::new(children)) } else { Scrollable::new(ContentChildren::new(children)) };
        Content {
            layout: Stack::new(Offset::Center, offset, width, Size::Fill, Padding::default()),
            children,
            validation,
            last_input: None,
        }
    }

    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.children.1.inner().iter_mut().find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.children.1.inner().get_mut(i).and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) = self.children.1.inner().iter().position(|item| (**item).as_any().is::<T>()) {
            let boxed = self.children.1.inner().remove(pos);
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    pub fn children(&self) -> &Vec<Box<dyn Drawable>> {&self.children.1.1}
    pub fn children_mut(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.children.1.1}
}

impl Content {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let routed = if let Some(TextInputEvent::Set(data)) = event.downcast_ref::<TextInputEvent>() {
            Some(data.clone())
        } else if let Some(QRCodeScannedEvent(data)) = event.downcast_ref::<QRCodeScannedEvent>() {
            Some(data.clone())
        } else {
            None
        };

        if let Some(data) = routed {
            let inputs = self.children().iter().enumerate()
                .filter(|(_, c)| (***c).as_any().is::<TextInput>())
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            if inputs.is_empty() { return vec![event]; }

            let focused = inputs.iter().copied().find(|&i| {
                self.children()[i].downcast_ref::<TextInput>().is_some_and(|t| t.is_focused)
            });
            let target = focused
                .or(self.last_input.filter(|i| inputs.contains(i)))
                .unwrap_or(inputs[0]);

            if let Some(input) = self.children_mut()[target].downcast_mut::<TextInput>() {
                input.set_value(&data);
            }
            return Vec::new();
        }

        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(i) = self.children().iter().position(|c| c.downcast_ref::<TextInput>().is_some_and(|t| t.is_focused)) {
                self.last_input = Some(i);
            }

            let event = InterfaceEvent::Disable(!(self.validation)(ctx, self.children.1.inner().iter_mut().collect()));
            ctx.emit(event);
        } else if let Some(InterfaceEvent::FocusTextInput(true)) = event.downcast_ref::<InterfaceEvent>() {
            let mut total_height = 0.0;
            for c in self.children_mut().iter_mut() {
                if let Some(input) = c.downcast_mut::<TextInput>() && input.is_focused {
                    ctx.emit(ptsd::events::Scrollable::ScrollTo(total_height));
                    break;
                } else {
                    let size = c.size_range();
                    total_height += size.0.max_height;
                }
            }
        }
        vec![event]
    }
}

#[derive(Debug, Component, Clone)]
pub struct ContentChildren(Column, Vec<Box<dyn Drawable>>);
impl ContentChildren {
    pub fn new(children: Vec<Box<dyn Drawable>>) -> Self {
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::default(), None);
        ContentChildren(layout, children)
    }

    pub fn inner(&mut self) -> &mut Vec<Box<dyn Drawable>> {&mut self.1}
    pub fn column(&mut self) -> &mut Column {&mut self.0}
}

#[derive(Debug, Component, Clone)]
pub struct Header {
    layout: Row,
    pub left: HeaderIcon,
    pub center: Box<dyn Drawable>,
    pub right: HeaderIcon
}


impl Header {
    pub fn home(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        Self::_new(theme, title, None, icon, TextSize::H3)
    }

    pub fn stack(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        let closure = |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Pop);
        Self::_new(theme, title, Some((Icons::Left, Box::new(closure))), icon, TextSize::H4)
    }

    pub fn stack_end(theme: &Theme, title: &str) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Reset);
        Self::_new(theme, title, Some((Icons::Close, Box::new(closure))), None, TextSize::H4)
    }

    pub fn messaging(theme: &Theme, title: String, profiles: Vec<Profile>, exact_len: usize, info: Option<Box<dyn Callback>>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| (0..exact_len).for_each(|_| ctx.emit(NavigationEvent::Pop));
        let l_icon = HeaderIcon::new(theme, Icons::Left, closure);
        let r_icon = info.map(|i| HeaderIcon::new(theme, Icons::Info, i)).unwrap_or_default();

        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        Header {
            layout,
            left: l_icon,
            center: Box::new(MessageHeader::new(theme, title, profiles)),
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
        let text = ExpandableText::new(theme, title, size, TextStyle::Heading, Align::Center, Some(1));

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
impl MessageHeader {
    pub fn new(theme: &Theme, title: String, profiles: Vec<Profile>) -> Self {
        let text = ExpandableText::new(theme, &title, TextSize::H4, TextStyle::Heading, Align::Center, Some(1));
        let avatars = profiles.iter().map(|p| p.pfp.clone()).collect::<Vec<_>>();
        MessageHeader(Column::center(8.0), AvatarGroup::new(theme, avatars), text)
    }
}

#[derive(Debug, Component, Clone)]
pub struct HeaderIcon {
    layout: Stack,
    pub icon: Option<GhostIconButton>
}
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

impl Bumper {
    pub fn home(theme: &Theme, first: Option<(String, Box<dyn Callback>)>, second: Option<(String, Box<dyn Callback>)>) -> Self {
        let mut content: Vec<Box<dyn Drawable>> = vec![];
        if let Some((l, c)) = first {content.push(Box::new(PrimaryButton::new(theme, &l, c))); }
        if let Some((l, c)) = second { content.push(Box::new(PrimaryButton::new(theme, &l, c))); }
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    pub fn stack(
        theme: &Theme,
        label: Option<&str>,
        on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        secondary: Option<(String, Box<dyn Callback>)>,
    ) -> Self {
        let mut content = vec![Box::new(PrimaryButton::new(theme, label.unwrap_or("Continue"), Box::new(on_click))) as Box<dyn Drawable>];
        if let Some((l, c)) = secondary { content.push(Box::new(SecondaryButton::large(theme, &l, c))); }
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content) }
    }

    pub fn stack_end(theme: &Theme, exact_pages: Option<usize>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| match exact_pages {
            Some(num) => (0..num).for_each(|_| ctx.emit(NavigationEvent::Pop)),
            None => ctx.emit(NavigationEvent::Reset),
        };

        let content = SecondaryButton::large(theme, "Done", Box::new(closure));
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(vec![Box::new(content)]) }
    }

    pub fn input(theme: &Theme, placeholder: &str, on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl FnMut(&mut Context, &mut String) -> bool + Clone + 'static) -> Self {
        let bar = MessageBar::new(theme, placeholder, on_submit, valid);
        let (layout, background) = Self::layout(theme);
        let mut content = BumperContent::new(vec![Box::new(bar)]);
        content.layout.padding().3 = 8.0;
        Bumper { layout, background, content }
    }

    pub fn message_bar(&mut self) -> Option<&mut MessageBar> {
        self.content.children.first_mut().and_then(|child| child.downcast_mut::<MessageBar>())
    }

    pub fn alert(theme: &Theme, message: &str) -> Self {
        let (layout, background) = Self::layout(theme);
        let alert = Alert::new(theme, message);
        Bumper { layout, background, content: BumperContent::new(vec![Box::new(alert)]) }
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
impl BumperContent {
    fn new(children: Vec<Box<dyn Drawable>>) -> Self {
        let padding = if children.is_empty() {Padding::default()} else {Padding(0.0, 16.0, 0.0, 16.0)};
        BumperContent{ layout: Row::new(16.0, Offset::Center, Size::Fit, padding), children }
    }
}

#[derive(Debug, Clone)]
pub enum AdjustScrollEvent {
    Vertical(f32),
    Horizontal(f32),
}

impl prism::events::Event for AdjustScrollEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Clone)]
pub enum InterfaceEvent {
    Disable(bool),
    FocusTextInput(bool)
}

impl prism::events::Event for InterfaceEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Screen(Stack, Pages, Option<Bin<Stack, Rectangle>>);

impl Screen {
    fn on_event(&mut self, _ctx: &mut Context, mut event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {*v = vec![1];}

        if let Some(MobileKeyboardEvent::FullScreen(full)) = event.downcast_ref::<MobileKeyboardEvent>() {
            self.0.3 = if *full { Size::Static(0.0) } else { Size::Fit };
        }
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
