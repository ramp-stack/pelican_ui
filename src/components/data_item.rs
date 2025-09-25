use mustache::events::OnEvent;
use mustache::drawable::{Color, Drawable, Component, Align, Shape};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::components::{Text, ExpandableText, TextStyle, Circle, Rectangle};
use crate::components::button::Button;
use crate::layout::{Column, Bin, Row, Stack, Padding, Offset, Size};

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
///     ctx,
///     Some("1"),
///     "Confirm Shipping Address",
///     Some("Confirm the address below matches your shipping address."),
///     None,
///     Some(vec![
///         ("Street", "123 Feather Ln"),
///         ("City", "Nestville"),
///         ("ZIP", "44556"),
///     ]), 
///     Some(vec![
///         Button::secondary(ctx, Some("edit"), "Edit Address", None, |ctx: &mut Context| println!("Editing address...")),
///     ]),
/// );
/// ```
#[derive(Debug, Component)]
pub struct DataItem(Row, Option<Number>, DataItemContent);
impl OnEvent for DataItem {}

impl DataItem {
    pub fn new(
        ctx: &mut Context,
        number: Option<&str>,
        label: &str,
        text: Option<&str>,
        secondary: Option<&str>,
        table: Option<Vec<(&str, &str)>>,
        quick_actions: Option<Vec<Button>>,
    ) -> Self {
        DataItem (
            Row::new(32.0, Offset::Start, Size::Fit, Padding::default()),
            number.map(|n| Number::new(ctx, n)),
            DataItemContent::new(ctx, label, text, secondary, table, quick_actions)
        )
    }

    pub fn label(&mut self) -> &mut String {
        &mut self.2.1.text().spans[0].text
    }

    pub fn buttons(&mut self) -> Option<&mut Vec<Button>> {
        self.2.5.as_mut().map(|qa| qa.buttons())
    }
}

#[derive(Debug, Component)]
struct Number(Stack, Shape, Text);
impl OnEvent for Number {}

impl Number {
    pub fn new(ctx: &mut Context, txt: &str) -> Self {
        let theme = &ctx.theme;
        let (color, font_size) = (theme.colors.background.secondary, theme.fonts.size.h5);
        Number(
            Stack::center(),
            Circle::new(32.0, color, false),
            Text::new(ctx, txt, TextStyle::Heading, font_size, Align::Left), 
        )
    }
}

#[derive(Debug, Component)]
struct DataItemContent(Column, Text, Option<ExpandableText>, Option<ExpandableText>, Option<Table>, Option<QuickActions>);
impl OnEvent for DataItemContent {}

impl DataItemContent {
    fn new(
        ctx: &mut Context,
        label: &str,
        text: Option<&str>,
        secondary: Option<&str>,
        table: Option<Vec<(&str, &str)>>,
        quick_actions: Option<Vec<Button>>,
    ) -> Self {
        let font_size = ctx.theme.fonts.size;
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, f32::MAX));
        DataItemContent(
            Column::new(16.0, Offset::Start, width, Padding::default()),
            Text::new(ctx, label, TextStyle::Heading, font_size.h5, Align::Left),
            text.map(|t| ExpandableText::new(ctx, t, TextStyle::Primary, font_size.md, Align::Left, None)),
            secondary.map(|t|ExpandableText::new(ctx, t, TextStyle::Secondary, font_size.sm, Align::Left, None)),
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
        Table (
            Column::new(0.0, Offset::Start, Size::Fit, Padding::default()),
            items.iter().map(|(name, data)| Tabular::new(ctx, name, data)).collect()
        )
    }
}

#[derive(Debug, Component)]
struct Tabular(Row, Text, Bin<Stack, Rectangle>, Text);
impl OnEvent for Tabular {}

impl Tabular {
    fn new(ctx: &mut Context, name: &str, data: &str) -> Self {
        let font_size = ctx.theme.fonts.size.sm;
        Tabular (
            Row::new(8.0, Offset::Start, Size::Fit, Padding(0.0, 4.0, 0.0, 4.0)),
            Text::new(ctx, name, TextStyle::Primary, font_size, Align::Left),
            Bin(
                Stack(Offset::Center, Offset::Center, Size::Fit, Size::Static(1.0), Padding::default()),
                Rectangle::new(Color::TRANSPARENT, 0.0, None),
            ),
            Text::new(ctx, data, TextStyle::Primary, font_size, Align::Left),
        )
    }
}

#[derive(Debug, Component)]
struct QuickActions(Row, Vec<Button>); // Row should be wrap
impl OnEvent for QuickActions {}

impl QuickActions {
    fn new(buttons: Vec<Button>) -> Self {
        QuickActions(Row::new(8.0, Offset::Start, Size::Fit, Padding::default()), buttons)
    }

    fn buttons(&mut self) -> &mut Vec<Button> {&mut self.1}
}