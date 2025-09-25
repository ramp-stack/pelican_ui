use mustache::{Assets, resources, Context};

use std::collections::HashMap;

/// A collection of icons used throughout the application.
///
/// # Adding a New Icon
/// ```rust
/// let theme = Theme::default();
/// theme.insert(ctx, "ice_cream");
/// ```
/// 
/// - Icons must be `.svg` files located in `resources/icons/`.
/// - The file name must match the name passed to the `insert` function.
///   For example: `"ice_cream"` corresponds to `resources/icons/ice_cream.svg`.
///
/// # Default Icons
/// - ![accounts](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/accounts.svg) `accounts`
/// - ![add](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/add.svg) `add`
/// - ![app_store](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/app_store.svg) `app_store`
/// - ![back](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/back.svg) `back`
/// - ![block](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/block.svg) `block`
/// - ![unblock](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/unblock.svg) `unblock`
/// - ![boot](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/boot.svg) `boot`
/// - ![unboot](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/unboot.svg) `unboot`
/// - ![backspace](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/backspace.svg) `backspace`
/// - ![bitcoin](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/bitcoin.svg) `bitcoin`
/// - ![camera](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/camera.svg) `camera`
/// - ![cancel](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/cancel.svg) `cancel`
/// - ![capslock](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/capslock.svg) `capslock`
/// - ![capslock_on](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/capslock_on.svg) `capslock_on`
/// - ![checkmark](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/checkmark.svg) `checkmark`
/// - ![close](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/close.svg) `close`
/// - ![copy](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/copy.svg) `copy`
/// - ![credential](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/credential.svg) `credential`
/// - ![down_arrow](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/down_arrow.svg) `down_arrow`
/// - ![delete](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/delete.svg) `delete`
/// - ![discord](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/discord.svg) `discord`
/// - ![door](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/door.svg) `door`
/// - ![down](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/down.svg) `down`
/// - ![edit](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/edit.svg) `edit`
/// - ![emoji](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/emoji.svg) `emoji`
/// - ![error](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/error.svg) `error`
/// - ![explore](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/explore.svg) `explore`
/// - ![facebook](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/facebook.svg) `facebook`
/// - ![forward](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/forward.svg) `forward`
/// - ![gif](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/gif.svg) `gif`
/// - ![group](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/group.svg) `group`
/// - ![heart](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/heart.svg) `heart`
/// - ![home](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/home.svg) `home`
/// - ![infinite](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/infinite.svg) `infinite`
/// - ![info](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/info.svg) `info`
/// - ![instagram](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/instagram.svg) `instagram`
/// - ![left](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/left.svg) `left`
/// - ![link](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/link.svg) `link`
/// - ![megaphone](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/megaphone.svg) `megaphone`
/// - ![messages](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/messages.svg) `messages`
/// - ![microphone](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/microphone.svg) `microphone`
/// - ![monitor](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/monitor.svg) `monitor`
/// - ![notification](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/notification.svg) `notification`
/// - ![paste](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/paste.svg) `paste`
/// - ![pelican_ui](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/pelican_ui.svg) `pelican_ui`
/// - ![photos](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/photos.svg) `photos`
/// - ![play_store](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/play_store.svg) `play_store`
/// - ![profile](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/profile.svg) `profile`
/// - ![qr_code](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/qr_code.svg) `qr_code`
/// - ![radio_filled](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/radio_filled.svg) `radio_filled`
/// - ![radio](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/radio.svg) `radio`
/// - ![right](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/right.svg) `right`
/// - ![scan](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/scan.svg) `scan`
/// - ![search](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/search.svg) `search`
/// - ![send](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/send.svg) `send`
/// - ![settings](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/settings.svg) `settings`
/// - ![up](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/up.svg) `up`
/// - ![wallet](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/wallet.svg) `wallet`
/// - ![warning](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/warning.svg) `warning`
/// - ![x](https://raw.githubusercontent.com/ramp-stack/pelican_ui/master/resources/icons/x.svg) `x`
pub struct IconResources(HashMap<&'static str, resources::Image>);

impl IconResources {
    pub const QUALITY: f32 = 8.0;
    pub fn default(assets: &mut Assets) -> Self {
        let mut icons = HashMap::new();

        icons.insert("accounts", assets.add_svg(&assets.load_file("icons/accounts.svg").unwrap(), Self::QUALITY));
        icons.insert("add", assets.add_svg(&assets.load_file("icons/add.svg").unwrap(), Self::QUALITY));
        icons.insert("app_store", assets.add_svg(&assets.load_file("icons/app_store.svg").unwrap(), Self::QUALITY));
        icons.insert("back", assets.add_svg(&assets.load_file("icons/back.svg").unwrap(), Self::QUALITY));
        icons.insert("block", assets.add_svg(&assets.load_file("icons/block.svg").unwrap(), Self::QUALITY));
        icons.insert("unblock", assets.add_svg(&assets.load_file("icons/unblock.svg").unwrap(), Self::QUALITY));
        icons.insert("boot", assets.add_svg(&assets.load_file("icons/boot.svg").unwrap(), Self::QUALITY));
        icons.insert("unboot", assets.add_svg(&assets.load_file("icons/unboot.svg").unwrap(), Self::QUALITY));
        icons.insert("backspace", assets.add_svg(&assets.load_file("icons/backspace.svg").unwrap(), Self::QUALITY));
        icons.insert("bitcoin", assets.add_svg(&assets.load_file("icons/bitcoin.svg").unwrap(), Self::QUALITY));
        icons.insert("camera", assets.add_svg(&assets.load_file("icons/camera.svg").unwrap(), Self::QUALITY));
        icons.insert("cancel", assets.add_svg(&assets.load_file("icons/cancel.svg").unwrap(), Self::QUALITY));
        icons.insert("capslock", assets.add_svg(&assets.load_file("icons/capslock.svg").unwrap(), Self::QUALITY));
        icons.insert("capslock_on", assets.add_svg(&assets.load_file("icons/capslock_on.svg").unwrap(), Self::QUALITY));
        icons.insert("checkmark", assets.add_svg(&assets.load_file("icons/checkmark.svg").unwrap(), Self::QUALITY));
        icons.insert("close", assets.add_svg(&assets.load_file("icons/close.svg").unwrap(), Self::QUALITY));
        icons.insert("copy", assets.add_svg(&assets.load_file("icons/copy.svg").unwrap(), Self::QUALITY));
        icons.insert("credential", assets.add_svg(&assets.load_file("icons/credential.svg").unwrap(), Self::QUALITY));
        icons.insert("down_arrow", assets.add_svg(&assets.load_file("icons/down_arrow.svg").unwrap(), Self::QUALITY));
        icons.insert("delete", assets.add_svg(&assets.load_file("icons/delete.svg").unwrap(), Self::QUALITY));
        icons.insert("discord", assets.add_svg(&assets.load_file("icons/discord.svg").unwrap(), Self::QUALITY));
        icons.insert("door", assets.add_svg(&assets.load_file("icons/door.svg").unwrap(), Self::QUALITY));
        icons.insert("down", assets.add_svg(&assets.load_file("icons/down.svg").unwrap(), Self::QUALITY));
        icons.insert("edit", assets.add_svg(&assets.load_file("icons/edit.svg").unwrap(), Self::QUALITY));
        icons.insert("emoji", assets.add_svg(&assets.load_file("icons/emoji.svg").unwrap(), Self::QUALITY));
        icons.insert("error", assets.add_svg(&assets.load_file("icons/error.svg").unwrap(), Self::QUALITY));
        icons.insert("explore", assets.add_svg(&assets.load_file("icons/explore.svg").unwrap(), Self::QUALITY));
        icons.insert("facebook", assets.add_svg(&assets.load_file("icons/facebook.svg").unwrap(), Self::QUALITY));
        icons.insert("forward", assets.add_svg(&assets.load_file("icons/forward.svg").unwrap(), Self::QUALITY));
        icons.insert("gif", assets.add_svg(&assets.load_file("icons/gif.svg").unwrap(), Self::QUALITY));
        icons.insert("group", assets.add_svg(&assets.load_file("icons/group.svg").unwrap(), Self::QUALITY));
        icons.insert("heart", assets.add_svg(&assets.load_file("icons/heart.svg").unwrap(), Self::QUALITY));
        icons.insert("home", assets.add_svg(&assets.load_file("icons/home.svg").unwrap(), Self::QUALITY));
        icons.insert("infinite", assets.add_svg(&assets.load_file("icons/infinite.svg").unwrap(), Self::QUALITY));
        icons.insert("info", assets.add_svg(&assets.load_file("icons/info.svg").unwrap(), Self::QUALITY));
        icons.insert("instagram", assets.add_svg(&assets.load_file("icons/instagram.svg").unwrap(), Self::QUALITY));
        icons.insert("left", assets.add_svg(&assets.load_file("icons/left.svg").unwrap(), Self::QUALITY));
        icons.insert("link", assets.add_svg(&assets.load_file("icons/link.svg").unwrap(), Self::QUALITY));
        icons.insert("megaphone", assets.add_svg(&assets.load_file("icons/megaphone.svg").unwrap(), Self::QUALITY));
        icons.insert("messages", assets.add_svg(&assets.load_file("icons/messages.svg").unwrap(), Self::QUALITY));
        icons.insert("microphone", assets.add_svg(&assets.load_file("icons/microphone.svg").unwrap(), Self::QUALITY));
        icons.insert("monitor", assets.add_svg(&assets.load_file("icons/monitor.svg").unwrap(), Self::QUALITY));
        icons.insert("notification", assets.add_svg(&assets.load_file("icons/notification.svg").unwrap(), Self::QUALITY));
        icons.insert("paste", assets.add_svg(&assets.load_file("icons/paste.svg").unwrap(), Self::QUALITY));
        icons.insert("pelican_ui", assets.add_svg(&assets.load_file("icons/pelican_ui.svg").unwrap(), Self::QUALITY));
        icons.insert("photos", assets.add_svg(&assets.load_file("icons/photos.svg").unwrap(), Self::QUALITY));
        icons.insert("play_store", assets.add_svg(&assets.load_file("icons/play_store.svg").unwrap(), Self::QUALITY));
        icons.insert("profile", assets.add_svg(&assets.load_file("icons/profile.svg").unwrap(), Self::QUALITY));
        icons.insert("qr_code", assets.add_svg(&assets.load_file("icons/qr_code.svg").unwrap(), Self::QUALITY));
        icons.insert("radio_filled", assets.add_svg(&assets.load_file("icons/radio_filled.svg").unwrap(), Self::QUALITY));
        icons.insert("radio", assets.add_svg(&assets.load_file("icons/radio.svg").unwrap(), Self::QUALITY));
        icons.insert("right", assets.add_svg(&assets.load_file("icons/right.svg").unwrap(), Self::QUALITY));
        icons.insert("scan", assets.add_svg(&assets.load_file("icons/scan.svg").unwrap(), Self::QUALITY));
        icons.insert("search", assets.add_svg(&assets.load_file("icons/search.svg").unwrap(), Self::QUALITY));
        icons.insert("send", assets.add_svg(&assets.load_file("icons/send.svg").unwrap(), Self::QUALITY));
        icons.insert("settings", assets.add_svg(&assets.load_file("icons/settings.svg").unwrap(), Self::QUALITY));
        icons.insert("up", assets.add_svg(&assets.load_file("icons/up.svg").unwrap(), Self::QUALITY));
        icons.insert("wallet", assets.add_svg(&assets.load_file("icons/wallet.svg").unwrap(), Self::QUALITY));
        icons.insert("warning", assets.add_svg(&assets.load_file("icons/warning.svg").unwrap(), Self::QUALITY));
        icons.insert("x", assets.add_svg(&assets.load_file("icons/x.svg").unwrap(), Self::QUALITY));

        Self(icons)
    }

    pub fn get(&self, name: &'static str) -> resources::Image {
        self.0.get(name).unwrap_or_else(|| self.0.get("pelican_ui").unwrap()).clone()
    }

    pub fn insert(&mut self, ctx: &mut Context, icon_name: &'static str) {
        let path = format!("icons/{icon_name}.svg");
        let svg = &ctx.assets.load_file(&path).unwrap();
        let icon = ctx.assets.add_svg(svg, Self::QUALITY);
        self.0.insert(icon_name, icon);
    }

    pub fn all(&self) -> HashMap<&'static str, resources::Image> {self.0.clone()}
}
