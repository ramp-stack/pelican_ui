use prism::event::OnEvent;
use prism::Context;
use prism::drawable::Component;
use prism::layout::Column;

use ptsd::interactions;

use crate::Callback;
use crate::theme::Theme;
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
#[derive(Component, Debug, Clone)]
pub struct RadioSelector(Column, pub Vec<interactions::Selectable>);
impl OnEvent for RadioSelector {}
impl RadioSelector {
    pub fn new(theme: &Theme, index: usize, items: Vec<(&str, &str, Box<dyn Callback>)>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let selectables = items.into_iter().enumerate().map(|(i, (t, s, mut c))| {
            let title = t.to_string();
            let selected = ListItem::new(theme, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None), None, Some("radio_filled"), None, |_, _| {});
            let default = ListItem::new(theme, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None), None, Some("radio"), None, |_, _| {});

            let theme = theme.clone();
            let callback = Box::new(move |ctx: &mut Context| (c)(ctx, &theme));
            interactions::Selectable::new(default, selected, i == index, false, callback, group_id)
        }).collect::<Vec<_>>();


        RadioSelector(Column::center(0.0), selectables)
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, 0, vec![
            ("Option A", "Press this to select option A", Box::new(|_: &mut Context, _: &Theme| println!("Option A Selected")) as Box<dyn Callback>),
            ("Option B", "Press this to select option B", Box::new(|_: &mut Context, _: &Theme| println!("Option B Selected")) as Box<dyn Callback>),
            ("Option C", "Press this to select option C", Box::new(|_: &mut Context, _: &Theme| println!("Option C Selected")) as Box<dyn Callback>)
        ])
    }
}

