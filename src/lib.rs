use futures::{channel::mpsc, lock::Mutex, StreamExt};
use prism::events::ChangeSource;
pub use prism::*;
pub use ptsd::{colors, utils, navigation};
pub use ptsd::{Validate, Validator, Watched};
pub use image;

use std::sync::Arc;

extern crate self as pelican_ui;

pub mod components;
pub mod interface;
pub mod theme;
use theme::Theme;

macro_rules! closure_trait {
    ($name:ident, $($signature:tt)+) => {
        pub trait $name: $($signature)+ + 'static {
            fn clone_box(&self) -> Box<dyn $name>;
        }

        impl<F: $($signature)+ + Clone + 'static> $name for F {
            fn clone_box(&self) -> Box<dyn $name> { Box::new(self.clone()) }
        }

        impl Clone for Box<dyn $name> {
            fn clone(&self) -> Self { self.as_ref().clone_box() }
        }

        impl PartialEq for dyn $name { fn eq(&self, _: &Self) -> bool { true } }

        impl std::fmt::Debug for dyn $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!($name))
            }
        }
    };
}

use closure_trait;

closure_trait!(Callback, FnMut(&mut Context, &Theme));

pub struct Task<T>(Arc<Mutex<mpsc::Receiver<T>>>);
impl<T: Send + 'static> Task<T> {
    pub fn spawn(work: impl FnOnce() -> T + Send + 'static) -> Self {
        let (mut sender, receiver) = mpsc::channel(1);
        std::thread::spawn(move || {
            let _ = sender.try_send(work());
        });
        Self(Arc::new(Mutex::new(receiver)))
    }
}
impl<T> Clone for Task<T> {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}
impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("Task") }
}
impl<T: 'static> ChangeSource for Task<T> {
    type Update = T;
    async fn next(&mut self) -> Option<T> { self.0.lock().await.next().await }
}
