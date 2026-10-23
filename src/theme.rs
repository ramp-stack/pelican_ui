use include_dir::{include_dir, Dir};
use prism::drawable::Svg;
use prism::hardware::Image;
use ptsd::{ColorResources, IconResources, FontResources, utils::Assets};
pub use ptsd::Color;
use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::components::Gifs;

#[derive(Clone, Debug)]
pub struct BrandResources {
    pub wordmark: Svg,
    pub logo: Svg,
    pub app_icon: Svg,
    pub error: Svg,
    pub qr_code: Image,
    pub images: HashMap<String, Image>,
    pub svgs: HashMap<String, Svg>,
}

impl BrandResources {
    fn new(assets: &Assets) -> Self {
        let get = |name: &str| {
            assets.get_asset_svg(name).unwrap_or_else(|| panic!("Missing asset: {name}"))
        };
        let mut images = HashMap::new();
        let mut svgs = HashMap::new();
        for file in assets.files() {
            let path = file.path().to_string_lossy();
            let Some((name, ext)) = path.rsplit_once('.') else { continue };
            match ext {
                "svg" => {
                    svgs.entry(name.to_string()).or_insert_with(|| assets.get_svg(&path).unwrap());
                }
                "png" | "jpg" | "jpeg" => {
                    images.entry(name.to_string())
                        .or_insert_with(|| assets.get_image(&path).unwrap());
                }
                _ => {}
            }
        }
        Self {
            logo: get("logo"), wordmark: get("wordmark"), app_icon: get("app_icon"),
            error: get("error"), qr_code: assets.get_asset_image("qr_code").unwrap(),
            images, svgs,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColors {
    pub primary: ButtonColorSet,
    pub secondary: ButtonColorSet,
    pub ghost: ButtonColorSet,
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColorSet {
    pub default: ButtonColorScheme,
    pub disabled: ButtonColorScheme,
    pub hover: ButtonColorScheme,
    pub pressed: ButtonColorScheme,
}

#[derive(Copy, Clone, Debug)]
pub struct ButtonColorScheme {
    pub background: Color,
    pub label: Color,
    pub outline: Color,
}

impl ButtonColorScheme {
    fn from(resources: &ColorResources, variant: Variant, state: State) -> Self {
        ButtonColorScheme {
            background: resources.get(Button(variant, state, Slot::Background)),
            label: resources.get(Button(variant, state, Slot::Label)),
            outline: resources.get(Button(variant, state, Slot::Outline)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant { Primary, Secondary, Ghost }
impl Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Variant::{}", match self {
            Variant::Primary => "Primary", Variant::Secondary => "Secondary",
            Variant::Ghost => "Ghost",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State { Default, Hover, Pressed, Disabled }
impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State::{}", match self {
            State::Default => "Default", State::Hover => "Hover", State::Pressed => "Pressed",
            State::Disabled => "Disabled",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot { Background, Label, Outline }
impl Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slot::{}", match self {
            Slot::Background => "Background", Slot::Label => "Label", Slot::Outline => "Outline",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Button(pub Variant, pub State, pub Slot);
impl Display for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Button({}, {}, {})", self.0, self.1, self.2)
    }
}

impl Button {
    pub fn get(resources: &ColorResources, variant: Variant) -> ButtonColorSet {
        ButtonColorSet {
            default: ButtonColorScheme::from(resources, variant, State::Default),
            disabled: ButtonColorScheme::from(resources, variant, State::Disabled),
            hover: ButtonColorScheme::from(resources, variant, State::Hover),
            pressed: ButtonColorScheme::from(resources, variant, State::Pressed),
        }
    }

    fn map(resources: &mut ColorResources, is_dark: bool, brand: Color) {
        use Slot::*;
        use State::*;
        use Variant::*;

        resources.insert(Button(Primary, Default, Background), brand);
        resources.insert(Button(Primary, Default, Label), brand.contrasted());
        resources.insert(Button(Primary, Default, Outline), Color::transparent());
        resources.insert(Button(Primary, Hover, Background), brand.darken(0.75));
        resources.insert(Button(Primary, Hover, Label), brand.contrasted());
        resources.insert(Button(Primary, Hover, Outline), Color::transparent());
        resources.insert(Button(Primary, Pressed, Background), brand.darken(0.7));
        resources.insert(Button(Primary, Pressed, Label), brand.contrasted());
        resources.insert(Button(Primary, Pressed, Outline), Color::transparent());

        match is_dark {
            false => {
                resources.insert(
                    Button(Primary, Disabled, Background), Color::from_hex("#9e9e9e", 255),
                );
                resources.insert(Button(Primary, Disabled, Label), Color::from_hex("#585250", 255));
                resources.insert(Button(Primary, Disabled, Outline), Color::transparent());
                resources.insert(Button(Secondary, Default, Background), Color::transparent());
                resources.insert(Button(Secondary, Default, Label), Color::black());
                resources.insert(
                    Button(Secondary, Default, Outline), Color::from_hex("#585250", 255),
                );
                resources.insert(
                    Button(Secondary, Hover, Background), Color::from_hex("#DDDDDD", 255),
                );
                resources.insert(Button(Secondary, Hover, Label), Color::black());
                resources.insert(Button(Secondary, Hover, Outline), Color::from_hex("#585250", 255));
                resources.insert(
                    Button(Secondary, Pressed, Background), Color::from_hex("#DDDDDD", 255),
                );
                resources.insert(Button(Secondary, Pressed, Label), Color::black());
                resources.insert(Button(Secondary, Pressed, Outline), Color::black());
                resources.insert(
                    Button(Secondary, Disabled, Background), Color::from_hex("#443f3f", 255),
                );
                resources.insert(Button(Secondary, Disabled, Label), Color::black());
                resources.insert(
                    Button(Secondary, Disabled, Outline), Color::from_hex("#585250", 255),
                );
                resources.insert(Button(Ghost, Default, Background), Color::transparent());
                resources.insert(Button(Ghost, Default, Label), Color::black());
                resources.insert(Button(Ghost, Default, Outline), Color::transparent());
                resources.insert(Button(Ghost, Hover, Background), Color::from_hex("#DDDDDD", 255));
                resources.insert(Button(Ghost, Hover, Label), Color::black());
                resources.insert(Button(Ghost, Hover, Outline), Color::transparent());
                resources.insert(Button(Ghost, Pressed, Background), Color::from_hex("#DDDDDD", 255));
                resources.insert(Button(Ghost, Pressed, Label), Color::from_hex("#585250", 255));
                resources.insert(Button(Ghost, Pressed, Outline), Color::transparent());
                resources.insert(Button(Ghost, Disabled, Background), Color::transparent());
                resources.insert(Button(Ghost, Disabled, Label), Color::from_hex("#9e9e9e", 255));
                resources.insert(Button(Ghost, Disabled, Outline), Color::transparent());
            }
            true => {
                resources.insert(
                    Button(Primary, Disabled, Background), Color::from_hex("#443f3f", 255),
                );
                resources.insert(Button(Primary, Disabled, Label), Color::black());
                resources.insert(Button(Primary, Disabled, Outline), Color::transparent());
                resources.insert(Button(Secondary, Default, Background), Color::transparent());
                resources.insert(Button(Secondary, Default, Label), Color::white());
                resources.insert(
                    Button(Secondary, Default, Outline), Color::from_hex("#585250", 255),
                );
                resources.insert(
                    Button(Secondary, Hover, Background), Color::from_hex("#262322", 255),
                );
                resources.insert(Button(Secondary, Hover, Label), Color::white());
                resources.insert(Button(Secondary, Hover, Outline), Color::from_hex("#585250", 255));
                resources.insert(
                    Button(Secondary, Pressed, Background), Color::from_hex("#262322", 255),
                );
                resources.insert(Button(Secondary, Pressed, Label), Color::white());
                resources.insert(Button(Secondary, Pressed, Outline), Color::white());
                resources.insert(Button(Secondary, Disabled, Background), Color::transparent());
                resources.insert(Button(Secondary, Disabled, Label), Color::from_hex("#443f3f", 255));
                resources.insert(
                    Button(Secondary, Disabled, Outline), Color::from_hex("#443f3f", 255),
                );
                resources.insert(Button(Ghost, Default, Background), Color::transparent());
                resources.insert(Button(Ghost, Default, Label), Color::white());
                resources.insert(Button(Ghost, Default, Outline), Color::transparent());
                resources.insert(Button(Ghost, Hover, Background), Color::from_hex("#262322", 255));
                resources.insert(Button(Ghost, Hover, Label), Color::white());
                resources.insert(Button(Ghost, Hover, Outline), Color::transparent());
                resources.insert(Button(Ghost, Pressed, Background), Color::from_hex("#262322", 255));
                resources.insert(Button(Ghost, Pressed, Label), Color::white());
                resources.insert(Button(Ghost, Pressed, Outline), Color::transparent());
                resources.insert(Button(Ghost, Disabled, Background), Color::transparent());
                resources.insert(Button(Ghost, Disabled, Label), Color::from_hex("#585250", 255));
                resources.insert(Button(Ghost, Disabled, Outline), Color::transparent());
            }
        }
    }
}

macro_rules! icons {
    ($($variant:ident => $name:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum Icons {
            $($variant),*
        }

        impl Icons {
            pub fn map(icons: &mut IconResources) { $(icons.insert(Icons::$variant, $name);)* }
        }

        impl std::fmt::Display for Icons {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self { $(Icons::$variant => write!(f, $name),)* }
            }
        }
    };
}

icons! {
    Accounts => "accounts", Add => "add", AddUser => "add_user", AppStore => "app_store",
    Baby => "baby", Back => "back", Backspace => "backspace", Bitcoin => "bitcoin",
    Block => "block", Boot => "boot", Camera => "camera", Cancel => "cancel",
    CapslockOn => "capslock_on", Capslock => "capslock", Check => "check", Checkmark => "checkmark",
    Close => "close", Copy => "copy", Credential => "credential", Delete => "delete",
    Discord => "discord", Door => "door", DownArrow => "down_arrow", Down => "down", Edit => "edit",
    Emoji => "emoji", Error => "error", Explore => "explore", Facebook => "facebook",
    Forward => "forward", Gif => "gif", Group => "group", Heart => "heart", Home => "home",
    Infinite => "infinite", Info => "info", Instagram => "instagram", Keyboard => "keyboard",
    Left => "left", Link => "link", Megaphone => "megaphone", Messages => "messages",
    Microphone => "microphone", Monitor => "monitor", Notification => "notification",
    Paste => "paste", PelicanUI => "pelican_ui", Photos => "photos", Play => "play",
    PlayStore => "play_store", Profile => "profile", QrCode => "qr_code",
    RadioFilled => "radio_filled", Radio => "radio", Right => "right", Scan => "scan",
    Search => "search", Send => "send", Senior => "senior", Settings => "settings",
    Sticker => "sticker", Unblock => "unblock", Unboot => "unboot", Unchecked => "unchecked",
    Up => "up", Wallet => "wallet", Warning => "warning", X => "x",
}

#[derive(Debug, Clone)]
pub struct Theme(ptsd::Theme, BrandResources, Gifs);
impl Theme {
    pub fn from(assets: &Vec<Dir<'static>>, color: Color) -> Self {
        let assets = Assets::new(
            assets.iter().cloned()
                .chain([include_dir!("resources/icons"), include_dir!("resources")]).collect(),
        );
        let (theme, is_dark) = ptsd::Theme::from(&assets, color);
        Theme::new(&assets, theme, is_dark, color)
    }

    pub fn dark(assets: &Vec<Dir<'static>>, color: Color) -> Self {
        let assets = Assets::new(
            assets.iter().cloned()
                .chain([include_dir!("resources/icons"), include_dir!("resources")]).collect(),
        );
        Theme::new(&assets, ptsd::Theme::dark(&assets, color), true, color)
    }
    pub fn light(assets: &Vec<Dir<'static>>, color: Color) -> Self {
        let assets = Assets::new(
            assets.iter().cloned()
                .chain([include_dir!("resources/icons"), include_dir!("resources")]).collect(),
        );
        Theme::new(&assets, ptsd::Theme::light(&assets, color), false, color)
    }

    pub fn gifs(&self) -> &Gifs { &self.2 }
    pub fn colors(&self) -> &ColorResources { &self.0.colors }
    pub fn icons(&self) -> &IconResources { &self.0.icons }
    pub fn fonts(&self) -> &FontResources { &self.0.fonts }
    pub fn brand(&self) -> &BrandResources { &self.1 }

    pub fn colors_mut(&mut self) -> &mut ColorResources { &mut self.0.colors }
    pub fn icons_mut(&mut self) -> &mut IconResources { &mut self.0.icons }
    pub fn fonts_mut(&mut self) -> &mut FontResources { &mut self.0.fonts }
    pub fn brand_mut(&mut self) -> &mut BrandResources { &mut self.1 }

    fn new(assets: &Assets, mut inner: ptsd::Theme, is_dark: bool, color: Color) -> Self {
        Icons::map(&mut inner.icons);
        Button::map(&mut inner.colors, is_dark, color);
        Theme(inner, BrandResources::new(assets), Gifs::default())
    }
}
impl Default for Theme {
    fn default() -> Self {Self::dark(&vec![], Color::from_hex("#00a2ff", 255))}
}
