use mustache::{Component, Context, drawables};
use mustache::events::{Event, OnEvent};
use mustache::drawable::{Drawable, Align};

use crate::components::{TextStyle, Text, Icon};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};
use crate::components::interactions;

use crate::layout::{Stack, Offset};
use crate::plugin::PelicanUI;
use crate::utils::ElementID;

pub type PageBuilder = Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>;

/// This trait is used to define pages in the application.
/// 
/// Every page must implement this trait. 
///
/// Every page must implement [`Debug`] and [`Component`].
///
///
pub trait AppPage: Drawable + std::fmt::Debug + 'static {
    /// This is called to navigate away from the current page.
    ///
    /// The `index` parameter is the index that was triggered. Match on the index to navigate to
    /// the desired page. The returned value must be an `Ok` variant with a boxed `dyn AppPage`.
    ///
    /// If the index is not an expected value, return `Err(self)` and the user will be navigated
    /// to an error page where `self` acts as the **"go back"** button.
    ///
    /// ```rust
    /// fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) {
    ///     match index {
    ///         0 => Ok(Box::new(Home::new(ctx))),
    ///         1 => Ok(Box::new(Settings::new(ctx))),
    ///         2 => Ok(Box::new(Search::new(ctx))),
    ///         _ => Err(self),
    ///     }
    /// }
    /// ```
    ///
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) 
        -> Result<Box<dyn AppPage>, Box<dyn AppPage>>;

    /// Returns whether a navigation bar is visible (mobile specific).
    fn has_nav(&self) -> bool {true}
}

/// Event used to navigate between pages of the app.
#[derive(Debug, Clone)]
pub struct NavigateEvent(pub usize);

impl Event for NavigateEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

pub struct NavigateInfo {
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) get_page: Option<PageBuilder>
}

impl NavigateInfo {
    pub fn icon(icon: &'static str, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        NavigateInfo {
            icon,
            label: label.to_string(),
            avatar: None,
            get_page: Some(Box::new(get_page))
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        NavigateInfo {
            icon: "profile",
            label: label.to_string(),
            avatar: Some(avatar),
            get_page: Some(Box::new(get_page))
        }
    }
}


#[derive(Debug, Component)]
pub struct NavigatorSelectable(Stack, interactions::Selectable);
impl OnEvent for NavigatorSelectable {}
impl NavigatorSelectable {
    pub fn desktop_icon(ctx: &mut Context, icon: &'static str, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: ElementID) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font(ctx);
            let icon_size = ButtonSize::Large.icon();
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let icon = Icon::new(ctx, icon, colors.label, icon_size);
            Button::new(drawables![icon, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(on_click, default, selected, is_selected, group_id))
    }

    pub fn desktop_avatar(ctx: &mut Context, avatar: AvatarContent, label: &str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: ElementID) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.default, colors.pressed].map(|colors| {
            let font_size = ButtonSize::Large.font(ctx);
            let text = Text::new(ctx, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            let avatar = Avatar::new(ctx, avatar.clone(), None, false, AvatarSize::Xs, None);
            Button::new(drawables![avatar, text], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.outline)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(on_click, default, selected, is_selected, group_id))
    }

    pub fn mobile(ctx: &mut Context, icon: &'static str, on_click: impl FnMut(&mut Context) + 'static, is_selected: bool, group_id: ElementID) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let [default, selected] = [colors.disabled, colors.default].map(|colors| {
            IconButton::new(ctx, icon, ButtonStyle::Ghost, ButtonSize::Large, colors.background, colors.outline, colors.label)
        });
        NavigatorSelectable(Stack::default(), interactions::Selectable::new(on_click, default, selected, is_selected, group_id))
    }

    pub fn inner(&mut self) -> &mut interactions::Selectable {&mut self.1}
}

/// Selects the [`NavigationButton`] with the given [`ElementID`].
#[derive(Debug, Clone)]
pub struct NavigatorSelect(pub ElementID);

impl Event for NavigatorSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Navigates to the page at the given `index`. See [`AppPage`] for details on navigation.
#[derive(Debug, Clone)]
pub struct NavigatorEvent(pub usize);

impl Event for NavigatorEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}