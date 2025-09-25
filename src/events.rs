use mustache::events::Event;
use mustache::Context;
use crate::utils::ElementID;

/// Event used to navigate between pages of the app.
#[derive(Debug, Clone)]
pub struct NavigateEvent(pub usize);

impl Event for NavigateEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event used to bring up or hide the keyboard.
#[derive(Debug, Clone)]
pub struct KeyboardActiveEvent(pub Option<bool>);

impl Event for KeyboardActiveEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Clears the contents of the active text input.
#[derive(Debug, Clone)]
pub struct ClearActiveInput;

impl Event for ClearActiveInput {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Sets the contents of the active [`TextInput`] with the provided `String`
#[derive(Debug, Clone)]
pub struct SetActiveInput(pub String);

impl Event for SetActiveInput {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Selects the [`TextInput`] with the given [`ElementID`] and deselects all other items.
#[derive(Debug, Clone)]
pub struct TextInputSelect(pub ElementID);

impl Event for TextInputSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Selects the [`ListItem`] with the given [`ElementID`] and deselects all other items.
#[derive(Debug, Clone)]
pub struct ListItemSelect(pub ElementID);

impl Event for ListItemSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Selects the [`NavigationButton`] with the given [`ElementID`] and deselects all other items.
#[derive(Debug, Clone)]
pub struct NavigatorSelect(pub ElementID);

impl Event for NavigatorSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Navigates to the page at the given `index`. See [`AppPage`] for details on navigation.
#[derive(Debug, Clone)]
pub struct NavigatorEvent(pub usize);

impl Event for NavigatorEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event triggered by the [`Searchbar`] component when its contents are edited and the input field is Focused.
#[derive(Debug, Clone)]
pub struct SearchEvent(pub String);

impl Event for SearchEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event trigger by [`TextInput`] when contents are edited. 
#[derive(Debug, Clone)]
pub struct InputEditedEvent;

impl Event for InputEditedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Adjust the scroll value of a [`Scroll`] layout.
#[derive(Debug, Clone)]
pub enum AdjustScrollEvent {
    Vertical(f32),
    Horizontal(f32),
}

impl Event for AdjustScrollEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event triggered when the [`QRScanner`] component detects a QR code.
#[derive(Debug, Clone)]
pub struct QRCodeScannedEvent(pub String);

impl Event for QRCodeScannedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentEvent(pub String);

impl Event for AttachmentEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}