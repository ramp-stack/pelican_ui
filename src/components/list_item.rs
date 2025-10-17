use mustache::events::{OnEvent, MouseState, MouseEvent, Event};
use mustache::drawable::{Align, Color, Image};
use mustache::{Context, Component};

use crate::components::{Rectangle, Icon, Text, ExpandableText, TextStyle};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::components::interactions::ButtonState;
use crate::layout::{Column, Stack, Row, Padding, Offset, Size};
use crate::plugin::PelicanUI;
use crate::utils::Callback;
use crate::utils::TitleSubtitle;

/// ## List Item
///
/// A component used for lists, menus, or settings screens.  
/// Supports a title, optional flair (badge), subtitle, description, right-aligned  
/// content, and avatar icons.  
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/list_item.png"
///      alt="List Item Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let item = ListItem::new(
///     ctx,
///     Some(AvatarContent::Icon("settings", AvatarIconStyle::Success)),
///     ListItemInfoLeft::new("Wi-Fi", "Home Network", None, None),
///     Some(TitleSubtitle::new("Connected", "Secure, WPA2")),
///     None,
///     None,
///     |ctx: &mut Context| println!("Clicked Wi-Fi")
/// );
/// ```
#[derive(Component)]
pub struct ListItem(Stack, Rectangle, ListItemContent, #[skip] ButtonState, #[skip] Callback, #[skip] bool);

impl ListItem {
    pub fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<TitleSubtitle>,
        icon_l: Option<&'static str>,
        icon_r: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let content = ListItemContent::new(ctx, avatar, left, right, icon_l, icon_r);
        let layout = Stack(Offset::Start, Offset::Center, Size::Fill, Size::custom(|heights: Vec<(f32, f32)>| heights[1]), Padding(0.0, 16.0, 0.0, 16.0));
        ListItem(layout, Rectangle::new(background, 0.0, None), content, ButtonState::Default, Box::new(on_click), false)
    }
}

impl OnEvent for ListItem {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                self.5 = true;
            } else if let MouseEvent{state: MouseState::Released, position: Some(_)} = event {
                if self.5 && matches!(self.3, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
                    ctx.hardware.haptic();
                    (self.4)(ctx)
                }
                self.5 = false;
            }
        }
        false
    }
}

impl std::fmt::Debug for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListItem")
    }
}

#[derive(Debug, Component)]
struct ListItemContent(Row, Option<Image>, Option<Avatar>, ListItemData, Option<Image>);
impl OnEvent for ListItemContent {}

impl ListItemContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<TitleSubtitle>,
        icon_l: Option<&'static str>,
        icon_r: Option<&'static str>,
    ) -> Self {
        let avatar = avatar.map(|data| Avatar::new(ctx, data, None, false, AvatarSize::Md, None));
        let content = ListItemData::new(ctx, left, right);
        let icon_l = icon_l.map(|i| {let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.primary; Icon::new(ctx, i, c, 24.0)});
        let icon_r = icon_r.map(|i| {let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.primary; Icon::new(ctx, i, c, 16.0)});
        ListItemContent(Row::center(16.0), icon_l, avatar, content, icon_r)
    }
}

#[derive(Debug, Component)]
struct ListItemData(Row, LeftData, Option<RightData>);
impl OnEvent for ListItemData {}

impl ListItemData {
    fn new(ctx: &mut Context, left: ListItemInfoLeft, right: Option<TitleSubtitle>) -> Self {
        ListItemData(Row::start(8.0), LeftData::new(ctx, left), right.map(|info| RightData::new(ctx, info)))
    }
}

#[derive(Debug, Component)]
struct LeftData(Column, TitleRow, ExpandableText, Option<ExpandableText>);
impl OnEvent for LeftData {}

impl LeftData {
    pub fn new(ctx: &mut Context, info: ListItemInfoLeft) -> Self {
        let layout = Column::new(4.0, Offset::Start, Size::Fill, Padding::default());
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.xs;
        let subtitle = ExpandableText::new(ctx, &info.title.subtitle, size, TextStyle::Secondary, Align::Left, Some(2));
        let description = info.description.map(|text| ExpandableText::new(ctx, &text, size, TextStyle::Secondary, Align::Left, Some(2)));
        LeftData(layout, TitleRow::new(ctx, &info.title.title, info.flair), subtitle, description)
    }
}

#[derive(Debug, Component)]
struct TitleRow(Row, Text, Option<Image>);
impl OnEvent for TitleRow {}

impl TitleRow {
    fn new(ctx: &mut Context, title: &str, flair: Option<(&'static str, Color)>) -> Self {
        let layout = Row::new(4.0, Offset::Center, Size::Fit, Padding::default());
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.h5;
        let text = Text::new(ctx, title, size, TextStyle::Heading, Align::Left, Some(1));
        let flair = flair.map(|(name, color)| Icon::new(ctx, name, color, 16.0));
        TitleRow(layout, text, flair)
    }
}

#[derive(Debug, Component)]
struct RightData(Column, Text, Text);
impl OnEvent for RightData {}

impl RightData {
    fn new(ctx: &mut Context, info: TitleSubtitle) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let title = Text::new(ctx, &info.title, size.h5, TextStyle::Heading, Align::Left, Some(1));
        let subtitle = Text::new(ctx, &info.subtitle, size.xs, TextStyle::Secondary, Align::Left, Some(2));
        RightData(Column::end(4.0), title, subtitle)
    }
}

pub struct ListItemInfoLeft {
    title: TitleSubtitle,
    flair: Option<(&'static str, Color)>,
    description: Option<String>,
}

impl ListItemInfoLeft {
    pub fn new(title: &str, subtitle: &str, description: Option<&str>, flair: Option<(&'static str, Color)>) -> Self {
        ListItemInfoLeft {
            title: TitleSubtitle::new(title, subtitle),
            description: description.map(|text| text.to_string()),
            flair,
        }
    }
}
