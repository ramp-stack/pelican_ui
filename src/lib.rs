use crate::components::interface::Interface;

use prism::drawable::{Drawable, Component};
use prism::event::OnEvent;
use prism::layout::Stack;

use std::hash::Hash;
use std::fmt::Debug;

pub mod components;
pub mod interactions;
pub mod utils;

pub mod theme;
use theme::Theme;

pub use prism::*;

extern crate self as pelican_ui;

pub struct PelicanUI;

impl PelicanUI {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(interface: impl FnOnce(&Theme) -> Interface) -> Interface {
        (interface)(&Theme::default())
    }
}

#[derive(Debug, Component)]
pub struct Listener<D: Drawable, T: Hash + Debug + Clone + 'static>(Stack, prism::Listener<D, T>);
impl<D: Drawable, T: Hash + Debug + Clone + 'static> OnEvent for Listener<D, T> {}
impl<D: Drawable, T: Hash + Debug + Clone + 'static> Listener<D, T> {
    pub fn new(ctx: &mut Context, theme: &Theme, inner: D, updated_on: impl Fn(&mut Context, &Theme, &mut D, T) + 'static) -> Self {
        let theme = theme.clone();
        let updated_on = move |ctx: &mut Context, inner: &mut D, other: T| (updated_on)(ctx, &theme.clone(), inner, other);
        Listener(Stack::default(), prism::Listener::new(ctx, inner, updated_on))
    }
}


// #[derive(Clone, Debug)]
// pub struct BrandResources {
//     pub wordmark: Arc<RgbaImage>,
//     pub logo: Arc<RgbaImage>,
//     pub app_icon: Arc<RgbaImage>,
//     pub error: Arc<RgbaImage>,
// }

// impl Default for BrandResources {
//     fn default() -> Self {
//         let dir = include_dir!("resources/brand");
//         BrandResources {
//             logo: Arc::new(load_svg(&load_file(&dir, "logo.svg").unwrap())),
//             wordmark: Arc::new(load_svg(&load_file(&dir, "wordmark.svg").unwrap())),
//             app_icon: Arc::new(load_svg(&load_file(&dir, "app_icon.svg").unwrap())),
//             error: Arc::new(load_svg(&load_file(&dir, "error.svg").unwrap())),
//         }
//     }
// }


// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Variant { Primary, Secondary, Ghost }
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum State { Default, Hover, Pressed, Disabled }
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Slot { Background, Label, Outline }
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct Button(Variant, State, Slot);
// impl ColorType for Button {}

// impl Button {
//     fn map(theme: ThemeStyle, brand: Color) -> HashMap<dyn ColorType, Color> {
//         use Slot::*;
//         use State::*;
//         use Variant::*;

//         let mut primary = HashMap::from(vec![
//             (Button(Primary, Default, Background), brand),
//             (Button(Primary, Default, Label), brand.contrasted()),
//             (Button(Primary, Default, Outline), Color::TRANSPARENT),
//             (Button(Primary, Hover, Background), brand.darken(0.75)),
//             (Button(Primary, Hover, Label), brand.contrasted()),
//             (Button(Primary, Hover, Outline), Color::TRANSPARENT),
//             (Button(Primary, Pressed, Background),brand.darken(0.7)),
//             (Button(Primary, Pressed, Label), brand.contrasted()),
//             (Button(Primary, Pressed, Outline), Color::TRANSPARENT),
//             (Button(Primary, Disabled, Background), Color::from_hex("#443f3f", 255)),
//             (Button(Primary, Disabled, Label), Color::BLACK),
//             (Button(Primary, Disabled, Outline), Color::TRANSPARENT),
//         ]);

//         let secondary = match theme {
//             ThemeStyle::Light => HashMap::from(vec![
//                 (Button(Secondary, Default, Background), Color::TRANSPARENT),
//                 (Button(Secondary, Default, Label), Color::BLACK),
//                 (Button(Secondary, Default, Outline), Color::from_hex("#585250", 255)),
//                 (Button(Secondary, Hover, Background), Color::from_hex("#DDDDDD", 255)),
//                 (Button(Secondary, Hover, Label), Color::BLACK),
//                 (Button(Secondary, Hover, Outline), Color::from_hex("#585250", 255)),
//                 (Button(Secondary, Pressed, Background), Color::from_hex("#DDDDDD", 255)),
//                 (Button(Secondary, Pressed, Label), Color::BLACK),
//                 (Button(Secondary, Pressed, Outline), Color::BLACK),
//                 (Button(Secondary, Disabled, Background), Color::from_hex("#443f3f", 255)),
//                 (Button(Secondary, Disabled, Label), Color::BLACK),
//                 (Button(Secondary, Disabled, Outline), Color::from_hex("#585250", 255)),
//             ]),
//             ThemeStyle::Dark => HashMap::from(vec![
//                 (Button(Secondary, Default, Background), Color::TRANSPARENT),
//                 (Button(Secondary, Default, Label), Color::WHITE),
//                 (Button(Secondary, Default, Outline), Color::from_hex("#585250", 255)),
//                 (Button(Secondary, Hover, Background), Color::from_hex("#262322", 255)),
//                 (Button(Secondary, Hover, Label), Color::WHITE),
//                 (Button(Secondary, Hover, Outline), Color::from_hex("#585250", 255)),
//                 (Button(Secondary, Pressed, Background), Color::from_hex("#262322", 255)),
//                 (Button(Secondary, Pressed, Label), Color::BLACK),
//                 (Button(Secondary, Pressed, Outline), Color::WHITE),
//                 (Button(Secondary, Disabled, Background), Color::from_hex("#443f3f", 255)),
//                 (Button(Secondary, Disabled, Label), Color::BLACK),
//                 (Button(Secondary, Disabled, Outline), Color::from_hex("#585250", 255)),
//             ])
//         };

//         let ghost = match theme {
//             ThemeStyle::Light => HashMap::from(vec![
//                 (Button(Ghost, Default, Background), Color::TRANSPARENT),
//                 (Button(Ghost, Default, Label), Color::BLACK),
//                 (Button(Ghost, Default, Outline), Color::TRANSPARENT),
//                 (Button(Ghost, Hover, Background), Color::from_hex("#DDDDDD", 255)),
//                 (Button(Ghost, Hover, Label), Color::BLACK),
//                 (Button(Ghost, Hover, Outline), Color::TRANSPARENT),
//                 (Button(Ghost, Pressed, Background), Color::from_hex("#DDDDDD", 255)),
//                 (Button(Ghost, Pressed, Label), Color::BLACK),
//                 (Button(Ghost, Pressed, Outline), Color::TRANSPARENT),
//                 (Button(Ghost, Disabled, Background), Color::from_hex("#443f3f", 255)),
//                 (Button(Ghost, Disabled, Label), Color::BLACK),
//                 (Button(Ghost, Disabled, Outline), Color::TRANSPARENT),
//             ]),
//             ThemeStyle::Dark => HashMap::from(vec![
//                 (Button(Ghost, Default, Background), Color::TRANSPARENT),
//                 (Button(Ghost, Default, Label), Color::WHITE),
//                 (Button(Ghost, Default, Outline), Color::TRANSPARENT,),
//                 (Button(Ghost, Hover, Background), Color::from_hex("#262322", 255)),
//                 (Button(Ghost, Hover, Label), Color::WHITE),
//                 (Button(Ghost, Hover, Outline), Color::TRANSPARENT),
//                 (Button(Ghost, Pressed, Background), Color::from_hex("#262322", 255)),
//                 (Button(Ghost, Pressed, Label), Color::BLACK),
//                 (Button(Ghost, Pressed, Outline), Color::TRANSPARENT),
//                 (Button(Ghost, Disabled, Background), Color::from_hex("#443f3f", 255)),
//                 (Button(Ghost, Disabled, Label), Color::BLACK),
//                 (Button(Ghost, Disabled, Outline), Color::TRANSPARENT),
//             ])
//         };

//         primary.extend(secondary);
//         primary.extend(ghost);
//         primary
//     }
// }
