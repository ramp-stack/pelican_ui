use prism::drawable::{Component, Drawable, SizedTree};
use prism::Context;
use prism::event::{OnEvent, Event};
use prism::layout::{Area, Column, Offset, Padding, Size, Stack, Row};
use prism::display::{Bin, Opt, EitherOr, Enum};

use crate::Theme;
// use crate::pages::Error;
use crate::components::interface::system::MobileKeyboard;
use crate::components::interface::navigation::Navigator;
use crate::components::Rectangle;
use crate::components::interface::navigation::{AppPage, NavigationEvent, RootInfo};

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
        if let Some(old) = self.pages.right().replace(page) { 
            self.history.push(old);
        }
        self.pages.display_left(false);
    }

    pub fn pop(&mut self) {
        if self.pages.right().is_some() {
            match self.history.pop() {
                Some(page) => *self.pages.right() = Some(page),
                None => self.root(None)
            }
        }
    }

    // pub fn pages(&mut self) -> { &mut self.pages }

    pub fn _current(&mut self) -> &mut Box<dyn Drawable> {
        if !self.history.is_empty() || self.pages.right().is_some() {
            self.pages.right().as_mut().unwrap()
        } else {
            self.pages.left().drawable().inner()
        }
    }
}

#[derive(Component, Debug)]
pub enum Interface {
    Mobile {
        layout: Column,
        safe_area_left: Box<Bin<Stack, Rectangle>>,
        pages: Box<Pages>,
        keyboard: Box<Opt<MobileKeyboard>>,
        navigator: Box<Option<Opt<Navigator>>>,
        safe_area_right: Box<Bin<Stack, Rectangle>>,
    },

    Desktop {
        layout: Row, 
        navigator: Box<Option<Opt<Navigator>>>,
        separator: Bin<Stack, Rectangle>, 
        pages: Box<Pages> 
    },

    Web {
        layout: Column, 
        navigator: Box<Option<Opt<Navigator>>>, 
        pages: Box<Pages>
    }
}


impl OnEvent for Interface {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Interface::Mobile{keyboard, ..} = self {
            if event.downcast_mut::<NavigationEvent>().is_some() {
                keyboard.display(false);
            } else if let Some(ShowKeyboard(b)) = event.downcast_ref::<ShowKeyboard>() {
                keyboard.display(*b);
            }
        }

        if let Some(event) = event.downcast_mut::<NavigationEvent>() {
            match event {
                NavigationEvent::Pop => self.pages().pop(),
                NavigationEvent::Push(page) => self.pages().push(page.take().unwrap()),
                NavigationEvent::Reset => self.pages().root(None),
                NavigationEvent::Error(e) => panic!("ERROR {e}"), //self.pages().push(Box::new(Error::new(ctx, e.to_string()))),
                NavigationEvent::Root(r) => self.pages().root(Some(r.to_string())),
            }

            if prism::IS_MOBILE {
                let no_history = self.pages().history.is_empty();
                if let Some(navigator) = self.navigator() {
                    navigator.display(no_history);
                }
            }
        }

        vec![event]
    }
}

impl Interface {
    pub fn desktop(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let color = ctx.state.get_or_default::<Theme>().colors.outline.secondary;
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::desktop(ctx, navigation), true));
        let line_layout = Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default());
        let separator = Bin(line_layout, Rectangle::new(color, 0.0, None));
        Interface::Desktop {
            layout: Row::start(0.0), 
            navigator: Box::new(navigator), 
            separator, 
            pages: Box::new(Pages::new(pages))
        }
    }

    pub fn mobile(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::mobile(ctx, navigation), true));
        // let (_left, _right, top, bottom) = ctx.send(Request::Hardware(Hardware::SafeAreaInsets));
        let (top, bottom) = (18.0, 18.0);
        let layout = Column::new(0.0, Offset::Center, Size::Fit, Padding::default(), None);

        Interface::Mobile {
            layout,
            safe_area_left: Box::new(Spacer::new(ctx, top)),
            pages: Box::new(Pages::new(pages)),
            keyboard: Box::new(Opt::new(MobileKeyboard::new(ctx, true), false)),
            navigator: Box::new(navigator),
            safe_area_right: Box::new(Spacer::new(ctx, bottom))
        }
    }

    pub fn web(ctx: &mut Context, mut navigation: Vec<RootInfo>) -> Self {
        let pages: Vec<(String, Box<dyn Drawable>)> = navigation.iter_mut().map(|nav| (nav.label.to_string(), nav.page.take().unwrap() as Box<dyn Drawable>)).collect();
        let navigator = (navigation.len() > 1).then_some(Opt::new(Navigator::web(ctx, navigation), true));
        let layout = Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None);
        Interface::Web{layout, navigator: Box::new(navigator), pages: Box::new(Pages::new(pages))}
    }

    pub fn pages(&mut self) -> &mut Pages {
        match self {
            Interface::Desktop {pages, ..} => pages,
            Interface::Mobile {pages, ..} => pages,
            Interface::Web {pages, ..} => pages,
        }
    }

    pub fn navigator(&mut self) -> &mut Option<Opt<Navigator>> {
        match self {
            Interface::Desktop {navigator, ..} => navigator,
            Interface::Mobile {navigator, ..} => navigator,
            Interface::Web {navigator, ..} => navigator,
        }
    }
}

/// Event used to open or close keyboard.
#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

pub struct Spacer; 
impl Spacer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, height: f32) -> Bin<Stack, Rectangle> {
        let color = ctx.state.get_or_default::<Theme>().colors.background.primary;
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(height), Padding::default());
        Bin(layout, Rectangle::new(color, 0.0, None))
    }
}