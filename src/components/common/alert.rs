use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component, Align, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::components::{Icon, Text, TextStyle};
use crate::layout::{Offset, Padding, Row, Size};

/// ## Alert
///
/// Displays a warning message with an icon.  
///  
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/alert.png"
///      alt="Alert Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let alert = Alert::new(&mut ctx, "Offline. Check Your Connection.");
/// ```
#[derive(Debug, Component)]
pub struct Alert(Row, Image, Text);
impl OnEvent for Alert {}

impl Alert {
    pub fn new(ctx: &mut Context, message: &str) -> Self {
        let color = ctx.theme.colors.status.warning;
        let font_size = ctx.theme.fonts.size.md;

        Alert(
            Row::new(4.0, Offset::Center, Size::Fit, Padding::default()),
            Icon::new(ctx, "warning", color, 32.0),
            Text::new(ctx, message, TextStyle::Primary, font_size, Align::Left)
        )
    }
}
