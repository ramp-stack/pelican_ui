use prism::event::OnEvent;
use prism::Context;
use prism::drawable::Component;
use prism::layout::{Stack, Column};

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
            move |ctx: &mut Context| {ctx.state.insert(CheckboxState(tag.to_string(), "true".to_string()));},
        );
        
        let default = ListItem::new(ctx,
            None, ListItemInfoLeft::new(title, subtitle.as_deref(), None, None), None, Some("unchecked"), None, 
            move |ctx: &mut Context| {ctx.state.insert(CheckboxState(second_tag.to_string(), "false".to_string()));},
        );

        let selectable = interactions::Selectable::new(default, selected, is_selected, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4());

        Checkbox(Stack::default(), selectable)
    }

    pub fn default(ctx: &mut Context) -> Self {
        Self::new(ctx, "Checkbox", None, false, "checkbox")
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

pub struct CheckboxState(String, String);