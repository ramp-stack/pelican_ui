use pelican_ui::{
    Align, Area, Color, Component, Context,
    Drawable, Event, Image, Layout,
    MouseEvent, MouseState, OnEvent,
    SizeRequest,
};

use crate::events::ListItemSelect;
use crate::components::{Rectangle, Icon, Text, ExpandableText, TextStyle};
use crate::components::avatar::{Avatar, AvatarContent};
use crate::components::button::ButtonState;
use crate::layout::{Column, Stack, Row, Padding, Offset, Size, Opt};
use crate::utils::ElementID;

/// ## List Item
///
/// A versatile row-style component used for lists, menus, or settings screens.  
/// Supports a title, optional flair (badge), subtitle, description, right-aligned  
/// content, radio buttons, and avatar/circle icons.  
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/list_item.png"
///      alt="List Item Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let item = ListItem::new(
///     ctx,
///     true,
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
pub struct ListItem(Stack, Rectangle, ListItemContent, #[skip] ButtonState, #[skip] Box<dyn FnMut(&mut Context)>, #[skip] Option<ElementID>, #[skip] bool);

impl ListItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &mut Context,
        caret: bool,
        title: &str,
        flair: Option<(&'static str, Color)>,
        subtitle: Option<&str>,
        description: Option<&str>,
        right_title: Option<&str>,
        right_subtitle: Option<&str>,
        radio_button: Option<bool>,
        circle_icon: Option<AvatarContent>,
        element_id: Option<ElementID>,
        max_lines: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let color = ctx.theme.colors.background.primary;
        let content = ListItemContent::new(
            ctx, caret, title, flair, subtitle, description, right_title, 
            right_subtitle, radio_button, circle_icon, max_lines
        );
        let layout = Stack(
            Offset::Start, Offset::Center, 
            Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, f32::MAX)), 
            Size::custom(|heights: Vec<(f32, f32)>| heights[1]), 
            Padding(0.0, 16.0, 0.0, 16.0)
        );

        ListItem(layout, Rectangle::new(color, 0.0, None), content, ButtonState::Default, Box::new(on_click), element_id, false)
    }

    pub fn title(&mut self) -> &mut TitleRow {self.2.data().left().title()}
    pub fn subtitle(&mut self) -> &mut Option<ExpandableText> {self.2.data().left().subtitle()}
    pub fn avatar(&mut self) -> &mut Option<Avatar> {&mut self.2.2}

    pub fn is_selected(&self) -> bool {
        self.2.1.as_ref().map(|r| r.2).unwrap_or(false)
    }
}

impl OnEvent for ListItem {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                self.6 = true;
            } else if let MouseEvent{state: MouseState::Released, position: Some(_)} = event {
                if self.6 {
                    if let Some(radio) = self.2.1.as_mut() {
                        radio.select(ctx);
                        ctx.trigger_event(ListItemSelect(self.5.expect("Selectable List Items Require ElementIDs")));
                    }
                    match self.3 {
                        ButtonState::Default | ButtonState::Hover | ButtonState::Pressed => {
                            ctx.hardware.haptic();
                            (self.4)(ctx)
                        },
                        _ => {}
                    }
                    self.6 = false;
                }
            }
        } else if let Some(ListItemSelect(id)) = event.downcast_ref::<ListItemSelect>() {
            if let Some(self_id) = &self.5 {
                if *id != *self_id {
                    if let Some(radio) = self.2.1.as_mut() {
                        radio.deselect(ctx);
                    }
                }
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
struct ListItemContent(Row, Option<RadioButton>, Option<Avatar>, ListItemData, Option<Image>);
impl OnEvent for ListItemContent {}

impl ListItemContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        caret: bool,
        title: &str,
        flair: Option<(&'static str, Color)>,
        subtitle: Option<&str>,
        description: Option<&str>,
        right_title: Option<&str>,
        right_subtitle: Option<&str>,
        radio_button: Option<bool>,
        circle_icon: Option<AvatarContent>,
        max_lines: bool,
    ) -> Self {
        let color = ctx.theme.colors.text.secondary;
        ListItemContent(
            Row::new(16.0, Offset::Center, Size::Fit, Padding::default()),
            radio_button.map(|enabled| RadioButton::new(ctx, enabled)), 
            circle_icon.map(|data| Avatar::new(ctx, data, None, false, 48.0, None)),
            ListItemData::new(ctx, title, flair, subtitle, description, right_title, right_subtitle, max_lines),
            caret.then(|| Icon::new(ctx, "forward", color, 16.0)),
        )
    }

    fn data(&mut self) -> &mut ListItemData {&mut self.3}
}

#[derive(Debug, Component)]
struct RadioButton(Row, Image, #[skip] bool); // is selected
impl OnEvent for RadioButton {}

impl RadioButton {
    fn new(ctx: &mut Context, is_enabled: bool) -> Self {
        let color = ctx.theme.colors.text.heading;
        let icon = if is_enabled { "radio_filled" } else { "radio"};
        RadioButton(Row::center(0.0), Icon::new(ctx, icon, color, 32.0), is_enabled)
    }

    fn select(&mut self, ctx: &mut Context) {
        self.2 = true;
        let color = ctx.theme.colors.text.heading;
        self.1 =  Icon::new(ctx, "radio_filled", color, 32.0);
    }

    fn deselect(&mut self, ctx: &mut Context) {
        self.2 = false;
        let color = ctx.theme.colors.text.heading;
        self.1 =  Icon::new(ctx, "radio", color, 32.0);
    }
}

#[derive(Debug, Component)]
struct ListItemData(Row, LeftData, Option<RightData>);
impl OnEvent for ListItemData {}

impl ListItemData {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        title: &str,
        flair: Option<(&'static str, Color)>,
        subtitle: Option<&str>,
        description: Option<&str>,
        right_title: Option<&str>,
        right_subtitle: Option<&str>,
        max_lines: bool,
    ) -> Self {
        ListItemData(
            Row::new(8.0, Offset::Start, Size::Fit, Padding::default()),
            LeftData::new(ctx, title, flair, subtitle, description, max_lines),
            right_title.map(|r_title| RightData::new(ctx, r_title, right_subtitle)), 
        )
    }

    fn left(&mut self) -> &mut LeftData {&mut self.1}
}

#[derive(Debug, Component)]
pub struct TitleRow(Row, Text, Option<Image>);
impl OnEvent for TitleRow {}

impl TitleRow {
    fn new(ctx: &mut Context, title: &str, flair: Option<(&'static str, Color)>) -> Self {
        let font_size = ctx.theme.fonts.size.h5;
        TitleRow(
            Row::new(4.0, Offset::Center, Size::Fit, Padding::default()),
            Text::new(ctx, title, TextStyle::Heading, font_size, Align::Left),
            flair.map(|(name, color)| Icon::new(ctx, name, color, 16.0)),
        )
    }

    pub fn update_flair(&mut self, ctx: &mut Context, flair: Option<(&'static str, Color)>) {
        self.2 = flair.map(|(i, c)| Icon::new(ctx, i, c, 20.0))
    }

    pub fn title(&mut self) -> &mut Text {&mut self.1}
}

#[derive(Debug, Component)]
struct LeftData(Column, TitleRow, Option<ExpandableText>, Option<ExpandableText>);
impl OnEvent for LeftData {}

impl LeftData {
    pub fn new(
        ctx: &mut Context,
        title: &str,
        flair: Option<(&'static str, Color)>,
        subtitle: Option<&str>,
        description: Option<&str>,
        max_lines: bool,
    ) -> Self {
        let max = max_lines.then_some(2);
        let font_size = ctx.theme.fonts.size.xs;
        LeftData (
            Column::new(4.0, Offset::Start, Size::custom(|widths: Vec<(f32, f32)>| (widths[0].0, f32::MAX)), Padding::default()),
            TitleRow::new(ctx, title, flair),
            subtitle.map(|text| ExpandableText::new(ctx, text, TextStyle::Secondary, font_size, Align::Left, max)),
            description.map(|text| ExpandableText::new(ctx, text, TextStyle::Secondary, font_size, Align::Left, max)),
        )
    }

    fn title(&mut self) -> &mut TitleRow {&mut self.1}
    fn subtitle(&mut self) -> &mut Option<ExpandableText> {&mut self.2}
}

#[derive(Debug, Component)]
struct RightData(Column, Text, Option<Text>);
impl OnEvent for RightData {}

impl RightData {
    pub fn new(ctx: &mut Context, title: &str, subtitle: Option<&str>) -> Self {
        let font_size = ctx.theme.fonts.size;
        RightData (
            Column::new(4.0, Offset::End, Size::Fit, Padding::default()),
            Text::new(ctx, title, TextStyle::Heading, font_size.h5, Align::Left),
            subtitle.map(|text| Text::new(ctx, text, TextStyle::Secondary, font_size.xs, Align::Left)),
        )
    }
}

impl ListItem {  
    /// Creates a list item for a radio selection.
    /// Displays a title, subtitle, and description, and supports a selected state.
    pub fn selection(
        ctx: &mut Context,
        selected: bool,
        title: &str,
        subtitle: &str,
        description: Option<&str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        ListItem::new(ctx, false, title, None, Some(subtitle), description, None, None, Some(selected), None, Some(ElementID::new()), false, on_click)
    }
}

/// ## List Item Selector
///
/// The first item is always marked as **selected**.  
/// The second item is always present but unselected.  
/// The third and fourth items are optional, and if provided, will also be unselected.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/selector.png"
///      alt="Selector Example"
///      width="400">
///
/// # Example
/// ```rust
/// let selector = ListItemSelector::new(
///     ctx,
///     ("Standard Shipping", "3–5 business days", Some("Free")),
///     ("Express Shipping", "1–2 business days", Some("$9.99")),
///     Some(("Overnight Shipping", "Arrives Tomorrow", Some("$19.99"))),
///     None,
/// );
/// ```
#[derive(Debug, Component)]
pub struct ListItemSelector(
    Column,       // The layout column for organizing the items vertically.
    ListItem,     // The first list item (selected).
    ListItem,     // The second list item (unselected).
    Option<ListItem>,  // The third list item (optional, unselected).
    Option<ListItem>,  // The fourth list item (optional, unselected).
);

impl OnEvent for ListItemSelector {}
impl ListItemSelector {
    pub fn new(
        ctx: &mut Context, 
        first: (&str, &str, Option<&str>), // title, subtitle, description
        second: (&str, &str, Option<&str>), 
        third: Option<(&str, &str, Option<&str>)>, 
        fourth: Option<(&str, &str, Option<&str>)>
    ) -> Self {
        ListItemSelector(
            Column::center(0.0), 
            ListItem::selection(ctx, true, first.0, first.1, first.2, |_: &mut Context| ()),
            ListItem::selection(ctx, false, second.0, second.1, second.2, |_: &mut Context| ()),
            third.map(|third| ListItem::selection(ctx, false, third.0, third.1, third.2, |_: &mut Context| ())),
            fourth.map(|fourth| ListItem::selection(ctx, false, fourth.0, fourth.1, fourth.2, |_: &mut Context| ())),
        )
    }

    /// Returns the index of the selected list item.
    pub fn index(&self) -> Option<u8> {
        if self.1.is_selected() { return Some(0) }
        if self.2.is_selected() { return Some(1) }
        if self.3.as_ref().map(|s| s.is_selected()).unwrap_or(false) { return Some(2) }
        if self.4.as_ref().map(|s| s.is_selected()).unwrap_or(false) { return Some(3) }
        None
    }
}

/// # List Item Group
/// A group of [`ListItem`]s arranged in a vertical [`Column`].
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/li_group.png"
///      alt="List Group Example"
///      width="400">
///
/// # Example
///
/// ```rust
/// let wifi = ListItem::new(
///     ctx,
///     true,
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
///
/// let bluetooth = ListItem::new(
///     ctx,
///     true,
///     "Bluetooth",
///     None,
///     None,
///     Some("Connect to accessories"),
///     Some("On")
///     Some("7 Devices"),
///     None,
///     Some(AvatarContent::Icon("bluetooth", AvatarIconStyle::Primary)),
///     None,
///     false,
///     |ctx: &mut Context| println!("Clicked Bluetooth")
/// );
///
/// let mut group = ListItemGroup::new(vec![wifi, bluetooth]);
/// ```
#[derive(Debug, Component)]
pub struct ListItemGroup(Column, Vec<Opt<ListItem>>);
impl OnEvent for ListItemGroup {}

impl ListItemGroup {
    pub fn new(list_items: Vec<ListItem>) -> Self {
        let list_items = list_items.into_iter().map(|item| Opt::new(item, true)).collect();
        ListItemGroup(Column::center(0.0), list_items)
    }

    /// Returns a vector of optional list items. 
    pub fn inner(&mut self) -> &mut Vec<Opt<ListItem>> {&mut self.1}

    /// Hide or show an item in the list.
    pub fn hide(&mut self, hide: bool, i: usize) {
        if let Some(item) = self.inner().get_mut(i) {item.display(!hide);}
    }
}
