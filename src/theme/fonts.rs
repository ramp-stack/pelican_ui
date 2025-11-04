use roost_ui::{Assets, resources};

/// Represents a collection of font resources, including fonts and font sizes.
#[derive(Clone)]
pub struct FontResources {
    /// The fonts used for various UI elements (e.g., headings, text, labels, etc.).
    pub fonts: Fonts,
    /// The font sizes used throughout the UI.
    pub size: FontSize,
}

impl FontResources {
    /// Creates a new `FontResources` instance with the specified fonts and font sizes.
    pub fn new(fonts: Fonts, size: FontSize) -> Self {
        Self { fonts, size }
    }

    /// Loads the default font resources using the Assets to load fonts.
    pub fn default(assets: &mut Assets) -> Self {
        FontResources {
            fonts: Fonts::default(assets),
            size: FontSize::default(),
        }
    }
}

/// Defines a collection of fonts used throughout the application for various elements (headings, text, labels, etc.).
#[derive(Clone)]
pub struct Fonts {
    /// The font used for headings.
    pub heading: resources::Font,
    /// The font used for regular text.
    pub text: resources::Font,
    /// The font used for labels.
    pub label: resources::Font,
    /// The font used for keyboard elements.
    pub keyboard: resources::Font,
    // The font used for emoji characters.
    // pub emoji: resources::Font,
}

impl Fonts {
    /// Creates a new `Fonts` struct with the specified fonts.
    pub fn new(
        heading: resources::Font, 
        text: resources::Font, 
        label: resources::Font, 
        keyboard: resources::Font, 
        // emoji: resources::Font
    ) -> Self {
        Self { heading, text, label, keyboard }
    }

    /// Loads the default fonts using the provided Assets.
    pub fn default(assets: &mut Assets) -> Self {
        let bold = assets.load_font("fonts/outfit_bold.ttf").unwrap();
        let medium = assets.load_font("fonts/outfit_medium.ttf").unwrap();
        let regular = assets.load_font("fonts/outfit_regular.ttf").unwrap();
        // let emoji = assets.load_font("fonts/noto_color_emoji.ttf").unwrap();
        
        Self {
            heading: bold.clone(),
            text: regular,
            label: bold,
            keyboard: medium,
            // emoji,
        }
    }
}

/// Defines a struct that holds font sizes for various UI elements.
#[derive(Copy, Clone)]
pub struct FontSize {
    /// The font size used for title text.
    pub title: f32,
    /// The font size used for h1 headers.
    pub h1: f32,
    /// The font size used for h2 headers.
    pub h2: f32,
    /// The font size used for h3 headers.
    pub h3: f32,
    /// The font size used for h4 headers.
    pub h4: f32,
    /// The font size used for h5 headers.
    pub h5: f32,
    /// The font size used for h6 headers.
    pub h6: f32,
    /// The font size used for extra-large text.
    pub xl: f32,
    /// The font size used for large text.
    pub lg: f32,
    /// The font size used for medium text.
    pub md: f32,
    /// The font size used for small text.
    pub sm: f32,
    /// The font size used for extra-small text.
    pub xs: f32,
}

impl Default for FontSize {
    /// Returns the default font sizes used throughout the application.
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
