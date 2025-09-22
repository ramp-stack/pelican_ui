use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest};
use pelican_ui::drawable::{ShapeType, Image, Color};
use pelican_ui::maverick_os::ImageOrientation;
use pelican_ui::{Context, resources};
use std::io::BufWriter;

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::RgbaImage;
use image::ColorType;

use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image as FirImage;
use image::GenericImageView;
use base64::{engine::general_purpose, Engine};


/// ## Icon
///
/// A square icon component.
///
/// If an Icon is built with an icon name that doesn't exist, it will default to the Pelican UI logo.
///
/// For all available icons, [`go here`](pelican_ui::IconResources)
///
/// To learn how to add new icons to the registration, [`go here`](pelican_ui::IconResources)
///
/// ![Icon Example](https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/icon.png)
///
/// ### Example
/// ```rust
/// let color = ctx.theme.colors.shades.white;
/// let icon = Icon::new(ctx, "potion", color, 64.0);
/// ```
#[derive(Clone, Debug)]
pub struct Icon;
impl Icon {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, name: &'static str, color: Color, size: f32) -> Image {
        let icon = ctx.theme.icons.get(name);
        Image{shape: ShapeType::Rectangle(0.0, (size, size), 0.0), image: icon, color: Some(color)}
    }
}

/// ## Aspect Ratio Image
///
/// Displays an image that scales to fit within the given size
/// while preserving its aspect ratio.
///
/// This is the recommended way to display an image.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ar_image.png"
///      alt="AsRa Image Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let img = ctx.theme.brand.illustrations.get("fish_image");
/// let image = AspectRatioImage::new(img, (100.0, 100.0));
/// ```
///
/// For adding a new image to the illustrations go here: [`Illustrations`](pelican_ui::Illustrations)
#[derive(Clone, Debug)]
pub struct AspectRatioImage;
impl AspectRatioImage {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(image: resources::Image, size: (f32, f32)) -> Image {
        let (w, h) = image.size();
        let r = h as f32 / w as f32;

        let tw = size.0;
        let th = tw * r;

        Image{shape: ShapeType::Rectangle(0.0, (tw, th), 0.0), image, color: None}
    }
}

/// ## Encoded Image
///
/// Encode an RgbaImage or image bytes as [`general_purpose::STANDARD`].
/// Then, later decode as [`resources::Image`] or [`RgbaImage`].
///
/// ### Example
/// ```rust
/// let image = EncodedImage::encode(rgba.as_bytes(), ImageOrientation::Up);
/// ```
pub struct EncodedImage;
impl EncodedImage {
    pub fn encode(bytes: Vec<u8>, orientation: ImageOrientation) -> Option<String> {
        if let Ok(dynamic) = image::load_from_memory(&bytes) {
            let src_image = orientation.apply_to(image::DynamicImage::ImageRgba8(dynamic.to_rgba8()));
            let (w, h) = src_image.dimensions();
            let s = 256.0 / w.min(h) as f32;
            let (w, h) = ((w as f32 * s) as u32, (h as f32 * s) as u32);
            let mut dst_image = FirImage::new(w, h, src_image.pixel_type().unwrap());
            Resizer::new().resize(&src_image, &mut dst_image, None).unwrap();

            let mut result_buf = BufWriter::new(Vec::new());
            PngEncoder::new(&mut result_buf).write_image(dst_image.buffer(), w, h, src_image.color().into()).unwrap();
            let result_buf = result_buf.into_inner().unwrap(); 
            return Some(general_purpose::STANDARD.encode(&result_buf))
        }
        None
    }

    pub fn decode(ctx: &mut Context, bytes: &String) -> resources::Image {
        let png_bytes = general_purpose::STANDARD.decode(bytes).unwrap();
        let image = image::load_from_memory(&png_bytes).unwrap();
        ctx.assets.add_image(image.into())  
    }

    pub fn encode_rgba(image: RgbaImage) -> String {
        let (width, height) = image.dimensions();
        let raw = image.into_raw();

        let mut result_buf = BufWriter::new(Vec::new());
        PngEncoder::new(&mut result_buf)
            .write_image(&raw, width, height, ColorType::Rgba8.into())
            .unwrap();

        let png_bytes = result_buf.into_inner().unwrap();
        general_purpose::STANDARD.encode(&png_bytes)
    }

    pub fn decode_rgba(data: &str) -> RgbaImage {
        let png_bytes = general_purpose::STANDARD
            .decode(data)
            .expect("Base64 decode failed");

        image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
            .expect("Failed to load PNG")
            .to_rgba8()
    }
}

/// # ExpandableImage
///
/// A wrapper around an [`Image`] that optionally supports custom dimensions.  
/// If no size is provided, the image defaults to `(0.0, 0.0)` and will expand according to its container.
///
/// ## Example
/// ```rust
/// let img = resources::Image::load("logo.png");
/// let expandable = ExpandableImage::new(img, Some((100.0, 100.0)));
/// ```
#[derive(Debug)]
pub struct ExpandableImage(Image, Option<(f32, f32)>);

impl ExpandableImage {
    pub fn new(image: resources::Image, size: Option<(f32, f32)>) -> Self {
        let dims = size.unwrap_or((0.0, 0.0));
        ExpandableImage(Image{shape: ShapeType::Rectangle(0.0, dims, 0.0), image, color: None}, size)
    }

    pub fn image(&mut self) -> &mut Image { &mut self.0 }
    pub fn dimensions(&mut self) -> &mut Option<(f32, f32)> {&mut self.1}
}

impl OnEvent for ExpandableImage {}
impl Component for ExpandableImage {
    fn children_mut(&mut self) -> Vec<&mut dyn Drawable> { vec![&mut self.0] }
    fn children(&self) -> Vec<&dyn Drawable> { vec![&self.0] }
    fn request_size(&self, _ctx: &mut Context, _children: Vec<SizeRequest>) -> SizeRequest {
        if let Some((_, orig_h)) = self.1 {
            SizeRequest::new(0.0, orig_h, f32::MAX, orig_h)
        } else {
            SizeRequest::fill()
        }
    }

    fn build(&mut self, _ctx: &mut Context, size: (f32, f32), _children: Vec<SizeRequest>) -> Vec<Area> {
        if let Some((orig_w, orig_h)) = self.1 {
            let width = size.0;
            let height = width * (orig_h / orig_w);
            self.1 = Some((width, height));

            if let ShapeType::Rectangle(_, s, _) = &mut self.0.shape {
                *s = (width, height);
            }

            vec![Area { offset: (0.0, 0.0), size: (width, height) }]
        } else {
            if let ShapeType::Rectangle(_, s, _) = &mut self.0.shape {
                *s = (size.0, size.1);
            }

            vec![Area { offset: (0.0, 0.0), size, }]
        }
    }
}