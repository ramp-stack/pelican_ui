use mustache::events::OnEvent;
use mustache::drawable::{Drawable, ShapeType, Shape, Color};
use mustache::layout::{Area, SizeRequest};
use mustache::{Context, Component};
use crate::layout::Stack;

/// # Rectangle
///
/// A rectangle component with a customizable background and optional outline.  
/// Supports rounded corners and adjustable stroke thickness.
///
/// Rectangles expand to fill available space, so wrap them in a [`Bin`]  
/// with a [`Stack`] layout to control the size.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/outlined_rectangle.png"
///      alt="Rectangle Example"
///      width="400">
///
/// ## Example
/// ```rust
/// let rect = Rectangle::new(black, 8.0, Some(8.0, blue));
/// let layout = Stack(Offset::Center, Offset::Center, Size::Static(100.0), Size::Static(100.0), Padding::default());
/// let shape = Bin(layout, rect);
/// ```
#[derive(Debug, Component)]
pub struct Rectangle(Stack, _Rectangle, Option<_Rectangle>);
impl OnEvent for Rectangle {}

impl Rectangle {
    pub fn new(background: Color, radius: f32, outline: Option<(f32, Color)>) -> Self {
        Rectangle(
            Stack::default(),
            _Rectangle::new(0.0, radius, background),
            outline.map(|(s, c)| _Rectangle::new(s, radius, c))
        )
    }

    pub fn background(&mut self) -> &mut Color {&mut self.1.shape().color}
    pub fn outline(&mut self) -> Option<&mut Color> {self.2.as_mut().map(|s| &mut s.shape().color)}
    pub fn size(&self) -> (f32, f32) {self.1.0.shape.size()}
}

#[derive(Debug)]
struct _Rectangle(Shape);

impl _Rectangle {
    fn new(s: f32, r: f32, color: Color) -> Self {
        _Rectangle(Shape{shape: ShapeType::RoundedRectangle(s, (0.0, 0.0), r, 0.0), color})
    }
    fn shape(&mut self) -> &mut Shape { &mut self.0 }
}

impl OnEvent for _Rectangle {}
impl Component for _Rectangle {
    fn children_mut(&mut self) -> Vec<&mut dyn Drawable> {vec![&mut self.0]}
    fn children(&self) -> Vec<&dyn Drawable> {vec![&self.0]}
    fn request_size(&self, _ctx: &mut Context, _children: Vec<SizeRequest>) -> SizeRequest {
        SizeRequest::fill()
    }
    fn build(&mut self, _ctx: &mut Context, size: (f32, f32), _children: Vec<SizeRequest>) -> Vec<Area> {
        if let ShapeType::RoundedRectangle(_, s, _, _) = &mut self.0.shape {
            *s = size;
        }
        vec![Area { offset: (0.0, 0.0), size }]
    }
}


/// # Circle
///
/// Creates a circle with a specified size and color.
///
/// - `size`: diameter of the circle
/// - `color`: fill color
/// - `outlined`: if true, a proportional outline is added
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/circle.png"
///      alt="Circle Example"
///      width="200">
///
/// ## Example
/// ```rust
/// let color = ctx.theme.colors.brand;
/// let circle = Circle::new(100.0, color, true); 
/// ```
pub struct Circle;

impl Circle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(size: f32, color: Color, outlined: bool) -> Shape {
        let outline = if outlined { size * 0.06 } else { 0.0 };
        Shape { shape: ShapeType::Ellipse(outline, (size, size), 0.0), color }
    }
}
