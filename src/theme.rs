use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use image::{ImageReader, RgbaImage};
use include_dir::{DirEntry, Dir, include_dir};
use prism::canvas;

#[derive(Default)]
pub struct Theme {
    pub colors: ColorResources,
    pub fonts: FontResources,
    pub icons: IconResources,
    pub brand: BrandResources,
}

/// Represents a collection of font resources, including fonts and font sizes.
#[derive(Clone, Default)]
pub struct FontResources {
    pub fonts: Fonts,
    pub size: FontSize,
}

/// Defines a collection of fonts used throughout the application for various elements (headings, text, labels, etc.).
#[derive(Clone)]
pub struct Fonts {
    pub heading: canvas::Font,
    pub text: canvas::Font,
    pub label: canvas::Font,
    pub keyboard: canvas::Font,
}

impl Default for Fonts {
    fn default() -> Self {
        let bold = canvas::Font::from_bytes(include_bytes!("../resources/fonts/outfit_bold.ttf").to_vec().as_slice()).unwrap();
        let medium = canvas::Font::from_bytes(include_bytes!("../resources/fonts/outfit_medium.ttf").to_vec().as_slice()).unwrap();
        let regular = canvas::Font::from_bytes(include_bytes!("../resources/fonts/outfit_regular.ttf").to_vec().as_slice()).unwrap();

        Self { heading: bold.clone(), text: regular, label: bold, keyboard: medium }
    }
}

#[derive(Copy, Clone)]
pub struct FontSize {
    pub title: f32,
    pub h1: f32,
    pub h2: f32,
    pub h3: f32,
    pub h4: f32,
    pub h5: f32,
    pub h6: f32,
    pub xl: f32,
    pub lg: f32,
    pub md: f32,
    pub sm: f32,
    pub xs: f32,
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize {
            title: 72.0,
            h1: 48.0,
            h2: 32.0,
            h3: 24.0,
            h4: 20.0,
            h5: 16.0,
            h6: 14.0,
            xl: 24.0,
            lg: 20.0,
            md: 16.0,
            sm: 14.0,
            xs: 12.0,
        }
    }
}

/// A collection of icons used throughout the application.
///
/// - Icons will automatically be adde to resources when they meet these conditions:
///     - Icons must be `.svg` files.
///     - Icons must be located in `project/resources/icons/`.
pub struct IconResources(HashMap<String, Arc<RgbaImage>>);

impl Default for IconResources {
    fn default() -> Self {
        let result = include_dir!("resources/icons").entries().iter().filter_map(|e| match e {
            DirEntry::File(f) => Some(f),
            _ => None,
        }).filter(|p| {
            p.path().to_str().unwrap().ends_with(".svg")
        }).collect::<Vec<_>>();

        Self(result.iter().map(|p| {
            let name = p.path().to_str().unwrap().strip_suffix(".svg").unwrap().replace(' ', "_");
            (name, Arc::new(load_svg(p.contents())))
        }).collect())
    }
}

impl IconResources {
    pub fn get(&self, name: &str) -> Arc<RgbaImage> {
        self.0.get(name).unwrap_or_else(|| {
            println!("Failed to get icon by name {name:?}. Defaulting to pelican_ui");
            self.0.get("pelican_ui").unwrap()
        }).clone()
    }
}

#[derive(Clone, Debug)]
pub struct BrandResources {
    pub wordmark: Arc<RgbaImage>,
    pub logo: Arc<RgbaImage>,
    pub app_icon: Arc<RgbaImage>,
    pub error: Arc<RgbaImage>,
}

impl Default for BrandResources {
    fn default() -> Self {
        let dir = include_dir!("resources/brand");
        BrandResources {
            logo: Arc::new(load_svg(&load_file(&dir, "logo.svg").unwrap())),
            wordmark: Arc::new(load_svg(&load_file(&dir, "wordmark.svg").unwrap())),
            app_icon: Arc::new(load_svg(&load_file(&dir, "app_icon.svg").unwrap())),
            error: Arc::new(load_svg(&load_file(&dir, "error.svg").unwrap())),
        }
    }
}

fn load_file(dir: &Dir, file: &str) -> Option<Vec<u8>> {
    dir.entries().iter().find_map(|e| match e {
        DirEntry::File(f) => (f.path().to_str().unwrap() == file).then_some(f.contents().to_vec()),
        _ => None,
    })
}

fn load_svg(svg: &[u8]) -> RgbaImage {
    let svg = std::str::from_utf8(svg).unwrap();
    let svg = nsvg::parse_str(svg, nsvg::Units::Pixel, 96.0).unwrap();
    let rgba = svg.rasterize(8.0).unwrap();
    let size = rgba.dimensions();
    RgbaImage::from_raw(size.0, size.1, rgba.into_raw()).unwrap()
}


#[derive(Clone, Copy, Debug)]
pub struct ColorResources {
    pub background: BackgroundColor,
    pub outline: OutlineColor,
    pub status: StatusColor,
    pub text: TextColor,
    pub button: ButtonColors,
    pub brand: Color,
}

impl Default for ColorResources {
    fn default() -> Self {
        Self::dark(Color::from_hex("#036ffc", 255))
    }
}

impl ColorResources {
    pub fn dark(brand: Color) -> Self {
        let p = NeutralPalette::dark();

        Self {
            background: BackgroundColor::from_palette(&p),
            outline: OutlineColor::from_palette(&p),
            text: TextColor::from_palette(&p),
            status: StatusColor::default(),
            button: ButtonColors::from(brand, &p),
            brand,
        }
    }

    pub fn light(brand: Color) -> Self {
        let p = NeutralPalette::light();

        Self {
            background: BackgroundColor::from_palette(&p),
            outline: OutlineColor::from_palette(&p),
            text: TextColor::from_palette(&p),
            status: StatusColor::default(),
            button: ButtonColors::from(brand, &p),
            brand,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BackgroundColor {
    pub primary: Color,
    pub secondary: Color,
}

impl BackgroundColor {
    fn from_palette(p: &NeutralPalette) -> Self {
        Self {
            primary: p.bg_primary,
            secondary: p.bg_secondary,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OutlineColor {
    pub primary: Color,
    pub secondary: Color,
}

impl OutlineColor {
    fn from_palette(p: &NeutralPalette) -> Self {
        Self {
            primary: p.outline,
            secondary: p.text_secondary,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextColor {
    pub heading: Color,
    pub primary: Color,
    pub secondary: Color,
}

impl TextColor {
    fn from_palette(p: &NeutralPalette) -> Self {
        Self {
            heading: p.text_primary,
            primary: p.text_primary,
            secondary: p.text_secondary,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StatusColor {
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for StatusColor {
    fn default() -> Self {
        Self {
            success: Color::from_hex("#3ccb5a", 255),
            warning: Color::from_hex("#f5bd14", 255),
            danger: Color::from_hex("#ff330a", 255),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColors {
    pub primary: ButtonVariants,
    pub secondary: ButtonVariants,
    pub ghost: ButtonVariants,
}

impl ButtonColors {
    fn from(brand: Color, p: &NeutralPalette) -> Self {
        Self {
            primary: ButtonVariants::primary(brand),
            secondary: ButtonVariants::secondary(p),
            ghost: ButtonVariants::ghost(p),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonVariants {
    pub default: ButtonColorScheme,
    pub hover: ButtonColorScheme,
    pub pressed: ButtonColorScheme,
    pub disabled: ButtonColorScheme,
}

impl ButtonVariants {
    fn primary(brand: Color) -> Self {
        Self {
            default: ButtonColorScheme::new(brand, Color::WHITE, Color::TRANSPARENT),
            hover: ButtonColorScheme::new(Color::darken(brand, 0.85), Color::WHITE, Color::TRANSPARENT),
            pressed: ButtonColorScheme::new(Color::darken(brand, 0.8), Color::WHITE, Color::TRANSPARENT),
            disabled: ButtonColorScheme::new(Color::from_hex("#443f3f", 255), Color::BLACK, Color::TRANSPARENT),
        }
    }

    fn secondary(p: &NeutralPalette) -> Self {
        Self {
            default: ButtonColorScheme::new(Color::TRANSPARENT, p.text_primary, p.outline),
            hover: ButtonColorScheme::new(p.bg_secondary, p.text_primary, p.outline),
            pressed: ButtonColorScheme::new(p.bg_secondary, p.text_primary, p.text_primary),
            disabled: ButtonColorScheme::new(p.disabled, Color::BLACK, p.outline),
        }
    }

    fn ghost(p: &NeutralPalette) -> Self {
        Self {
            default: ButtonColorScheme::new(Color::TRANSPARENT, p.text_primary, Color::TRANSPARENT),
            hover: ButtonColorScheme::new(p.bg_secondary, p.text_primary, Color::TRANSPARENT),
            pressed: ButtonColorScheme::new(p.bg_secondary, p.text_primary, Color::TRANSPARENT),
            disabled: ButtonColorScheme::new(Color::TRANSPARENT, p.disabled, Color::TRANSPARENT),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColorScheme {
    pub background: Color,
    pub label: Color,
    pub outline: Color,
}

impl ButtonColorScheme {
    const fn new(background: Color, label: Color, outline: Color) -> Self {
        Self { background, label, outline }
    }
}

#[derive(Copy, Clone)]
struct NeutralPalette {
    bg_primary: Color,
    bg_secondary: Color,
    text_primary: Color,
    text_secondary: Color,
    outline: Color,
    disabled: Color,
}

impl NeutralPalette {
    fn light() -> Self {
        Self {
            bg_primary: Color::WHITE,
            bg_secondary: Color::from_hex("#DDDDDD", 255),
            text_primary: Color::BLACK,
            text_secondary: Color::from_hex("#9e9e9e", 255),
            outline: Color::from_hex("#585250", 255),
            disabled: Color::from_hex("#78716c", 255),
        }
    }

    fn dark() -> Self {
        Self {
            bg_primary: Color::BLACK,
            bg_secondary: Color::from_hex("#262322", 255),
            text_primary: Color::WHITE,
            text_secondary: Color::from_hex("#a7a29d", 255),
            outline: Color::from_hex("#585250", 255),
            disabled: Color::from_hex("#443f3f", 255),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color(canvas::Color);

impl Color {
    pub const WHITE: Color = Color(canvas::Color(255, 255, 255, 255));
    pub const BLACK: Color = Color(canvas::Color(0, 0, 0, 255));
    pub const TRANSPARENT: Color = Color(canvas::Color(0, 0, 0, 0));

    pub fn from_hex(color: &str, alpha: u8) -> Self {
        let ce = "Color was not a Hex Value";
        let c = hex::decode(color.strip_prefix('#').unwrap_or(color)).expect(ce);
        Color(canvas::Color(c[0], c[1], c[2], alpha))
    }

    pub fn darken(c: Color, factor: f32) -> Color {
        let c: canvas::Color = c.into();
        let avg = ((c.0 as f32 + c.1 as f32 + c.2 as f32) / 3.0) * 0.1;
        let f = |ch: u8| {
            let chf = ch as f32 * factor;
            ((avg + (chf - avg) * 0.1).clamp(0.0, 255.0)) as u8
        };
        Color(canvas::Color(f(c.0), f(c.1), f(c.2), c.3))
    }

    pub fn is_high_contrast(c: Color) -> bool {
        let c: canvas::Color = c.into();
        0.299*(c.0 as f32) + 0.587*(c.1 as f32) + 0.114*(c.2 as f32) > 128.0
    }
}

impl From<Color> for canvas::Color {
    fn from(val: Color) -> Self {
        val.0
    }
}
