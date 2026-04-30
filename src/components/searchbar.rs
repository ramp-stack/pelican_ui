use prism::event::{Event, TickEvent, OnEvent};
use prism::{Request, Context};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Wrap, Stack, Column};

use ptsd::interactions;

use crate::Callback;
use crate::theme::{Theme, Icons};
use crate::components::TextInput;
use crate::components::list_item::ListItem;
use crate::components::button::SecondaryButton;

use air::names::Name;

#[derive(Debug, Component, Clone)]
pub struct SearchBar(Column, TextInput, SelectedItems, SearchableItems);
impl OnEvent for SearchBar {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() { self.3.sort(&self.1.value()); }
        vec![event]
    }
}

impl SearchBar {
    pub fn new(theme: &Theme, items: Vec<(ListItem, Name)>) -> Self {
        let input = TextInput::new(theme, None, None, None, None, None);
        SearchBar(Column::start(12.0), input, SelectedItems::new(theme, vec![]), SearchableItems::new(theme, items))
    }

    pub fn results(&self) -> Vec<Name> {
        self.2.1.iter().map(|p| p.2).collect::<Vec<_>>()
    }
}

#[derive(Debug, Component, Clone)]
pub struct SelectedItems(Wrap, Vec<SearchPill>, #[skip] Theme);
impl OnEvent for SelectedItems {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<SearchbarEvent>() {
            match event {
                SearchbarEvent::Remove(id) => {
                    let id = id.clone();
                    if let Some(pos) = self.1.iter().position(|l| l.2 == id) {
                        let removed = self.1.remove(pos);
                        ctx.emit(SearchbarEvent::Store(removed.3, id));
                    }
                },
                SearchbarEvent::Relist(item, id) => {
                    let id = id.clone();
                    self.1.push(SearchPill::new(
                        SecondaryButton::medium(&self.2, Icons::Close, &item.title().clone(), None, move |ctx: &mut Context, _theme: &Theme| {
                            ctx.emit(SearchbarEvent::Remove(id))
                        }), item.clone(), id
                    ))
                },
                _ => {}
            }
        }
        vec![event]
    }
}

impl SelectedItems {
    pub fn new(theme: &Theme, items: Vec<SearchBarListItem>) -> Self {
        SelectedItems(Wrap::start(8.0, 8.0), items.into_iter().map(|item| {
            let id = item.id();
            SearchPill::new(SecondaryButton::medium(theme, Icons::Close, &item.3.clone(), None, move |ctx: &mut Context, _theme: &Theme| {
                ctx.emit(SearchbarEvent::Remove(id))
            }), item.4.clone(), item.id())
        }).collect::<Vec<_>>(), theme.clone())
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchableItems(Column, Vec<SearchBarListItem>, #[skip] Theme);
impl OnEvent for SearchableItems {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<SearchbarEvent>() {
            match event {
                SearchbarEvent::Select(id) => {
                    let id = id.clone();
                    if let Some(pos) = self.1.iter().position(|l| l.2 == id) {
                        let removed = self.1.remove(pos);
                        ctx.emit(SearchbarEvent::Relist(removed.4, id))
                    }
                },
                SearchbarEvent::Store(item, id) => {
                    let id = id.clone();
                    self.1.push(SearchBarListItem::new(&self.2, item.clone(), Box::new(move |ctx: &mut Context, _theme: &Theme| {
                        ctx.emit(SearchbarEvent::Select(id))
                    }), id))
                },
                _ => {}
            }
        }
        vec![event]
    }
}

impl SearchableItems {
    pub fn new(theme: &Theme, items: Vec<(ListItem, Name)>) -> Self {
        SearchableItems(Column::default(), items.into_iter().map(|(item, id)| 
            SearchBarListItem::new(theme, item, Box::new(move |ctx: &mut Context, _theme: &Theme| {
                ctx.emit(SearchbarEvent::Select(id))
            }), id)).collect::<Vec<_>>(), theme.clone()
        )
    }

    pub fn sort(&mut self, val: &str) {
        let val = &val.to_lowercase();
        self.1.sort_by_key(|item| {
            let s = &item.3.to_lowercase();

            match s == val {
                true => 0,
                false if s.starts_with(val) => 1,
                false if s.contains(val) => 2,
                false => 3
            }
        });
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchPill(Stack, SecondaryButton, #[skip] Name, #[skip] ListItem);
impl OnEvent for SearchPill {}
impl SearchPill {
    pub fn new(button: SecondaryButton, item: ListItem, id: Name) -> Self {
        SearchPill(Stack::default(), button, id, item)
    }
}

#[derive(Clone, Debug)]
pub enum SearchbarEvent {
    Remove(Name),
    Select(Name),
    Relist(ListItem, Name),
    Store(ListItem, Name),
}

impl Event for SearchbarEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Component, Debug, Clone)]
pub struct SearchBarListItem(Stack, pub interactions::Button, #[skip] Name, #[skip] String, #[skip] ListItem);
impl OnEvent for SearchBarListItem {}
impl SearchBarListItem {
    pub fn new(theme: &Theme, item: ListItem, mut callback: Box<dyn Callback>, id: Name) -> Self {
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (callback)(ctx, &theme));
        let button = interactions::Button::new(item.clone(), None::<ListItem>, None::<ListItem>, None::<ListItem>, callback, false);
        SearchBarListItem(Stack::default(), button, id, item.title().to_string(), item)
    }

    pub fn id(&self) -> Name {self.2}
}

