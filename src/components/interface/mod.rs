mod general;
pub use general::{Interface, Page, Content, Bumper, Header};
mod interfaces;
pub use interfaces::ShowKeyboard;
mod system;
mod navigation;
pub use navigation::{RootInfo, AppPage};
