use mustache::events::OnEvent;
use mustache::drawable::{Color, Drawable, Component, ShapeType, Image};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::layout::{Padding, Size, Offset, Stack, Bin};
use crate::components::{Rectangle, AspectRatioImage};

use image::{Rgb, RgbImage, DynamicImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use qrcode::{QrCode, EcLevel};

/// ## QR Code
///
/// Renders a scannable QR code with a centered brand/logo overlay.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/qr_code.png"
///      alt="QR Code Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let qr = QRCode::new(ctx, "https://ramp-stack.com/pelican_ui");
/// ```
#[derive(Debug, Component)]
pub struct QRCode(Stack, Bin<Stack, Rectangle>, Image, Image);
impl OnEvent for QRCode {}

impl QRCode {
    pub fn new(ctx: &mut Context, data: &str) -> Self {
        let theme = &ctx.theme;
        let app_icon = theme.brand.app_icon.clone();
        let qr_size = 300.0;
        let logo_size = 64.0;

        let image = generate_qr_code(data);
        let img = DynamicImage::ImageRgb8(image).to_rgba8();
        QRCode (
            Stack::center(),
            Bin(
                Stack(Offset::Center, Offset::Center, Size::Static(qr_size), Size::Static(qr_size), Padding::default()),
                Rectangle::new(Color::TRANSPARENT, 8.0, None),
            ),
            Image{shape: ShapeType::RoundedRectangle(0.0, (qr_size - 16.0, qr_size - 16.0), 8.0, 0.0), image: ctx.assets.add_image(img), color: None},
            // QRModules::new(ctx, data, qr_size, logo_size),  - NO CUSTOM STYLIZATION FOR THIS RELEASE
            AspectRatioImage::new(app_icon, (logo_size, logo_size))
        )
    }
}

fn generate_qr_code(data: &str) -> RgbImage {
    let scale = 60;

    let fg_color = Rgb([0, 0, 0]);
    let bg_color = Rgb([255, 255, 255]);

    let code = QrCode::with_error_correction_level(data, EcLevel::H).expect("Failed to create QR");
    let module_count = code.width();
    let img_size = module_count * scale;

    let mut img = RgbImage::from_pixel(img_size as u32, img_size as u32, bg_color);
    let logo_size_px: f32 = img_size as f32 / 3.8;

    let finder_size = 7;

    let module_size = img_size as f32 / module_count as f32;
    let logo_modules = ceil_to_odd(logo_size_px / module_size);
    let logo_start = (module_count - logo_modules) / 2;
    let logo_end = logo_start + logo_modules;

    for &(fx, fy) in &[
        (0, 0),
        (0, module_count - finder_size),
        (module_count - finder_size, 0),
    ] {
        draw_classic_finder_pattern(
            &mut img,
            fx * scale,
            fy * scale,
            scale,
            fg_color,
            bg_color,
        );
    }

    for y in 0..module_count {
        for x in 0..module_count {
            if is_in_finder_pattern_area(x, y, module_count, finder_size) {
                continue;
            }

            if x >= logo_start && x < logo_end && y >= logo_start && y < logo_end {
                continue;
            }

            if code[(x, y)] == qrcode::Color::Dark {
                let cx = (x * scale + scale / 2) as i32;
                let cy = (y * scale + scale / 2) as i32;
                let radius = (scale / 2) as i32;
                draw_filled_circle_mut(&mut img, (cx, cy), radius, fg_color);
            }
        }
    }

    img
}

fn draw_classic_finder_pattern(
    img: &mut RgbImage,
    start_x: usize,
    start_y: usize,
    scale: usize,
    fg_color: Rgb<u8>,
    bg_color: Rgb<u8>,
) {
    draw_rounded_square(img, start_x, start_y, 7 * scale, scale, fg_color);
    draw_rounded_square(img, start_x + scale, start_y + scale, 5 * scale, scale / 2, bg_color);
    draw_rounded_square(img, start_x + 2 * scale, start_y + 2 * scale, 3 * scale, scale / 2, fg_color);
}

fn draw_rounded_square(
    img: &mut RgbImage,
    x: usize,
    y: usize,
    size: usize,
    corner_radius: usize,
    color: Rgb<u8>,
) {
    let x = x as i32;
    let y = y as i32;
    let size = size as i32;
    let r = corner_radius.min((size / 2) as usize) as i32;

    let rect_h = Rect::at(x + r - 1, y).of_size((size - 2 * r + 2) as u32, size as u32);
    draw_filled_rect_mut(img, rect_h, color);

    let rect_v = Rect::at(x, y + r - 1).of_size(size as u32, (size - 2 * r + 2) as u32);
    draw_filled_rect_mut(img, rect_v, color);

    draw_filled_circle_mut(img, (x + r, y + r), r, color);
    draw_filled_circle_mut(img, (x + size - r - 1, y + r), r, color);
    draw_filled_circle_mut(img, (x + r, y + size - r - 1), r, color);
    draw_filled_circle_mut(img, (x + size - r - 1, y + size - r - 1), r, color);
}

fn is_in_finder_pattern_area(
    x: usize,
    y: usize,
    module_count: usize,
    finder_size: usize,
) -> bool {
    let in_top_left = x < finder_size && y < finder_size;
    let in_bottom_left = x < finder_size && y >= module_count - finder_size;
    let in_top_right = x >= module_count - finder_size && y < finder_size;
    in_top_left || in_bottom_left || in_top_right
}

fn ceil_to_odd(val: f32) -> usize {
    let mut v = val.ceil() as usize;
    if v % 2 == 0 { v += 1 }
    v
}

