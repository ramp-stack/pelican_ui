use roost_ui::{Component, Context, drawables};
use roost_ui::events::{Event, OnEvent};
use roost_ui::drawable::{Drawable, Align, Color, Image};
use roost_ui::layouts::{Row, Column, Size, Padding, Offset, Bin, Stack};

use crate::interactions;
use crate::components::{TextStyle, Text, Icon, AspectRatioImage, Rectangle};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};
use crate::plugin::PelicanUI;

/// This trait is used to define pages in the application.
/// 
/// Every page must implement this trait. 
///
/// Every page must implement [`Debug`] and [`Component`].
///
///
pub trait AppPage: Drawable + std::fmt::Debug + 'static {}

/// Event used to navigate between pages of the app.

#[derive(Debug)]
pub enum NavigationEvent {
    Pop,
    Push(Option<Box<dyn AppPage>>),
    Reset,
    Root(String),
    Error(String)
}

impl Event for NavigationEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, _children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        vec![Some(Box::new(_NavEvent(self)))]
    }
}

#[derive(Debug)]
pub struct _NavEvent(Box<NavigationEvent>);

impl Event for _NavEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, _children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        vec![None, Some(self.0)]
    }
}

#[derive(Debug)]
pub struct RootInfo {
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) page: Option<Box<dyn AppPage>>,
}

impl RootInfo {
    pub fn icon(icon: &'static str, label: &str, page: impl AppPage) -> Self {
        RootInfo {
            icon,
            label: label.to_string(),
            avatar: None,
            page: Some(Box::new(page))
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, page: impl AppPage) -> Self {
        RootInfo {
            icon: "profile",
            label: label.to_string(),
            avatar: Some(avatar),
            page: Some(Box::new(page))
        }
    }
}


#[derive(Debug, Component)]
pub struct NavigatorSelectable(Stack, interactions::Selectable);
impl OnEvent for NavigatorSelectable {}
impl NavigatorSelectable {
    pub fn desktop_icon(ctx: &mut Context, icon: &'static str, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let icon = Icon::new(ctx, icon, Some(colors.label), icon_size);
            Button::new(drawables![icon, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, on_click, group_id))
    }

    pub fn desktop_avatar(ctx: &mut Context, avatar: AvatarContent, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let avatar = Avatar::new(ctx, avatar.clone(), None, false, AvatarSize::Xs, None);
            Button::new(drawables![avatar, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, on_click, group_id))
    }

    pub fn mobile(ctx: &mut Context, icon: &'static str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.disabled, colors.default].map(|colors| {
            IconButton::new(ctx, icon, ButtonStyle::Ghost, ButtonSize::Large, colors.background, colors.outline, colors.label)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, on_click, group_id))
    }
}

/// Selects the [`NavigationButton`] with the given [`uuid::Uuid`].
#[derive(Debug, Clone)]
pub struct NavigatorSelect(pub uuid::Uuid);

impl Event for NavigatorSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Component)]
pub enum Navigator {
    Mobile(NavigatorMobile),
    Desktop(NavigatorDesktop),
    Web(NavigatorWeb),
}

impl OnEvent for Navigator {}

impl Navigator {
    pub fn mobile(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Navigator::Mobile(NavigatorMobile::new(ctx, navigation))
    }

    pub fn desktop(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Navigator::Desktop(NavigatorDesktop::new(ctx, navigation))
    }

    pub fn web(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        Navigator::Web(NavigatorWeb::new(ctx, navigation))
    }
}

#[derive(Debug, Component)]
pub struct NavigatorMobile(Stack, Rectangle, MobileNavigatorContent);
impl OnEvent for NavigatorMobile {}

impl NavigatorMobile {
    pub fn new(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;

        let group_id = uuid::Uuid::new_v4();
        let mut tabs = Vec::new();
        for (i, info) in navigation.into_iter().enumerate() {
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigationEvent::Root(info.label.to_string()));
            tabs.push(NavigatorSelectable::mobile(ctx, info.icon, closure, 0 == i, group_id));
        }

        NavigatorMobile(
            Stack(Offset::Center, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(background, 0.0, None),
            MobileNavigatorContent::new(tabs)
        )
    }
}

#[derive(Debug, Component)]
pub struct NavigatorDesktop(Column, Image, ButtonColumn, Option<Bin<Stack, Rectangle>>, Option<ButtonColumn>);
impl OnEvent for NavigatorDesktop {}

impl NavigatorDesktop {
    pub fn new(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let (mut top_col, mut bot_col) = (Vec::new(), Vec::new());
        let mut i = 0;
        let mut has_profile = false;

        navigation.into_iter().for_each(|info| {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigationEvent::Root(root.clone()));

            match info.avatar {
                Some(a) => {
                    has_profile = true;
                    bot_col.push(NavigatorSelectable::desktop_avatar(ctx, a, &info.label, closure, 0 == i, group_id));
                }
                None => top_col.push(NavigatorSelectable::desktop_icon(ctx, info.icon, &info.label, closure, 0 == i, group_id))
            };
            i += 1;
        });

        let spacer = has_profile.then(|| {
            let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, widths[1].1));
            let spacer = Stack(Offset::Center, Offset::Center, width, Size::Fill, Padding::default());
            Bin(spacer, Rectangle::new(Color::TRANSPARENT, 0.0, None))
        });

        let wordmark = ctx.get::<PelicanUI>().get().0.theme().brand.wordmark.clone();
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 200.0));

        NavigatorDesktop(
            Column::new(32.0, Offset::Center, width, Padding(16.0, 32.0, 16.0, 32.0)),
            AspectRatioImage::new(wordmark, (100.0, 25.0)), 
            ButtonColumn::new(top_col), 
            spacer,
            (!bot_col.is_empty()).then_some(ButtonColumn::new(bot_col))
        )
    }
}

#[derive(Debug, Component)]
pub struct NavigatorWeb(Row, Image, Bin<Stack, Rectangle>, ButtonRow);
impl OnEvent for NavigatorWeb {}

impl NavigatorWeb {
    pub fn new(ctx: &mut Context, navigation: Vec<RootInfo>) -> Self {
        let mut buttons = Vec::new();
        let group_id = uuid::Uuid::new_v4();

        for (index, info) in navigation.into_iter().enumerate() {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context| ctx.trigger_event(NavigationEvent::Root(root.clone()));
            buttons.push(NavigatorSelectable::desktop_icon(ctx, info.icon, &info.label, closure, 0 == index, group_id));
        }

        let wordmark = ctx.get::<PelicanUI>().get().0.theme().brand.wordmark.clone();

        let bin_layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(5.0), Padding::default());
        NavigatorWeb(
            Row::new(32.0, Offset::Center, Size::Fit, Padding::new(48.0)),
            AspectRatioImage::new(wordmark, (150.0, 35.0)),
            Bin (bin_layout, Rectangle::new(Color::TRANSPARENT, 0.0, None)),
            ButtonRow::new(buttons)
        )
    }
}

#[derive(Debug, Component)]
struct MobileNavigatorContent(Row, Vec<NavigatorSelectable>);
impl OnEvent for MobileNavigatorContent {}

impl MobileNavigatorContent {
    fn new(tabs: Vec<NavigatorSelectable>) -> Self {
        let layout = Row::new(48.0, Offset::Center, Size::Fit, Padding(0.0, 8.0, 0.0, 8.0));
        MobileNavigatorContent(layout, tabs)
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

#[derive(Debug, Component)]
struct ButtonRow(Row, Vec<NavigatorSelectable>);
impl OnEvent for ButtonRow {}

impl ButtonRow {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {
        ButtonRow(Row::center(8.0), buttons)
    }

    // fn buttons(&mut self) -> &mut Vec<NavigatorSelectable> {&mut self.1}
}
