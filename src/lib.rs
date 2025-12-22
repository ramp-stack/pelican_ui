use prism::{Context, drawable::Drawable};
use crate::components::interface::general::Interface;

pub mod components;
pub mod interactions;
pub mod utils;

pub mod theme;
use theme::Theme;

extern crate self as pelican_ui;

pub struct PelicanUI;

impl PelicanUI {
    #[allow(clippy::new_ret_no_self)]
    async fn new(ctx: &mut Context, 
        interface: impl FnOnce(&mut Context) -> Interface, 
        on_tick: impl FnMut(&mut Context) + 'static
    ) -> impl Drawable {
        ctx.state.insert(Theme::default());
        (interface)(ctx).set_on_tick(on_tick)
    }
}

// #[doc(hidden)]
// pub mod __private {
//     pub use roost_ui::start as roost_start;
//     pub use pelican_ui::PelicanUI;
// }

// #[macro_export]
// macro_rules! start {
//     ($app:ty) => {
//         pub use $crate::__private::*;

//         roost_start!(PelicanUI<$app>);
//     };
// }