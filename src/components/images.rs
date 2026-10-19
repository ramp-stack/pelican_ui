use prism::drawable::{Component, Drawable, LayoutTree, RangeTree, Rect, Instruction, Item};
use prism::drawable::shape::{Shape, Variant};
use prism::hardware::Image;
use ptsd::display::Bin;
use ptsd::layout::{Stack, Offset, Size, Padding};

use crate::theme::{Theme, Color, Icons};

#[derive(Component, Clone, Debug)]
pub enum Icon {
    Raw {layout: Stack, image: Image},
    Tinted {layout: Stack, image: prism::drawable::Icon},
}

impl Icon {
    pub fn new(theme: &Theme, name: Icons, color: Option<Color>, size: f32) -> Self {
        let layout = Stack::new(Offset::Center, Offset::Center,
            Size::Static(size), Size::Static(size), Padding::default());
        let image = theme.icons().get(name);
        match color {
            Some(color) => Self::Tinted {layout, image: prism::drawable::Icon(image, color.into())},
            None => Self::Raw {layout, image},
        }
    }
}

#[derive(Clone, Debug)]
pub struct AspectRatioImage;
impl AspectRatioImage {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(image: Image, size: (f32, f32)) -> Bin<Stack, Image> {
        let height = size.0 * image.height() as f32 / image.width() as f32;
        Bin(Stack::new(Offset::Center, Offset::Center,
            Size::Static(size.0), Size::Static(height), Padding::default()), image)
    }
}

#[derive(Debug, Clone)]
pub struct MaskedImage(pub Image, pub Variant);

impl Drawable for MaskedImage {
    fn size_range(&self) -> RangeTree {self.0.size_range()}
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let shape = Shape::new(layout.0.0, layout.0.1, prism::drawable::Color::WHITE, self.1);
        vec![Instruction {offset, bound, item: Item::Image(shape, self.0.clone(), None)}]
    }
}

#[derive(Debug, Clone)]
pub struct ExpandableImage(Image, Option<(f32, f32)>);

impl ExpandableImage {
    pub fn new(image: Image, size: Option<(f32, f32)>) -> Self {Self(image, size)}
    pub fn image(&mut self) -> &mut Image {&mut self.0}
    pub fn dimensions(&mut self) -> &mut Option<(f32, f32)> {&mut self.1}
}

impl Drawable for ExpandableImage {
    fn size_range(&self) -> RangeTree {
        let mut range = self.0.size_range();
        if let Some((_, height)) = self.1 {
            range.0.min_height = height;
            range.0.max_height = height;
        }
        range
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.0.draw(layout, offset, bound)
    }
}

#[derive(Debug, Clone)]
pub enum ImageOrientation {
    Up,
    Down,
    Left,
    Right,
    UpMirrored,
    DownMirrored,
    LeftMirrored,
    RightMirrored,
}

impl ImageOrientation {
    pub fn from_ios_value(orientation: i64) -> Self {
        match orientation {
            0 => ImageOrientation::Up,
            1 => ImageOrientation::Down,
            2 => ImageOrientation::Left,
            3 => ImageOrientation::Right,
            4 => ImageOrientation::UpMirrored,
            5 => ImageOrientation::DownMirrored,
            6 => ImageOrientation::LeftMirrored,
            7 => ImageOrientation::RightMirrored,
            _ => ImageOrientation::Up,
        }
    }

    pub fn apply_to(&self, image: image::DynamicImage) -> image::DynamicImage {
        match self {
            ImageOrientation::Up => image,
            ImageOrientation::Down => image.rotate180(),
            ImageOrientation::Left => image.rotate270(),
            ImageOrientation::Right => image.rotate90(),
            ImageOrientation::UpMirrored => image.fliph(),
            ImageOrientation::DownMirrored => image.fliph().rotate180(),
            ImageOrientation::LeftMirrored => image.fliph().rotate90(),
            ImageOrientation::RightMirrored => image.fliph().rotate270(),
        }
    }
}
