use prism::event::OnEvent;
use prism::Context;
use prism::drawable::Component;
use prism::layout::{Stack, Column};

use crate::interactions;
use crate::utils::Callback;
use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;
use crate::theme::Theme;

#[derive(Debug, Component)]
pub struct Checkbox(Stack, pub interactions::Selectable);
impl OnEvent for Checkbox {}
impl Checkbox {
    pub fn new(theme: &Theme, title: &str, subtitle: Option<String>, is_selected: bool, on_check: Callback, on_uncheck: Callback) -> Self {
        let selected = ListItem::new(theme, None, ListItemInfoLeft::new(title, subtitle.as_deref(), None, None), None, Some("check"), None, on_uncheck);
        let default = ListItem::new(theme, None, ListItemInfoLeft::new(title, subtitle.as_deref(), None, None), None, Some("unchecked"), None, on_check);

        let selectable = interactions::Selectable::new(default, selected, is_selected, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4());

        Checkbox(Stack::default(), selectable)
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "Checkbox", None, false, Box::new(|_| println!("Checked")), Box::new(|_| println!("Un-checked")))
    }
}

#[derive(Debug, Component)]
pub struct CheckboxList(Column, Vec<Checkbox>);
impl OnEvent for CheckboxList {}
impl CheckboxList {
    pub fn new(items: Vec<Checkbox>) -> Self {
        CheckboxList(Column::center(0.0), items)
    }
}