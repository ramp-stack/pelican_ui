use prism::drawable::{Component, Drawable, LayoutTree, Rect, Instruction, Item};
use prism::drawable::shape::Variant;
use prism::drawable::Svg;
use media::Image;
use ptsd::display::Bin;
use ptsd::layout::{Stack, Offset, Size, Padding};

use crate::theme::{Theme, Color, Icons};

#[derive(Component, Clone, Debug)]
pub enum Icon {
    Raw { layout: Stack, image: Svg },
    Tinted { layout: Stack, image: prism::drawable::Icon },
}

impl Icon {
    pub fn new(theme: &Theme, name: Icons, color: Option<Color>, size: f32) -> Self {
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(size), Size::Static(size), Padding::default(),
        );
        let image = theme.icons().get(name);
        match color {
            Some(color) => Self::Tinted { layout, image: prism::drawable::Icon(image, color.into()) },
            None => Self::Raw { layout, image },
        }
    }
}

#[derive(Clone, Debug)]
pub struct AspectRatioImage;
impl AspectRatioImage {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(image: Image, width: f32) -> Bin<Stack, Image> {
        let dimensions = image.dimensions;
        let height = width * dimensions.1 as f32 / dimensions.0 as f32;
        Bin(Stack::new(
            Offset::Center, Offset::Center, Size::Static(width), Size::Static(height),
            Padding::default(),
        ), image)
    }

    pub fn svg(image: Svg, width: f32) -> Bin<Stack, Svg> {
        let dimensions = image.dimensions();
        let height = width * dimensions.1 / dimensions.0;
        Bin(Stack::new(
            Offset::Center, Offset::Center, Size::Static(width), Size::Static(height),
            Padding::default(),
        ), image)
    }
}

#[derive(Debug, Clone)]
pub struct MaskedImage<D: Drawable + Clone = Image>(pub D, pub Variant);

impl<D: Drawable + Clone> Drawable for MaskedImage<D> {
    fn request_width(&mut self) -> (f32, f32) {self.0.request_width()}
    fn request_height(&mut self, width: f32) -> (f32, f32) {self.0.request_height(width)}
    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {self.0.layout(size)}
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let mut instructions = self.0.draw(layout, offset, bound);
        for instruction in &mut instructions {
            match &mut instruction.item {
                Item::Image(shape, ..) | Item::Svg(shape, ..) => shape.variant = self.1,
                _ => {}
            }
        }
        instructions
    }
}
