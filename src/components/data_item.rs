use mustache::events::OnEvent;
use mustache::drawable::{Align};
use mustache::{Context, Component};

use crate::components::{Text, ExpandableText, TextStyle};
use crate::layout::{Column, Row, Padding, Offset, Size, Wrap};
use crate::components::button::SecondaryButton;
use crate::plugin::PelicanUI;
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
/// let data = DataItem::new(
///     ctx, "Confirm Shipping Address",
///     Some("Confirm the address below matches your shipping address."),
///     None,
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
    pub fn new(
        ctx: &mut Context,
        primary: &str,
        secondary: Option<&str>,
        description: Option<&str>,
        table: Option<Vec<(&str, &str)>>,
        quick_actions: Option<Vec<SecondaryButton>>,
    ) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        DataItem(
            Column::new(16.0, Offset::Start, Size::Fill, Padding::default()),
            Text::new(ctx, primary, size.h5, TextStyle::Heading, Align::Left, None),
            secondary.map(|t| ExpandableText::new(ctx, t, size.md, TextStyle::Primary, Align::Left, None)),
            description.map(|t|ExpandableText::new(ctx, t, size.sm, TextStyle::Secondary, Align::Left, None)),
            table.map(|tabulars| Table::new(ctx, tabulars)),
            quick_actions.map(QuickActions::new)
        )
    }
}

#[derive(Debug, Component)]
struct Table(pub Column, pub Vec<Tabular>);
impl OnEvent for Table {}

impl Table {
    pub fn new(ctx: &mut Context, items: Vec<(&str, &str)>) -> Self {
        Table(Column::center(0.0), items.iter().map(|(name, data)| Tabular::new(ctx, name, data)).collect())
    }
}

#[derive(Debug, Component)]
struct Tabular(Row, ExpandableText, Text);
impl OnEvent for Tabular {}

impl Tabular {
    fn new(ctx: &mut Context, name: &str, data: &str) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.sm;
        Tabular (
            Row::new(8.0, Offset::Start, Size::Fit, Padding(0.0, 4.0, 0.0, 4.0)),
            ExpandableText::new(ctx, name, size, TextStyle::Primary, Align::Left, Some(1)),
            Text::new(ctx, data, size, TextStyle::Primary, Align::Left, Some(1)),
        )
    }
}

#[derive(Debug, Component)]
pub struct QuickActions(Wrap, pub Vec<SecondaryButton>); 
impl OnEvent for QuickActions {}

impl QuickActions {
    pub fn new(buttons: Vec<SecondaryButton>) -> Self {
        QuickActions(Wrap::new(8.0, 8.0), buttons)
    }
}