use prism::event::OnEvent;
use prism::canvas::{Align, Image};
use prism::layout::{Column, Stack, Row, Padding, Offset, Size};
use prism::drawable::Component;
use prism::Context;

use crate::{interactions, Theme, theme::Color};
use crate::components::text::{Text, TextSize, ExpandableText, TextStyle};
use crate::components::Icon;
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};
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
        let list_item = ListItemContent::new(ctx, avatar, left, right, icon_l, icon_r);
        ListItem(Stack::default(), interactions::Button::new(list_item, None::<ListItemContent>, None::<ListItemContent>, None::<ListItemContent>, false, Box::new(on_click), None))
    }

    pub fn default(ctx: &mut Context) -> Self {
        Self::new(ctx, None, ListItemInfoLeft::new("List Item", Some("Click me for details"), None, None), None, None, Some("forward"), |_: &mut Context| println!("Pressed..."))
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
        let icon_l = icon_l.map(|i| {let c = ctx.state.get_or_default::<Theme>().colors.text.primary; Icon::new(ctx, i, Some(c), 24.0)});
        let icon_r = icon_r.map(|i| {let c = ctx.state.get_or_default::<Theme>().colors.text.primary; Icon::new(ctx, i, Some(c), 16.0)});
        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        ListItemContent(layout, icon_l, avatar, content, icon_r)
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
struct LeftData(Column, TitleRow, Option<ExpandableText>, Option<ExpandableText>);
impl OnEvent for LeftData {}

impl LeftData {
    pub fn new(ctx: &mut Context, info: ListItemInfoLeft) -> Self {
        let layout = Column::new(4.0, Offset::Start, Size::Fill, Padding::default(), None);
        let subtitle = info.title.subtitle.map(|s| ExpandableText::new(ctx, &s, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
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
struct RightData(Column, Text, Option<Text>);
impl OnEvent for RightData {}

impl RightData {
    fn new(ctx: &mut Context, info: TitleSubtitle) -> Self {
        let title = Text::new(ctx, &info.title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let subtitle = info.subtitle.map(|s| Text::new(ctx, &s, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
        RightData(Column::end(4.0), title, subtitle)
    }
}

pub struct ListItemInfoLeft {
    title: TitleSubtitle,
    flair: Option<(&'static str, Color)>,
    description: Option<String>,
}

impl ListItemInfoLeft {
    pub fn new(title: &str, subtitle: Option<&str>, description: Option<&str>, flair: Option<(&'static str, Color)>) -> Self {
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


#[derive(Debug, Component)]
pub struct ListItemSection(Column, Option<ExpandableText>, ListItemGroup);
impl OnEvent for ListItemSection {}
impl ListItemSection {
    pub fn new(ctx: &mut Context, label: Option<String>, items: Vec<ListItem>) -> Self {
        let text = label.as_ref().map(|l| ExpandableText::new(ctx, l, TextSize::H5, TextStyle::Heading, Align::Left, None));
        ListItemSection(Column::center(16.0), text, ListItemGroup::new(items))
    }

    pub fn group(&mut self) -> &mut ListItemGroup { &mut self.2 }
}

