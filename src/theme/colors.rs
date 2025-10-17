use mustache::drawable::Color;

/// Represents a collection of color resources used throughout the UI, including background, text, button, and status colors.
#[derive(Clone, Debug, Default, Copy)]
pub struct ColorResources {
    /// Defines the background colors.
    pub background: BackgroundColor,
    /// Defines the outline colors.
    pub outline: OutlineColor,
    /// Defines the colors representing various status indicators.
    pub status: StatusColor,
    /// Defines the colors for various text elements.
    pub text: TextColor,
    /// Defines the colors for buttons in various states (default, disabled, hover, etc.).
    pub button: ButtonColors,
    /// Defines brand-specific colors.
    pub brand: Color,
}

impl ColorResources {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        background: BackgroundColor,
        outline: OutlineColor,
        status: StatusColor,
        text: TextColor,
        brand: Color,
        button: ButtonColors,
    ) -> Self {
        ColorResources { background, outline, status, text, brand, button }
    }

    /// Create a light theme from the given primary color.
    pub fn light(brand: Color) -> Self {
        ColorResources { 
            background: BackgroundColor::light(),
            outline: OutlineColor::light(),
            status: StatusColor::default(),
            text: TextColor::light(),
            brand,
            button: ButtonColors::from(brand, true),
        }
    }

    /// Create a dark theme from the given primary color.
    pub fn dark(brand: Color) -> Self {
        ColorResources { 
            background: BackgroundColor::dark(),
            outline: OutlineColor::dark(),
            status: StatusColor::default(),
            text: TextColor::dark(),
            brand,
            button: ButtonColors::from(brand, false),
        }
    }

    /// Create a new theme from the brand color.
    /// Chooses light or dark depending on brightness of the primary color.
    pub fn from(brand: Color) -> Self {
        match Color::is_high_contrast(brand) {
            true => Self::dark(brand),
            false => Self::light(brand)
        }
    }
}

/// Defines the background colors.   
#[derive(Copy, Clone, Debug)]
pub struct BackgroundColor {
    pub primary: Color,
    pub secondary: Color,
}

impl BackgroundColor {
    pub fn dark() -> Self {
        BackgroundColor {
            primary: Color::BLACK,
            secondary: Color::from_hex("#262322", 255),
        }
    }
    pub fn light() -> Self {
        BackgroundColor {
            primary: Color::WHITE,
            secondary: Color::from_hex("#DDDDDD", 255),
        }
    }
}

impl Default for BackgroundColor {
    fn default() -> Self { Self::dark() }
}

/// Defines the outline colors.
#[derive(Copy, Clone, Debug)]
pub struct OutlineColor {
    pub primary: Color,
    pub secondary: Color,
}

impl OutlineColor {
    pub fn dark() -> Self {
        OutlineColor {
            primary: Color::WHITE,
            secondary: Color::from_hex("#585250", 255),
        }
    }
    pub fn light() -> Self {
        OutlineColor {
            primary: Color::from_hex("#9e9e9e", 255),
            secondary: Color::from_hex("#444444", 255),
        }
    }
}

impl Default for OutlineColor {
    fn default() -> Self { Self::dark() }
}

/// Defines the colors of text elements.
#[derive(Copy, Clone, Debug)]
pub struct TextColor {
    pub heading: Color,
    pub primary: Color,
    pub secondary: Color,
}

impl TextColor {
    pub fn dark() -> Self {
        TextColor {
            heading: Color::WHITE,
            primary: Color::from_hex("#e2e1df", 255),
            secondary: Color::from_hex("#a7a29d", 255),
        }
    }
    pub fn light() -> Self {
        TextColor {
            heading: Color::BLACK,
            primary: Color::BLACK,
            secondary: Color::from_hex("#9e9e9e", 255),
        }
    }
}

impl Default for TextColor {
    fn default() -> Self { Self::dark() }
}

/// Defines the colors representing various status indicators.
#[derive(Copy, Clone, Debug)]
pub struct StatusColor {
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for StatusColor {
    fn default() -> Self {
        StatusColor {
            success: Color::from_hex("#3ccb5a", 255),
            warning: Color::from_hex("#f5bd14", 255),
            danger: Color::from_hex("#ff330a", 255),
        }
    }
}

/// Defines the colors for buttons in various states, including default, disabled, hover, pressed, etc.
#[derive(Copy, Clone, Debug)]
pub struct ButtonColors {
    pub primary: ButtonColorSet,
    pub secondary: ButtonColorSet,
    pub ghost: ButtonColorSet,
}

impl ButtonColors {
    pub fn from(brand: Color, is_light: bool) -> Self {
        let label = if is_light {Color::WHITE} else {Color::BLACK};
        ButtonColors {
            primary: ButtonColorSet::primary(brand, label),
            secondary: ButtonColorSet::secondary(is_light),
            ghost: ButtonColorSet::ghost(is_light),
        }
    }
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            primary: ButtonColorSet::primary(Color::from_hex("#02f0cc", 255), Color::WHITE),
            secondary: ButtonColorSet::secondary(false),
            ghost: ButtonColorSet::ghost(false),
        }
    }
}

/// Defines a color scheme for a button, including background, label, and outline colors.
#[derive(Copy, Clone, Debug)]
pub struct ButtonColorScheme {
    /// The background color of the button.
    pub background: Color,
    /// The color of the button label (text).
    pub label: Color,
    /// The color of the button outline (border).
    pub outline: Color,
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColorSet {
    pub default: ButtonColorScheme,
    pub disabled: ButtonColorScheme,
    pub hover: ButtonColorScheme,
    pub pressed: ButtonColorScheme,
}

impl ButtonColorSet {
    pub fn primary(brand: Color, label: Color) -> Self {
        ButtonColorSet {
            default: ButtonColorScheme {
                background: brand,
                label,
                outline: Color::TRANSPARENT,
            },
            disabled: ButtonColorScheme {
                background: Color::from_hex("#443f3f", 255),
                label: Color::BLACK,
                outline: Color::TRANSPARENT,
            },
            hover: ButtonColorScheme {
                background: Color::darken(brand, 0.85),
                label,
                outline: Color::TRANSPARENT,
            },
            pressed: ButtonColorScheme {
                background: Color::darken(brand, 0.8),
                label,
                outline: Color::TRANSPARENT
            }
        }
    }

    pub fn secondary(is_light: bool) -> Self {
        ButtonColorSet {
            default: ButtonColorScheme {
                background: Color::TRANSPARENT,
                label: if is_light {Color::BLACK} else {Color::WHITE},
                outline: Color::from_hex("#585250", 255),
            },
            disabled: ButtonColorScheme {
                background: Color::from_hex("#78716c", 255),
                label: Color::BLACK,
                outline:Color::from_hex("#585250", 255),
            },
            hover: ButtonColorScheme {
                background: if is_light {Color::from_hex("#dcdbda", 255)} else {Color::from_hex("#262322", 255)},
                label: Color::WHITE,
                outline: Color::from_hex("#585250", 255),
            },
            pressed: ButtonColorScheme {
                background: if is_light {Color::from_hex("#dcdbda", 255)} else {Color::from_hex("#262322", 255)},
                label: Color::WHITE,
                outline: if is_light {Color::BLACK} else {Color::WHITE},
            }
        }
    }

    pub fn ghost(is_light: bool) -> Self {
        ButtonColorSet {
            default: ButtonColorScheme {
                background: Color::TRANSPARENT,
                label: if is_light {Color::BLACK} else {Color::WHITE},
                outline: Color::TRANSPARENT,
            },
            disabled: ButtonColorScheme {
                background: Color::TRANSPARENT,
                label: Color::from_hex("#78716c", 255),
                outline: Color::TRANSPARENT,
            },
            hover: ButtonColorScheme {
                background: if is_light {Color::from_hex("#dcdbda", 255)} else {Color::from_hex("#262322", 255)},
                label: if is_light {Color::BLACK} else {Color::WHITE},
                outline: Color::TRANSPARENT,
            },
            pressed: ButtonColorScheme {
                background: if is_light {Color::from_hex("#dcdbda", 255)} else {Color::from_hex("#262322", 255)},
                label: if is_light {Color::BLACK} else {Color::WHITE},
                outline: Color::TRANSPARENT,
            }
        }
    }
}