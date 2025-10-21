use mustache::events::OnEvent;
use mustache::{Context, Component};

use crate::layout::Column;
use crate::utils::Callback;
use crate::utils::ElementID;

use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;
use crate::components::interactions;

/// A column of selectable radio-style list items.
///
/// # Example
///
///<img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/selector.png"
///      alt="Radio Example"
///      width="400">
///
/// ```rust
/// let selector = RadioSelector::new(ctx, 0, vec![
///     ("Light Mode", "Bright theme with light background", |_| println!("Selected Light Mode")),
///     ("Dark Mode", "Dim theme for low-light environments", |_| println!("Selected Dark Mode")),
///     ("System Default", "Match the system appearance setting", |_| println!("Selected System Default")),
/// ]);
/// ```
#[derive(Debug, Component)]
pub struct RadioSelector(Column, pub Vec<interactions::Selectable>);
impl OnEvent for RadioSelector {}
impl RadioSelector {
    pub fn new(ctx: &mut Context, index: usize, items: Vec<(&str, &str, Callback)>) -> Self {
        let group_id = ElementID::new();
        let selectables = items.into_iter().enumerate().map(|(i, (t, s, c))| {
            let selected = ListItem::new(ctx, None, ListItemInfoLeft::new(t, s, None, None), None, Some("radio_filled"), None, |_| {});
            let default = ListItem::new(ctx, None, ListItemInfoLeft::new(t, s, None, None), None, Some("radio"), None, |_| {});

            interactions::Selectable::new(c, default, selected, i == index, group_id)
        }).collect::<Vec<_>>();

        RadioSelector(Column::center(0.0), selectables)
    }
}