use prism::event::OnEvent;
use prism::drawable::Component;
use prism::canvas::Align;
use prism::Context;
use prism::layout::{Column, Row, Padding, Offset, Size};

use crate::components::text::{Text, ExpandableText, TextStyle, TextSize};
use crate::components::button::{QuickActions, QuickAction};

/// ## Data Item
///
/// A component for presenting information in a clear, structured format.
///  
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/data_item.png"
///      alt="Data Item Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let data = DataItem::text(ctx, 
///     "Confirm Shipping Address",
///     Some("Confirm the address below matches your shipping address."),
///     Some(vec![
///         SecondaryButton::medium(ctx, "edit, "Edit Address", |ctx: &mut Context| println!("Editing address...")),
///     ]),
/// );
///
/// let data = DataItem::table(
///     ctx, "Confirm Shipping Address",
///     Some(vec![
///         ("Street", "123 Feather Ln"),
///         ("City", "Nestville"),
///         ("ZIP", "44556"),
///     ]), 
///     Some(vec![
///         SecondaryButton::medium(ctx, "edit, "Edit Address", |ctx: &mut Context| println!("Editing address...")),
///     ]),
/// );
/// ```
#[derive(Debug, Component)]
pub struct DataItem(Column, Text, Option<ExpandableText>, Option<ExpandableText>, Option<Table>, Option<QuickActions>);
impl OnEvent for DataItem {}

impl DataItem {
    pub fn text(ctx: &mut Context, label: &str, secondary: &str, description: &str, quick_actions: Option<Vec<QuickAction>>) -> Self {
        DataItem(
            Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            Text::new(ctx, label, TextSize::H5, TextStyle::Heading, Align::Left, None),
            Some(ExpandableText::new(ctx, secondary, TextSize::Md, TextStyle::Primary, Align::Left, None)),
            Some(ExpandableText::new(ctx, description, TextSize::Sm, TextStyle::Secondary, Align::Left, None)),
            None, quick_actions.map(|actions| QuickActions::new(ctx, actions))
        )
    }

    pub fn table(ctx: &mut Context, label: &str, table: Vec<(String, String)>, quick_actions: Option<Vec<QuickAction>>) -> Self {
        DataItem(
            Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            Text::new(ctx, label, TextSize::H5, TextStyle::Heading, Align::Left, None),
            None, None, Some(Table::new(ctx, table)),
            quick_actions.map(|actions| QuickActions::new(ctx, actions))
        )
    }
}

#[derive(Debug, Component)]
struct Table(pub Column, pub Vec<Tabular>);
impl OnEvent for Table {}

impl Table {
    pub fn new(ctx: &mut Context, items: Vec<(String, String)>) -> Self {
        Table(Column::center(0.0), items.iter().map(|(name, data)| Tabular::new(ctx, name, data)).collect())
    }
}

#[derive(Debug, Component)]
struct Tabular(Row, ExpandableText, Text);
impl OnEvent for Tabular {}

impl Tabular {
    fn new(ctx: &mut Context, name: &str, data: &str) -> Self {
        Tabular (
            Row::new(8.0, Offset::Start, Size::Fit, Padding(0.0, 4.0, 0.0, 4.0)),
            ExpandableText::new(ctx, name, TextSize::Sm, TextStyle::Primary, Align::Left, Some(1)),
            Text::new(ctx, data, TextSize::Sm, TextStyle::Primary, Align::Left, Some(1)),
        )
    }
}
