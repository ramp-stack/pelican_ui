use prism::{Context, drawables, Request};
use prism::event::OnEvent;
use prism::drawable::Component;
use prism::canvas::{Align, Image};
use prism::display::Bin;
use prism::layout::{Row, Column, Size, Padding, Offset, Stack};

use ptsd::interactions;

pub use ptsd::navigation::{NavigationEvent, AppPage, Flow, FlowContainer};

use crate::theme::{self, Theme, Color, Variant};
use crate::components::{Icon, AspectRatioImage, Rectangle};
use crate::components::text::{TextStyle, Text};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};


#[derive(Debug, Clone)]
pub struct RootInfo {
    pub(crate) icon: String,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) page: Option<Box<dyn AppPage>>,
}

impl RootInfo {
    pub fn icon(icon: &str, label: &str, page: Box<dyn AppPage>) -> Self {
        RootInfo {
            icon: icon.to_string(),
            label: label.to_string(),
            avatar: None,
            page: Some(page)
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, page: Box<dyn AppPage>) -> Self {
        RootInfo {
            icon: "profile".to_string(),
            label: label.to_string(),
            avatar: Some(avatar),
            page: Some(page)
        }
    }
}


#[derive(Debug, Component, Clone)]
pub struct NavigatorSelectable(Stack, interactions::Selectable);
impl OnEvent for NavigatorSelectable {}
impl NavigatorSelectable {
    pub fn desktop_icon(theme: &Theme, icon: &str, label: &str, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let text = Text::new(theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let icon = Icon::new(theme, icon, Some(colors.label), icon_size);
            Button::new(drawables![icon, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });

        let theme = theme.clone();
        let callback = move |ctx: &mut Context| (on_click)(ctx, &theme);
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, false, callback, group_id))
    }

    pub fn desktop_avatar(theme: &Theme, avatar: AvatarContent, label: &str, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let avatar = Avatar::new(theme, avatar.clone(), None, false, AvatarSize::Xs, None);
            Button::new(drawables![avatar, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, false, callback, group_id))
    }

    pub fn mobile(theme: &Theme, icon: &str, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static, is_selected: bool, group_id: uuid::Uuid) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.disabled, colors.default].map(|colors| {
            IconButton::new(theme, icon, ButtonStyle::Ghost, ButtonSize::Large, colors.background, colors.outline, colors.label)
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(default, selected, is_selected, false, callback, group_id))
    }
}

// /// Selects the [`NavigationButton`] with the given [`uuid::Uuid`].
// #[derive(Debug, Clone)]
// pub struct NavigatorSelect(pub uuid::Uuid);

// impl Event for NavigatorSelect {
//     fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
//         children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
//     }
// }

#[derive(Debug, Component, Clone)]
pub(crate) enum Navigator {
    Desktop {
        layout: Column, 
        brandmark: Image, 
        top: ButtonColumn, 
        spacer: Option<Bin<Stack, Rectangle>>, 
        bottom: Option<ButtonColumn>
    },

    Mobile {
        layout: Stack, 
        background: Rectangle, 
        content: MobileNavigatorContent
    },

    Web {
        layout: Row, 
        brandmark: Image, 
        spacer: Bin<Stack, Rectangle>, 
        content: ButtonRow
    }
}

impl OnEvent for Navigator {}
impl ptsd::interfaces::Navigator for Navigator {}

impl Navigator {
    pub(crate) fn desktop(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let (mut top_col, mut bot_col) = (Vec::new(), Vec::new());
        let mut i = 0;
        let mut has_profile = false;

        navigation.into_iter().for_each(|info| {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context, _: &Theme| ctx.send(Request::Event(Box::new(NavigationEvent::Root(root.clone()))));

            match info.avatar {
                Some(a) => {
                    has_profile = true;
                    bot_col.push(NavigatorSelectable::desktop_avatar(theme, a, &info.label, closure, 0 == i, group_id));
                }
                None => top_col.push(NavigatorSelectable::desktop_icon(theme, &info.icon, &info.label, closure, 0 == i, group_id))
            };
            i += 1;
        });

        let spacer = has_profile.then(|| {
            let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, widths[1].1));
            let spacer = Stack(Offset::Center, Offset::Center, width, Size::Fill, Padding::default());
            Bin(spacer, Rectangle::new(Color::TRANSPARENT, 0.0, None))
        });

        let wordmark = theme.brand().wordmark.clone();
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 200.0));

        Navigator::Desktop {
            layout: Column::new(32.0, Offset::Center, width, Padding(16.0, 32.0, 16.0, 32.0), None),
            brandmark: AspectRatioImage::new(wordmark, (100.0, 25.0)), 
            top: ButtonColumn::new(top_col), 
            spacer,
            bottom: (!bot_col.is_empty()).then_some(ButtonColumn::new(bot_col))
        }
    }

    pub(crate) fn mobile(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let background = theme.colors().get(ptsd::Background::Primary);

        let group_id = uuid::Uuid::new_v4();
        let mut tabs = Vec::new();
        for (i, info) in navigation.into_iter().enumerate() {
            let closure = move |ctx: &mut Context, _: &Theme| ctx.send(Request::Event(Box::new(NavigationEvent::Root(info.label.to_string()))));
            tabs.push(NavigatorSelectable::mobile(theme, &info.icon, closure, 0 == i, group_id));
        }

        Navigator::Mobile {
            layout: Stack(Offset::Center, Offset::Start, Size::Fill, height, Padding::default()), 
            background: Rectangle::new(background, 0.0, None),
            content: MobileNavigatorContent::new(tabs)
        }
    }


    pub(crate) fn web(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let mut buttons = Vec::new();
        let group_id = uuid::Uuid::new_v4();

        for (index, info) in navigation.into_iter().enumerate() {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context, _: &Theme| ctx.send(Request::Event(Box::new(NavigationEvent::Root(root.clone()))));
            buttons.push(NavigatorSelectable::desktop_icon(theme, &info.icon, &info.label, closure, 0 == index, group_id));
        }

        let wordmark = theme.brand().wordmark.clone();

        let bin_layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(5.0), Padding::default());
        Navigator::Web {
            layout: Row::new(32.0, Offset::Center, Size::Fit, Padding::new(48.0)),
            brandmark: AspectRatioImage::new(wordmark, (150.0, 35.0)),
            spacer: Bin (bin_layout, Rectangle::new(Color::TRANSPARENT, 0.0, None)),
            content: ButtonRow::new(buttons)
        }
    }
}


#[derive(Debug, Component, Clone)]
pub struct MobileNavigatorContent(Row, Vec<NavigatorSelectable>);
impl OnEvent for MobileNavigatorContent {}

impl MobileNavigatorContent {
    fn new(tabs: Vec<NavigatorSelectable>) -> Self {
        let layout = Row::new(48.0, Offset::Center, Size::Fit, Padding(0.0, 8.0, 0.0, 8.0));
        MobileNavigatorContent(layout, tabs)
    }
}

#[derive(Debug, Component, Clone)]
pub struct ButtonColumn(Column, Vec<NavigatorSelectable>);
impl OnEvent for ButtonColumn {}

impl ButtonColumn {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {
        ButtonColumn(Column::center(8.0), buttons)
    }
}

#[derive(Debug, Component, Clone)]
pub struct ButtonRow(Row, Vec<NavigatorSelectable>);
impl OnEvent for ButtonRow {}

impl ButtonRow {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {
        ButtonRow(Row::center(8.0), buttons)
    }

    // fn buttons(&mut self) -> &mut Vec<NavigatorSelectable> {&mut self.1}
}
