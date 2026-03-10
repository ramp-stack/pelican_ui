use prism::event::{Event, TickEvent, OnEvent};
use prism::{Request, Context};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Wrap, Stack, Column};

use ptsd::interactions;

use crate::Callback;
use crate::theme::Theme;
use crate::components::TextInput;
use crate::components::list_item::ListItem;
use crate::components::button::SecondaryButton;

#[derive(Debug, Component, Clone)]
pub struct SearchBar(Column, TextInput, SelectedItems, SearchableItems);
impl OnEvent for SearchBar {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() { self.3.sort(&self.1.value()); }
        vec![event]
    }
}

impl SearchBar {
    pub fn new(theme: &Theme, items: Vec<ListItem>) -> Self {
        SearchBar(Column::start(12.0), TextInput::default(theme), SelectedItems::new(theme, vec![]), SearchableItems::new(theme, items))
    }

    pub fn results(&self) -> Vec<ListItem> {
        self.2.1.iter().map(|p| p.3.clone()).collect::<Vec<_>>()
    }
}

#[derive(Debug, Component, Clone)]
pub struct SelectedItems(Wrap, Vec<SearchPill>, #[skip] Theme);
impl OnEvent for SelectedItems {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<SearchbarEvent>() {
            match event {
                SearchbarEvent::Remove(i) => if let Some(pos) = self.1.iter().position(|item| item.2 == *i) {
                    let removed = self.1.remove(pos);
                    ctx.send(Request::event(SearchbarEvent::Store(removed.3)))
                },
                SearchbarEvent::Relist(item) => {
                    let i = self.1.len();
                    self.1.push(SearchPill::new(
                        SecondaryButton::medium(&self.2, "close", &item.title().clone(), None, move |ctx: &mut Context, _theme: &Theme| {
                            ctx.send(Request::event(SearchbarEvent::Remove(i)))
                        }), i, item.clone()
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
        SelectedItems(Wrap::start(8.0, 8.0), items.into_iter().enumerate().map(|(i, item)| 
            SearchPill::new(SecondaryButton::medium(theme, "close", &item.3.clone(), None, move |ctx: &mut Context, _theme: &Theme| {
                ctx.send(Request::event(SearchbarEvent::Remove(i)))
            }), i, item.4.clone())).collect::<Vec<_>>(), theme.clone()
        )
    }
}

#[derive(Debug, Component, Clone)]
pub struct SearchableItems(Column, Vec<SearchBarListItem>, #[skip] Theme);
impl OnEvent for SearchableItems {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<SearchbarEvent>() {
            match event {
                SearchbarEvent::Select(i) => if let Some(pos) = self.1.iter().position(|item| item.2 == *i) {
                    let removed = self.1.remove(pos);
                    ctx.send(Request::event(SearchbarEvent::Relist(removed.4)))
                },
                SearchbarEvent::Store(item) => {
                    let i = self.1.len();
                    self.1.push(SearchBarListItem::new(&self.2, item.clone(), Box::new(move |ctx: &mut Context, _theme: &Theme| {
                        ctx.send(Request::event(SearchbarEvent::Select(i)))
                    }), i))
                },
                _ => {}
            }
        }
        vec![event]
    }
}

impl SearchableItems {
    pub fn new(theme: &Theme, items: Vec<ListItem>) -> Self {
        SearchableItems(Column::default(), items.into_iter().enumerate().map(|(i, item)| 
            SearchBarListItem::new(theme, item, Box::new(move |ctx: &mut Context, _theme: &Theme| {
                ctx.send(Request::event(SearchbarEvent::Select(i)))
            }), i)).collect::<Vec<_>>(), theme.clone()
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
pub struct SearchPill(Stack, SecondaryButton, #[skip] usize, #[skip] ListItem);
impl OnEvent for SearchPill {}
impl SearchPill {
    pub fn new(button: SecondaryButton, i: usize, item: ListItem) -> Self {
        SearchPill(Stack::default(), button, i, item)
    }
}

#[derive(Clone, Debug)]
pub enum SearchbarEvent {
    Remove(usize),
    Select(usize),
    Relist(ListItem),
    Store(ListItem),
}

impl Event for SearchbarEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Component, Debug, Clone)]
pub struct SearchBarListItem(Stack, pub interactions::Button, #[skip] usize, #[skip] String, #[skip] ListItem);
impl OnEvent for SearchBarListItem {}
impl SearchBarListItem {
    pub fn new(theme: &Theme, item: ListItem, mut callback: Box<dyn Callback>, i: usize) -> Self {
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (callback)(ctx, &theme));
        let button = interactions::Button::new(item.clone(), None::<ListItem>, None::<ListItem>, None::<ListItem>, callback, false);
        SearchBarListItem(Stack::default(), button, i, item.title().to_string(), item)
    }
}

