use mustache::events::OnEvent;
use mustache::drawable::Align;
use mustache::{Context, Component};

use crate::components::{Text, ExpandableText, TextStyle, Circle, Rectangle};
use mustache::layouts::Column;
use crate::plugin::PelicanUI;
use crate::components::interactions;

/// ## Slider
///
/// A UI component that allows users to select a value along a continuous range. 
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/slider.png"
///      alt="Secondary Icons Example"
///      width="350">
///
/// ### Example
/// ```rust
/// let slider = Slider::new(
///     ctx,
///     50.0,
///     Some("Volume"),
///     None,
///     |ctx: &mut Context, percentage: f32| {
///         println!("Slider released at %{percentage}");
///     }
/// );
/// ```
#[derive(Debug, Component)]
pub struct Slider(Column, Option<Text>, Option<ExpandableText>, interactions::Slider); // last f32 = value 0.0..1.0
impl OnEvent for Slider {}
impl Slider {
    pub fn new(
        ctx: &mut Context,
        start: f32,
        label: Option<&str>,
        description: Option<&str>,
        on_change: impl FnMut(&mut Context, f32) + 'static,
    ) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;
        Slider(Column::start(8.0),
            label.map(|l| Text::new(ctx, l, size.h5, TextStyle::Heading, Align::Left, None)),
            description.map(|t| ExpandableText::new(ctx, t, size.md, TextStyle::Primary, Align::Left, None)),
            interactions::Slider::new(start, Rectangle::new(colors.outline.primary, 3.0, None), Rectangle::new(colors.brand, 3.0, None), Circle::new(18.0, colors.brand, false), on_change),
        )
    }
}