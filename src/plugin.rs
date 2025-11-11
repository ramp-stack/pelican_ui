use roost_ui::{Context, Plugin, include_dir};
use crate::theme::Theme;

pub struct PelicanUI(Theme);
impl Plugin for PelicanUI {}
impl PelicanUI {
    pub fn new(ctx: &mut Context, theme: Theme) -> Self {
        ctx.assets.include_assets(include_dir!("./resources"));
        PelicanUI(theme)
    }

    pub fn theme(&self) -> &Theme {&self.0}
    pub fn theme_mut(&mut self) -> &mut Theme {&mut self.0}
}

// impl Default for PelicanUI {
//     fn default() -> Self {
//         PelicanUI(Theme::default())
//     }
// }