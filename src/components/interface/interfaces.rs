use roost_ui::{Component, Context};
use roost_ui::events::{OnEvent, Event};
use roost_ui::drawable::Drawable;
use roost_ui::layouts::{Bin, Column, Offset, Opt, Padding, Row, Size, Stack, EitherOr, Enum};

use crate::pages::Error;
use crate::components::interface::system::MobileKeyboard;
use crate::components::interface::navigation::Navigator;
use crate::components::Rectangle;
use crate::components::interface::navigation::{AppPage, NavigationEvent, RootInfo};
use crate::plugin::PelicanUI;

#[derive(Debug, Component)]
pub struct Pages {
    layout: Stack,
    pages: EitherOr<Enum, Option<Box<dyn Drawable>>>,
    #[skip] history: Vec<Box<dyn Drawable>>
}

impl OnEvent for Pages {}

impl Pages {
    pub fn new(pages: Vec<(String, Box<dyn Drawable>)>) -> Self {
        let first = pages[0].0.to_string();
        let pages = Enum::new(pages, first);
        Pages {
            layout: Stack::default(),
            pages: EitherOr::new(pages, None),
            history: Vec::new(),
        }
    }

    pub fn root(&mut self, page: Option<String>) {
        self.pages.display_left(true);
        if let Some(p) = page { self.pages.left().display(&p); }
        self.history = vec![];
        *self.pages.right() = None;
    }

    pub fn push(&mut self, page: Box<dyn AppPage>) {
        println!("PUSHING PAGE");
        self.pages.display_left(false);
        if let Some(old) = self.pages.right().replace(page) { self.history.push(old); }
    }

    pub fn pop(&mut self) {
        if self.pages.right().is_some() {
            match self.history.pop() {
                Some(page) => *self.pages.right() = Some(page),
                None => self.root(None)
            }
        }
    }
}

#[derive(Component, Debug)]
pub enum Interface {
    Desktop(Box<InterfaceDesktop>),
    Mobile(Box<InterfaceMobile>),
    Web(Box<InterfaceWeb>)
}

impl OnEvent for Interface {
    fn on_event(&mut self, ctx: &mut Context, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Interface::Mobile(interface) = self {
            if event.downcast_mut::<NavigationEvent>().is_some() {
                interface.keyboard().display(false);
            } else if let Some(ShowKeyboard(b)) = event.downcast_ref::<ShowKeyboard>() {
                interface.keyboard().display(*b);
            }
        }

        if let Some(event) = event.downcast_mut::<NavigationEvent>() {
            let mut is_root = false;
            match event {
                NavigationEvent::Pop => self.pages().pop(),
                NavigationEvent::Push(page) => self.pages().push(page.take().unwrap()),
                NavigationEvent::Reset => self.pages().root(None),
                NavigationEvent::Error(e) => self.pages().push(Box::new(Error::new(ctx, e.to_string()))),
                NavigationEvent::Root(r) => {
                    is_root = true;
                    self.pages().root(Some(r.to_string()));
                }
            }

            if let Some(navigator) = self.navigator() {navigator.display(!(roost_ui::IS_MOBILE && is_root));}
        }

        vec![event]
    }
}

impl Interface {
    pub fn desktop(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Interface::Desktop(Box::new(InterfaceDesktop::new(ctx, navigation)))
    }

    pub fn mobile(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Interface::Mobile(Box::new(InterfaceMobile::new(ctx, navigation)))
    }

    pub fn web(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Interface::Web(Box::new(InterfaceWeb::new(ctx, navigation)))
    }

    pub fn pages(&mut self) -> &mut Pages {
        match self {
            Interface::Desktop(s) => s.pages(),
            Interface::Mobile(s) => s.pages(),
            Interface::Web(s) => s.pages(),
        }
    }

    pub fn navigator(&mut self) -> &mut Option<Opt<Navigator>> {
        match self {
            Interface::Desktop(s) => s.navigator(),
            Interface::Mobile(s) => s.navigator(),
            Interface::Web(s) => s.navigator(),
        }
    }
}

#[derive(Component, Debug)]
pub struct InterfaceMobile(Column, Pages, Opt<MobileKeyboard>, Option<Opt<Navigator>>);
impl OnEvent for InterfaceMobile {}

impl InterfaceMobile {
    pub fn new(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::mobile(ctx, navigation), true));
        let insets = ctx.hardware.safe_area_insets();
        let padding = Padding(insets.0, insets.2, insets.1, insets.3);
        let layout = Column::new(0.0, Offset::Center, Size::Fit, padding);
        InterfaceMobile(layout, Pages::new(pages), Opt::new(MobileKeyboard::new(ctx, true), false), navigator)
    }

    pub fn keyboard(&mut self) -> &mut Opt<MobileKeyboard> {&mut self.2}
    pub fn pages(&mut self) -> &mut Pages {&mut self.1}
    pub fn navigator(&mut self) -> &mut Option<Opt<Navigator>> {&mut self.3}
}

#[derive(Component, Debug)]
pub struct InterfaceDesktop(Row, Option<Opt<Navigator>>, Bin<Stack, Rectangle>, Pages);
impl OnEvent for InterfaceDesktop {}

impl InterfaceDesktop {
    pub fn new(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.outline.secondary;
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::desktop(ctx, navigation), true));
        let line_layout = Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default());
        let separator = Bin(line_layout, Rectangle::new(color, 0.0, None));
        InterfaceDesktop(Row::start(0.0), navigator, separator, Pages::new(pages))
    }

    pub fn pages(&mut self) -> &mut Pages {&mut self.3}
    pub fn navigator(&mut self) -> &mut Option<Opt<Navigator>> {&mut self.1}
}

#[derive(Component, Debug)]
pub struct InterfaceWeb(Column, Option<Opt<Navigator>>, Pages);
impl OnEvent for InterfaceWeb {}

impl InterfaceWeb {
    pub fn new(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::web(ctx, navigation), true));
        let layout = Column::new(0.0, Offset::Start, Size::Fill, Padding::default());
        InterfaceWeb(layout, navigator, Pages::new(pages))
    }


    pub fn pages(&mut self) -> &mut Pages {&mut self.2}
    pub fn navigator(&mut self) -> &mut Option<Opt<Navigator>> {&mut self.1}
}

/// Event used to open or close keyboard.
#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

