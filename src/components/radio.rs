use prism::Context;
use prism::drawable::Component;
use prism::events::{OnEvent, on_event, DynEvent};
use uuid::Uuid;
use ptsd::layout::Column;
use ptsd::interactions;

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::list_item::ListItem;
use crate::components::list_item::ListItemInfoLeft;
use crate::interface::general::FormEvent;

#[derive(Component, Debug, Clone)]
pub struct RadioSelector(Column, pub Vec<interactions::Selectable>, #[skip] Uuid, #[skip] Option<Uuid>);
impl RadioSelector {
    pub fn new(theme: &Theme, index: usize, items: Vec<(&str, &str, Box<dyn Callback>)>) -> Self {
        let group_id = uuid::Uuid::new_v4();
        let selectables = items.into_iter().enumerate().map(|(i, (t, s, mut c))| {
            let title = t.to_string();
            let selected = ListItem::new(
                theme, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None),
                None, Some(Icons::RadioFilled), None, |_, _| {},
            );
            let default = ListItem::new(
                theme, None, ListItemInfoLeft::new(&title.to_string(), Some(s), None, None),
                None, Some(Icons::Radio), None, |_, _| {},
            );

            let theme = theme.clone();
            let callback = Box::new(move |ctx: &mut Context| (c)(ctx, &theme));
            interactions::Selectable::new(default, selected, i == index, false, callback, group_id)
        }).collect::<Vec<_>>();

        RadioSelector(Column::center(0.0), selectables, group_id, None)
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, 0, vec![
            ("Option A", "Press this to select option A", Box::new(|_: &mut Context, _: &Theme| {
                println!("Option A Selected")
            }) as Box<dyn Callback>),
            ("Option B", "Press this to select option B", Box::new(|_: &mut Context, _: &Theme| {
                println!("Option B Selected")
            }) as Box<dyn Callback>),
            ("Option C", "Press this to select option C", Box::new(|_: &mut Context, _: &Theme| {
                println!("Option C Selected")
            }) as Box<dyn Callback>),
        ])
    }

    pub fn value(&self) -> usize {
        for (index, selectable) in self.1.iter().enumerate() {
            if selectable.is_selected() {return index}
        }

        0
    }
}

#[on_event]
impl OnEvent<FormEvent> for RadioSelector {
    fn on(&mut self, _ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Bind(id) = event {self.3 = Some(id);}
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<ptsd::events::Selectable> for RadioSelector {
    fn on(&mut self, ctx: &mut Context, event: ptsd::events::Selectable) -> Vec<Box<dyn DynEvent>> {
        if let ptsd::events::Selectable::Pressed(_, group) = &event
            && *group == self.2.to_string() && let Some(form) = self.3 {
            // Queued events run after every child has applied the selection.
            ctx.emit(FormEvent::Refresh(form));
        }
        prism::events![event]
    }
}
