use prism::drawable::{Drawable, Component, RangeTree, LayoutTree, Offset, Rect, Instruction, Item};
use prism::drawable::shape::{Shape, Variant};
use ptsd::layout::Stack;

use crate::theme::Color;

#[derive(Debug, Component, Clone)]
pub struct Rectangle(Stack, ExpandableShape, Option<ExpandableShape>);

impl Rectangle {
    pub fn new(background: Color, radius: f32, outline: Option<(f32, Color)>) -> Self {
        Rectangle(
            Stack::default(),
            ExpandableShape::rounded_rectangle(0.0, radius, background),
            outline.map(|(s, c)| ExpandableShape::rounded_rectangle(s, radius, c))
        )
    }

    pub fn background(&mut self) -> &mut prism::drawable::Color {&mut self.1.0.color}
    pub fn outline(&mut self) -> Option<&mut prism::drawable::Color> {
        self.2.as_mut().map(|s| &mut s.0.color)
    }
    pub fn size(&self) -> (f32, f32) {self.1.0.size()}
}

#[derive(Debug, Clone)]
pub struct ExpandableShape(Shape);

impl ExpandableShape {
    pub fn rounded_rectangle(stroke: f32, radius: f32, color: Color) -> Self {
        Self(Shape {stroke, ..Shape::new(0.0, 0.0, color.into(), Variant::RoundedRectangle(radius))})
    }
    pub fn rectangle(stroke: f32, color: Color) -> Self {
        Self(Shape {stroke, ..Shape::new(0.0, 0.0, color.into(), Variant::Rectangle)})
    }
    pub fn ellipse(stroke: f32, color: Color) -> Self {
        Self(Shape {stroke, ..Shape::new(0.0, 0.0, color.into(), Variant::Ellipse)})
    }
}

impl Drawable for ExpandableShape {
    fn size_range(&self) -> RangeTree {
        RangeTree(prism::drawable::SizeRange {
            min_width: 0.0, max_width: f32::INFINITY,
            min_height: 0.0, max_height: f32::INFINITY,
        }, vec![])
    }
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        let size = range.0.clamp(size);
        self.0.width = size.0;
        self.0.height = size.1;
        LayoutTree(size, vec![])
    }
    fn draw(&self, layout: &LayoutTree, offset: Offset, bound: Rect) -> Vec<Instruction> {
        let shape = Shape {width: layout.0.0, height: layout.0.1, ..self.0};
        vec![Instruction {offset, bound, item: Item::Shape(shape)}]
    }
}

pub struct Circle;

impl Circle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(size: f32, color: Color, outlined: bool) -> Shape {
        let stroke = if outlined {size * 0.06} else {0.0};
        Shape {stroke, ..Shape::new(size, size, color.into(), Variant::Ellipse)}
    }
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Shape {Self::new(48.0, Color::WHITE, false)}
}
