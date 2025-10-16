use mustache::events::OnEvent;
use mustache::{Context, Component};

use crate::layout::Column;
use crate::utils::Callback;
use crate::utils::ElementID;
use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;
use crate::components::interactions;

#[derive(Debug, Component)]
pub struct RadioSelector(Column, pub Vec<interactions::Selectable>);
impl OnEvent for RadioSelector {}
impl RadioSelector {
    pub fn new(ctx: &mut Context, index: usize, items: Vec<(&str, &str, Option<&str>, Callback)>) -> Self {
        let group_id = ElementID::new();
        let selectables = items.into_iter().enumerate().map(|(i, (t, s, d, c))| {
            let selected = ListItem::new(ctx, None, ListItemInfoLeft::new(t, s, d, None), None, Some("radio_filled"), None, |_| {});
            let default = ListItem::new(ctx, None, ListItemInfoLeft::new(t, s, d, None), None, Some("radio"), None, |_| {});

            interactions::Selectable::new(c, default, selected, i == index, group_id)
        }).collect::<Vec<_>>();

        RadioSelector(Column::center(0.0), selectables)
    }
}