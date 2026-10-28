use prism::air::Name;
use prism::events::{Event, DynEvent, OnEvent, on_event};
use uuid::Uuid;
use prism::Context;
use prism::drawable::Component;
use ptsd::layout::{Wrap, Stack, Column};
use ptsd::interactions;

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::{TextInput, TextInputEvent};
use crate::components::list_item::ListItem;
use crate::components::button::{SecondaryButton, QuickActions};
use crate::interface::general::FormEvent;

#[derive(Clone, Debug)]
pub enum SearchbarEvent {
    Remove(Name),
    Select(Name),
    Relist(ListItem, Name),
    Store(ListItem, Name),
}

impl Event for SearchbarEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchBar(Column, TextInput, Option<QuickActions>, SelectedItems, SearchableItems);
#[on_event]
impl OnEvent<TextInputEvent> for SearchBar {
    fn on(&mut self, _ctx: &mut Context, event: TextInputEvent) -> Vec<Box<dyn DynEvent>> {
        if let TextInputEvent::Changed {id, value} = &event && *id == self.1.id() {
            self.4.sort(value);
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<FormEvent> for SearchBar {
    fn on(&mut self, _ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Bind(id) = event {
            self.3.3 = Some(id);
            return vec![];
        }
        prism::events![event]
    }
}

impl SearchBar {
    pub fn new(theme: &Theme, id: Uuid, label: &str, items: Vec<(ListItem, Name)>, actions: Option<QuickActions>) -> Self {
        let input = TextInput::new(
            theme, id, None, None, Some(&format!("Search {}...", label.to_lowercase())),
            None, None,
        );
        SearchBar(
            Column::start(12.0), input, actions, SelectedItems::new(theme, vec![]),
            SearchableItems::new(theme, items),
        )
    }

    pub fn results(&self) -> Vec<Name> {self.3.1.iter().map(|p| p.2).collect::<Vec<_>>()}
}

#[derive(Debug, Component, Clone)]
pub struct SelectedItems(Wrap, Vec<SearchPill>, #[skip] Theme, #[skip] Option<Uuid>);
impl SelectedItems {
    pub fn new(theme: &Theme, items: Vec<SearchBarListItem>) -> Self {
        SelectedItems(Wrap::start(8.0, 8.0), items.into_iter().map(|item| {
            let id = item.id();
            SearchPill::new(SecondaryButton::medium(
                theme, Icons::Close, &item.3.0.clone(), None, Stack::default(),
                move |ctx, _| ctx.emit(SearchbarEvent::Remove(id)),
            ), item.4.clone(), item.id())
        }).collect::<Vec<_>>(), theme.clone(), None)
    }
}

#[on_event]
impl OnEvent<SearchbarEvent> for SelectedItems {
    fn on(&mut self, ctx: &mut Context, event: SearchbarEvent) -> Vec<Box<dyn DynEvent>> {
        match &event {
            SearchbarEvent::Remove(id) => {
                let id = *id;
                if let Some(pos) = self.1.iter().position(|l| l.2 == id) {
                    let removed = self.1.remove(pos);
                    ctx.emit(SearchbarEvent::Store(removed.3, id));
                    if let Some(form) = self.3 {ctx.emit(FormEvent::Refresh(form));}
                }
            },
            SearchbarEvent::Relist(item, id) => {
                let id = *id;
                self.1.push(SearchPill::new(
                    SecondaryButton::medium(
                        &self.2, Icons::Close, &item.title().clone(), None, Stack::default(),
                        move |ctx, _| ctx.emit(SearchbarEvent::Remove(id)),
                    ), item.clone(), id,
                ));
                if let Some(form) = self.3 {ctx.emit(FormEvent::Refresh(form));}
            },
            _ => {}
        }
        prism::events![event]
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchableItems(Column, Vec<SearchBarListItem>, #[skip] Theme);
impl SearchableItems {
    pub fn new(theme: &Theme, items: Vec<(ListItem, Name)>) -> Self {
        SearchableItems(Column::default(), items.into_iter().map(|(item, id)| {
            SearchBarListItem::new(theme, item, Box::new(move |ctx: &mut Context, _theme: &Theme| {
                ctx.emit(SearchbarEvent::Select(id))
            }), id)
        }).collect::<Vec<_>>(), theme.clone())
    }

    pub fn sort(&mut self, val: &str) {
        let val = &val.to_lowercase();
        self.1.sort_by_key(|item| {
            let title = &item.3.0.to_lowercase();
            let subtitle = &item.3.1.to_lowercase();

            match title == val {
                true => 0,
                false if title.starts_with(val) => 1,
                false if title.contains(val) => 2,
                false => match subtitle == val {
                    true => 0,
                    false if subtitle.starts_with(val) => 1,
                    false if subtitle.contains(val) => 2,
                    false => 3
                }
            }
        });
    }
}

#[on_event]
impl OnEvent<SearchbarEvent> for SearchableItems {
    fn on(&mut self, ctx: &mut Context, event: SearchbarEvent) -> Vec<Box<dyn DynEvent>> {
        match &event {
            SearchbarEvent::Select(id) => {
                let id = *id;
                if let Some(pos) = self.1.iter().position(|l| l.2 == id) {
                    let removed = self.1.remove(pos);
                    ctx.emit(SearchbarEvent::Relist(removed.4, id));
                } else {
                    let mut title = id.to_string();
                    let len = title.chars().count();

                    if len >= 20 {
                        let chars: Vec<char> = title.chars().collect();

                        let start: String = chars.iter().take(7).collect();
                        let end: String = chars.iter().rev().take(3).collect::<Vec<_>>()
                            .iter().rev().copied().collect();

                        title = format!("{}...{}", start, end);
                    }

                    let fallback_item = ListItem::default(&self.2, title.clone());
                    ctx.emit(SearchbarEvent::Relist(fallback_item, id));
                }
            }
            SearchbarEvent::Store(item, id) => {
                let id = *id;
                self.1.push(SearchBarListItem::new(
                    &self.2, item.clone(), Box::new(move |ctx: &mut Context, _theme: &Theme| {
                        ctx.emit(SearchbarEvent::Select(id))
                    }), id,
                ))
            },
            _ => {}
        }
        prism::events![event]
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchPill(Stack, SecondaryButton, #[skip] Name, #[skip] ListItem);
impl SearchPill {
    pub fn new(button: SecondaryButton, item: ListItem, id: Name) -> Self {
        SearchPill(Stack::default(), button, id, item)
    }
}

#[derive(Component, Debug, Clone)]
pub struct SearchBarListItem(
    Stack, pub interactions::Button, #[skip] Name, #[skip] (String, String), #[skip] ListItem,
);
impl SearchBarListItem {
    pub fn new(theme: &Theme, item: ListItem, mut callback: Box<dyn Callback>, id: Name) -> Self {
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (callback)(ctx, &theme));
        let button = interactions::Button::new_triggers_on_release(
            item.clone(), None::<ListItem>, None::<ListItem>, None::<ListItem>, None::<ListItem>,
            callback, false,
        );
        SearchBarListItem(
            Stack::default(), button, id, (item.title().to_string(), item.subtitle().to_string()), item,
        )
    }

    pub fn id(&self) -> Name {self.2}
}
