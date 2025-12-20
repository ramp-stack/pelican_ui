use roost_ui::events::OnEvent;
use roost_ui::{Context, Component};
use roost_ui::layouts::{Stack, Column};

use crate::interactions;
use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;


#[derive(Debug, Component)]
pub struct Checkbox(Stack, pub interactions::Selectable);
impl OnEvent for Checkbox {}
impl Checkbox {
    pub fn new(ctx: &mut Context, title: &str, subtitle: Option<String>, is_selected: bool, tag: &str) -> Self {
        let tag = tag.to_string();
        let second_tag = tag.to_string();

        let selected = ListItem::new(ctx, None, ListItemInfoLeft::new(title, subtitle.as_deref(), None, None), None, Some("check"), None,
            move |ctx: &mut Context| ctx.state().set_named(tag.to_string(), "true"),
        );
        
        let default = ListItem::new(ctx,
            None, ListItemInfoLeft::new(title, subtitle.as_deref(), None, None), None, Some("unchecked"), None, 
            move |ctx: &mut Context| ctx.state().set_named(second_tag.to_string(), "false"),
        );

        let selectable = interactions::Selectable::new(default, selected, is_selected, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4());

        Checkbox(Stack::default(), selectable)
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