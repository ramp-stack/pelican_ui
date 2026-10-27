use prism::drawable::{Component, Shape, shape::Variant};
use ptsd::display::Bin;
use ptsd::layout::{Stack, Size, Offset, Padding};

use crate::theme::Color;

#[derive(Debug, Component, Clone)]
pub struct Rectangle(Stack, Shape, Option<Shape>);

impl Rectangle {
    pub fn new(background: Color, radius: f32, outline: Option<(f32, Color)>) -> Self {
        Rectangle(
            Stack::default(),
            Shape::new(background.into(), Variant::RoundedRectangle(radius)),
            outline.map(|(stroke, color)| Shape {
                stroke, ..Shape::new(color.into(), Variant::RoundedRectangle(radius))
            }),
        )
    }

    pub fn background(&mut self) -> &mut prism::drawable::Color {&mut self.1.color}
    pub fn outline(&mut self) -> Option<&mut prism::drawable::Color> {
        self.2.as_mut().map(|shape| &mut shape.color)
    }
}

pub struct Circle;

impl Circle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(size: f32, color: Color, outlined: bool) -> Bin<Stack, Shape> {
        let stroke = if outlined {size * 0.06} else {0.0};
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(size), Size::Static(size), Padding::default(),
        );
        Bin(layout, Shape {stroke, ..Shape::new(color.into(), Variant::Ellipse)})
    }
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Bin<Stack, Shape> {Self::new(48.0, Color::white(), false)}
}
