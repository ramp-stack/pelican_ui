use mustache::{Assets, resources, Context};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BrandResources {
    pub wordmark: resources::Image,
    pub logo: resources::Image,
    pub app_icon: resources::Image,
    pub error: resources::Image,
    pub illustrations: Illustrations,
}

impl BrandResources {
    pub const QUALITY: f32 = 8.0;

    pub fn new(
        logo: resources::Image, 
        wordmark: resources::Image,
        app_icon: resources::Image,
        error: resources::Image,
        illustrations: Illustrations
    ) -> Self {
        BrandResources { logo, wordmark, app_icon, error, illustrations }
    }

    pub fn default(assets: &mut Assets) -> Self {
        BrandResources {
            logo: assets.add_svg(&assets.load_file("brand/logo.svg").unwrap(), Self::QUALITY),
            wordmark: assets.add_svg(&assets.load_file("brand/wordmark.svg").unwrap(), Self::QUALITY),
            app_icon: assets.add_svg(&assets.load_file("brand/app_icon.svg").unwrap(), Self::QUALITY),
            error: assets.add_svg(&assets.load_file("brand/error.svg").unwrap(), Self::QUALITY),
            illustrations: Illustrations::default(),
        }
    }
}
 
/// Branding illustrations and images.
///
/// Store and display branding illustrations and images.
///
/// This is separate from [`icons`](crate::theme::IconResources)
///
///
/// # Adding a New Illustration
/// ```rust
/// let theme = Theme::default();
/// theme.brand.illustrations.insert(ctx, "fish_image", "fish_image.png");
/// ```
/// 
/// - The first string is the name of the image you will reference it as.
/// - The second string is the file path. Illustrations must be `.svg` or `.png` files located in `resources/brand/`.
#[derive(Clone, Debug, Default)]
pub struct Illustrations(pub HashMap<String, resources::Image>);

impl Illustrations {
    pub const QUALITY: f32 = 8.0;

    /// Get an illustration by it's name
    pub fn get(&self, name: &str) -> Option<resources::Image> {
        self.0.get(name).cloned()
    }

    /// Remove an illustration by it's name.
    pub fn remove(&mut self, name: &str)  {
        self.0.remove(name);
    }

    /// Insert an illustration.
    ///
    /// - The first string is the name of the image you will reference it as.
    /// - The second string is the file path. Illustrations must be `.svg` or `.png` files located in `resources/brand/`.
    pub fn insert(&mut self, ctx: &mut Context, name: &str, path: &str) {
        let mut illustration: Option<resources::Image> = None;
        if path.ends_with(".png") { illustration = Some(ctx.assets.add_image(image::load_from_memory(&ctx.assets.load_file(path).unwrap()).unwrap().into()))}
        if path.ends_with(".svg") { illustration = Some(ctx.assets.add_svg(&ctx.assets.load_file(path).unwrap(), 2.0)) }

        if let Some(i) = illustration {self.0.insert(name.to_string(), i);}
    }
}