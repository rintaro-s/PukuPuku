/* application.rs
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

use std::cell::{BorrowError, Cell, Ref, RefCell};
use std::collections::HashMap;

use adw::glib::g_warning;
use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, g_critical, property::PropertySet},
};

use magpie_types::apps::icon::Icon;

use crate::about_system_dialog::AboutSystemDialog;
use crate::{config::VERSION, i18n::i18n, magpie_client::Readings};

pub const INTERVAL_STEP: f64 = 0.05;
pub const BASE_INTERVAL: f64 = 1f64;

#[macro_export]
macro_rules! app {
    () => {{
        use ::gtk::glib::object::Cast;
        ::gtk::gio::Application::default()
            .and_then(|app| app.downcast::<$crate::MissionCenterApplication>().ok())
            .expect("Failed to get MissionCenterApplication instance")
    }};
}

#[macro_export]
macro_rules! settings {
    () => {
        $crate::app!().settings()
    };
}

mod imp {
    use super::*;
    use crate::setup_readable_settings_cache;

    pub struct MissionCenterApplication {
        pub settings: Cell<Option<gio::Settings>>,
        pub sys_info: RefCell<Option<crate::magpie_client::MagpieClient>>,
        pub window: RefCell<Option<crate::MissionCenterWindow>>,

        pub apps_icons_cache: Cell<Option<HashMap<String, Icon>>>,
    }

    impl Default for MissionCenterApplication {
        fn default() -> Self {
            Self {
                settings: Cell::new(None),
                sys_info: RefCell::new(None),
                window: RefCell::new(None),
                apps_icons_cache: Cell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MissionCenterApplication {
        const NAME: &'static str = "MissioncenterApplication";
        type Type = super::MissionCenterApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MissionCenterApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_default();

            self.settings
                .set(Some(gio::Settings::new("io.github.rinta.PukuPuku")));

            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for MissionCenterApplication {
        fn activate(&self) {
            use gtk::glib::*;

            let application = self.obj();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.window() {
                window
            } else {
                let settings = unsafe { self.settings.take().unwrap_unchecked() };
                self.settings.set(Some(settings.clone()));

                let sys_info = crate::magpie_client::MagpieClient::new();

                let window = crate::MissionCenterWindow::new(&*application, &settings, &sys_info);

                setup_readable_settings_cache(&settings);

                window.connect_default_height_notify({
                    move |window| {
                        let settings = settings!();
                        settings
                            .set_int("window-height", window.default_height())
                            .unwrap_or_else(|err| {
                                g_critical!(
                                    "MissionCenter",
                                    "Failed to save window height: {}",
                                    err
                                );
                            });
                    }
                });
                window.connect_default_width_notify({
                    move |window| {
                        let settings = settings!();
                        settings
                            .set_int("window-width", window.default_width())
                            .unwrap_or_else(|err| {
                                g_critical!(
                                    "MissionCenter",
                                    "Failed to save window width: {}",
                                    err
                                );
                            });
                    }
                });

                window
                    .set_default_size(settings.int("window-width"), settings.int("window-height"));

                window.connect_maximized_notify({
                    move |window| {
                        let settings = settings!();
                        settings
                            .set_boolean("is-maximized", window.is_maximized())
                            .unwrap_or_else(|err| {
                                g_critical!(
                                    "MissionCenter",
                                    "Failed to save window maximization: {}",
                                    err
                                );
                            });
                    }
                });

                window.set_maximized(settings.boolean("is-maximized"));

                sys_info.set_core_count_affects_percentages(
                    settings.boolean("apps-page-core-count-affects-percentages"),
                );

                settings.connect_changed(
                    Some("apps-page-core-count-affects-percentages"),
                    move |settings, _| {
                        let app = app!();
                        match app.sys_info() {
                            Ok(sys_info) => {
                                sys_info.set_core_count_affects_percentages(
                                    settings.boolean("apps-page-core-count-affects-percentages"),
                                );
                            }
                            Err(e) => {
                                g_critical!(
                                    "MissionCenter",
                                    "Failed to get sys_info from MissionCenterApplication: {}",
                                    e
                                );
                            }
                        };
                    },
                );

                self.sys_info.set(Some(sys_info));

                let provider = gtk::CssProvider::new();
                provider.load_from_bytes(&Bytes::from_static(include_bytes!(
                    "../resources/ui/style.css"
                )));

                gtk::style_context_add_provider_for_display(
                    &gtk::gdk::Display::default().expect("Could not connect to a display."),
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );

                window.upcast()
            };

            window.present();

            self.window
                .set(window.downcast_ref::<crate::MissionCenterWindow>().cloned());
        }
    }

    impl GtkApplicationImpl for MissionCenterApplication {}

    impl AdwApplicationImpl for MissionCenterApplication {}
}

glib::wrapper! {
    pub struct MissionCenterApplication(ObjectSubclass<imp::MissionCenterApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MissionCenterApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        use glib::g_message;

        let this: Self = glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build();

        g_message!(
            "MissionCenter::Application",
            "Starting Mission Center v{}",
            env!("CARGO_PKG_VERSION")
        );

        this
    }

    pub fn set_initial_readings(&self, readings: Readings) {
        use gtk::glib::*;

        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to refresh data"
            );
            return;
        };

        window.set_initial_readings(readings)
    }

    pub fn set_app_icons(&self, icons: HashMap<String, Icon>) {
        self.imp().apps_icons_cache.set(Some(icons))
    }

    pub fn merge_app_icons(&self, icons: HashMap<String, Icon>) {
        let old = self.imp().apps_icons_cache.take();

        let Some(mut old) = old else {
            self.set_app_icons(icons);
            return;
        };

        for (app_id, icon) in icons {
            old.insert(app_id, icon);
        }

        self.set_app_icons(old);
    }

    pub fn missing_icons(&self, mut appids: Vec<&String>) -> Option<Vec<String>> {
        let this = self.imp();
        let icons = this.apps_icons_cache.take();

        this.apps_icons_cache.set(icons.clone());

        let Some(apps) = icons else {
            return Some(appids.drain(..).map(|app_id| app_id.to_string()).collect());
        };

        let out: Vec<_> = appids
            .drain(..)
            .filter_map(|appid| {
                if !apps.contains_key(appid) {
                    Some(appid.clone())
                } else {
                    None
                }
            })
            .collect();

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    pub fn get_app_icon(&self, app_id: &str) -> Icon {
        let this = self.imp();
        let icons = this.apps_icons_cache.take();

        this.apps_icons_cache.set(icons.clone());

        icons
            .map(|map| map.get(app_id).cloned())
            .flatten()
            .unwrap_or_default()
    }

    pub fn setup_animations(&self) {
        use gtk::glib::*;

        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to refresh data"
            );
            return;
        };

        window.setup_animations()
    }

    pub fn refresh_readings(&self, readings: &mut Readings) -> bool {
        use gtk::glib::*;

        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to refresh data"
            );
            return false;
        };

        window.update_readings(readings)
    }

    pub fn settings(&self) -> gio::Settings {
        unsafe { (&*self.imp().settings.as_ptr()).as_ref().unwrap_unchecked() }.clone()
    }

    pub fn sys_info(&self) -> Result<Ref<'_, crate::magpie_client::MagpieClient>, BorrowError> {
        match self.imp().sys_info.try_borrow() {
            Ok(sys_info_ref) => Ok(Ref::map(sys_info_ref, |sys_info_opt| match sys_info_opt {
                Some(sys_info) => sys_info,
                None => {
                    panic!("MissionCenter::Application::sys_info() called before sys_info was initialized");
                }
            })),
            Err(e) => Err(e),
        }
    }

    pub fn window(&self) -> Option<crate::MissionCenterWindow> {
        unsafe { &*self.imp().window.as_ptr() }.clone()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| {
                app.show_preferences();
            })
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let about_system_action = gio::ActionEntry::builder("system-about")
            .activate(move |app: &Self, _, _| app.show_system_about())
            .build();
        let keyboard_shortcuts_action = gio::ActionEntry::builder("keyboard-shortcuts")
            .activate(move |app: &Self, _, _| app.show_keyboard_shortcuts())
            .build();

        self.add_action_entries([
            quit_action,
            preferences_action,
            about_action,
            about_system_action,
            keyboard_shortcuts_action,
        ]);

        self.set_accels_for_action("app.preferences", &["<Control>comma"]);
        self.set_accels_for_action("app.keyboard-shortcuts", &["<Control>question"]);
    }

    fn show_preferences(&self) {
        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to show preferences"
            );
            return;
        };

        let preferences = crate::preferences::PreferencesDialog::new();
        preferences.present(Some(&window));
    }

    fn show_keyboard_shortcuts(&self) {
        let Some(app_window) = self.window() else {
            return;
        };

        let builder =
            gtk::Builder::from_resource("/io/github/rinta/PukuPuku/ui/keyboard_shortcuts.ui");
        let shortcuts_window = builder
            .object::<gtk::ShortcutsWindow>("keyboard_shortcuts")
            .expect("Failed to get shortcuts window");

        shortcuts_window.set_transient_for(Some(&app_window));
        shortcuts_window.set_modal(true);
        shortcuts_window.present();
    }

    fn show_system_about(&self) {
        let app = app!();
        let Ok(magpie) = app.sys_info() else {
            g_warning!("MissionCenter::Disk", "Failed to get magpie client");
            return;
        };

        let about = magpie.about_system();

        let dialog = AboutSystemDialog::new(about);

        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to show about dialog"
            );
            return;
        };

        dialog.present(Some(&window));
    }

    fn show_about(&self) {
        let Some(window) = self.window() else {
            g_critical!(
                "MissionCenter::Application",
                "No active window, when trying to show about dialog"
            );
            return;
        };

        let about = adw::AboutDialog::builder()
            .application_name("PukuPuku")
            .application_icon("io.github.rinta.PukuPuku")
            .developer_name("PukuPuku")
            .developers(["PukuPuku", "Romeo Calota", "QwertyChouskie", "jojo2357", "Jan Luca"])
            .translator_credits(i18n("translator-credits"))
            .version(VERSION)
            .issue_url("https://github.com/rinta/PukuPuku/issues")
            .copyright("© 2026 PukuPuku\n© 2023-2025 Mission Center Developers")
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/rinta/PukuPuku")
            .release_notes(r#"<p>Initial release of PukuPuku!</p>
<ul>
<li>Character-based status display with 3 mood states</li>
<li>CPU performance benchmark</li>
<li>I/O speed benchmark (disk read/write)</li>
<li>System monitoring (CPU, Memory, Disk, Network, GPU)</li>
<li>Hand-drawn illustration style design</li>
<li>Per-app and per-process resource breakdown</li>
<li>Services monitoring and control</li>
</ul>
<p>Based on Mission Center v1.1.0 architecture</p>"#)
            .build();

        about.add_credit_section(
            Some("Standing on the shoulders of giants"),
            &[
                "GTK https://www.gtk.org/",
                "GNOME https://www.gnome.org/",
                "Libadwaita https://gitlab.gnome.org/GNOME/libadwaita",
                "Blueprint Compiler https://jwestman.pages.gitlab.gnome.org/blueprint-compiler/",
                "NVTOP https://github.com/Syllo/nvtop",
                "Workbench https://github.com/sonnyp/Workbench",
                "And many more... Thank you all!",
            ],
        );

        about.present(Some(&window));
    }
}
