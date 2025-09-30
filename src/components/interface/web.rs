use mustache::events::{OnEvent, Event};
use mustache::drawable::{Color, Drawable, Component, Image};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::components::{Rectangle, AspectRatioImage};
use crate::events::{NavigatorSelect, NavigateEvent, NavigatorEvent};
use crate::layout::{Column, Stack, Bin, Row, Padding, Offset, Size};
use crate::components::interface::general::NavigatorGhostButton;
use crate::components::button::GhostIconButton;
use crate::utils::ElementID;
use crate::pages::AppPage;
use crate::pages::Error;
use crate::plugin::PelicanUI;

use std::fmt::Debug;
use crate::components::interface::general::{NavigationButton, NavigateInfo, PageBuilder};

#[derive(Component)]
pub struct WebInterface(Column, Option<WebNavigator>, Option<Box<dyn AppPage>>, Option<WebFooter>, #[skip] Option<Vec<PageBuilder>>);

impl WebInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        mut navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>,
        socials: Option<Vec<(&'static str, String)>>
    ) -> Self {
        // let color = ctx.theme.colors.outline.secondary;
        let pages = navigation.as_mut().map(|(_, a, b)| {
            let new: Vec<&mut NavigateInfo> = match b {
                Some(nav) => a.iter_mut().chain(nav.iter_mut()).collect(),
                None => a.iter_mut().collect(),
            };
            new.into_iter().map(|t| t.get_page.take().unwrap()).collect::<Vec<_>>()
        });
        let navigator = navigation.map(|n| WebNavigator::new(ctx, n));
        let footer = socials.map(|s| WebFooter::new(ctx, s)); 

        WebInterface(
            Column::new(0.0, Offset::Start, Size::Fill, Padding::default()),
            navigator,
            Some(start_page),
            footer,
            pages,
        )
    }

    pub fn page(&mut self) -> &mut Option<Box<dyn AppPage>> { &mut self.2 }
    pub fn navigator(&mut self) -> &mut Option<WebNavigator> { &mut self.1 }
}

impl OnEvent for WebInterface {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigateEvent(index)) = event.downcast_mut::<NavigateEvent>() {
            self.2 = match self.2.take().unwrap().navigate(ctx, *index) {
                Ok(p) => Some(p),
                Err(e) => Some(Box::new(Error::new(ctx, "404 Page Not Found", e)))
            };
        } else if let Some(NavigatorEvent(index)) = event.downcast_mut::<NavigatorEvent>() {
            if let Some(nav) = self.4.as_mut() { self.2 = Some(nav[*index](ctx)); }
        }
        true
    }
}

impl std::fmt::Debug for WebInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Desktop")
    }
}


#[derive(Debug, Component)]
pub struct WebNavigator(Row, Image, Bin<Stack, Rectangle>, ButtonRow);

impl WebNavigator {
    pub fn new(
        ctx: &mut Context, 
        mut navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>),
    ) -> Self {
        let mut buttons = Vec::new();

        if let Some(n) = navigation.2 { navigation.1.extend(n); }
        for (index, info) in navigation.1.into_iter().enumerate() {
            let id = ElementID::new();
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(NavigatorSelect(id));
                ctx.trigger_event(NavigatorEvent(index));
            };

            let button = NavigatorGhostButton::desktop_icon(ctx, info.icon, &info.label, closure, navigation.0 == index);
            buttons.push(NavigationButton::new(id, button));
        }

        let wordmark = ctx.get::<PelicanUI>().get().0.theme().brand.wordmark.clone();

        WebNavigator(
            Row::new(32.0, Offset::Center, Size::Fit, Padding::new(48.0)),
            AspectRatioImage::new(wordmark, (150.0, 35.0)),
            Bin (
                Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(5.0), Padding::default()), 
                Rectangle::new(Color::TRANSPARENT, 0.0, None)
            ),
            ButtonRow::new(buttons)
        )
    }

    // pub fn update_avatar(&mut self, avatar_content: AvatarContent) {
    //     if let Some(avatar) = self.avatar() {
    //         if avatar.avatar().image().is_none() {
    //             avatar.set_content(avatar_content)
    //         } else if let AvatarContent::Image(ref image) = avatar_content {
    //             if avatar.avatar().image().as_ref().unwrap().image != *image {
    //                 avatar.set_content(avatar_content)
    //             }
    //         }
    //     };
    // }

    // pub fn update_username(&mut self, username: String) {
    //     self.4.buttons()[0].button().as_mut().unwrap().label().as_mut().unwrap().text().spans[0].text = username;
    // }

    // pub fn avatar(&mut self) -> Option<&mut Avatar> {
    //     self.4.buttons().iter_mut().flat_map(|nb| nb.button()).flat_map(|button| button.avatar()).next()
    // }

    pub fn buttons(&mut self) -> Vec<&mut NavigatorGhostButton> {
        self.3.buttons().iter_mut().map(|nb| nb.inner()).collect()
    }
}

impl OnEvent for WebNavigator {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigatorSelect(id)) = event.downcast_ref::<NavigatorSelect>() {
            let mut buttons: Vec<&mut NavigationButton> = self.3.buttons().iter_mut().collect();
            // buttons.extend(self.4.buttons().iter_mut());
            buttons.iter_mut().for_each(|b| {
                let is_selected = b.id() == *id;
                b.inner().inner().selected(is_selected)
            });
        }
        true
    }
}

#[derive(Debug, Component)]
struct ButtonRow(Row, Vec<NavigationButton>);
impl OnEvent for ButtonRow {}

impl ButtonRow {
    fn new(buttons: Vec<NavigationButton>) -> Self {
        ButtonRow(Row::center(8.0), buttons)
    }

    fn buttons(&mut self) -> &mut Vec<NavigationButton> {&mut self.1}
}

#[derive(Debug, Component)]
struct WebFooter(Row, Vec<GhostIconButton>);
impl OnEvent for WebFooter {}

impl WebFooter {
    fn new(
        ctx: &mut Context, 
        socials: Vec<(&'static str, String)>
    ) -> Self {
        let buttons = socials.into_iter().map(|(i, _)| GhostIconButton::new(ctx, i, |_: &mut Context| {})).collect();

        // let wordmark = ctx.theme.brand.wordmark.clone();
        // let white = ctx.theme.colors.shades.white;
        // let mut logo = AspectRatioImage::new(wordmark, (150.0, 35.0));
        // logo.color = Some(white);

        let layout = Row::new(8.0, Offset::Center, Size::Fit, Padding::new(48.0));
        WebFooter(layout, buttons)
    }

    // fn buttons(&mut self) -> Vec<&mut Button> {
    //     self.2.buttons().iter_mut().flat_map(|nb| nb.button()).collect::<Vec<_>>()
    // }
}

