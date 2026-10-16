#![allow(clippy::type_complexity)]

use prism::event::OnEvent;
use prism::drawable::Component;
use prism::canvas::Align;
use prism::layout::{Column, Row, Padding, Offset, Size};
use ptsd::theme::TextSize;
use crate::components::text::{Text, ExpandableText, TextStyle};
use crate::components::button::{QuickActions, ActionData};
use crate::theme::Theme;

#[derive(Debug, Component, Clone)]
pub struct DataItem(Column, Text, Option<ExpandableText>, Option<ExpandableText>, Option<Table>, Option<QuickActions>);
impl OnEvent for DataItem {}

impl DataItem {
    pub fn text(theme: &Theme, label: &str, secondary: Option<&str>, description: &str, quick_actions: Option<Vec<ActionData>>) -> Self {
        DataItem(
            Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            Text::new(theme, label, TextSize::H5, TextStyle::Heading, Align::Left, None),
            secondary.map(|s| ExpandableText::new(theme, s, TextSize::Md, TextStyle::Primary, Align::Left, None)),
            Some(ExpandableText::new(theme, description, TextSize::Sm, TextStyle::Secondary, Align::Left, None)),
            None, quick_actions.map(|actions| QuickActions::new(theme, Offset::Start, actions))
        )
    }

    pub fn table(theme: &Theme, label: &str, table: Vec<(String, String)>, quick_actions: Option<Vec<ActionData>>) -> Self {
        DataItem(
            Column::new(16.0, Offset::Start, Size::Fill, Padding::default(), None),
            Text::new(theme, label, TextSize::H5, TextStyle::Heading, Align::Left, None),
            None, None, Some(Table::new(theme, table)),
            quick_actions.map(|actions| QuickActions::new(theme, Offset::Start, actions))
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Table(pub Column, pub Vec<Tabular>);
impl OnEvent for Table {}

impl Table {
    pub fn new(theme: &Theme, items: Vec<(String, String)>) -> Self {
        Table(Column::center(0.0), items.iter().map(|(name, data)| Tabular::new(theme, name, data)).collect())
    }
}

#[derive(Debug, Component, Clone)]
struct Tabular(Row, ExpandableText, Text);
impl OnEvent for Tabular {}

impl Tabular {
    fn new(theme: &Theme, name: &str, data: &str) -> Self {
        Tabular (
            Row::new(8.0, Offset::Start, Size::Fit, Padding(0.0, 4.0, 0.0, 4.0)),
            ExpandableText::new(theme, name, TextSize::Sm, TextStyle::Primary, Align::Left, Some(1)),
            Text::new(theme, data, TextSize::Sm, TextStyle::Primary, Align::Left, Some(1)),
        )
    }
}
