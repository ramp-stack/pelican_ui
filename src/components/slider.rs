use mustache::events::OnEvent;
use mustache::drawable::{Color, Align};
use mustache::{Context, Component};

use crate::components::{Text, ExpandableText, TextStyle, Circle, Rectangle};
use crate::layout::{Column, Offset, Size, Padding};
use crate::plugin::PelicanUI;
use crate::components::interactions;

/// ## Slider
///
/// A UI component that allows users to select a value along a continuous range. 
///
/// ![Slider Example](https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/slider.png)
///
/// ### Example
/// ```rust
/// let slider = Slider::new(
///     ctx,V
///     50.0,
///     Some("Volume"),
///     Some("Adjust the sound level"),
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
        on_release: impl FnMut(&mut Context, f32) + 'static,
    ) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        let brand = ctx.get::<PelicanUI>().get().0.theme().colors.brand;
        Slider(
            Column::new(8.0, Offset::Start, Size::Fit, Padding::default()),
            label.map(|l| Text::new(ctx, l, size.h5, TextStyle::Heading, Align::Left, None)),
            description.map(|t| ExpandableText::new(ctx, t, size.md, TextStyle::Primary, Align::Left, None)),
            interactions::Slider::new(ctx, start, Rectangle::new(Color::WHITE, 3.0, None), Rectangle::new(brand, 3.0, None), Circle::new(18.0, brand, false), on_release),
        )
    }

    // pub fn set_value(&mut self, value: f32) {
    //     self.pending_value = Some(value.clamp(0.0, 1.0));
    //     let track_width = match self.content.background.layout().2 {
    //         Size::Static(w) => w,
    //         _ => 0.0,
    //     };
    //     self.content.set_knob_position(self.pending_value.unwrap() * track_width, track_width);
    // }
}



// Rectangle::new(Color::WHITE, 3.0, None)
// Rectangle::new(color, 3.0, None)?
// let color = ctx.get::<PelicanUI>().get().0.theme().colors.brand;

// let color = ctx.get::<PelicanUI>().get().0.theme().colors.brand;
// Circle::new(18.0, color, false)