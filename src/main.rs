/* main.rs
 *
 * Copyright 2024 Romeo Calota
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */
use adw::gdk;
use adw::glib::Bytes;
use application::MissionCenterApplication;
use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::gio::Settings;
use gtk::{gio, prelude::*, Image};
use i18n::{i18n, i18n_f};

use magpie_types::apps::icon::Icon;

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};
use window::MissionCenterWindow;

use crate::i18n::ni18n_f;

const APP_ID: &str = "io.github.rinta.PukuPuku";

mod about_system_dialog;
mod application;
mod apps_page;
mod benchmark;
mod benchmark_page;
mod i18n;
mod magpie_client;
mod performance_page;
mod preferences;
mod services_page;
mod table_view;
mod widgets;
mod window;

#[macro_export]
macro_rules! glib_clone {
    ($var:expr) => {{
        unsafe { &*$var.as_ptr() }.clone()
    }};
}

mod config {
    include!(concat!(env!("BUILD_ROOT"), "/src/config.rs"));
}

#[allow(deprecated)]
pub fn apply_icon_to_image(image: &gtk::Image, icon: Icon, width: i32) -> bool {
    #[inline]
    fn apply_blank(image: &gtk::Image) -> bool {
        image.set_icon_name(Some("application-x-executable"));

        false
    }

    fn set_icon_from_stringlike(image: &gtk::Image, icon_name: &str) -> bool {
        let display = gdk::Display::default().unwrap();
        let icon_theme = gtk::IconTheme::for_display(&display);

        if icon_theme.has_icon(icon_name) {
            image.set_icon_name(Some(icon_name));

            true
        } else {
            apply_blank(image);

            false
        }
    }

    fn apply_bytes(image: &Image, img: &Vec<u8>, width: i32) -> bool {
        let input_stream = gio::MemoryInputStream::from_bytes(&Bytes::from(img));
        let Ok(pixbuf) = gdk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
            &input_stream,
            width,
            -1,
            true,
            None::<&gio::Cancellable>,
        ) else {
            apply_blank(image);

            return false;
        };

        image.set_from_pixbuf(Some(&pixbuf));

        true
    }

    match icon {
        Icon::Empty(_) => apply_blank(image),
        Icon::Id(id) => set_icon_from_stringlike(image, &id),
        Icon::Data(img) => apply_bytes(image, &img, width),
    }
}

fn user_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();

    HOME.get_or_init(|| {
        env::var_os("HOME")
            .or(env::var_os("USERPROFILE"))
            .map(|v| PathBuf::from(v))
            .unwrap_or(if cfg!(windows) {
                "C:\\Windows\\Temp".into()
            } else {
                "/tmp".into()
            })
    })
    .as_path()
}

fn flatpak_data_dir() -> &'static Path {
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

    DATA_DIR
        .get_or_init(|| {
            let path = user_home().join(format!(".var/app/{}/data", APP_ID));
            std::fs::create_dir_all(&path).expect("Failed to create flatpak data directory");
            path
        })
        .as_path()
}

pub fn is_flatpak() -> bool {
    static IS_FLATPAK: OnceLock<bool> = OnceLock::new();
    *IS_FLATPAK.get_or_init(|| Path::new("/.flatpak-info").exists())
}

// tysm gdu
pub fn to_long_human_readable_time(seconds: u64) -> String {
    const USEC_PER_YEAR: u64 = 60 * 60 * 6 * 1461; // ((60 * 60 * 24) as f32 * 365.25);
    const USEC_PER_MONTH: u64 = 60 * 30 * 1461; // ((60 * 60 * 24) as f32 * 365.25 / 12.0);
    const USEC_PER_DAY: u64 = 60 * 60 * 24;
    const USEC_PER_HOUR: u64 = 60 * 60;
    const USEC_PER_MINUTE: u64 = 60;

    pub fn years_to_string(time: u32) -> String {
        ni18n_f("{} year", "{} years", time, &[&time.to_string()])
    }

    pub fn months_to_string(time: u32) -> String {
        ni18n_f("{} month", "{} months", time, &[&time.to_string()])
    }

    pub fn days_to_string(time: u32) -> String {
        ni18n_f("{} day", "{} days", time, &[&time.to_string()])
    }

    pub fn hours_to_string(time: u32) -> String {
        ni18n_f("{} hour", "{} hours", time, &[&time.to_string()])
    }

    pub fn minutes_to_string(time: u32) -> String {
        ni18n_f("{} minute", "{} minutes", time, &[&time.to_string()])
    }

    pub fn seconds_to_string(time: u32) -> String {
        ni18n_f("{} second", "{} seconds", time, &[&time.to_string()])
    }

    let mut t = seconds;
    let years = t / USEC_PER_YEAR;
    t -= years * USEC_PER_YEAR;

    let months = t / USEC_PER_MONTH;
    t -= months * USEC_PER_MONTH;

    let days = t / USEC_PER_DAY;
    t -= days * USEC_PER_DAY;

    let hours = t / USEC_PER_HOUR;
    t -= hours * USEC_PER_HOUR;

    let minutes = t / USEC_PER_MINUTE;
    t -= minutes * USEC_PER_MINUTE;

    let seconds = t;

    let string3 = years_to_string(years as u32);
    let years_str = string3.as_str();
    let string2 = months_to_string(months as u32);
    let months_str = string2.as_str();
    let string1 = days_to_string(days as u32);
    let days_str = string1.as_str();
    let string = hours_to_string(hours as u32);
    let hours_str = string.as_str();
    let string4 = minutes_to_string(minutes as u32);
    let minutes_str = string4.as_str();
    let string5 = seconds_to_string(seconds as u32);
    let seconds_str = string5.as_str();

    if years > 0 {
        /* Translators: Used for duration greater than one year. First %s is number of years, second %s is months, third %s is days */
        i18n_f("{}, {} and {}", &[years_str, months_str, days_str])
    } else if months > 0 {
        /* Translators: Used for durations less than one year but greater than one month. First %s is number of months, second %s is days */
        i18n_f("{} and {}", &[months_str, days_str])
    } else if days > 0 {
        /* Translators: Used for durations less than one month but greater than one day. First %s is number of days, second %s is hours */
        i18n_f("{} and {}", &[days_str, hours_str])
    } else if hours > 0 {
        /* Translators: Used for durations less than one day but greater than one hour. First %s is number of hours, second %s is minutes */
        i18n_f("{} and {}", &[hours_str, minutes_str])
    } else if minutes > 0 {
        String::from(minutes_str)
    } else if seconds == 0 {
        seconds_str.to_string()
    } else {
        i18n("Less than a minute")
    }
}

pub fn to_short_human_readable_time(seconds: u32) -> String {
    let mins = seconds / 60;
    let seconds = seconds % 60;

    let seconds_string = ni18n_f("{} second", "{} seconds", seconds, &[&seconds.to_string()]);

    let mins_string = ni18n_f("{} minute", "{} minutes", mins, &[&(mins).to_string()]);

    if mins > 0 {
        format!("{} {}", mins_string, seconds_string)
    } else {
        format!("{}", seconds_string)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DataType {
    MemoryBytes,
    DriveBytes,
    DriveBytesPerSecond,
    NetworkBytes,
    NetworkBytesPerSecond,
    Hertz,
    Watts,
}

impl DataType {
    const CACHABLE_TYPES: [DataType; 5] = [
        DataType::MemoryBytes,
        DataType::DriveBytes,
        DataType::DriveBytesPerSecond,
        DataType::NetworkBytes,
        DataType::NetworkBytesPerSecond,
    ];
}

// i would LOVE to use variables, but i dunno how to make this const (compile time concat)
const fn data_type_setting_byte_setting(data_type: &DataType) -> &'static str {
    match data_type {
        // because we biffed it. see https://gitlab.com/mission-center-devs/mission-center/-/issues/385
        DataType::MemoryBytes => &const { concat!("performance-page-", "memory2", "-use-bytes") },
        DataType::DriveBytes => &const { concat!("performance-page-", "drive", "-use-bytes") },
        DataType::DriveBytesPerSecond => {
            &const { concat!("performance-page-", "drive", "-use-bytes") }
        }
        DataType::NetworkBytes => &const { concat!("performance-page-", "network", "-use-bytes") },
        DataType::NetworkBytesPerSecond => {
            &const { concat!("performance-page-", "network", "-use-bytes") }
        }
        DataType::Hertz => {
            panic!("Hertz data type not supported yet");
        }
        DataType::Watts => {
            panic!("Watts data type not supported yet");
        }
    }
}

// i would LOVE to use variables, but i dunno how to make this const (compile time concat)
const fn data_type_setting_base_setting(data_type: &DataType) -> &'static str {
    match data_type {
        // because we biffed it. see https://gitlab.com/mission-center-devs/mission-center/-/issues/385
        DataType::MemoryBytes => &const { concat!("performance-page-", "memory2", "-use-base2") },
        DataType::DriveBytes => &const { concat!("performance-page-", "drive", "-use-base2") },
        DataType::DriveBytesPerSecond => {
            &const { concat!("performance-page-", "drive", "-use-base2") }
        }
        DataType::NetworkBytes => &const { concat!("performance-page-", "network", "-use-base2") },
        DataType::NetworkBytesPerSecond => {
            &const { concat!("performance-page-", "network", "-use-base2") }
        }
        DataType::Hertz => {
            panic!("Hertz data type not supported yet");
        }
        DataType::Watts => {
            panic!("Watts data type not supported yet");
        }
    }
}

pub fn to_human_readable_adv_str(
    value_bytes: f32,
    use_bytes: bool,
    use_binary: bool,
    per_second: bool,
    unit_label: &str,
    min_exponent: usize,
) -> String {
    const UNITS: [&'static str; 9] = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];

    let divisor = if use_binary { 1024. } else { 1000. };

    let (mut value, label) = if use_bytes {
        (value_bytes, if per_second { "/s" } else { "" })
    } else {
        (value_bytes * 8., if per_second { "ps" } else { "" })
    };

    let mut exponent = 0;

    // Only display bit/byte values in the given range or higher, not individual bits/bytes
    // This sacrifices some precision for the sake of readability
    while exponent < min_exponent {
        value /= divisor;
        exponent += 1;
    }

    while value >= divisor && exponent < UNITS.len() - 1 {
        value /= divisor;
        exponent += 1;
    }

    // Calculate number of decimals to display
    // Only display fractional values for bit/byte numbers in the defined range and bigger
    // This sacrifices some precision for the sake of readability
    let dec_to_display = if exponent > min_exponent {
        if value < 10.0 {
            2
        } else if value < 100.0 {
            1
        } else {
            0
        }
    } else {
        0
    };

    format!(
        "{0:.1$} {2}{3}{4}{5}",
        value,
        dec_to_display,
        UNITS[exponent],
        if use_binary { "i" } else { "" },
        unit_label,
        label
    )
}

static BOOLEAN_DICT_CACHE: LazyLock<Mutex<HashMap<String, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn setup_readable_settings_cache(settings: &Settings) {
    for data_type in DataType::CACHABLE_TYPES {
        let byte_key = data_type_setting_byte_setting(&data_type);

        settings.connect_changed(Some(byte_key), |settings, _| {
            let Ok(mut hm) = BOOLEAN_DICT_CACHE.lock() else {
                return;
            };

            hm.insert(byte_key.to_string(), settings.boolean(byte_key));
        });
        let base_key = data_type_setting_base_setting(&data_type);

        settings.connect_changed(Some(base_key), |settings, _| {
            let Ok(mut hm) = BOOLEAN_DICT_CACHE.lock() else {
                return;
            };

            hm.insert(base_key.to_string(), settings.boolean(base_key));
        });
    }
}

pub fn to_human_readable_nice(value_bytes: f32, data_type: &DataType) -> String {
    let label = match data_type {
        DataType::Hertz => "Hz",
        DataType::Watts => "W",
        _ => {
            let dict_lock = BOOLEAN_DICT_CACHE.lock();

            let byte_key = data_type_setting_byte_setting(data_type);
            let base_key = data_type_setting_base_setting(data_type);

            let (bytes, bases) = if let Ok(mut dict) = dict_lock {
                let bytes = if let Some(b) = dict.get(byte_key) {
                    *b
                } else {
                    let b = settings!().boolean(byte_key);

                    dict.insert(byte_key.to_string(), b);

                    b
                };

                let bases = if let Some(b) = dict.get(base_key) {
                    *b
                } else {
                    let b = settings!().boolean(base_key);

                    dict.insert(base_key.to_string(), b);

                    b
                };

                (bytes, bases)
            } else {
                let settings = settings!();
                (settings.boolean(byte_key), settings.boolean(base_key))
            };

            return to_human_readable_adv_str(
                value_bytes,
                bytes,
                bases,
                *data_type == DataType::NetworkBytesPerSecond
                    || *data_type == DataType::DriveBytesPerSecond,
                if bytes { "B" } else { "b" },
                1,
            );
        }
    };

    to_human_readable_adv_str(value_bytes, true, false, false, label, 0)
}

pub fn show_error_dialog_and_exit(message: &str) -> ! {
    use crate::i18n::*;
    use adw::prelude::*;

    let message = Arc::<str>::from(message);
    gtk::glib::idle_add_once(move || {
        let app_window = app!().window();

        let error_dialog =
            adw::AlertDialog::new(Some("A fatal error has occurred"), Some(message.as_ref()));
        error_dialog.add_responses(&[("close", &i18n("_Quit"))]);
        error_dialog.set_response_appearance("close", adw::ResponseAppearance::Destructive);
        error_dialog.connect_response(None, |dialog, _| {
            dialog.close();
            std::process::exit(-1);
        });
        error_dialog.present(app_window.as_ref());
    });

    loop {}
}

fn main() {
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let gresource_dir = if let Ok(gresource_dir) = std::env::var("MC_RESOURCE_DIR") {
        gresource_dir
    } else {
        PKGDATADIR.to_owned()
    };

    let resources = gio::Resource::load(&format!("{gresource_dir}/pukupuku.gresource"))
        .expect("Could not load resources");
    gio::resources_register(&resources);

    let app = MissionCenterApplication::new(
        APP_ID,
        &gio::ApplicationFlags::empty(),
    );
    gtk::Application::set_default(app.upcast_ref::<gtk::Application>());

    let exit_code = app.run();
    std::process::exit(exit_code.get() as _);
}
