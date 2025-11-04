use roost_ui::{Component, Context, drawables};
use roost_ui::events::{Event, OnEvent};
use roost_ui::drawable::{Drawable, Align};

use crate::interactions;
use crate::components::{TextStyle, Text, Icon};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::button::{Button, ButtonStyle, ButtonSize, ButtonWidth, IconButton};

use roost_ui::layouts::{Stack, Offset};
use crate::plugin::PelicanUI;

pub enum PelicanError {
    Err(String, Option<Box<dyn AppPage>>),
    InvalidPage(Option<Box<dyn AppPage>>),
    NoOutlet
}

impl From<String> for PelicanError {
    fn from(s: String) -> PelicanError {
        PelicanError::Err(s, None)
    }
}

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
    ///         0 => page!(Home::new(ctx)),
    ///         1 => page!(Settings::new(ctx))),
    ///         2 => page!(Search::new(ctx))),
    ///         _ => Err(PelicanError::InvalidPage(Some(self))),
    ///     }
    /// }
    /// ```
    ///
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) 
        -> Result<Box<dyn AppPage>, PelicanError>;

    /// Returns whether a navigation bar is visible
    fn has_navigator(&self) -> bool {true}
}

/// Event used to navigate between pages of the app.
#[derive(Debug, Clone)]
pub struct NavigateEvent(pub usize);

impl Event for NavigateEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

pub type PageBuilder = Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>;

pub struct RootInfo {
    pub(crate) icon: &'static str,
    pub(crate) label: String,
    pub(crate) avatar: Option<AvatarContent>,
    pub(crate) get_page: Option<PageBuilder>
}

impl RootInfo {
    pub fn icon(icon: &'static str, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        RootInfo {
            icon,
            label: label.to_string(),
            avatar: None,
            get_page: Some(Box::new(get_page))
        }
    }

    pub fn avatar(avatar: AvatarContent, label: &str, get_page: impl FnMut(&mut Context) -> Box<dyn AppPage> + 'static) -> Self {
        RootInfo {
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

/// Navigates to the page at the given `index`. See [`AppPage`] for details on navigation.
#[derive(Debug, Clone)]
pub struct NavigatorEvent(pub usize);

impl Event for NavigatorEvent {
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
