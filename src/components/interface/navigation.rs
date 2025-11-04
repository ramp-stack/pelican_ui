use roost::{Component, Context, drawables};
use roost::events::{Event, OnEvent};
use roost::drawable::{Drawable, Align};

use crate::interactions;
use crate::components::{TextStyle, Text, Icon};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};

use roost::layouts::{Stack, Offset};
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
    Pop(usize),
    Push(Option<Box<dyn AppPage>>),
    Reset,
    Root(String),
    Error(String)
}

impl NavigationEvent {
    pub fn push(page: impl AppPage) -> Self {
        NavigationEvent::Push(Some(Box::new(page)))
    }
}

impl Event for NavigationEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, _children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        // children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
        vec![Some(self)]
    }
}

pub struct RootInfo {
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) page: Option<Box<dyn AppPage>>
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

#[macro_export]
macro_rules! page {
    ($result:expr, $self:expr) => {
        $result
            .map(|p| Box::new(p) as Box<dyn AppPage>)
            .map_err(|e| PelicanError::Err(e, Some($self)))
    };
}
