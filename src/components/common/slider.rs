use pelican_ui::events::{OnEvent, MouseState, Event, MouseEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Align, Component, Shape};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::components::{Text, ExpandableText, TextStyle, Circle, Rectangle};
use crate::layout::{Column, Stack, Offset, Size, Padding, Bin};

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
pub struct Slider(Column, Option<Text>, Option<ExpandableText>, SliderContent, #[skip] f32); // last f32 = value 0.0..1.0

impl Slider {
    pub fn new(
        ctx: &mut Context,
        start: f32,
        label: Option<&str>,
        description: Option<&str>,
        on_release: impl FnMut(&mut Context, f32) + 'static,
    ) -> Self {
        let font_size = ctx.theme.fonts.size;
        Slider(
            Column::new(8.0, Offset::Start, Size::Fit, Padding::default()),
            label.map(|l| Text::new(ctx, l, TextStyle::Heading, font_size.h5, Align::Left)),
            description.map(|t| ExpandableText::new(ctx, t, TextStyle::Primary, font_size.md, Align::Left, None)),
            SliderContent::new(ctx, start, on_release),
            start.clamp(0.0, 1.0),
        )
    }

    pub fn set_value(&mut self, value: f32) {
        self.4 = value.clamp(0.0, 1.0);
        let track_width = self.3.track_width();
        self.3.3.adjust_position(self.4 * track_width, track_width);
    }

    pub fn trigger_event(&mut self, ctx: &mut Context) {
        (self.3.5)(ctx, self.4);
    }
}

impl OnEvent for Slider {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            self.set_value(self.4);
        }
        true
    }
}

type SliderClosure = Box<dyn FnMut(&mut Context, f32)>;

#[derive(Component)]
pub struct SliderContent(Stack, Bin<Stack, Rectangle>, Bin<Stack, Rectangle>, SliderKnob, #[skip] f32, #[skip] SliderClosure, #[skip] bool);

impl SliderContent {
    pub fn new(ctx: &mut Context, start: f32, on_release: impl FnMut(&mut Context, f32) + 'static) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(300.0), f32::MAX));
        let track = Stack(Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default());
        let fill = Stack(Offset::Start, Offset::Start, Size::Static(30.0), Size::Static(6.0), Padding::default());
        let layout = Stack(Offset::Start, Offset::Center, Size::Fit, Size::Fit, Padding::default());
        let color = ctx.theme.colors.brand.primary;
        let white = ctx.theme.colors.shades.white;

        SliderContent(
            layout,
            Bin(track, Rectangle::new(white, 3.0, None)),
            Bin(fill, Rectangle::new(color, 3.0, None)),
            SliderKnob::new(ctx),
            start, 
            Box::new(on_release),
            false,
        )
    }

    pub fn track_width(&mut self) -> f32 {
        self.1.inner().size().0
    }

    pub fn percentage(&self) -> f32 {
        self.4 * 100.0
    }

    fn set_knob_pixel(&mut self, px: f32, track_width: f32) {
        let clamped = px.clamp(0.0, track_width);
        self.3.adjust_position(clamped, track_width);
        self.2.layout().2 = Size::Static(clamped);
    }
}

impl std::fmt::Debug for SliderContent { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "SliderContent") 
    } 
}
impl OnEvent for SliderContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        let width = self.track_width();

        if let Some(MouseEvent { state: MouseState::Pressed, position: Some((x, _)) }) = event.downcast_ref::<MouseEvent>() {
            self.6 = true;

            if width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.4 = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p =  self.percentage() / 100.0;
                (self.5)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Scroll(..), position: Some((x, _))}) = event.downcast_ref::<MouseEvent>() {
            if self.6 && width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.4 = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p =  self.percentage() / 100.0;
                (self.5)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Moved, position: Some((x, _)) }) = event.downcast_ref::<MouseEvent>() {
            if self.6 && width > 0.0 {
                let clamped_x = x.clamp(0.0, width);
                self.4 = (clamped_x / width).clamp(0.0, 1.0);
                self.set_knob_pixel(clamped_x, width);
                let p =  self.percentage() / 100.0;
                (self.5)(ctx, p);
            }
        } else if let Some(MouseEvent { state: MouseState::Released, .. }) = event.downcast_ref::<MouseEvent>() {
            if self.6 {
                self.6 = false;
                let p =  self.percentage() / 100.0;
                (self.5)(ctx, p);
            }
        } else if event.downcast_ref::<TickEvent>().is_some() && width > 0.0 {
            self.set_knob_pixel(self.4 * width, width);
        }

        true
    }
}

#[derive(Debug, Component)]
pub struct SliderKnob(Stack, Shape);
impl OnEvent for SliderKnob {}

impl SliderKnob {
    pub fn new(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.brand.primary;
        SliderKnob(Stack::default(), Circle::new(18.0, color, false))
    }

    pub fn adjust_position(&mut self, x: f32, track_width: f32) {
        let clamped_x = x.clamp(9.0, track_width);
        self.0.0 = Offset::Static(clamped_x - 9.0);
    }
}
