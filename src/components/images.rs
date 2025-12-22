use prism::layout::SizeRequest;
use prism::drawable::{Drawable, SizedTree, RequestTree, Rect}; 
use prism::canvas::{ShapeType, Image, Area as CanvasArea, Item as CanvasItem};
use prism::Context;

use crate::theme::{Theme, Color};

// use std::io::BufWriter;
use std::sync::Arc;

// use image::codecs::png::PngEncoder;
// use image::ImageEncoder;
use image::RgbaImage;
// use image::ColorType;

// use fast_image_resize::{IntoImageView, Resizer};
// use fast_image_resize::images::Image as FirImage;
// use image::GenericImageView;
// use base64::{engine::general_purpose, Engine};

// pub struct Images(Vec<)

#[derive(Clone, Debug)]
pub struct Icon;
impl Icon {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, name: &str, color: Option<Color>, size: f32) -> Image {
        let icon = ctx.state.get_or_default::<Theme>().icons.get(name);
        Image{shape: ShapeType::Rectangle(0.0, (size, size), 0.0), image: icon, color: color.map(|c| c.into())}
    }
}

/// ## Aspect Ratio Image
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ar_image.png"
///      alt="AsRa Image Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let img = ctx.theme.brand.illustrations.get("fish_image");
/// let image = AspectRatioImage::new(img, (100.0, 100.0));
/// ```
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

// /// Encode an RgbaImage or image bytes as [`general_purpose::STANDARD`].
// /// Then, later decode as [`resources::Image`] or [`RgbaImage`].
// pub struct EncodedImage;
// impl EncodedImage {
//     pub fn encode(bytes: Vec<u8>, orientation: ImageOrientation) -> Option<String> {
//         if let Ok(dynamic) = image::load_from_memory(&bytes) {
//             let src_image = orientation.apply_to(image::DynamicImage::ImageRgba8(dynamic.to_rgba8()));
//             let (w, h) = src_image.dimensions();
//             let s = 256.0 / w.min(h) as f32;
//             let (w, h) = ((w as f32 * s) as u32, (h as f32 * s) as u32);
//             let mut dst_image = FirImage::new(w, h, src_image.pixel_type().unwrap());
//             Resizer::new().resize(&src_image, &mut dst_image, None).unwrap();

//             let mut result_buf = BufWriter::new(Vec::new());
//             PngEncoder::new(&mut result_buf).write_image(dst_image.buffer(), w, h, src_image.color().into()).unwrap();
//             let result_buf = result_buf.into_inner().unwrap(); 
//             return Some(general_purpose::STANDARD.encode(&result_buf))
//         }
//         None
//     }

//     pub fn decode(ctx: &mut Context, bytes: &String) -> Arc<RgbaImage> {
//         let png_bytes = general_purpose::STANDARD.decode(bytes).unwrap();
//         image::load_from_memory(&png_bytes).unwrap().into()
//     }

//     pub fn encode_rgba(image: RgbaImage) -> String {
//         let (width, height) = image.dimensions();
//         let raw = image.into_raw();

//         let mut result_buf = BufWriter::new(Vec::new());
//         PngEncoder::new(&mut result_buf)
//             .write_image(&raw, width, height, ColorType::Rgba8.into())
//             .unwrap();

//         let png_bytes = result_buf.into_inner().unwrap();
//         general_purpose::STANDARD.encode(&png_bytes)
//     }

//     pub fn decode_rgba(data: &str) -> RgbaImage {
//         let png_bytes = general_purpose::STANDARD
//             .decode(data)
//             .expect("Base64 decode failed");

//         image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
//             .expect("Failed to load PNG")
//             .to_rgba8()
//     }
// }

/// A wrapper around an [`Image`] that optionally supports custom dimensions.  
/// If no size is provided, the image defaults to `(0.0, 0.0)` and will expand according to its container.
#[derive(Debug)]
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

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<(CanvasArea, CanvasItem)> {
        if let Some((orig_w, orig_h)) = self.1 {
            let width = sized.0.0;
            let height = width * (orig_h / orig_w);

            let shape = match self.0.shape {
                ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, (width, height), a, r),
                ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, (width, height), a),
                ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, (width, height), a),
            };

            vec![(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Image(Image{shape, image: self.0.image.clone(), color: self.0.color }))]
        } else {
            let shape = match self.0.shape {
                ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, sized.0, a, r),
                ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, sized.0, a),
                ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, sized.0, a),
            };

            vec![(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Image(Image{shape, image: self.0.image.clone(), color: self.0.color }))]
        }
    }
}

#[derive(Debug)]
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