use roost_ui::{Component, Context};
use roost_ui::events::OnEvent;
use roost_ui::drawable::{Color, Drawable, Image};

use crate::components::{Rectangle, AspectRatioImage};
use crate::components::button::GhostIconButton;
use crate::components::interface::general::InterfaceTrait;
use crate::components::interface::navigation::{AppPage, RootInfo, NavigatorEvent, NavigatorSelectable};

use roost_ui::layouts::{Bin, Column, Offset, Opt, Padding, Row, Size, Stack};
use crate::plugin::PelicanUI;

#[derive(Component, Debug)]
pub struct WebInterface(Column, Option<Opt<Box<dyn Drawable>>>, Option<Box<dyn AppPage>>, Option<WebFooter>);
impl OnEvent for WebInterface {}

impl InterfaceTrait for WebInterface {
    fn app_page(&mut self) -> &mut Option<Box<dyn AppPage>> {&mut self.2}
    fn navigator(&mut self) -> &mut Option<Opt<Box<dyn Drawable>>> {&mut self.1}
}

impl WebInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: Box<dyn AppPage>,
        navigation: Option<(Vec<RootInfo>, Option<Vec<RootInfo>>)>,
        socials: Option<Vec<(&'static str, String)>>
    ) -> Self {
        let navigator = navigation.map(|n| Opt::new(Box::new(WebNavigator::new(ctx, n)) as Box<dyn Drawable>, true));
        let footer = socials.map(|s| WebFooter::new(ctx, s)); 
        let layout = Column::new(0.0, Offset::Start, Size::Fill, Padding::default());
        WebInterface(layout, navigator, Some(start_page), footer)
    }
}

#[derive(Debug, Component)]
pub struct WebNavigator(Row, Image, Bin<Stack, Rectangle>, ButtonRow);
impl OnEvent for WebNavigator {}

impl WebNavigator {
    pub fn new(
        ctx: &mut Context, 
        mut navigation: (Vec<RootInfo>, Option<Vec<RootInfo>>),
    ) -> Self {
        let mut buttons = Vec::new();
        let group_id = uuid::Uuid::new_v4();

        if let Some(n) = navigation.1 { navigation.0.extend(n); }
        for (index, info) in navigation.0.into_iter().enumerate() {
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigatorEvent(index));
            buttons.push(NavigatorSelectable::desktop_icon(ctx, info.icon, &info.label, closure, 0 == index, group_id));
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

    // pub fn buttons(&mut self) -> Vec<&mut NavigatorSelectable> {
    //     self.3.buttons().iter_mut().map(|nb| nb.inner()).collect()
    // }
}

#[derive(Debug, Component)]
struct ButtonRow(Row, Vec<NavigatorSelectable>);
impl OnEvent for ButtonRow {}

impl ButtonRow {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {
        ButtonRow(Row::center(8.0), buttons)
    }

    // fn buttons(&mut self) -> &mut Vec<NavigatorSelectable> {&mut self.1}
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

