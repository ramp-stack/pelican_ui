use mustache::events::{OnEvent, MouseState, MouseEvent, Event};
use mustache::drawable::{Align, Color, Image};
use mustache::{Context, Component};

use crate::components::{Rectangle, Icon, Text, ExpandableText, TextStyle};
use crate::components::avatar::{Avatar, AvatarContent};
use crate::components::interactions::ButtonState;
use crate::layout::{Column, Stack, Row, Padding, Offset, Size};
use crate::plugin::PelicanUI;
use crate::utils::Callback;
use crate::utils::ElementID;

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
///     "Wi-Fi",
///     None,
///     None,
///     Some("Home Network"),
///     Some("Connected")
///     Some("Secure, WPA2"),
///     None,
///     Some(AvatarContent::Icon("wifi", AvatarIconStyle::Success)),
///     None,
///     false,
///     |ctx: &mut Context| println!("Clicked Wi-Fi")
/// );
/// ```
#[derive(Component)]
pub struct ListItem(Stack, Rectangle, ListItemContent, #[skip] ButtonState, #[skip] Callback, #[skip] bool);
impl std::fmt::Debug for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListItem")
    }
}

pub struct ListItemInfoLeft {
    title: String,
    subtitle: String,
    flair: Option<(&'static str, Color)>,
    description: Option<String>,
}

impl ListItemInfoLeft {
    pub fn new(title: &str, subtitle: &str, description: Option<&str>, flair: Option<(&'static str, Color)>) -> Self {
        ListItemInfoLeft {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            description: description.map(|text| text.to_string()),
            flair,
        }
    }
}

pub struct ListItemInfoRight {
    title: String,
    subtitle: String,
}

impl ListItemInfoRight {
    pub fn new(title: &str, subtitle: &str) -> Self {
        ListItemInfoRight {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
        }
    }
}

impl ListItem {
    pub fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<ListItemInfoRight>,
        icon: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.primary;
        let content = ListItemContent::new(ctx, avatar, left, right, icon);
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

#[derive(Debug, Component)]
struct ListItemContent(Row, Option<Avatar>, ListItemData, Option<Image>);
impl OnEvent for ListItemContent {}

impl ListItemContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        left: ListItemInfoLeft,
        right: Option<ListItemInfoRight>,
        icon: Option<&'static str>,
    ) -> Self {
        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding::default());
        let avatar = avatar.map(|data| Avatar::new(ctx, data, None, false, 48.0, None));
        let content = ListItemData::new(ctx, left, right);
        let icon = icon.map(|i| {let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.primary; Icon::new(ctx, i, c, 16.0)});
        ListItemContent(layout, avatar, content, icon)
    }
}

#[derive(Debug, Component)]
struct ListItemData(Row, LeftData, Option<RightData>);
impl OnEvent for ListItemData {}

impl ListItemData {
    fn new(ctx: &mut Context, left: ListItemInfoLeft, right: Option<ListItemInfoRight>) -> Self {
        let layout = Row::new(8.0, Offset::Start, Size::Fit, Padding::default());
        ListItemData(layout, LeftData::new(ctx, left), right.map(|info| RightData::new(ctx, info)))
    }
}

#[derive(Debug, Component)]
struct LeftData(Column, TitleRow, ExpandableText, Option<ExpandableText>);
impl OnEvent for LeftData {}

impl LeftData {
    pub fn new(ctx: &mut Context, info: ListItemInfoLeft) -> Self {
        let layout = Column::new(4.0, Offset::Start, Size::Fill, Padding::default());
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.xs;
        let subtitle = ExpandableText::new(ctx, &info.subtitle, size, TextStyle::Secondary, Align::Left, Some(2));
        let description = info.description.map(|text| ExpandableText::new(ctx, &text, size, TextStyle::Secondary, Align::Left, Some(2)));
        LeftData(layout, TitleRow::new(ctx, &info.title, info.flair), subtitle, description)
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
    fn new(ctx: &mut Context, info: ListItemInfoRight) -> Self {
        let layout = Column::new(4.0, Offset::End, Size::Fit, Padding::default());
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let title = Text::new(ctx, &info.title, size.h5, TextStyle::Heading, Align::Left, Some(1));
        let subtitle = Text::new(ctx, &info.subtitle, size.xs, TextStyle::Secondary, Align::Left, Some(2));
        RightData(layout, title, subtitle)
    }
}

/// Selects the [`ListItem`] with the given [`ElementID`] and deselects all other items.
#[derive(Debug, Clone)]
pub struct ListItemSelect(pub ElementID);

impl Event for ListItemSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


// /// ## List Item Selector
// ///
// /// The first item is always marked as **selected**.  
// /// The second item is always present but unselected.  
// /// The third and fourth items are optional, and if provided, will also be unselected.
// ///
// /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/selector.png"
// ///      alt="Selector Example"
// ///      width="400">
// ///
// /// # Example
// /// ```rust
// /// let selector = ListItemSelector::new(
// ///     ctx,
// ///     ("Standard Shipping", "3–5 business days", Some("Free")),
// ///     ("Express Shipping", "1–2 business days", Some("$9.99")),
// ///     Some(("Overnight Shipping", "Arrives Tomorrow", Some("$19.99"))),
// ///     None,
// /// );
// /// ```
// #[derive(Debug, Component)]
// pub struct ListItemSelector(
//     Column,       // The layout column for organizing the items vertically.
//     ListItem,     // The first list item (selected).
//     ListItem,     // The second list item (unselected).
//     Option<ListItem>,  // The third list item (optional, unselected).
//     Option<ListItem>,  // The fourth list item (optional, unselected).
// );

// impl OnEvent for ListItemSelector {}
// impl ListItemSelector {
//     pub fn new(
//         ctx: &mut Context, 
//         first: (&str, &str, Option<&str>), // title, subtitle, description
//         second: (&str, &str, Option<&str>), 
//         third: Option<(&str, &str, Option<&str>)>, 
//         fourth: Option<(&str, &str, Option<&str>)>
//     ) -> Self {
//         ListItemSelector(
//             Column::center(0.0), 
//             ListItem::selection(ctx, true, first.0, first.1, first.2, |_: &mut Context| ()),
//             ListItem::selection(ctx, false, second.0, second.1, second.2, |_: &mut Context| ()),
//             third.map(|third| ListItem::selection(ctx, false, third.0, third.1, third.2, |_: &mut Context| ())),
//             fourth.map(|fourth| ListItem::selection(ctx, false, fourth.0, fourth.1, fourth.2, |_: &mut Context| ())),
//         )
//     }

//     /// Returns the index of the selected list item.
//     pub fn index(&self) -> Option<u8> {
//         if self.1.is_selected() { return Some(0) }
//         if self.2.is_selected() { return Some(1) }
//         if self.3.as_ref().map(|s| s.is_selected()).unwrap_or(false) { return Some(2) }
//         if self.4.as_ref().map(|s| s.is_selected()).unwrap_or(false) { return Some(3) }
//         None
//     }
// }

// /// # List Item Group
// /// A group of [`ListItem`]s arranged in a vertical [`Column`].
// ///
// /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/li_group.png"
// ///      alt="List Group Example"
// ///      width="400">
// ///
// /// let mut group = ListItemGroup::new(vec![item1, item2]);
// /// ```
// #[derive(Debug, Component)]
// pub struct ListItemGroup(Column, Vec<Opt<ListItem>>);
// impl OnEvent for ListItemGroup {}

// impl ListItemGroup {
//     pub fn new(list_items: Vec<ListItem>) -> Self {
//         let list_items = list_items.into_iter().map(|item| Opt::new(item, true)).collect();
//         ListItemGroup(Column::center(0.0), list_items)
//     }

//     /// Returns a vector of optional list items. 
//     pub fn inner(&mut self) -> &mut Vec<Opt<ListItem>> {&mut self.1}

//     /// Hide or show an item in the list.
//     pub fn hide(&mut self, hide: bool, i: usize) {
//         if let Some(item) = self.inner().get_mut(i) {item.display(!hide);}
//     }
// }
