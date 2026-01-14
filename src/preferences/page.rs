/* preferences/page.rs
 *
 * Copyright 2023 Romeo Calota
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

use adw::{prelude::*, subclass::prelude::*, SpinRow, SwitchRow};
use gtk::{gio, glib, CheckButton, Scale};

use crate::preferences::{MAX_POINTS, MIN_POINTS};
use crate::settings;

const MAX_INTERVAL_TICKS: u64 = 200;
const MIN_INTERVAL_TICKS: u64 = 10;

macro_rules! connect_switch_to_setting {
    ($this: expr, $switch_row: expr, $setting: literal) => {
        $switch_row.connect_active_notify({
            move |switch_row| {
                if let Err(e) = settings!().set_boolean($setting, switch_row.is_active()) {
                    gtk::glib::g_critical!(
                        "MissionCenter::Preferences",
                        "Failed to set {} setting: {}",
                        $setting,
                        e
                    );
                }
            }
        });
    };
}

macro_rules! connect_checkbutton_pair_to_setting {
    ($truthy: expr, $falsy: expr, $setting: literal) => {
        {
            let truthy = $truthy.clone();
            $truthy.connect_active_notify(move |_| {
                if let Err(e) = settings!().set_boolean($setting, truthy.is_active()) {
                    gtk::glib::g_critical!(
                        "MissionCenter::Preferences",
                        "Failed to set {} setting: {}",
                        $setting,
                        e
                    );
                }
            });
        }

        {
            let truthy = $truthy.clone();
            $falsy.connect_active_notify(move |_| {
                if let Err(e) = settings!().set_boolean($setting, truthy.is_active()) {
                    gtk::glib::g_critical!(
                        "MissionCenter::Preferences",
                        "Failed to set {} setting: {}",
                        $setting,
                        e
                    );
                }
            });
        }
    };
}
mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/io/github/rinta/PukuPuku/ui/preferences/page.ui")]
    pub struct PreferencesPage {
        #[template_child]
        pub update_interval: TemplateChild<SpinRow>,
        #[template_child]
        pub data_points: TemplateChild<Scale>,

        #[template_child]
        pub smooth_graphs: TemplateChild<SwitchRow>,
        #[template_child]
        pub sliding_graphs: TemplateChild<SwitchRow>,
        #[template_child]
        pub network_dynamic_scaling: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_cpu: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_memory: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_disks: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_network: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_gpus: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_fans: TemplateChild<SwitchRow>,

        #[template_child]
        pub merged_process_stats: TemplateChild<SwitchRow>,
        #[template_child]
        pub remember_sorting: TemplateChild<SwitchRow>,
        #[template_child]
        pub remember_column_order: TemplateChild<SwitchRow>,
        #[template_child]
        pub core_count_affects_percentages: TemplateChild<SwitchRow>,
        #[template_child]
        pub show_column_separators: TemplateChild<SwitchRow>,

        #[template_child]
        pub toggle_memory_unit_bits: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_memory_unit_bytes: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_memory_base_2: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_memory_base_10: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_drive_unit_bits: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_drive_unit_bytes: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_drive_base_2: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_drive_base_10: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_net_unit_bits: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_net_unit_bytes: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_net_base_2: TemplateChild<CheckButton>,
        #[template_child]
        pub toggle_net_base_10: TemplateChild<CheckButton>,

        #[template_child]
        pub character_size_scale: TemplateChild<Scale>,
    }

    impl PreferencesPage {
        pub fn configure_update_speed(&self) {
            use crate::application::INTERVAL_STEP;
            use glib::g_critical;

            let settings = settings!();

            let new_interval = (self.update_interval.value() / INTERVAL_STEP).round() as u64;
            let new_points = self.data_points.value() as i32;

            if new_interval <= MAX_INTERVAL_TICKS && new_interval >= MIN_INTERVAL_TICKS {
                if settings
                    .set_uint64("app-update-interval-u64", new_interval)
                    .is_err()
                {
                    g_critical!(
                        "MissionCenter::Preferences",
                        "Failed to set update interval setting",
                    );
                }
            } else {
                g_critical!(
                    "MissionCenter::Preferences",
                    "Update interval out of bounds",
                );
            }

            if new_points <= MAX_POINTS && new_points >= MIN_POINTS {
                if settings
                    .set_int("performance-page-data-points", new_points)
                    .is_err()
                {
                    g_critical!(
                        "MissionCenter::Preferences",
                        "Failed to set update points setting",
                    );
                }
            } else {
                g_critical!(
                    "MissionCenter::Preferences",
                    "Points interval out of bounds",
                );
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesPage {
        const NAME: &'static str = "PreferencesPage";
        type Type = super::PreferencesPage;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            SwitchRow::ensure_type();

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.data_points
                .downcast_ref::<Scale>()
                .unwrap()
                .connect_value_changed({
                    let this = self.obj().downgrade();
                    move |_| {
                        if let Some(this) = this.upgrade() {
                            this.imp().configure_update_speed();
                        }
                    }
                });

            self.update_interval
                .downcast_ref::<SpinRow>()
                .unwrap()
                .connect_changed({
                    let this = self.obj().downgrade();
                    move |_| {
                        if let Some(this) = this.upgrade() {
                            this.imp().configure_update_speed();
                        }
                    }
                });

            connect_switch_to_setting!(self, self.smooth_graphs, "performance-smooth-graphs");
            connect_switch_to_setting!(self, self.sliding_graphs, "performance-sliding-graphs");
            connect_switch_to_setting!(
                self,
                self.network_dynamic_scaling,
                "performance-page-network-dynamic-scaling"
            );
            connect_switch_to_setting!(self, self.show_cpu, "performance-show-cpu");
            connect_switch_to_setting!(self, self.show_memory, "performance-show-memory");
            connect_switch_to_setting!(self, self.show_disks, "performance-show-disks");
            connect_switch_to_setting!(self, self.show_network, "performance-show-network");
            connect_switch_to_setting!(self, self.show_gpus, "performance-show-gpus");
            connect_switch_to_setting!(self, self.show_fans, "performance-show-fans");

            connect_switch_to_setting!(
                self,
                self.merged_process_stats,
                "apps-page-merged-process-stats"
            );
            connect_switch_to_setting!(self, self.remember_sorting, "apps-page-remember-sorting");
            connect_switch_to_setting!(
                self,
                self.remember_column_order,
                "apps-page-remember-column-order"
            );
            connect_switch_to_setting!(
                self,
                self.core_count_affects_percentages,
                "apps-page-core-count-affects-percentages"
            );
            connect_switch_to_setting!(
                self,
                self.show_column_separators,
                "apps-page-show-column-separators"
            );

            connect_checkbutton_pair_to_setting!(
                self.toggle_memory_unit_bytes,
                self.toggle_memory_unit_bits,
                "performance-page-memory2-use-bytes"
            );
            connect_checkbutton_pair_to_setting!(
                self.toggle_memory_base_2,
                self.toggle_memory_base_10,
                "performance-page-memory2-use-base2"
            );
            connect_checkbutton_pair_to_setting!(
                self.toggle_drive_unit_bytes,
                self.toggle_drive_unit_bits,
                "performance-page-drive-use-bytes"
            );
            connect_checkbutton_pair_to_setting!(
                self.toggle_drive_base_2,
                self.toggle_drive_base_10,
                "performance-page-drive-use-base2"
            );
            connect_checkbutton_pair_to_setting!(
                self.toggle_net_unit_bytes,
                self.toggle_net_unit_bits,
                "performance-page-network-use-bytes"
            );
            connect_checkbutton_pair_to_setting!(
                self.toggle_net_base_2,
                self.toggle_net_base_10,
                "performance-page-network-use-base2"
            );

            // Connect character size scale to GSettings
            self.character_size_scale
                .set_value(settings!().int("character-size").clamp(40, 800) as f64);
            self.character_size_scale.connect_value_changed({
                move |scale| {
                    let new_size = (scale.value().round() as i32).clamp(40, 800);
                    if let Err(e) = settings!().set_int("character-size", new_size) {
                        gtk::glib::g_critical!(
                            "MissionCenter::Preferences",
                            "Failed to set character-size setting: {}",
                            e
                        );
                    }
                }
            });
        }
    }

    impl WidgetImpl for PreferencesPage {}

    impl PreferencesPageImpl for PreferencesPage {}
}

glib::wrapper! {
    pub struct PreferencesPage(ObjectSubclass<imp::PreferencesPage>)
        @extends adw::PreferencesPage, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl PreferencesPage {
    pub fn new() -> Self {
        let this: Self = glib::Object::builder().build();

        this.set_initial_update_speed();

        let imp = this.imp();
        let settings = settings!();

        imp.smooth_graphs
            .set_active(settings.boolean("performance-smooth-graphs"));
        imp.sliding_graphs
            .set_active(settings.boolean("performance-sliding-graphs"));
        imp.network_dynamic_scaling
            .set_active(settings.boolean("performance-page-network-dynamic-scaling"));
        imp.show_cpu
            .set_active(settings.boolean("performance-show-cpu"));
        imp.show_memory
            .set_active(settings.boolean("performance-show-memory"));
        imp.show_disks
            .set_active(settings.boolean("performance-show-disks"));
        imp.show_network
            .set_active(settings.boolean("performance-show-network"));
        imp.show_gpus
            .set_active(settings.boolean("performance-show-gpus"));
        imp.show_fans
            .set_active(settings.boolean("performance-show-fans"));

        imp.merged_process_stats
            .set_active(settings.boolean("apps-page-merged-process-stats"));
        imp.remember_sorting
            .set_active(settings.boolean("apps-page-remember-sorting"));
        imp.remember_column_order
            .set_active(settings.boolean("apps-page-remember-column-order"));
        imp.core_count_affects_percentages
            .set_active(settings.boolean("apps-page-core-count-affects-percentages"));
        imp.show_column_separators
            .set_active(settings.boolean("apps-page-show-column-separators"));

        let memory_use_bytes = settings.boolean("performance-page-memory2-use-bytes");
        imp.toggle_memory_unit_bytes.set_active(memory_use_bytes);
        imp.toggle_memory_unit_bits.set_active(!memory_use_bytes);

        let memory_use_base2 = settings.boolean("performance-page-memory2-use-base2");
        imp.toggle_memory_base_2.set_active(memory_use_base2);
        imp.toggle_memory_base_10.set_active(!memory_use_base2);

        let drive_use_bytes = settings.boolean("performance-page-drive-use-bytes");
        imp.toggle_drive_unit_bytes.set_active(drive_use_bytes);
        imp.toggle_drive_unit_bits.set_active(!drive_use_bytes);

        let drive_use_base2 = settings.boolean("performance-page-drive-use-base2");
        imp.toggle_drive_base_2.set_active(drive_use_base2);
        imp.toggle_drive_base_10.set_active(!drive_use_base2);

        let net_use_bytes = settings.boolean("performance-page-network-use-bytes");
        imp.toggle_net_unit_bytes.set_active(net_use_bytes);
        imp.toggle_net_unit_bits.set_active(!net_use_bytes);

        let net_use_base2 = settings.boolean("performance-page-network-use-base2");
        imp.toggle_net_base_2.set_active(net_use_base2);
        imp.toggle_net_base_10.set_active(!net_use_base2);

        this
    }

    fn set_initial_update_speed(&self) {
        use crate::application::INTERVAL_STEP;

        let settings = settings!();

        let data_points = settings.int("performance-page-data-points");
        let update_interval_s = (settings.uint64("app-update-interval-u64") as f64) * INTERVAL_STEP;
        let this = self.imp();

        this.data_points.set_value(data_points as f64);
        this.update_interval.set_value(update_interval_s);
    }
}
