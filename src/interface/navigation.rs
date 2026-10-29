use prism::Context;
use prism::drawable::Component;
use prism::drawable::text::Align;
use prism::drawable::Svg;
use ptsd::display::Bin;
use ptsd::layout::{Row, Column, Size, Padding, Offset, Stack};
use ptsd::interactions;
pub use ptsd::navigation::{NavigationEvent, AppPage, Flow, FlowContainer};

use crate::theme::{self, Theme, Color, Variant, Icons};
use crate::components::{Icon, AspectRatioImage, Rectangle};
use crate::components::text::{TextStyle, Text};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};

#[derive(Debug, Clone)]
pub struct RootInfo {
    pub icon: Icons,
    pub label: String,
    pub avatar: Option<AvatarContent>,
    pub page: Option<Box<dyn AppPage>>,
}

impl RootInfo {
    pub fn icon(icon: Icons, label: &str, page: Box<dyn AppPage>) -> Self {
        RootInfo {
            icon,
            label: label.to_string(),
            avatar: None,
            page: Some(page)
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, page: Box<dyn AppPage>) -> Self {
        RootInfo {
            icon: Icons::Profile,
            label: label.to_string(),
            avatar: Some(avatar),
            page: Some(page)
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct NavigatorSelectable(Stack, interactions::Selectable);
impl NavigatorSelectable {
    pub fn desktop_icon(
        theme: &Theme, icon: Icons, label: &str, layout: Stack,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        is_selected: bool, group_id: uuid::Uuid,
    ) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let text = Text::new(
                theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None,
            ).fit();
            let icon = Icon::new(theme, icon, Some(colors.label), icon_size);
            Bin(layout.clone(), Button::new(
                vec![Box::new(icon), Box::new(text)], ButtonSize::Large, ButtonWidth::Fill,
                Offset::Start, colors.background, colors.outline,
            ))
        });

        let theme = theme.clone();
        let callback = move |ctx: &mut Context| (on_click)(ctx, &theme);
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(
            default, selected, is_selected, false, callback, group_id,
        ))
    }

    pub fn desktop_avatar(
        theme: &Theme, avatar: AvatarContent, label: &str, layout: Stack,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        is_selected: bool, group_id: uuid::Uuid,
    ) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(
                theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None,
            ).fit();
            let avatar = Avatar::new(theme, avatar.clone(), None, AvatarSize::Xs, None);
            Bin(layout.clone(), Button::new(
                vec![Box::new(avatar), Box::new(text)], ButtonSize::Large, ButtonWidth::Fill,
                Offset::Start, colors.background, colors.outline,
            ))
        });
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(
            default, selected, is_selected, false, callback, group_id,
        ))
    }

    pub fn mobile(
        theme: &Theme, icon: Icons, padding: Padding,
        on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        is_selected: bool, group_id: uuid::Uuid,
    ) -> Self {
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::custom(|_| (48.0, f32::MAX)),
            Size::Fit, padding,
        );
        Self::mobile_with_layout(theme, icon, layout, on_click, is_selected, group_id)
    }

    fn mobile_with_layout(
        theme: &Theme, icon: Icons, layout: Stack,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        is_selected: bool, group_id: uuid::Uuid,
    ) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let [default, selected] = [colors.disabled, colors.default].map(|colors| {
            Bin(layout.clone(), IconButton::new(
                theme, icon, ButtonStyle::Ghost, ButtonSize::Medium,
                colors.background, colors.outline, colors.label,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(
            default, selected, is_selected, false, callback, group_id,
        ))
    }
}

#[derive(Debug, Component, Clone)]
pub enum Navigator {
    Desktop {
        layout: Column,
        brandmark: Bin<Stack, Svg>,
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
        brandmark: Bin<Stack, Svg>,
        spacer: Bin<Stack, Rectangle>,
        content: ButtonRow
    }
}

impl ptsd::interfaces::Navigator for Navigator {}

impl Navigator {
    pub fn desktop(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let (mut top_col, mut bot_col) = (Vec::new(), Vec::new());
        let mut i = 0;
        let mut has_profile = false;

        navigation.into_iter().for_each(|info| {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::root(&root.clone()));

            let layout = Stack::new(
                Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(16.0, 0.0, 16.0, 0.0),
            );
            match info.avatar {
                Some(a) => {
                    has_profile = true;
                    bot_col.push(NavigatorSelectable::desktop_avatar(
                        theme, a, &info.label, layout, closure, 0 == i, group_id,
                    ));
                }
                None => top_col.push(NavigatorSelectable::desktop_icon(
                    theme, info.icon, &info.label, layout, closure, 0 == i, group_id,
                )),
            };
            i += 1;
        });

        let spacer = has_profile.then(|| {
            let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, widths[0].1));
            let spacer = Stack::new(
                Offset::Center, Offset::Center, width, Size::Fill, Padding::default(),
            );
            Bin(spacer, Rectangle::new(Color::transparent(), 0.0, None))
        });

        let wordmark = theme.brand().wordmark.clone();
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 232.0));

        Navigator::Desktop {
            layout: Column::new(32.0, Offset::Center, width, Padding(0.0, 32.0, 0.0, 32.0), None),
            brandmark: AspectRatioImage::svg(wordmark, 100.0),
            top: ButtonColumn::new(top_col),
            spacer,
            bottom: (!bot_col.is_empty()).then_some(ButtonColumn::new(bot_col))
        }
    }

    pub fn mobile(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let height = Size::custom(move |heights: Vec<(f32, f32)>|(heights[1].0, heights[1].1));
        let background = theme.colors().get(ptsd::Background::Primary);

        let group_id = uuid::Uuid::new_v4();
        let mut tabs = Vec::new();
        let count = navigation.len();
        let button_width = ButtonSize::Medium.icon_button(ButtonStyle::Ghost).0;
        let side_padding = (100.0 - button_width) / 2.0;
        for (i, info) in navigation.into_iter().enumerate() {
            let closure = move |ctx: &mut Context, _: &Theme| {
                ctx.emit(NavigationEvent::root(&info.label))
            };
            // Only the outer tap areas expand. Align their icons toward the
            // middle so the visible group stays centered at 100-point intervals.
            let offset = if count == 1 {Offset::Center}
                else if i == 0 {Offset::End}
                else if i == count - 1 {Offset::Start}
                else {Offset::Center};
            let width = if i == 0 || i == count - 1 {
                Size::custom(move |_| (button_width, f32::MAX))
            } else {Size::Static(button_width)};
            let layout = Stack::new(
                offset, Offset::Center, width, Size::Fit,
                Padding(side_padding, 8.0, side_padding, 8.0),
            );
            tabs.push(NavigatorSelectable::mobile_with_layout(
                theme, info.icon, layout, closure, 0 == i, group_id,
            ));
        }

        Navigator::Mobile {
            layout: Stack::new(Offset::Center, Offset::Start, Size::Fill, height, Padding::default()),
            background: Rectangle::new(background, 0.0, None),
            content: MobileNavigatorContent::new(tabs)
        }
    }

    pub fn web(theme: &Theme, navigation: Vec<RootInfo>) -> Self {
        let mut buttons = Vec::new();
        let group_id = uuid::Uuid::new_v4();

        for (index, info) in navigation.into_iter().enumerate() {
            let root = info.label.to_string();
            let closure = move |ctx: &mut Context, _: &Theme| ctx.emit(NavigationEvent::root(&root.clone()));
            let layout = Stack::new(
                Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(0.0, 48.0, 0.0, 48.0),
            );
            buttons.push(NavigatorSelectable::desktop_icon(
                theme, info.icon, &info.label, layout, closure, 0 == index, group_id,
            ));
        }

        let wordmark = theme.brand().wordmark.clone();

        let bin_layout = Stack::new(
            Offset::Center, Offset::Center, Size::Fill, Size::Static(5.0), Padding::default(),
        );
        Navigator::Web {
            layout: Row::new(32.0, Offset::Center, Size::Fit, Padding(48.0, 0.0, 48.0, 0.0)),
            brandmark: AspectRatioImage::svg(wordmark, 150.0),
            spacer: Bin (bin_layout, Rectangle::new(Color::transparent(), 0.0, None)),
            content: ButtonRow::new(buttons)
        }
    }
}

/// Public because it is the content of the `Navigator::Mobile` variant.
#[derive(Debug, Component, Clone)]
pub struct MobileNavigatorContent(Row, Vec<NavigatorSelectable>);

impl MobileNavigatorContent {
    fn new(tabs: Vec<NavigatorSelectable>) -> Self {
        let layout = Row::new(0.0, Offset::Center, Size::Fit, Padding::default());
        MobileNavigatorContent(layout, tabs)
    }
}

/// Public because it is the content of the `Navigator::Desktop` variant.
#[derive(Debug, Component, Clone)]
pub struct ButtonColumn(Column, Vec<NavigatorSelectable>);

impl ButtonColumn {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {ButtonColumn(Column::center(8.0), buttons)}
}

/// Public because it is the content of the `Navigator::Web` variant.
#[derive(Debug, Component, Clone)]
pub struct ButtonRow(Row, Vec<NavigatorSelectable>);

impl ButtonRow {
    fn new(buttons: Vec<NavigatorSelectable>) -> Self {ButtonRow(Row::center(8.0), buttons)}
}
