use roost::{Assets, resources};

#[derive(Clone, Debug)]
pub struct BrandResources {
    pub wordmark: resources::Image,
    pub logo: resources::Image,
    pub app_icon: resources::Image,
    pub error: resources::Image,
}

impl BrandResources {
    pub const QUALITY: f32 = 8.0;

    pub fn new(
        logo: resources::Image, 
        wordmark: resources::Image,
        app_icon: resources::Image,
        error: resources::Image,
    ) -> Self {
        BrandResources { logo, wordmark, app_icon, error }
    }

    pub fn default(assets: &mut Assets) -> Self {
        BrandResources {
            logo: assets.add_svg(&assets.load_file("brand/logo.svg").unwrap(), Self::QUALITY),
            wordmark: assets.add_svg(&assets.load_file("brand/wordmark.svg").unwrap(), Self::QUALITY),
            app_icon: assets.add_svg(&assets.load_file("brand/app_icon.svg").unwrap(), Self::QUALITY),
            error: assets.add_svg(&assets.load_file("brand/error.svg").unwrap(), Self::QUALITY),        }
    }
}
