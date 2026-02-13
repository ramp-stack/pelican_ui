use prism::event::OnEvent;
use prism::canvas::Align;
use prism::Context;
use prism::drawable::Component;
use prism::layout::Column;

use ptsd::interactions;

use crate::theme::Theme;
use crate::components::text::{TextSize, TextStyle, ExpandableText, Text};
use crate::components::{Circle, Rectangle};

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
///     theme,
///     50.0,
///     Some("Volume"),
///     None,
///     |ctx: &mut Context, percentage: f32| {
///         println!("Slider released at %{percentage}");
///     }
/// );
/// ```
#[derive(Debug, Component, Clone)]
pub struct Slider(Column, Option<Text>, Option<ExpandableText>, interactions::Slider); // last f32 = value 0.0..1.0
impl OnEvent for Slider {}
impl Slider {
    pub fn new(
        theme: &Theme,
        start: f32,
        label: Option<&str>,
        description: Option<&str>,
        mut on_change: impl FnMut(&mut Context, &Theme, f32) + Clone + Send + Sync + 'static,
    ) -> Self {
        let colors = theme.colors();
        let background = Rectangle::new(colors.get(ptsd::Outline::Primary), 3.0, None);
        let foreground = Rectangle::new(colors.get(ptsd::Brand), 3.0, None);
        let handle = Circle::new(18.0, colors.get(ptsd::Brand), false);
        let label = label.map(|l| Text::new(theme, l, TextSize::H5, TextStyle::Heading, Align::Left, None));
        let description = description.map(|t| ExpandableText::new(theme, t, TextSize::Md, TextStyle::Primary, Align::Left, None));

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context, p: f32| (on_change)(ctx, &theme, p));
        Slider(Column::start(8.0), label, description, interactions::Slider::new(start, background, foreground, handle, callback))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, 0.5, Some("Slider"), None, |_: &mut Context, _: &Theme, p: f32| println!("Slider moved... {p:?}"))
    }
}