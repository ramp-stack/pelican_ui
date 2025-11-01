use roost::{Assets, resources};

use std::collections::HashMap;

/// A collection of icons used throughout the application.
///
/// - Icons will automatically be adde to resources when they meet these conditions:
///     - Icons must be `.svg` files.
///     - Icons must be located in `project/resources/icons/`.
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
pub struct IconResources(HashMap<String, resources::Image>);

impl IconResources {
    pub const QUALITY: f32 = 8.0;

    pub fn default(assets: &mut Assets) -> Self {
        let mut icons = HashMap::new();
        assets.dirs().to_vec().iter().for_each(|d| d.dirs().for_each(|dir|  {
            if dir.path().to_str().unwrap() == "icons" {
                for entry in dir.entries() {
                    if let Some(i) = entry.path().to_str() {
                        if i.ends_with(".svg") {
                            let name = i.strip_prefix("icons/").unwrap_or(i).strip_suffix(".svg").unwrap_or(i).replace(" ", "_");
                            icons.insert(name.to_string(), assets.add_svg(&assets.load_file(i).unwrap(), Self::QUALITY));
                        }
                    }
                }
            }
        }));

        // println!("ICONS {:#?}", icons);
        Self(icons)
    }

    pub fn get(&self, name: &str) -> resources::Image {
        self.0.get(name).unwrap_or_else(|| self.0.get("pelican_ui").unwrap()).clone()
    }
}
