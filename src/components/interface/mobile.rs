use mustache::events::{OnEvent, Event};
use mustache::drawable::{Drawable, Component};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::events::{KeyboardActiveEvent, NavigatorSelect, NavigateEvent, NavigatorEvent};
use crate::layout::{Column, Row, Padding, Offset, Size, Opt, Stack, Bin};
use crate::components::Rectangle;
use crate::utils::ElementID;
use crate::pages::AppPage;
use crate::pages::Error;
use crate::plugin::PelicanUI;
use crate::components::interactions::ButtonState;
use crate::components::interface::general::{NavigationButton, NavigateInfo, PageBuilder};
use crate::components::interface::system::MobileKeyboard;
use crate::components::interface::general::NavigatorGhostButton;

#[derive(Component)]
pub struct MobileInterface(Column, Option<Box<dyn AppPage>>, Option<MobileKeyboard>, Option<Opt<MobileNavigator>>, #[skip] Option<Vec<PageBuilder>>);

impl MobileInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        mut navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>
    ) -> Self {
        let pages = navigation.as_mut().map(|(_, a, b)| {
            let new: Vec<&mut NavigateInfo> = match b {
                Some(nav) => a.iter_mut().chain(nav.iter_mut()).collect(),
                None => a.iter_mut().collect(),
            };
            new.into_iter().map(|t| t.get_page.take().unwrap()).collect::<Vec<_>>()
        });

        let navigator = navigation.map(|n| Opt::new(MobileNavigator::new(ctx, n), true));
        let insets = ctx.hardware.safe_area_insets();
        let insets = Padding(insets.0, insets.2, insets.1, insets.3);
        let layout = Column::new(0.0, Offset::Center, Size::Fit, insets);
        MobileInterface(layout, Some(start_page), None, navigator, pages)
    }

    pub fn page(&mut self) -> &mut Option<Box<dyn AppPage>> { &mut self.1 }
    pub fn navigator(&mut self) -> &mut Option<Opt<MobileNavigator>> { &mut self.3 }
}

impl OnEvent for MobileInterface {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigateEvent(index)) = event.downcast_mut::<NavigateEvent>() {
            self.2 = None;
            self.1 = Some(self.1.take().unwrap().navigate(ctx, *index).unwrap_or_else(|e| {
                Box::new(Error::new(ctx, "404 Page Not Found", e))
            }));
            if let Some(navigator) = &mut self.3 {navigator.display(self.1.as_ref().map(|s| s.has_nav()).unwrap_or(false));}
        } else if let Some(NavigatorEvent(index)) = event.downcast_mut::<NavigatorEvent>() {
            self.3 = None;
            if let Some(nav) = self.4.as_mut() { self.1 = Some(nav[*index](ctx)); }
        } else if let Some(KeyboardActiveEvent(keyboard)) = event.downcast_ref::<KeyboardActiveEvent>() {
            match keyboard {
                Some(_) if self.2.is_some() => {},
                Some(a) => self.2 = Some(MobileKeyboard::new(ctx, *a)),
                None => self.2 = None
            }
        }
        true
    }
}

impl std::fmt::Debug for MobileInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mobile")
    }
}

#[derive(Debug, Component)]
pub struct MobileNavigator(Stack, Rectangle, MobileNavigatorContent);

impl MobileNavigator {
    pub fn new(ctx: &mut Context, navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)) -> Self {
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;

        MobileNavigator(
            Stack(Offset::Center, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(background, 0.0, None),
            MobileNavigatorContent::new(ctx, navigation)
        )
    }

    pub fn buttons(&mut self) -> Vec<&mut NavigatorGhostButton> {self.2.buttons()}
}

impl OnEvent for MobileNavigator {}

#[derive(Debug, Component)]
struct MobileNavigatorContent(Row, Vec<NavigationButton>);

impl MobileNavigatorContent {
    fn new(ctx: &mut Context, mut navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)) -> Self {
        let mut tabs = Vec::new();
        if let Some(n) = navigation.2 { navigation.1.extend(n); }
        for (i, info) in navigation.1.into_iter().enumerate() {
            let id = ElementID::new();
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(NavigatorSelect(id));
                ctx.trigger_event(NavigatorEvent(i));
            };

            let button = NavigatorGhostButton::mobile(ctx, info.icon, closure, navigation.0 == i);
            tabs.push(NavigationButton::new(id, button));
        }

        let layout = Row::new(48.0, Offset::Center, Size::Fit, Padding(0.0, 8.0, 0.0, 8.0));
        MobileNavigatorContent(layout, tabs)
    }

    fn buttons(&mut self) -> Vec<&mut NavigatorGhostButton> {
        self.1.iter_mut().map(|nb| nb.inner()).collect::<Vec<_>>()
    }
}


impl OnEvent for MobileNavigatorContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigatorSelect(id)) = event.downcast_ref::<NavigatorSelect>() {
            self.1.iter_mut().for_each(|b| {
                let is_selected = b.id() == *id;
                b.inner().inner().selected(is_selected);
            });
        }
        true
    }
}