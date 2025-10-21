use mustache::{Component, Context};
use mustache::drawable::Drawable;
use mustache::events::{Event, OnEvent};

use crate::layout::{Column, Row, Padding, Offset, Size, Opt, Stack};

use crate::components::Rectangle;
use crate::components::interface::general::InterfaceTrait;
use crate::components::interface::system::MobileKeyboard;
use crate::components::interface::navigation::{AppPage, NavigateEvent, NavigateInfo, NavigatorEvent, NavigatorSelectable};
use crate::plugin::PelicanUI;
use crate::utils::ElementID;

#[derive(Component, Debug)]
pub struct MobileInterface(Column, Option<Box<dyn AppPage>>, Option<MobileKeyboard>, Option<Opt<Box<dyn Drawable>>>);

impl InterfaceTrait for MobileInterface {
    fn app_page(&mut self) -> &mut Option<Box<dyn AppPage>> {&mut self.1}
    fn navigator(&mut self) -> &mut Option<Opt<Box<dyn Drawable>>> {&mut self.3}
}

impl MobileInterface {
    pub fn new(
        ctx: &mut Context, 
        start_page: impl AppPage,
        navigation: Option<(usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)>
    ) -> Self {
        let navigator = navigation.map(|n| Opt::new(Box::new(MobileNavigator::new(ctx, n)) as Box<dyn Drawable>, true));
        let insets = ctx.hardware.safe_area_insets();
        let padding = Padding(insets.0, insets.2, insets.1, insets.3);
        let layout = Column::new(0.0, Offset::Center, Size::Fit, padding);
        MobileInterface(layout, Some(Box::new(start_page)), None, navigator)
    }
}

impl OnEvent for MobileInterface {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_mut::<NavigateEvent>().is_some() {
            self.2 = None;
        } else if let Some(ShowKeyboard(b)) = event.downcast_ref::<ShowKeyboard>() {
            self.2 = b.then_some(MobileKeyboard::new(ctx, true));
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct MobileNavigator(Stack, Rectangle, MobileNavigatorContent);
impl OnEvent for MobileNavigator {}

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
}

#[derive(Debug, Component)]
struct MobileNavigatorContent(Row, Vec<NavigatorSelectable>);
impl OnEvent for MobileNavigatorContent {}

impl MobileNavigatorContent {
    fn new(ctx: &mut Context, mut navigation: (usize, Vec<NavigateInfo>, Option<Vec<NavigateInfo>>)) -> Self {
        let group_id = ElementID::new();
        let mut tabs = Vec::new();
        if let Some(n) = navigation.2 { navigation.1.extend(n); }
        for (i, info) in navigation.1.into_iter().enumerate() {
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigatorEvent(i));
            tabs.push(NavigatorSelectable::mobile(ctx, info.icon, closure, navigation.0 == i, group_id));
        }

        let layout = Row::new(48.0, Offset::Center, Size::Fit, Padding(0.0, 8.0, 0.0, 8.0));
        MobileNavigatorContent(layout, tabs)
    }
}

/// Event used to open or close keyboard.
#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}