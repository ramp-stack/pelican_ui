use crate::Assets;
use wgpu_canvas::Color;

pub mod colors;
pub use colors::*;
pub mod fonts;
pub use fonts::*;
pub mod icons;
pub use icons::*;
pub mod brand;
pub use brand::*;
pub mod layout;
pub use layout::*;

/// Contains all visual and layout resources for the 
/// application's theme, including colors, fonts, icons, 
/// branding, and layout.
pub struct Theme {
    pub colors: ColorResources,
    pub fonts: FontResources,
    pub icons: IconResources,
    pub brand: BrandResources,
    pub layout: LayoutResources,
}

impl Theme {
    pub fn default(ctx: &mut Assets) -> Self {
        Theme {
            colors: ColorResources::default(),
            fonts: FontResources::default(ctx),
            icons: IconResources::default(ctx),
            brand: BrandResources::default(ctx),
            layout: LayoutResources::default(),
        }
    }

    /// Creates a new instance of the Theme object.
    pub fn new(
        colors: ColorResources, 
        fonts: FontResources, 
        icons: IconResources,
        brand: BrandResources,
        layout: LayoutResources,
    ) -> Self { 
        Theme { colors, fonts, icons, brand, layout } 
    }

    pub fn new_from(ctx: &mut Assets, primary: Color) -> Self {
        Theme {
            colors: ColorResources::new_from(primary),
            fonts: FontResources::default(ctx),
            icons: IconResources::default(ctx),
            brand: BrandResources::default(ctx),
            layout: LayoutResources::default(),
        }
    }

    pub fn light(ctx: &mut Assets, primary: Color) -> Self {
        Theme {
            colors: ColorResources::light(primary),
            fonts: FontResources::default(ctx),
            icons: IconResources::default(ctx),
            brand: BrandResources::default(ctx),
            layout: LayoutResources::default(),
        }
    }

    pub fn dark(ctx: &mut Assets, primary: Color) -> Self {
        Theme {
            colors: ColorResources::dark(primary),
            fonts: FontResources::default(ctx),
            icons: IconResources::default(ctx),
            brand: BrandResources::default(ctx),
            layout: LayoutResources::default(),
        }
    }
}
