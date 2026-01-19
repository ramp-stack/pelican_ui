use prism::event::OnEvent;
use prism::Context;
use prism::drawable::Component;
use prism::layout::Column;

use crate::interactions;
use crate::utils::Callback;
use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;

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
#[derive(Component, Debug)]
pub struct RadioSelector(Column, pub Vec<interactions::Selectable>);
impl OnEvent for RadioSelector {}
impl RadioSelector {
    pub fn new(ctx: &mut Context, index: usize, items: Vec<(&str, &str, Callback)>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let selectables = items.into_iter().enumerate().map(|(i, (t, s, c))| {
            let title = t.to_string();
            let selected = ListItem::new(ctx, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None), None, Some("radio_filled"), None, |_| {});
            let default = ListItem::new(ctx, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None), None, Some("radio"), None, |_| {});

            interactions::Selectable::new(default, selected, i == index, false, c, group_id)
        }).collect::<Vec<_>>();


        RadioSelector(Column::center(0.0), selectables)
    }

    pub fn default(ctx: &mut Context) -> Self {
        Self::new(ctx, 0, vec![
            ("Option A", "Press this to select option A", Box::new(|_: &mut Context| println!("Option A Selected")) as Callback),
            ("Option B", "Press this to select option B", Box::new(|_: &mut Context| println!("Option B Selected")) as Callback),
            ("Option C", "Press this to select option C", Box::new(|_: &mut Context| println!("Option C Selected")) as Callback)
        ])
    }
}
