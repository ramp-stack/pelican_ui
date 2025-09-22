use pelican_ui::events::{OnEvent, Event};
use pelican_ui::drawable::{Color, Drawable, Component, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::components::{Rectangle, AspectRatioImage};
use crate::events::{NavigatorSelect, NavigateEvent, NavigatorEvent};
use crate::layout::{Column, Stack, Bin, Row, Padding, Offset, Size};
use crate::components::button::{Button, ButtonState};
use crate::components::avatar::{Avatar, AvatarContent};
use crate::utils::ElementID;
use crate::pages::AppPage;
use crate::pages::Error;

use std::fmt::Debug;
use crate::components::interface::general::{NavigationButton, NavigateInfo, PageBuilder};

#[derive(Component)]
pub struct DesktopInterface(Row, Option<DesktopNavigator>, Bin<Stack, Rectangle>, Option<Box<dyn AppPage>>, #[skip] PageBuilder);

impl DesktopInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        mut navigation: Option<(usize, Vec<NavigateInfo>, Vec<NavigateInfo>)>,
    ) -> Self {
        let color = ctx.theme.colors.outline.secondary;
        let pages = navigation.as_mut().map(|navi| navi.1.iter_mut().chain(navi.2.iter_mut()).map(|t| t.3.take().unwrap()).collect::<Vec<_>>());
        let navigator = navigation.map(|n| DesktopNavigator::new(ctx, n));

        DesktopInterface(
            Row::new(0.0, Offset::Start, Size::Fit, Padding::default()),
            navigator,
            Bin(
                Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fit, Padding::default()), 
                Rectangle::new(color, 0.0, None)
            ),
            Some(start_page),
            pages
        )
    }

    pub fn page(&mut self) -> &mut Option<Box<dyn AppPage>> { &mut self.3 }
    pub fn navigator(&mut self) -> &mut Option<DesktopNavigator> { &mut self.1 }
}

impl OnEvent for DesktopInterface {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigateEvent(index)) = event.downcast_mut::<NavigateEvent>() {
            self.3 = match self.3.take().unwrap().navigate(ctx, *index) {
                Ok(p) => Some(p),
                Err(e) => Some(Box::new(Error::new(ctx, "404 Page Not Found", e)))
            };
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
pub struct DesktopNavigator(Column, Image, ButtonColumn, Bin<Stack, Rectangle>, ButtonColumn);

impl DesktopNavigator {
    pub fn new(ctx: &mut Context, navigation: (usize, Vec<NavigateInfo>, Vec<NavigateInfo>)) -> Self {
        let mut top_col = Vec::new();
        let mut bot_col = Vec::new();

        let mut index = 0;

        for (icon, name, avatar, _) in navigation.1.into_iter() {
            let id = ElementID::new();
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(NavigatorSelect(id));
                ctx.trigger_event(NavigatorEvent(index));
            };

            if let Some(avatar) = avatar {
                let profile = Button::photo(ctx, &name, avatar, navigation.0 == index, closure);
                top_col.push(NavigationButton::new(id, Some(profile), None))
            } else {
                let button = Button::navigation(ctx, icon, &name, navigation.0 == index, closure);
                top_col.push(NavigationButton::new(id, Some(button), None))
            }

            index += 1;
        }

        for (icon, name, avatar, _) in navigation.2.into_iter() {
            let id = ElementID::new();
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(NavigatorSelect(id));
                ctx.trigger_event(NavigatorEvent(index));
            };

            if let Some(avatar) = avatar {
                let profile = Button::photo(ctx, &name, avatar, navigation.0 == index, closure);
                bot_col.push(NavigationButton::new(id, Some(profile), None))
            } else {
                let button = Button::navigation(ctx, icon, &name, navigation.0 == index, closure);
                bot_col.push(NavigationButton::new(id, Some(button), None))
            }

            index += 1;
        }

        let wordmark = ctx.theme.brand.wordmark.clone();

        DesktopNavigator(
            Column::new(32.0, Offset::Center, Size::Fill(100.0, 200.0), Padding(16.0, 32.0, 16.0, 32.0)),
            AspectRatioImage::new(wordmark, (108.0, 23.0)),
            ButtonColumn::new(top_col),
            Bin (
                Stack(Offset::Center, Offset::Center, Size::Fill(100.0, 200.0), Size::Fill(0.0, f32::MAX), Padding::default()), 
                Rectangle::new(Color::TRANSPARENT, 0.0, None)
            ),
            ButtonColumn::new(bot_col)
        )
    }

    pub fn update_avatar(&mut self, avatar_content: AvatarContent) {
        if let Some(avatar) = self.avatar() {
            if avatar.avatar().image().is_none() {
                avatar.set_content(avatar_content)
            } else if let AvatarContent::Image(ref image) = avatar_content {
                if avatar.avatar().image().as_ref().unwrap().image != *image {
                    avatar.set_content(avatar_content)
                }
            }
        };
    }

    pub fn update_username(&mut self, username: String) {
        self.4.buttons()[0].button().as_mut().unwrap().label().as_mut().unwrap().text().spans[0].text = username;
    }

    pub fn avatar(&mut self) -> Option<&mut Avatar> {
        self.4.buttons().iter_mut().flat_map(|nb| nb.button()).flat_map(|button| button.avatar()).next()
    }

    pub fn buttons(&mut self) -> Vec<&mut Button> {
        self.2.buttons().iter_mut().flat_map(|nb| nb.button()).collect::<Vec<_>>()
    }
}

impl OnEvent for DesktopNavigator {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(NavigatorSelect(id)) = event.downcast_ref::<NavigatorSelect>() {
            println!("Navigator selected");
            let mut buttons: Vec<&mut NavigationButton> = self.2.buttons().iter_mut().collect();
            buttons.extend(self.4.buttons().iter_mut());
            buttons.iter_mut().for_each(|button| {
                *button.button().as_mut().unwrap().status() = if button.id() == *id {ButtonState::Selected} else {ButtonState::Default};
                button.button().as_mut().unwrap().color(ctx);
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
