use prism::layout::SizeRequest;
use prism::drawable::{Drawable, SizedTree, RequestTree, Rect}; 
use prism::canvas::{ShapeType, Image, Area as CanvasArea, Item as CanvasItem, Instruction};

use crate::theme::{Theme, Color, Icons};

use std::sync::Arc;

use image::RgbaImage;

#[derive(Clone, Debug)]
pub struct Icon;
impl Icon {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(theme: &Theme, name: Icons, color: Option<Color>, size: f32) -> Image {
        Image{shape: ShapeType::Rectangle(0.0, (size, size), 0.0), image: theme.icons().get(name), color: color.map(|c| c.into())}
    }
}

#[derive(Clone, Debug)]
pub struct AspectRatioImage;
impl AspectRatioImage {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(image: Arc<RgbaImage>, size: (f32, f32)) -> Image {
        let (w, h) = (image.width(), image.height());
        let r = h as f32 / w as f32;

        let tw = size.0;
        let th = tw * r;

        Image{shape: ShapeType::Rectangle(0.0, (tw, th), 0.0), image, color: None}
    }
}

#[derive(Debug, Clone)]
pub struct ExpandableImage(Image, Option<(f32, f32)>);

impl ExpandableImage {
    pub fn new(image: Arc<RgbaImage>, size: Option<(f32, f32)>) -> Self {
        let dims = size.unwrap_or((0.0, 0.0));
        ExpandableImage(Image{shape: ShapeType::Rectangle(0.0, dims, 0.0), image, color: None}, size)
    }

    pub fn image(&mut self) -> &mut Image { &mut self.0 }
    pub fn dimensions(&mut self) -> &mut Option<(f32, f32)> {&mut self.1}
}

impl Drawable for ExpandableImage {
    fn request_size(&self) -> RequestTree { 
        RequestTree(if let Some((_, orig_h)) = self.1 {
            SizeRequest::new(0.0, orig_h, f32::MAX, orig_h)
        } else {
            SizeRequest::fill()
        }, vec![]) 
    }

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        if let Some((orig_w, orig_h)) = self.1 {
            let width = sized.0.0;
            let height = width * (orig_h / orig_w);

            let shape = match self.0.shape {
                ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, (width, height), a, r),
                ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, (width, height), a),
                ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, (width, height), a),
            };

            vec![Instruction(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Image(Image{shape, image: self.0.image.clone(), color: self.0.color }))]
        } else {
            let shape = match self.0.shape {
                ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, sized.0, a, r),
                ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, sized.0, a),
                ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, sized.0, a),
            };

            vec![Instruction(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Image(Image{shape, image: self.0.image.clone(), color: self.0.color }))]
        }
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
