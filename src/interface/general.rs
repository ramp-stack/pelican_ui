use prism::Context;
use prism::events::{DynEvent, Event, ChildEvent, OnChange, on_change};
use prism::events::{MouseEvent, WindowEvent, Timer, OnEvent, on_event};
use prism::drawable::{Drawable, Component};
use prism::drawable::text::Align;
use ptsd::display::Bin;
use ptsd::layout::{Column, Stack, Row, Padding, Offset, Size};
use ptsd::emitters::Scrollable;
use uuid::Uuid;
use ptsd::interfaces::{Body, Navigator as PTSDNavigator, ShowKeyboard};
use ptsd::navigation::{NavigationEvent, NavigationChanged, AppPage};
use ptsd::utils::ValidationFn;
pub use ptsd::navigation::Pages;

use std::time::Duration;

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::{Rectangle, ValidFn, Profile};
use crate::components::text::{TextStyle, TextSize, Text};
use crate::components::button::{GhostIconButton, PrimaryButton, SecondaryButton};
use crate::components::avatar::AvatarGroup;
use crate::interface::system::CustomKeyboard;
use crate::interface::message_bar::MessageBar;
use crate::interface::navigation::{RootInfo, Navigator};
use crate::components::Alert;

#[derive(Component, Debug, Clone)]
pub struct Interface {
    layout: Stack,
    inner: ptsd::interfaces::Interface,
}

impl Interface {
    pub fn new(ctx: &mut Context, theme: &Theme, mut roots: Vec<RootInfo>, keyboard: Option<Box<dyn Drawable>>) -> Self {
        ctx.background = theme.colors().get(ptsd::Background::Primary).into();
        let pages: Vec<(String, Box<dyn AppPage>)> = roots
            .iter_mut()
            .map(|r| (r.label.to_string(), r.page.take().unwrap() as Box<dyn AppPage>))
            .collect();
        Interface {
            layout: Stack::new(
                Offset::Start, Offset::Start, Size::Fill, Size::Fill, Padding::default(),
            ),
            inner: if ctx.window.is_mobile {
                let navigator = (pages.len() > 1)
                    .then_some(Box::new(Navigator::mobile(theme, roots)) as Box<dyn PTSDNavigator>);
                ptsd::interfaces::Interface::mobile(
                    navigator, Screen::mobile(Pages::new(pages)), keyboard,
                )
            } else {
                let navigator =
                    (pages.len() > 1).then_some(
                        Box::new(Navigator::desktop(theme, roots)) as Box<dyn PTSDNavigator>
                    );
                ptsd::interfaces::Interface::desktop(
                    navigator, Screen::desktop(theme, Pages::new(pages)),
                )
            },
        }
    }

    pub fn web(ctx: &mut Context, theme: &Theme, mut roots: Vec<RootInfo>) -> Self {
        ctx.background = theme.colors().get(ptsd::Background::Primary).into();
        let pages = roots
            .iter_mut()
            .map(|root| (root.label.to_string(), root.page.take().unwrap() as Box<dyn AppPage>))
            .collect();
        let navigator = (roots.len() > 1)
            .then_some(Box::new(Navigator::web(theme, roots)) as Box<dyn PTSDNavigator>);
        Self {
            layout: Stack::new(
                Offset::Start, Offset::Start, Size::Fill, Size::Fill, Padding::default(),
            ),
            inner: ptsd::interfaces::Interface::web(navigator, Screen::web(Pages::new(pages))),
        }
    }

    fn custom_keyboard(&mut self) -> Option<(&mut CustomKeyboard, bool)> {
        let ptsd::interfaces::Interface::Mobile {keyboard: Some(keyboard), ..} = &mut self.inner
            else {return None};
        let showing = keyboard.is_showing();
        keyboard.inner().downcast_mut::<CustomKeyboard>().map(|keyboard| (keyboard, showing))
    }
}

#[on_event]
impl OnEvent<MouseEvent> for Interface {
    fn on(&mut self, ctx: &mut Context, event: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        if let Some((keyboard, showing)) = self.custom_keyboard() {
            keyboard.observe_mouse(ctx.window, event.clone(), showing);
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<WindowEvent> for Interface {
    fn on(&mut self, _ctx: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if let Some((keyboard, showing)) = self.custom_keyboard() {
            keyboard.on_window_event(event, showing);
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<NavigationEvent> for Interface {
    fn on(&mut self, _ctx: &mut Context, event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        if let Some((keyboard, _)) = self.custom_keyboard() {keyboard.reset();}
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<ShowKeyboard> for Interface {
    fn on(&mut self, _ctx: &mut Context, event: ShowKeyboard) -> Vec<Box<dyn DynEvent>> {
        if let Some((keyboard, _)) = self.custom_keyboard() {keyboard.reset();}
        prism::events![event]
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
            layout: Column::new(12.0, Offset::Center, Size::Fill, Padding::default(), None),
            header, content, bumper,
        }
    }
}

#[on_event]
impl OnEvent<FormEvent> for Page {
    fn on(&mut self, _ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Validated { id, valid } = event
            && id == self.content.id()
            && let Some(bumper) = &mut self.bumper { bumper.disable(!valid); }
        prism::events![event]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FormEvent {
    Refresh(Uuid), Bind(Uuid), Changed(Uuid),
    Validated { id: Uuid, valid: bool },
}
impl Event for FormEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(*self); }
    }
}

/// Scrollable form content that reveals the focused field.
#[derive(Debug, Clone)]
pub struct Content {
    inner: ContentInner,
    focused: Option<Uuid>,
    last_size: (f32, f32),
    top: std::cell::Cell<f32>,
    last_top: f32,
    reveal: bool,
}

impl Content {
    pub fn new(offset: Offset, children: Vec<Box<dyn Drawable>>, validation: Box<dyn ValidationFn>) -> Self {
        Self {
            inner: ContentInner::new(offset, children, validation),
            focused: None, last_size: (0.0, 0.0), top: std::cell::Cell::new(0.0), last_top: 0.0,
            reveal: false,
        }
    }

    fn focused_input(&self, layout: &prism::drawable::LayoutTree) -> Option<(Uuid, f32, f32)> {
        let (scroll_offset, scroll_tree) = layout.1.first()?;
        let (body_offset, body_tree) = scroll_tree.1.first()?;
        let offset = (scroll_offset.0 + body_offset.0, scroll_offset.1 + body_offset.1);
        // Include off-screen inputs so programmatic focus can reveal them too.
        self.inner.children.1.draw(body_tree, offset, (-1e6, -1e6, 2e6, 2e6))
            .into_iter().find_map(|instruction| match instruction.item {
                prism::drawable::Item::Text(text) if text.focused && text.cursor.is_some() => {
                    Some((text.id, instruction.offset.1, text.size().1))
                }
                _ => None,
            })
    }
}

impl std::ops::Deref for Content {
    type Target = ContentInner;
    fn deref(&self) -> &Self::Target {&self.inner}
}
impl std::ops::DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.inner}
}

impl Drawable for Content {
    fn request_width(&mut self) -> (f32, f32) {self.inner.request_width()}
    fn request_height(&mut self, width: f32) -> (f32, f32) {self.inner.request_height(width)}

    fn layout(&mut self, size: (f32, f32)) -> prism::drawable::LayoutTree {
        let layout = Drawable::layout(&mut self.inner, size);
        let id = self.focused_input(&layout).map(|(id, ..)| id);
        let reposition = id != self.focused || size != self.last_size || self.top.get() != self.last_top;
        self.focused = id;
        self.last_size = size;
        self.last_top = self.top.get();
        if reposition {self.reveal = id.is_some();}
        layout
    }

    fn draw(&self, layout: &prism::drawable::LayoutTree, offset: (f32, f32), bound: prism::drawable::Rect) -> Vec<prism::drawable::Instruction> {
        self.top.set(offset.1);
        self.inner.draw(layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &prism::drawable::LayoutTree, event: Box<dyn DynEvent>) {
        if let Some(RevealFocusedInput(id)) = event.downcast_ref() && *id == self.id() {
            if let Some((_, y, height)) = self.focused_input(layout) {
                let target = (128.0 - self.top.get())
                    .clamp(16.0, (layout.0.1 - height - 16.0).max(16.0));
                self.event(ctx, layout, Box::new(AdjustScrollEvent::Vertical(y - target)));
            }
            return;
        }
        if let Some(AdjustScrollEvent::Vertical(delta)) = event.downcast_ref() {
            // The scroll layout clamps this to the real content bounds. Never
            // enlarge the content just to reach the preferred focus position.
            if delta.is_finite() {self.inner.children.adjust_scroll(*delta);}
            return;
        }
        self.inner.event(ctx, layout, event);
    }
    fn changed(&mut self) -> prism::events::ListenFuture<'_> {
        Box::pin(async move {
            if self.reveal {
                return Box::new(|target: &mut dyn std::any::Any, ctx: &mut Context| {
                    let target = target.downcast_mut::<Self>().unwrap();
                    target.reveal = false;
                    // Scope focus adjustments to this form, rather than
                    // scrolling every content area receiving a global event.
                    ctx.emit(RevealFocusedInput(target.id()));
                }) as prism::events::ListenCallback;
            }
            let callback = self.inner.changed().await;
            Box::new(move |target: &mut dyn std::any::Any, ctx: &mut Context| {
                let target = target.downcast_mut::<Self>().unwrap();
                callback(&mut target.inner, ctx);
            }) as prism::events::ListenCallback
        })
    }
}

#[derive(Debug, Clone)]
struct RevealFocusedInput(Uuid);

impl Event for RevealFocusedInput {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

/// Public as the dereference target of `Content`.
#[derive(Debug, Component, Clone)]
pub struct ContentInner {
    layout: Stack,
    pub children: Scrollable<ContentChildren>,
    #[skip]
    validation: Box<dyn ValidationFn>,
    #[skip]
    id: Uuid,
    #[skip]
    valid: Option<bool>,
    #[skip]
    dirty: Timer,
}

impl ContentInner {
    pub fn new(offset: Offset, children: Vec<Box<dyn Drawable>>, validation: Box<dyn ValidationFn>) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(375.0), 375.0));
        let children = if offset == Offset::End {
            Scrollable::end(ContentChildren::new(children))
        } else {
            Scrollable::new(ContentChildren::new(children))
        };
        ContentInner {
            layout: Stack::new(
                Offset::Center, offset, width, Size::Fill, Padding(24.0, 0.0, 24.0, 0.0),
            ), children, validation,
            id: Uuid::new_v4(),
            valid: None,
            dirty: Timer::new(Duration::ZERO),
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn refresh(&self, ctx: &mut Context) { ctx.emit(FormEvent::Refresh(self.id)); }

    fn invalidate(&mut self) { self.dirty.reset(Duration::ZERO); }

    pub fn find<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.invalidate();
        self.children
            .1
            .inner()
            .iter_mut()
            .find_map(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    pub fn find_at<T: std::any::Any>(&mut self, i: usize) -> Option<&mut T> {
        self.invalidate();
        self.children
            .1
            .inner()
            .get_mut(i)
            .and_then(|item| (**item).as_any_mut().downcast_mut::<T>())
    }

    pub fn remove<T: std::any::Any>(&mut self) -> Option<T> {
        if let Some(pos) =
            self.children.1.inner().iter().position(|item| (**item).as_any().is::<T>()) {
            let boxed = self.children.1.inner().remove(pos);
            self.invalidate();
            boxed.into_any().downcast::<T>().ok().map(|b| *b)
        } else {
            None
        }
    }

    pub fn children(&self) -> &Vec<Box<dyn Drawable>> { &self.children.1.1 }
    pub fn children_mut(&mut self) -> &mut Vec<Box<dyn Drawable>> {
        self.invalidate();
        &mut self.children.1.1
    }
}

#[on_event]
impl OnEvent<FormEvent> for ContentInner {
    fn on(&mut self, ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Refresh(id) = event && id == self.id {
            self.dirty.cancel();
            let valid = self.validation.validate(ctx, self.children.1.inner().iter_mut().collect());
            ctx.emit(FormEvent::Changed(self.id));
            if self.valid != Some(valid) {
                self.valid = Some(valid);
                ctx.emit(FormEvent::Validated { id: self.id, valid });
            }
            return prism::events![FormEvent::Bind(self.id)];
        } else if matches!(event, FormEvent::Bind(_)) {
            // A nested form owns its controls independently of the surrounding form.
            return vec![];
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<NavigationChanged> for ContentInner {
    fn on(&mut self, _ctx: &mut Context, event: NavigationChanged) -> Vec<Box<dyn DynEvent>> {
        self.invalidate();
        prism::events![event]
    }
}

#[on_change(self.dirty.next())]
impl OnChange<()> for ContentInner {
    fn on_change(&mut self, ctx: &mut Context, _: ()) { self.refresh(ctx); }
}
#[on_change(async {
    if self.validation.change().await.is_none() {std::future::pending::<()>().await;}
})]
impl OnChange<()> for ContentInner {
    fn on_change(&mut self, ctx: &mut Context, _: ()) { self.refresh(ctx); }
}

#[derive(Debug, Component, Clone)]
pub struct ContentChildren(Column, Vec<Box<dyn Drawable>>);
impl ContentChildren {
    pub fn new(children: Vec<Box<dyn Drawable>>) -> Self {
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::default(), None);
        ContentChildren(layout, children)
    }

    pub fn inner(&mut self) -> &mut Vec<Box<dyn Drawable>> { &mut self.1 }
    pub fn column(&mut self) -> &mut Column { &mut self.0 }
}

#[derive(Debug, Component, Clone)]
pub struct Header {
    layout: Row,
    pub left: HeaderIcon,
    pub center: Box<dyn Drawable>,
    pub right: HeaderIcon,
}

impl Header {
    pub fn home(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        let text = Text::new(
            theme, title, TextSize::H3, TextStyle::Heading, Align::Center, Some(1),
        );
        Self::new(theme, text, None, icon)
    }

    pub fn stack(theme: &Theme, title: &str, icon: Option<(Icons, Box<dyn Callback>)>) -> Self {
        let closure = |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Pop);
        let text = Text::new(
            theme, title, TextSize::H4, TextStyle::Heading, Align::Center, Some(1),
        );
        Self::new(theme, text, Some((Icons::Left, Box::new(closure))), icon)
    }

    pub fn stack_end(theme: &Theme, title: &str) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::Reset);
        let text = Text::new(
            theme, title, TextSize::H4, TextStyle::Heading, Align::Center, Some(1),
        );
        Self::new(theme, text, Some((Icons::Close, Box::new(closure))), None)
    }

    pub fn messaging(theme: &Theme, title: String, profiles: Vec<Profile>, exact_len: usize, info: Option<Box<dyn Callback>>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| {
            (0..exact_len).for_each(|_| ctx.emit(NavigationEvent::Pop))
        };
        Self::new(
            theme, MessageHeader::new(theme, title, profiles),
            Some((Icons::Left, Box::new(closure))), info.map(|action| (Icons::Info, action)),
        )
    }

    fn new(theme: &Theme, center: impl Drawable, left: Option<(Icons, Box<dyn Callback>)>, right: Option<(Icons, Box<dyn Callback>)>) -> Self {
        let height =
            Size::custom(|heights| (heights[1].0.max(48.0) + 32.0, heights[1].1.max(48.0) + 32.0));
        Header {
            layout: Row::new(0.0, Offset::Center, height, Padding::default()),
            left: HeaderIcon::new(theme, left, Padding(24.0, 16.0, 16.0, 16.0)),
            center: Box::new(center),
            right: HeaderIcon::new(theme, right, Padding(16.0, 16.0, 24.0, 16.0)),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageHeader(Column, AvatarGroup, Text);
impl MessageHeader {
    pub fn new(theme: &Theme, title: String, profiles: Vec<Profile>) -> Self {
        let text = Text::new(
            theme, &title, TextSize::H4, TextStyle::Heading, Align::Center, Some(1),
        );
        let avatars = profiles.iter().map(|p| p.pfp.clone()).collect::<Vec<_>>();
        MessageHeader(Column::center(8.0), AvatarGroup::new(theme, avatars), text)
    }
}

#[derive(Debug, Component, Clone)]
pub struct HeaderIcon {
    layout: Stack,
    pub icon: Option<GhostIconButton>,
}
impl HeaderIcon {
    pub fn new(theme: &Theme, icon: Option<(Icons, Box<dyn Callback>)>, padding: Padding) -> Self {
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(48.0), Size::custom(|_| (48.0, f32::MAX)),
            padding,
        );
        let icon =
            icon.map(|(icon, action)| GhostIconButton::new(theme, icon, layout.clone(), action));
        Self { layout: if icon.is_some() { Stack::default() } else { layout }, icon }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Bumper {
    layout: Stack,
    background: Bin<Stack, Rectangle>,
    content: BumperContent,
}

impl Bumper {
    pub fn home(theme: &Theme, first: Option<(String, Box<dyn Callback>)>, second: Option<(String, Box<dyn Callback>)>) -> Self {
        let actions: Vec<_> = [first, second].into_iter().flatten().collect();
        let count = actions.len();
        let content = actions
            .into_iter()
            .enumerate()
            .map(|(index, (label, action))| {
                Box::new(PrimaryButton::new(
                    theme, &label, Self::button_layout(index, count), action,
                )) as Box<dyn Drawable>
            })
            .collect();
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content, Padding::default()) }
    }

    pub fn stack(
        theme: &Theme, label: Option<&str>,
        on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        secondary: Option<(String, Box<dyn Callback>)>,
    ) -> Self {
        let count = if secondary.is_some() { 2 } else { 1 };
        let mut content = vec![Box::new(PrimaryButton::new(
            theme, label.unwrap_or("Continue"), Self::button_layout(0, count), on_click,
        )) as Box<dyn Drawable>];
        if let Some((label, action)) = secondary {
            content.push(Box::new(SecondaryButton::large(
                theme, &label, Self::button_layout(1, count), action,
            )));
        }
        let (layout, background) = Self::layout(theme);
        Bumper { layout, background, content: BumperContent::new(content, Padding::default()) }
    }

    pub fn stack_end(theme: &Theme, exact_pages: Option<usize>) -> Self {
        let closure = move |ctx: &mut Context, _: &Theme| match exact_pages {
            Some(num) => (0..num).for_each(|_| ctx.emit(NavigationEvent::Pop)),
            None => ctx.emit(NavigationEvent::Reset),
        };

        let content = SecondaryButton::large(theme, "Done", Self::button_layout(0, 1), closure);
        let (layout, background) = Self::layout(theme);
        Bumper {
            layout, background,
            content: BumperContent::new(vec![Box::new(content)], Padding::default()),
        }
    }

    pub fn input(theme: &Theme, placeholder: &str, on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl ValidFn) -> Self {
        let padding = Padding(24.0, 16.0, 24.0, 8.0);
        let bar = MessageBar::new(theme, placeholder, padding.3, on_submit, valid);
        let (layout, background) = Self::layout(theme);
        let content = BumperContent::new(vec![Box::new(bar)], padding);
        Bumper { layout, background, content }
    }

    pub fn message_bar(&mut self) -> Option<&mut MessageBar> {
        self.content.children.first_mut().and_then(|child| child.downcast_mut::<MessageBar>())
    }

    pub fn alert(theme: &Theme, message: &str) -> Self {
        let (layout, background) = Self::layout(theme);
        let alert = Alert::new(theme, message);
        Bumper {
            layout, background,
            content: BumperContent::new(vec![Box::new(alert)], Padding(24.0, 16.0, 24.0, 16.0)),
        }
    }

    fn button_layout(index: usize, count: usize) -> Stack {
        let padding = Padding(
            if index == 0 { 24.0 } else { 8.0 },
            16.0,
            if index + 1 == count { 24.0 } else { 8.0 },
            16.0,
        );
        Stack::new(Offset::Center, Offset::Center, Size::Fit, Size::Fit, padding)
    }

    fn layout(theme: &Theme) -> (Stack, Bin<Stack, Rectangle>) {
        let background = Bin(
            Stack::new(
                Offset::Center, Offset::Center, Size::Fill, Size::Fill,
                Padding(24.0, 0.0, 24.0, 0.0),
            ), Rectangle::new(theme.colors().get(ptsd::Background::Primary), 0.0, None),
        );
        let width = Size::custom(|widths| (widths[0].0.min(423.0), 423.0));
        let height = Size::custom(|heights| heights[1]);
        let layout = Stack::new(Offset::Center, Offset::Start, width, height, Padding::default());
        (layout, background)
    }

    pub fn on_click(&mut self) -> Vec<&mut Box<dyn ptsd::utils::Callback>> {
        self.content
            .children
            .iter_mut()
            .filter_map(|child| child.downcast_mut::<PrimaryButton>().map(|b| b.1.on_click()))
            .collect::<Vec<_>>()
    }

    pub fn disable(&mut self, disable: bool) {
        for child in &mut self.content.children {
            if let Some(button) = child.downcast_mut::<PrimaryButton>() {
                button.1.disable(disable);
            }
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::home(
            theme,
            Some((
                "Press me".to_string(),
                Box::new(|_: &mut Context, _: &Theme| println!("Pressed....")),
            )),
            Some((
                "No Press me".to_string(),
                Box::new(|_: &mut Context, _: &Theme| println!("Pressed....")),
            )),
        )
    }
}

#[derive(Debug, Component, Clone)]
pub struct BumperContent {
    layout: Row,
    children: Vec<Box<dyn Drawable>>,
}
impl BumperContent {
    fn new(children: Vec<Box<dyn Drawable>>, padding: Padding) -> Self {
        BumperContent { layout: Row::new(0.0, Offset::Center, Size::Fit, padding), children }
    }
}

#[derive(Debug, Clone)]
pub enum AdjustScrollEvent { Vertical(f32), Horizontal(f32) }

impl prism::events::Event for AdjustScrollEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Clone)]
pub enum InterfaceEvent { FocusTextInput(bool) }

impl prism::events::Event for InterfaceEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Component, Clone)]
pub struct Screen(Stack, Pages, Option<Bin<Stack, Rectangle>>);

impl Screen {
    pub fn desktop(theme: &Theme, pages: Pages) -> Self {
        let color = theme.colors().get(ptsd::Outline::Secondary);
        let line_layout = Stack::new(
            Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default(),
        );
        let border = Bin(line_layout, Rectangle::new(color, 0.0, None));

        Screen(Stack::default(), pages, Some(border))
    }

    pub fn mobile(pages: Pages) -> Self { Screen(Stack::default(), pages, None) }

    pub fn web(pages: Pages) -> Self { Screen(Stack::default(), pages, None) }
}

#[on_event]
impl OnEvent<NavigationEvent> for Screen {
    fn on(&mut self, _ctx: &mut Context, mut event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        if let NavigationEvent::Push(_, excluded) = &mut event {*excluded = vec![1];}
        prism::events![event]
    }
}

impl Body for Screen {
    fn pages(&mut self) -> &mut Pages { &mut self.1 }
}
