use prism::drawable::{Drawable, Component, RequestTree, SizedTree, Offset, Rect};
use prism::canvas::{ShapeType, Shape, Area as CanvasArea, Item as CanvasItem, self};
use prism::event::OnEvent;
use prism::layout::SizeRequest;
use prism::layout::Stack;

use crate::theme::Color;

#[derive(Debug, Component)]
pub struct Rectangle(Stack, ExpandableShape, Option<ExpandableShape>);
impl OnEvent for Rectangle {}

impl Rectangle {
    pub fn new(background: Color, radius: f32, outline: Option<(f32, Color)>) -> Self {
        Rectangle(
            Stack::default(),
            ExpandableShape::rounded_rectangle(0.0, radius, background),
            outline.map(|(s, c)| ExpandableShape::rounded_rectangle(s, radius, c))
        )
    }

    pub fn background(&mut self) -> &mut canvas::Color {&mut self.1.shape().color}
    pub fn outline(&mut self) -> Option<&mut canvas::Color> {self.2.as_mut().map(|s| &mut s.shape().color)}
    pub fn size(&self) -> (f32, f32) {self.1.0.shape.size()}
}

#[derive(Debug)]
pub struct ExpandableShape(Shape);

impl ExpandableShape {
    pub fn rounded_rectangle(s: f32, r: f32, color: Color) -> Self {
        ExpandableShape(Shape{shape: ShapeType::RoundedRectangle(s, (0.0, 0.0), 0.0, r), color: color.into()})
    }

    pub fn rectangle(s: f32, color: Color) -> Self {
        ExpandableShape(Shape{shape: ShapeType::Rectangle(s, (0.0, 0.0), 0.0), color: color.into()})
    }

    pub fn ellipse(s: f32, color: Color) -> Self {
        ExpandableShape(Shape{shape: ShapeType::Ellipse(s, (0.0, 0.0), 0.0), color: color.into()})
    }

    fn shape(&mut self) -> &mut Shape { &mut self.0 }
}

impl Drawable for ExpandableShape {
    fn request_size(&self) -> RequestTree {RequestTree(SizeRequest::fill(), vec![])}

    fn draw(&self, sized: &SizedTree, offset: Offset, bound: Rect) -> Vec<(CanvasArea, CanvasItem)> {
        let shape = match self.0.shape {
            ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, sized.0, a, r),
            ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, sized.0, a),
            ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, sized.0, a),
        };

        vec![(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Shape(Shape{shape, color: self.0.color}))]
    }
}

pub struct Circle;

impl Circle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(size: f32, color: Color, outlined: bool) -> Shape {
        let outline = if outlined { size * 0.06 } else { 0.0 };
        Shape { shape: ShapeType::Ellipse(outline, (size, size), 0.0), color: color.into() }
    }
}
