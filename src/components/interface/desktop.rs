use mustache::{Component, Context};
use mustache::events::OnEvent;
use mustache::drawable::{Color, Drawable, Image};
use mustache::layouts::{Bin, Column, Offset, Opt, Padding, Row, Size, Stack};

use crate::components::{Rectangle, AspectRatioImage};
use crate::components::interface::general::InterfaceTrait;
use crate::components::interface::navigation::{AppPage, NavigatorEvent, NavigateInfo, NavigatorSelectable};
use crate::plugin::PelicanUI;

#[derive(Component, Debug)]
pub struct DesktopInterface(Row, Option<Opt<Box<dyn Drawable>>>, Bin<Stack, Rectangle>, Option<Box<dyn AppPage>>);
impl OnEvent for DesktopInterface {}

impl InterfaceTrait for DesktopInterface {
    fn app_page(&mut self) -> &mut Option<Box<dyn AppPage>> {&mut self.3}
    fn navigator(&mut self) -> &mut Option<Opt<Box<dyn Drawable>>> {&mut self.1}
}

impl DesktopInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: impl AppPage,
        navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>,
    ) -> Self {
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.outline.secondary;
        let navigator = navigation.map(|n| Opt::new(Box::new(DesktopNavigator::new(ctx, n)) as Box<dyn Drawable>, true));
        let line_layout = Stack(Offset::default(), Offset::default(), Size::Static(1.0), Size::Fill, Padding::default());
        let separator = Bin(line_layout, Rectangle::new(color, 0.0, None));
        DesktopInterface(Row::start(0.0), navigator, separator, Some(Box::new(start_page)))
    }
}

#[derive(Debug, Component)]
pub struct DesktopNavigator(Column, Image, ButtonColumn, Option<Bin<Stack, Rectangle>>, Option<ButtonColumn>);
impl OnEvent for DesktopNavigator {}

impl DesktopNavigator {
    pub fn new(ctx: &mut Context, navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let (mut top_col, mut bot_col) = (Vec::new(), Vec::new());
        let mut i = 0;

        let spacer = navigation.2.as_ref().map(|_| {
            let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, widths[1].1));
            let spacer = Stack(Offset::Center, Offset::Center, width, Size::Fill, Padding::default());
            Bin(spacer, Rectangle::new(Color::TRANSPARENT, 0.0, None))
        });

        navigation.1.into_iter().for_each(|info| {
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigatorEvent(i));

            top_col.push(match info.avatar {
                Some(a) => NavigatorSelectable::desktop_avatar(ctx, a, &info.label, closure, navigation.0 == i, group_id),
                None => NavigatorSelectable::desktop_icon(ctx, info.icon, &info.label, closure, navigation.0 == i, group_id)
            });
            i += 1;
        });

        if let Some(n) = navigation.2 {
            n.into_iter().for_each(|info| {
                let closure = move |ctx: &mut Context| ctx.trigger_event(NavigatorEvent(i));

                bot_col.push(match info.avatar {
                    Some(a) => NavigatorSelectable::desktop_avatar(ctx, a, &info.label, closure, navigation.0 == i, group_id),
                    None => NavigatorSelectable::desktop_icon(ctx, info.icon, &info.label, closure, navigation.0 == i, group_id)
                });
                i += 1;
            });
        }

        let top_col = ButtonColumn::new(top_col);
        let bot_col = (!bot_col.is_empty()).then_some(ButtonColumn::new(bot_col));
        let wordmark = ctx.get::<PelicanUI>().get().0.theme().brand.wordmark.clone();
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 200.0));

        DesktopNavigator(
            Column::new(32.0, Offset::Center, width, Padding(16.0, 32.0, 16.0, 32.0)),
            AspectRatioImage::new(wordmark, (100.0, 25.0)), top_col, spacer, bot_col
        )
    }

}

#[derive(Debug, Component)]
struct ButtonColumn(Column, Vec<NavigatorSelectable>);
impl OnEvent for ButtonColumn {}

impl ButtonColumn {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {
        ButtonColumn(Column::center(8.0), buttons)
    }
}
