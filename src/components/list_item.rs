use roost::events::OnEvent;
use roost::drawable::{Align, Color, Image};
use roost::layouts::{Column, Stack, Row, Padding, Offset, Size};
use roost::{Context, Component};

use crate::interactions;
use crate::components::{Rectangle, Icon, Text, TextSize, ExpandableText, TextStyle};
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
use crate::plugin::PelicanUI;
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
///     Some(AvatarContent::Icon("wifi", AvatarIconStyle::Success)),
///     ListItemInfoLeft::new("Wi-Fi", "Home Network", None, None),
///     Some(TitleSubtitle::new("Connected", "Secure, WPA2")),
///     None,
///     None,
///     |ctx: &mut Context| println!("Clicked Wi-Fi")
/// );
/// ```
#[derive(Debug, Component)]
pub struct ListItem(Stack, interactions::Button);
impl OnEvent for ListItem {}

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
        let list_item = _ListItem::new(ctx, avatar, left, right, icon_l, icon_r);
        ListItem(Stack::default(), interactions::Button::new(list_item, None::<_ListItem>, None::<_ListItem>, None::<_ListItem>, false, Box::new(on_click)))
    }
}

#[derive(Component)]
pub struct _ListItem(Stack, Rectangle, ListItemContent);

impl _ListItem {
    pub fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<TitleSubtitle>,
        icon_l: Option<&'static str>,
        icon_r: Option<&'static str>,
    ) -> Self {
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let content = ListItemContent::new(ctx, avatar, left, right, icon_l, icon_r);
        let layout = Stack(Offset::Start, Offset::Center, Size::Fill, Size::custom(|heights: Vec<(f32, f32)>| heights[1]), Padding(0.0, 16.0, 0.0, 16.0));
        _ListItem(layout, Rectangle::new(background, 0.0, None), content)
    }
}

impl OnEvent for _ListItem {}

impl std::fmt::Debug for _ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_ListItem")
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
        let icon_l = icon_l.map(|i| {let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.primary; Icon::new(ctx, i, Some(c), 24.0)});
        let icon_r = icon_r.map(|i| {let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.primary; Icon::new(ctx, i, Some(c), 16.0)});
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
        let subtitle = ExpandableText::new(ctx, &info.title.subtitle, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2));
        let description = info.description.map(|text| ExpandableText::new(ctx, &text, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
        LeftData(layout, TitleRow::new(ctx, &info.title.title, info.flair), subtitle, description)
    }
}

#[derive(Debug, Component)]
struct TitleRow(Row, ExpandableText, Option<Image>);
impl OnEvent for TitleRow {}

impl TitleRow {
    fn new(ctx: &mut Context, title: &str, flair: Option<(&'static str, Color)>) -> Self {
        let layout = Row::new(4.0, Offset::Center, Size::Fit, Padding::default());
        let text = ExpandableText::new(ctx, title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let flair = flair.map(|(name, color)| Icon::new(ctx, name, Some(color), 16.0));
        TitleRow(layout, text, flair)
    }
}

#[derive(Debug, Component)]
struct RightData(Column, Text, Text);
impl OnEvent for RightData {}

impl RightData {
    fn new(ctx: &mut Context, info: TitleSubtitle) -> Self {
        let title = Text::new(ctx, &info.title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let subtitle = Text::new(ctx, &info.subtitle, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2));
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

#[derive(Debug, Component)]
pub struct ListItemGroup(Column, Vec<ListItem>);
impl OnEvent for ListItemGroup {}

impl ListItemGroup {
    pub fn new(items: Vec<ListItem>) -> Self {
        ListItemGroup(Column::start(0.0), items)
    }
}
