use prism::event::{OnEvent, Event, TickEvent};
use prism::canvas::{Align, Image};
use prism::layout::{Column, Stack, Row, Padding, Offset, Size};
use prism::drawable::{Component, SizedTree};
use prism::display::EitherOr;
use prism::Context;

use ptsd::interactions;
use ptsd::utils::TitleSubtitle;

use crate::theme::{Theme, Color, Icons};
use crate::components::text::{Text, TextSize, ExpandableText, TextStyle};
use crate::components::Icon;
use crate::components::avatar::{Avatar, AvatarContent, AvatarSize};

use std::sync::Arc;


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
#[derive(Debug, Component, Clone)]
pub struct ListItem(Stack, interactions::Button, #[skip] String);
impl OnEvent for ListItem {}

impl ListItem {
    pub fn new(
        theme: &Theme,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<TitleSubtitle>,
        icon_l: Option<Icons>,
        icon_r: Option<Icons>,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Self {
        let label = left.title.title.to_string();
        let list_item = ListItemContent::new(theme, avatar, left, right, icon_l, icon_r);

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        ListItem(Stack::default(), interactions::Button::new(list_item, None::<ListItemContent>, None::<ListItemContent>, None::<ListItemContent>, callback, false), label)
    }

    pub fn default(theme: &Theme, ) -> Self {
        Self::new(theme, None, ListItemInfoLeft::new("List Item", Some("Click me for details"), None, None), None, None, Some(Icons::Forward), |_: &mut Context, _: &Theme| println!("Pressed..."))
    }

    pub fn title(&self) -> &String {&self.2}
}

#[derive(Debug, Component, Clone)]
struct ListItemContent(Row, Option<Image>, Option<Avatar>, ListItemData, Option<Image>);
impl OnEvent for ListItemContent {}

impl ListItemContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        theme: &Theme, 
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<TitleSubtitle>,
        icon_l: Option<Icons>,
        icon_r: Option<Icons>,
    ) -> Self {
        let c = theme.colors().get(ptsd::Text::Primary); 
        let avatar = avatar.map(|data| Avatar::new(theme, data, None, false, AvatarSize::Md, None));
        let content = ListItemData::new(theme, left, right);
        let icon_l = icon_l.map(|i| Icon::new(theme, i, Some(c), 24.0));
        let icon_r = icon_r.map(|i| Icon::new(theme, i, Some(c), 16.0));
        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        ListItemContent(layout, icon_l, avatar, content, icon_r)
    }
}

#[derive(Debug, Component, Clone)]
struct ListItemData(Row, LeftData, Option<RightData>);
impl OnEvent for ListItemData {}

impl ListItemData {
    fn new(theme: &Theme, left: ListItemInfoLeft, right: Option<TitleSubtitle>) -> Self {
        ListItemData(Row::start(8.0), LeftData::new(theme, left), right.map(|info| RightData::new(theme, info)))
    }
}

#[derive(Debug, Component, Clone)]
struct LeftData(Column, TitleRow, Option<ExpandableText>, Option<ExpandableText>);
impl OnEvent for LeftData {}

impl LeftData {
    pub fn new(theme: &Theme, info: ListItemInfoLeft) -> Self {
        let layout = Column::new(4.0, Offset::Start, Size::Fill, Padding::default(), None);
        let subtitle = info.title.subtitle.map(|s| ExpandableText::new(theme, &s, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
        let description = info.description.map(|text| ExpandableText::new(theme, &text, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
        LeftData(layout, TitleRow::new(theme, &info.title.title, info.flair), subtitle, description)
    }
}

#[derive(Debug, Component, Clone)]
struct TitleRow(Row, ExpandableText, Option<Image>);
impl OnEvent for TitleRow {}

impl TitleRow {
    fn new(theme: &Theme, title: &str, flair: Option<(Icons, Color)>) -> Self {
        let layout = Row::new(4.0, Offset::Center, Size::Fit, Padding::default());
        let text = ExpandableText::new(theme, title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let flair = flair.map(|(name, color)| Icon::new(theme, name, Some(color), 16.0));
        TitleRow(layout, text, flair)
    }
}

#[derive(Debug, Component, Clone)]
struct RightData(Column, Text, Option<Text>);
impl OnEvent for RightData {}

impl RightData {
    fn new(theme: &Theme, info: TitleSubtitle) -> Self {
        let title = Text::new(theme, &info.title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let subtitle = info.subtitle.map(|s| Text::new(theme, &s, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)));
        RightData(Column::end(4.0), title, subtitle)
    }
}

#[derive(Clone, Debug)]
pub struct ListItemInfoLeft {
    title: TitleSubtitle,
    flair: Option<(Icons, Color)>,
    description: Option<String>,
}

impl ListItemInfoLeft {
    pub fn new(title: &str, subtitle: Option<&str>, description: Option<&str>, flair: Option<(Icons, Color)>) -> Self {
        ListItemInfoLeft {
            title: TitleSubtitle::new(title, subtitle),
            description: description.map(|text| text.to_string()),
            flair,
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct ListItemGroup(Column, Option<ExpandableText>, Vec<ListItem>);
impl OnEvent for ListItemGroup {}

impl ListItemGroup {
    pub fn new(theme: &Theme, label: Option<String>, items: Vec<ListItem>) -> Self {
        let text = label.as_ref().map(|l| ExpandableText::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None));
        ListItemGroup(Column::start(16.0), text, items)
    }
}


#[derive(Debug, Component, Clone)]
pub struct ListItemSection(Stack, EitherOr<ExpandableText, ListItemGroup>, #[skip] Arc<Box<dyn ListItemGetter>>);
impl OnEvent for ListItemSection {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let new_items = (self.2)(ctx);
            self.1.display_left(new_items.is_empty());
            self.0 = match new_items.is_empty() {
                true => Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default()),
                false => Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fill, Padding::default()),
            };
            self.1.right().2 = new_items;
        }
        vec![event]
    }
}

impl ListItemSection {
    pub fn new(theme: &Theme, label: Option<String>, instructions: Option<String>, item_getter: impl ListItemGetter + 'static) -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default());
        let instructions = ExpandableText::new(theme, &instructions.unwrap_or_default(), TextSize::Md, TextStyle::Secondary, Align::Center, None);
        ListItemSection(layout, EitherOr::new(instructions, ListItemGroup::new(theme, label, vec![])), Arc::new(Box::new(item_getter)))
    }

    // pub fn group(&mut self) -> &mut ListItemGroup { &mut self.2.left() }
}

pub trait ListItemGetter: Fn(&mut Context) -> Vec<ListItem> + 'static {
}

impl<F> ListItemGetter for F where F: Fn(&mut Context) -> Vec<ListItem> + Clone + 'static {

}



impl std::fmt::Debug for dyn ListItemGetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListItemGetter")
    }
}
