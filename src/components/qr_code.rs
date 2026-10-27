use prism::events::{on_change, OnChange};
use prism::drawable::{Drawable, Svg};
use prism::drawable::shape::Variant;
use prism::drawable::Component;
use ptsd::layout::{Padding, Size, Offset, Stack};
use ptsd::display::Bin;
use prism::Context;
use media::{Image, Pixel};

use qrcode::{QrCode, EcLevel};

use crate::theme::{Theme, Color};
use crate::components::{Rectangle, AspectRatioImage, MaskedImage};

#[derive(Debug, Component, Clone)]
pub struct QRCode(
    Stack, Rectangle, Bin<Stack, Box<dyn Drawable>>, Option<Bin<Stack, Svg>>,
    #[skip] Option<String>,
    #[skip] Svg,
);
#[on_change(async {
    match self.4.take() {Some(data) => data, None => std::future::pending().await}
})]
impl OnChange<String> for QRCode {
    fn on_change(&mut self, _ctx: &mut Context, data: String) {
        match Self::generate(&data) {
            Some(image) => {
                self.2.1 = Box::new(MaskedImage(image, Variant::RoundedRectangle(8.0)));
            }
            None => {
                self.2.1 = Box::new(MaskedImage(self.5.clone(), Variant::RoundedRectangle(8.0)));
                self.3 = None;
            }
        }
    }
}

impl QRCode {
    pub fn new(theme: &Theme, data: &str) -> Self {
        let app_icon = theme.brand().app_icon.clone();
        let dummy_qr_code = theme.brand().qr_code.clone();
        let qr_size = 300.0;
        let logo_size = 64.0;
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(qr_size - 16.0),
            Size::Static(qr_size - 16.0), Padding::default(),
        );
        QRCode(
            Stack::new(
                Offset::Center, Offset::Center, Size::Static(qr_size), Size::Static(qr_size),
                Padding::default(),
            ), Rectangle::new(Color::white(), 8.0, None),
            Bin(layout, Box::new(MaskedImage(dummy_qr_code, Variant::RoundedRectangle(8.0)))),
            Some(AspectRatioImage::svg(app_icon, logo_size)), Some(data.to_string()),
            theme.brand().error.clone(),
        )
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "https://ramp.com/design_systems/pelican_ui")
    }

    pub fn generate(data: &str) -> Option<Image> {
        let scale: usize = 60;
        let fg = [0u8, 0, 0];
        let bg = [255u8, 255, 255];

        let code = QrCode::with_error_correction_level(data, EcLevel::H).ok()?;

        let module_count = code.width();
        let img_size = module_count * scale;

        let logo_size_px = img_size as f32 / 3.8;
        let logo_modules = (logo_size_px / scale as f32).ceil() as usize | 1;
        let logo_start = (module_count - logo_modules) / 2;
        let logo_end = logo_start + logo_modules;

        let mut buf = vec![255u8; img_size * img_size * 3];

        let r = (scale / 2) as i32;
        let cx = r;
        let cy = r;
        let rr = r * r;

        let mut dot_rows = Vec::with_capacity(scale);

        for y in 0..scale as i32 {
            let mut row = Vec::new();
            for x in 0..scale as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= rr { row.push((x as usize * 3, fg)); }
            }
            dot_rows.push(row);
        }

        let mut skip = vec![false; module_count * module_count];
        let finder = 7;

        for y in 0..module_count {
            for x in 0..module_count {
                let in_finder = !(x >= finder || y >= finder && y < module_count - finder)
                    || (x >= module_count - finder && y < finder);

                let in_logo = x >= logo_start && x < logo_end && y >= logo_start && y < logo_end;

                skip[y * module_count + x] = in_finder || in_logo;
            }
        }

        for y in 0..module_count {
            let py = y * scale;
            for x in 0..module_count {
                if skip[y * module_count + x] || code[(x, y)] != qrcode::Color::Dark { continue; }
                let px = x * scale;
                Self::blit_dot(&mut buf, img_size, px, py, &dot_rows);
            }
        }

        for &(fx, fy) in &[(0, 0), (0, module_count - 7), (module_count - 7, 0)] {
            Self::draw_finder(&mut buf, img_size, fx * scale, fy * scale, scale, fg, bg);
        }

        let pixels = buf.chunks_exact(3).map(|pixel| {
            Pixel {red: pixel[0], green: pixel[1], blue: pixel[2], alpha: 255}
        }).collect();
        Some(Image::new((img_size as u32, img_size as u32), pixels))
    }

    fn blit_dot(buf: &mut [u8], img_size: usize, px: usize, py: usize, dot_rows: &[Vec<(usize, [u8; 3])>]) {
        for (dy, row) in dot_rows.iter().enumerate() {
            let base = ((py + dy) * img_size + px) * 3;
            for &(dx3, rgb) in row {
                let i = base + dx3;
                buf[i] = rgb[0];
                buf[i + 1] = rgb[1];
                buf[i + 2] = rgb[2];
            }
        }
    }

    fn draw_finder(buf: &mut [u8], img_size: usize, x: usize, y: usize, scale: usize, fg: [u8; 3], bg: [u8; 3]) {
        Self::fill_rounded_rect(buf, img_size, x, y, 7 * scale, scale, fg);
        Self::fill_rounded_rect(buf, img_size, x + scale, y + scale, 5 * scale, scale / 2, bg);
        Self::fill_rounded_rect(
            buf, img_size, x + 2 * scale, y + 2 * scale, 3 * scale, scale / 2, fg,
        );
    }

    fn fill_rounded_rect(buf: &mut [u8], img_size: usize, x: usize, y: usize, size: usize, radius: usize, color: [u8; 3]) {
        let r = radius.min(size / 2) as isize;
        let x0 = x as isize;
        let y0 = y as isize;
        let x1 = (x + size) as isize;
        let y1 = (y + size) as isize;

        for py in y0..y1 {
            for px in x0..x1 {
                let dx = if px < x0 + r {
                    x0 + r - px
                } else if px >= x1 - r {
                    px - (x1 - r - 1)
                } else { 0 };

                let dy = if py < y0 + r {
                    y0 + r - py
                } else if py >= y1 - r {
                    py - (y1 - r - 1)
                } else { 0 };

                if dx == 0 || dy == 0 || dx * dx + dy * dy <= r * r {
                    let i = ((py as usize) * img_size + (px as usize)) * 3;
                    buf[i] = color[0];
                    buf[i + 1] = color[1];
                    buf[i + 2] = color[2];
                }
            }
        }
    }
}
