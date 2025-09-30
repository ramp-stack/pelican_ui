use mustache::events::{OnEvent, Event};
use mustache::drawable::{Color, Drawable, Component, Image};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::components::{Rectangle, AspectRatioImage};
use crate::events::{NavigatorSelect, NavigateEvent, NavigatorEvent};
use crate::layout::{Column, Stack, Bin, Row, Padding, Offset, Size};
use crate::components::avatar::AvatarContent;
use crate::components::interactions::ButtonState;
use crate::components::interface::general::NavigatorGhostButton;
use crate::plugin::PelicanUI;

use crate::utils::ElementID;
use crate::pages::AppPage;
use crate::pages::Error;

use std::fmt::Debug;
use crate::components::interface::general::{NavigationButton, NavigateInfo, PageBuilder};

#[derive(Component)]
pub struct DesktopInterface(Row, Option<DesktopNavigator>, Bin<Stack, Rectangle>, Option<Box<dyn AppPage>>, #[skip] Option<Vec<PageBuilder>>);

impl DesktopInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        mut navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>,
    ) -> Self {
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.outline.secondary;
        let pages = navigation.as_mut().map(|(_, a, b)| {
            let new: Vec<&mut NavigateInfo> = match b {
                Some(nav) => a.iter_mut().chain(nav.iter_mut()).collect(),
                None => a.iter_mut().collect(),
            };
            new.into_iter().map(|t| t.get_page.take().unwrap()).collect::<Vec<_>>()
        });

        let navigator = navigation.map(|n| DesktopNavigator::new(ctx, n));

        let line_layout = Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default());
        let separator = Bin(line_layout, Rectangle::new(color, 0.0, None));

        let layout = Row::new(0.0, Offset::Start, Size::Fit, Padding::default());
        DesktopInterface(layout, navigator, separator, Some(start_page), pages)
    }

    pub fn page(&mut self) -> &mut Option<Box<dyn AppPage>> { &mut self.3 }
    pub fn navigator(&mut self) -> &mut Option<DesktopNavigator> { &mut self.1 }
}

impl OnEvent for DesktopInterface {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigateEvent(index)) = event.downcast_mut::<NavigateEvent>() {
            self.3 = Some(self.3.take().unwrap().navigate(ctx, *index).unwrap_or_else(|e| {
                Box::new(Error::new(ctx, "404 Page Not Found", e))
            }));
        } else if let Some(NavigatorEvent(index)) = event.downcast_mut::<NavigatorEvent>() {
            if let Some(nav) = self.4.as_mut() { self.3 = Some(nav[*index](ctx)); }
        }
        true
    }
}

impl std::fmt::Debug for DesktopInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Desktop")
    }
}

#[derive(Debug, Component)]
pub struct DesktopNavigator(Column, Image, ButtonColumn, Option<Bin<Stack, Rectangle>>, Option<ButtonColumn>);

impl DesktopNavigator {
    pub fn new(ctx: &mut Context, navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)) -> Self {
        let (mut top_col, mut bot_col) = (Vec::new(), Vec::new());
        let mut i = 0;

        let spacer = navigation.2.as_ref().map(|_| {
            let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, widths[1].1));
            let spacer = Stack(Offset::Center, Offset::Center, width, Size::Fill, Padding::default());
            Bin(spacer, Rectangle::new(Color::TRANSPARENT, 0.0, None))
        });

        navigation.1.into_iter().for_each(|info| {
            top_col.push(Self::_new(ctx, i, navigation.0 == i, info));
            i += 1;
        });

        if let Some(n) = navigation.2 {
            n.into_iter().for_each(|info| {
                bot_col.push(Self::_new(ctx, i, navigation.0 == i, info));
                i += 1;
            });
        }

        let top_col = ButtonColumn::new(top_col);
        let bot_col = (!bot_col.is_empty()).then_some(ButtonColumn::new(bot_col));
        let wordmark = ctx.get::<PelicanUI>().get().0.theme().brand.wordmark.clone();
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, widths[1].1));

        DesktopNavigator(
            Column::new(32.0, Offset::Center, width, Padding(16.0, 32.0, 16.0, 32.0)),
            AspectRatioImage::new(wordmark, (100.0, 25.0)), top_col, spacer, bot_col
        )
    }

    fn _new(ctx: &mut Context, index: usize, selected: bool, info: NavigateInfo) -> NavigationButton {
        let id = ElementID::new();

        let closure = move |ctx: &mut Context| {
            ctx.trigger_event(NavigatorSelect(id));
            ctx.trigger_event(NavigatorEvent(index));
        };

        match info.avatar {
            Some(a) => NavigationButton::new(id, NavigatorGhostButton::desktop_avatar(ctx, a, &info.label, closure, selected)),
            None => NavigationButton::new(id, NavigatorGhostButton::desktop_icon(ctx, info.icon, &info.label, closure, selected))
        }
    }

    pub fn buttons(&mut self) -> Vec<&mut NavigatorGhostButton> {
        self.2.buttons().iter_mut().map(|nb| nb.inner()).collect()
    }
}

impl OnEvent for DesktopNavigator {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigatorSelect(id)) = event.downcast_ref::<NavigatorSelect>() {
            let mut buttons: Vec<&mut NavigationButton> = self.2.buttons().iter_mut().collect();
            if let Some(b) = self.4.as_mut() { buttons.extend(b.buttons().iter_mut()) }
            buttons.iter_mut().for_each(|b| {
                let is_selected = b.id() == *id;
                b.inner().inner().selected(is_selected);
            });
        }
        true
    }
}

#[derive(Debug, Component)]
struct ButtonColumn(Column, Vec<NavigationButton>);
impl OnEvent for ButtonColumn {}

impl ButtonColumn {
    fn new(buttons: Vec<NavigationButton>) -> Self {
        ButtonColumn(Column::center(8.0), buttons)
    }

    fn buttons(&mut self) -> &mut Vec<NavigationButton> {&mut self.1}
}
