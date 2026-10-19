pub mod components;
pub mod interface;

pub mod theme;
use theme::Theme;

pub use prism::*;
pub use ptsd::{colors, utils, navigation};

pub use image;

use std::sync::{Arc, Mutex};

extern crate self as pelican_ui;

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
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", stringify!($name)) }
        }
    };
}

use closure_trait;

closure_trait!(Callback, FnMut(&mut Context, &Theme));

pub struct Task<T>(Arc<Mutex<Option<T>>>);

impl<T: Send + 'static> Task<T> {
    pub fn spawn(work: impl FnOnce() -> T + Send + 'static) -> Self {
        let slot = Arc::new(Mutex::new(None));
        let result = slot.clone();
        std::thread::spawn(move || {
            let value = work();
            if let Ok(mut slot) = result.lock() { *slot = Some(value); }
        });
        Task(slot)
    }

    pub fn take(&self) -> Option<T> { self.0.lock().ok()?.take() }
}

impl<T> Clone for Task<T> {
    fn clone(&self) -> Self { Task(self.0.clone()) }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Task") }
}

mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize<T: AsRef<[u8]> + ?Sized, S: Serializer>(bytes: &Arc<T>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes.as_ref().as_ref()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: From<Vec<u8>>>(deserializer: D) -> Result<T, D::Error> {
        let text = String::deserialize(deserializer)?;
        STANDARD.decode(text).map(T::from).map_err(serde::de::Error::custom)
    }
}
