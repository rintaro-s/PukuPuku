/* performance_page/gpu.rs
 *
 * Copyright 2025 Mission Center Developers
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

use std::cell::{Cell, RefCell};
use std::fmt::Write;

use adw::{self, subclass::prelude::*};
use arrayvec::ArrayString;
use glib::{g_critical, g_warning, ParamSpec, Properties, Value};
use gtk::{gio, glib, prelude::*};

use magpie_types::gpus::Gpu;
use magpie_types::gpus::OpenGlVariant;

use crate::performance_page::widgets::{DatasetGroup, GraphWidget, ScalingSettings};
use crate::{
    application::INTERVAL_STEP, i18n::*, settings, to_short_human_readable_time, DataType,
};

use super::{GpuDetails, PageExt};

mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::PerformancePageGpu)]
    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/github/rinta/PukuPuku/ui/performance_page/gpu.ui")]
    pub struct PerformancePageGpu {
        #[template_child]
        pub gpu_id: TemplateChild<gtk::Label>,
        #[template_child]
        pub device_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub graph_utilization: TemplateChild<GraphWidget>,
        #[template_child]
        pub container_bottom: TemplateChild<gtk::Box>,
        #[template_child]
        pub encode_decode_graph: TemplateChild<gtk::Box>,
        #[template_child]
        pub usage_graph_encode_decode: TemplateChild<GraphWidget>,
        #[template_child]
        pub memory_graph: TemplateChild<gtk::Box>,
        #[template_child]
        pub total_memory: TemplateChild<gtk::Label>,
        #[template_child]
        pub memory_graph_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub usage_graph_memory: TemplateChild<GraphWidget>,
        #[template_child]
        pub context_menu: TemplateChild<gtk::Popover>,
        #[template_child]
        pub graph_max_duration: TemplateChild<gtk::Label>,

        #[property(get = Self::name, set = Self::set_name, type = String)]
        name: RefCell<String>,
        #[property(get, set)]
        base_color: Cell<gtk::gdk::RGBA>,
        #[property(get, set)]
        summary_mode: Cell<bool>,

        #[property(get, set)]
        encode_decode_available: Cell<bool>,

        #[property(get = Self::infobar_content, type = Option < gtk::Widget >)]
        pub infobar_content: GpuDetails,

        show_enc_dec_action: gio::SimpleAction,
    }

    impl Default for PerformancePageGpu {
        fn default() -> Self {
            Self {
                gpu_id: Default::default(),
                device_name: Default::default(),
                graph_utilization: Default::default(),
                container_bottom: Default::default(),
                encode_decode_graph: Default::default(),
                usage_graph_encode_decode: Default::default(),
                memory_graph: Default::default(),
                total_memory: Default::default(),
                memory_graph_label: Default::default(),
                usage_graph_memory: Default::default(),
                context_menu: Default::default(),
                graph_max_duration: Default::default(),

                name: RefCell::new(String::new()),
                base_color: Cell::new(gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)),
                summary_mode: Cell::new(false),

                encode_decode_available: Cell::new(true),

                infobar_content: GpuDetails::new(),

                show_enc_dec_action: gio::SimpleAction::new_stateful(
                    "enc_dec_usage",
                    None,
                    &glib::Variant::from(true),
                ),
            }
        }
    }

    impl PerformancePageGpu {
        fn name(&self) -> String {
            self.name.borrow().clone()
        }

        fn set_name(&self, name: String) {
            if name == *self.name.borrow() {
                return;
            }

            self.name.replace(name);
        }

        fn infobar_content(&self) -> Option<gtk::Widget> {
            Some(self.infobar_content.clone().upcast())
        }
    }

    impl PerformancePageGpu {
        fn configure_actions(this: &super::PerformancePageGpu) {
            let actions = gio::SimpleActionGroup::new();
            this.insert_action_group("graph", Some(&actions));

            let action = gio::SimpleAction::new("copy", None);
            action.connect_activate({
                let this = this.downgrade();
                move |_, _| {
                    if let Some(this) = this.upgrade() {
                        let clipboard = this.clipboard();
                        clipboard.set_text(this.imp().data_summary().as_str());
                    }
                }
            });
            actions.add_action(&action);

            let action = &this.imp().show_enc_dec_action;
            action.set_enabled(true);
            action.connect_activate(move |action, _| {
                let visible = !action
                    .state()
                    .and_then(|v| v.get::<bool>())
                    .unwrap_or(false);

                settings!()
                    .set_boolean("performance-page-gpu-encode-decode-usage-visible", visible)
                    .unwrap_or_else(|_| {
                        g_critical!(
                            "MissionCenter::PerformancePage",
                            "Failed to save show encode/decode usage"
                        );
                    });
            });
            actions.add_action(action);
        }

        fn configure_context_menu(this: &super::PerformancePageGpu) {
            let right_click_controller = gtk::GestureClick::new();
            right_click_controller.set_button(3); // Secondary click (AKA right click)
            right_click_controller.connect_released({
                let this = this.downgrade();
                move |_click, _n_press, x, y| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return,
                    };
                    let this = this.imp();

                    this.context_menu
                        .set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                            x.round() as i32,
                            y.round() as i32,
                            1,
                            1,
                        )));
                    this.context_menu.popup();
                }
            });
            this.add_controller(right_click_controller);
        }
    }

    impl PerformancePageGpu {
        pub fn set_static_information(
            this: &super::PerformancePageGpu,
            index: Option<usize>,
            gpu: &Gpu,
        ) -> bool {
            let this = this.imp();

            if let Some(index) = index {
                this.gpu_id.set_text(&format!("GPU {}", index));
            } else {
                this.gpu_id.set_text("GPU");
            }

            this.device_name
                .set_text(gpu.device_name.as_ref().unwrap_or(&i18n("Unknown")));

            let settings = settings!();
            let show_enc_dec_usage =
                settings.boolean("performance-page-gpu-encode-decode-usage-visible");
            this.show_enc_dec_action
                .set_state(&glib::Variant::from(show_enc_dec_usage));
            settings.connect_changed(Some("performance-page-gpu-encode-decode-usage-visible"), {
                let this = this.obj().downgrade();
                move |settings, _| {
                    if let Some(this) = this.upgrade() {
                        let this = this.imp();

                        let show_enc_dec_usage =
                            settings.boolean("performance-page-gpu-encode-decode-usage-visible");

                        let action = &this.show_enc_dec_action;
                        this.obj()
                            .set_encode_decode_available(action.is_enabled() && show_enc_dec_usage);
                        this.show_enc_dec_action
                            .set_state(&glib::Variant::from(show_enc_dec_usage));

                        // The usage graph is `homogeneous: true`, so we need to hide the container if all
                        // contained graphs are hidden so that the usage graph expands to fill the available
                        // space.
                        this.container_bottom.set_visible(
                            this.memory_graph.property::<bool>("visible")
                                || this.encode_decode_available.get(),
                        );
                    }
                }
            });

            this.infobar_content
                .set_encode_decode_shared(gpu.encode_decode_shared);
            // ??? might have broken this
            if gpu.encode_decode_shared {
                this.infobar_content
                    .encode_label()
                    .set_label(&i18n("Video encode/decode"));
            }

            let mut ogl_version = ArrayString::<64>::new();
            if let Some(ogl_var) = gpu.opengl_variant {
                if ogl_var == OpenGlVariant::OpenGles as i32 {
                    ogl_version.push_str("ES ");
                }
            }

            if let Some(api_ver) = gpu.opengl_version.as_ref() {
                let _ = write!(&mut ogl_version, "{}.{}", api_ver.major, api_ver.minor);
            }

            if ogl_version.is_empty() {
                ogl_version.push_str(&i18n("Unknown"));
            }

            this.infobar_content
                .opengl_version()
                .set_text(ogl_version.as_str());

            let vk_version = if let Some(vulkan_version) = gpu.vulkan_version.as_ref() {
                format!(
                    "{}.{}.{}",
                    vulkan_version.major,
                    vulkan_version.minor,
                    vulkan_version.patch.unwrap_or(0)
                )
            } else {
                i18n("Unsupported")
            };
            this.infobar_content.vulkan_version().set_text(&vk_version);

            if let (Some(pcie_gen), Some(pcie_lanes)) = (gpu.pcie_gen, gpu.pcie_lanes) {
                this.infobar_content.set_pcie_info_visible(true);
                this.infobar_content
                    .pcie_speed()
                    .set_text(&format!("PCIe Gen {} x{} ", pcie_gen, pcie_lanes));
            } else {
                this.infobar_content.set_pcie_info_visible(false);
            }

            if let (Some(max_pcie_gen), Some(max_pcie_lanes)) =
                (gpu.max_pcie_gen, gpu.max_pcie_lanes)
            {
                this.infobar_content.set_max_pcie_info_visible(true);
                this.infobar_content
                    .max_pcie_speed()
                    .set_text(&format!("PCIe Gen {} x{} ", max_pcie_gen, max_pcie_lanes));
            } else {
                this.infobar_content.set_max_pcie_info_visible(false);
            }

            this.infobar_content.pci_addr().set_text(gpu.id.as_ref());

            true
        }

        pub fn update_readings(
            this: &super::PerformancePageGpu,
            gpu: &Gpu,
            index: Option<usize>,
        ) -> bool {
            let this = this.imp();

            if let Some(index) = index {
                this.gpu_id
                    .set_text(&i18n_f("GPU {}", &[&format!("{}", index)]));
            } else {
                this.gpu_id.set_text(&i18n("GPU"));
            }

            this.update_utilization(gpu);
            this.update_clock_speed(gpu);
            this.update_power_draw(gpu);
            this.update_memory_info(gpu);
            this.update_memory_speed(gpu);
            this.update_video_encode_decode(gpu);
            this.update_temperature(gpu);
            this.update_pcie(gpu);

            // The usage graph is `homogeneous: true`, so we need to hide the container if all
            // contained graphs are hidden so that the usage graph expands to fill the available
            // space.
            this.container_bottom.set_visible(
                this.memory_graph.property::<bool>("visible") || this.encode_decode_available.get(),
            );

            true
        }

        pub(crate) fn update_animations(this: &super::PerformancePageGpu, new_ticks: f32) -> bool {
            let this = this.imp();

            this.graph_utilization.update_animation(new_ticks);
            this.usage_graph_memory.update_animation(new_ticks);
            this.usage_graph_encode_decode.update_animation(new_ticks);

            true
        }

        fn data_summary(&self) -> String {
            format!(
                r#"{}

    {}

    OpenGL version:        {}
    Vulkan version:        {}
    PCI Express speed:     {}
    Max PCI Express speed: {}
    PCI bus address:       {}

    Utilization:   {}
    Memory usage:  {} / {}
    GTT usage:     {} / {}
    Clock speed:   {} / {}
    Memory speed:  {} / {}
    Power draw:    {}{}
    Encode/Decode: {} / {}
    Temperature:   {}"#,
                self.gpu_id.label(),
                self.device_name.label(),
                self.infobar_content.opengl_version().label(),
                self.infobar_content.vulkan_version().label(),
                self.infobar_content.pcie_speed().label(),
                self.infobar_content.max_pcie_speed().label(),
                self.infobar_content.pci_addr().label(),
                self.infobar_content.utilization().label(),
                self.infobar_content.memory_usage_current().label(),
                self.infobar_content.memory_usage_max().label(),
                self.infobar_content.shared_mem_usage_current().label(),
                self.infobar_content.shared_mem_usage_max().label(),
                self.infobar_content.clock_speed_current().label(),
                self.infobar_content.clock_speed_max().label(),
                self.infobar_content.memory_speed_current().label(),
                self.infobar_content.memory_speed_max().label(),
                self.infobar_content.power_draw_current().label(),
                self.infobar_content.power_draw_max().label(),
                self.infobar_content.encode_percent().label(),
                self.infobar_content.decode_percent().label(),
                self.infobar_content.temperature().label(),
            )
        }

        fn update_utilization(&self, gpu: &Gpu) {
            let overall_usage = gpu.utilization_percent.unwrap_or_else(|| {
                g_warning!(
                    "MissionCenter::PerformancePage",
                    "GPU '{}' utilization data is missing",
                    gpu.id
                );
                0.
            });

            self.graph_utilization
                .add_data_point(vec![vec![overall_usage]]);
            self.infobar_content
                .utilization()
                .set_text(&format!("{}%", overall_usage));
        }

        fn update_clock_speed(&self, gpu: &Gpu) {
            let mut clock_speed_available = false;

            if let Some(max_clock_speed) = gpu.max_clock_speed_mhz {
                self.infobar_content
                    .clock_speed_separator()
                    .set_visible(true);
                self.infobar_content.clock_speed_max().set_visible(true);

                let max_label = crate::to_human_readable_nice(
                    max_clock_speed as f32 * 1_000_000.,
                    &DataType::Hertz,
                );
                self.infobar_content.clock_speed_max().set_text(&max_label);
            } else {
                self.infobar_content
                    .clock_speed_separator()
                    .set_visible(false);
                self.infobar_content.clock_speed_max().set_visible(false);
            }

            if let Some(clock_speed) = gpu.clock_speed_mhz {
                clock_speed_available = true;

                let clock_label = crate::to_human_readable_nice(
                    clock_speed as f32 * 1_000_000.,
                    &DataType::Hertz,
                );

                self.infobar_content
                    .clock_speed_current()
                    .set_text(&clock_label);
            }

            self.infobar_content
                .set_clock_speed_available(clock_speed_available);
        }

        fn update_power_draw(&self, gpu: &Gpu) {
            let mut power_draw_available = false;

            if let Some(power_limit) = gpu.max_power_draw_watts {
                self.infobar_content
                    .power_draw_separator()
                    .set_visible(true);
                self.infobar_content.power_draw_max().set_visible(true);

                let power_limit = crate::to_human_readable_nice(power_limit, &DataType::Watts);
                self.infobar_content.power_draw_max().set_text(&power_limit);
            } else {
                self.infobar_content
                    .power_draw_separator()
                    .set_visible(false);
                self.infobar_content.power_draw_max().set_visible(false);
            }

            if let Some(power_draw) = gpu.power_draw_watts {
                power_draw_available = true;

                let power_draw = crate::to_human_readable_nice(power_draw, &DataType::Watts);
                self.infobar_content
                    .power_draw_current()
                    .set_text(&power_draw);
            }

            self.infobar_content
                .set_power_draw_available(power_draw_available);
        }

        fn update_memory_info(&self, gpu: &Gpu) {
            fn update_dedicated_memory(
                this: &PerformancePageGpu,
                gpu: &Gpu,
                has_memory_info: &mut bool,
            ) -> Option<String> {
                let mut total_memory_str_res = None;

                if let Some(total_memory) = gpu.total_memory {
                    let total_memory = total_memory as f32;
                    let total_memory_str =
                        crate::to_human_readable_nice(total_memory, &DataType::MemoryBytes);

                    this.usage_graph_memory
                        .set_dataset_scaling(0, ScalingSettings::Fixed);
                    this.usage_graph_memory
                        .set_dataset_max_scale(0, total_memory);
                    this.infobar_content.set_total_memory_valid(true);

                    this.infobar_content
                        .memory_usage_max()
                        .set_text(&total_memory_str);

                    total_memory_str_res = Some(total_memory_str);
                } else {
                    this.infobar_content.set_total_memory_valid(false);
                }

                if let Some(used_memory) = gpu.used_memory {
                    *has_memory_info = true;

                    this.infobar_content.set_used_memory_valid(true);
                    this.infobar_content
                        .memory_usage_title()
                        .set_text(&i18n("Memory Usage"));

                    this.usage_graph_memory
                        .add_single_data_point(0, vec![used_memory as f32]);

                    let used_memory = crate::to_human_readable_nice(
                        gpu.used_memory.unwrap_or(0) as f32,
                        &DataType::MemoryBytes,
                    );
                    this.infobar_content
                        .memory_usage_current()
                        .set_text(&used_memory);
                } else {
                    this.infobar_content.set_used_memory_valid(false);

                    if this.infobar_content.total_memory_valid() {
                        this.infobar_content
                            .memory_usage_title()
                            .set_text(&i18n("Total Memory"));
                    }
                }

                total_memory_str_res
            }

            fn update_shared_memory(
                this: &PerformancePageGpu,
                gpu: &Gpu,
                total_memory_str: Option<&str>,
                has_memory_info: &mut bool,
            ) {
                if let Some(total_shared_memory) = gpu.total_shared_memory {
                    let total_gtt = crate::to_human_readable_nice(
                        total_shared_memory as f32,
                        &DataType::MemoryBytes,
                    );

                    this.infobar_content.set_total_shared_memory_valid(true);
                    this.usage_graph_memory
                        .set_all_datasets_scaling(ScalingSettings::Fixed);
                    this.usage_graph_memory
                        .set_dataset_max_scale(1, total_shared_memory as f32);

                    if let Some(total_memory_str) = total_memory_str {
                        this.total_memory
                            .set_text(&format!("{total_memory_str} / {total_gtt}"));

                        this.memory_graph_label
                            .set_text(&i18n("Dedicated and shared memory usage over "));
                    } else {
                        this.total_memory.set_text(&total_gtt);

                        this.memory_graph_label
                            .set_text(&i18n("Shared memory usage over "));
                    }

                    this.infobar_content
                        .shared_mem_usage_max()
                        .set_text(&total_gtt);
                } else {
                    this.infobar_content.set_total_shared_memory_valid(false);
                }

                if let Some(used_shared_memory) = gpu.used_shared_memory {
                    *has_memory_info = true;

                    this.usage_graph_memory
                        .add_single_data_point(1, vec![used_shared_memory as f32]);

                    this.infobar_content.set_used_shared_memory_valid(true);
                    this.infobar_content
                        .shared_memory_usage_title()
                        .set_text(&i18n("Shared Memory Usage"));

                    let used_shared_mem_str = crate::to_human_readable_nice(
                        used_shared_memory as f32,
                        &DataType::MemoryBytes,
                    );

                    this.infobar_content
                        .shared_mem_usage_current()
                        .set_text(&used_shared_mem_str);
                } else {
                    this.infobar_content.set_used_shared_memory_valid(false);

                    if this.infobar_content.total_shared_memory_valid() {
                        this.infobar_content
                            .shared_memory_usage_title()
                            .set_text(&i18n("Total Shared Memory"));
                    }
                }
            }

            let mut has_memory_info = false;

            let total_memory_str = update_dedicated_memory(self, gpu, &mut has_memory_info);

            update_shared_memory(
                self,
                gpu,
                total_memory_str.as_ref().map(String::as_str),
                &mut has_memory_info,
            );

            if !self.infobar_content.total_memory_valid()
                && !self.infobar_content.total_shared_memory_valid()
            {
                self.usage_graph_memory
                    .set_all_datasets_scaling(ScalingSettings::StickyUp);
            }

            self.memory_graph.set_visible(has_memory_info);
        }

        fn update_memory_speed(&self, gpu: &Gpu) {
            let mut memory_speed_available = false;

            if let Some(max_memory_speed) = gpu.max_memory_speed_mhz {
                self.infobar_content
                    .memory_speed_separator()
                    .set_visible(true);
                self.infobar_content.memory_speed_max().set_visible(true);

                let ms_max = crate::to_human_readable_nice(
                    max_memory_speed as f32 * 1_000_000.,
                    &DataType::Hertz,
                );
                self.infobar_content.memory_speed_max().set_text(&ms_max);
            } else {
                self.infobar_content
                    .memory_speed_separator()
                    .set_visible(false);
                self.infobar_content.memory_speed_max().set_visible(false);
            }

            if let Some(memory_speed) = gpu.memory_speed_mhz {
                memory_speed_available = true;

                let memory_speed = crate::to_human_readable_nice(
                    memory_speed as f32 * 1_000_000.,
                    &DataType::Hertz,
                );
                self.infobar_content
                    .memory_speed_current()
                    .set_text(&memory_speed);
            }

            self.infobar_content
                .set_memory_speed_available(memory_speed_available);
        }

        fn update_video_encode_decode(&self, gpu: &Gpu) {
            let mut encode_decode_info_available = false;

            if let Some(encoder_percent) = gpu.encoder_percent {
                encode_decode_info_available = true;

                self.usage_graph_encode_decode
                    .add_single_data_point(0, vec![encoder_percent]);

                self.infobar_content
                    .encode_percent()
                    .set_text(&format!("{}%", encoder_percent));
            }

            if !gpu.encode_decode_shared {
                if let Some(decoder_percent) = gpu.decoder_percent {
                    encode_decode_info_available = true;

                    self.usage_graph_encode_decode
                        .add_single_data_point(1, vec![decoder_percent]);

                    self.infobar_content
                        .decode_percent()
                        .set_text(&format!("{}%", decoder_percent));
                }
            }

            self.show_enc_dec_action
                .set_enabled(encode_decode_info_available);
            self.obj().set_encode_decode_available(
                encode_decode_info_available
                    && self
                        .show_enc_dec_action
                        .state()
                        .and_then(|v| v.get::<bool>())
                        .unwrap_or(false),
            );
        }

        fn update_temperature(&self, gpu: &Gpu) {
            if let Some(temp) = gpu.temperature_c {
                self.infobar_content.box_temp().set_visible(true);

                self.infobar_content
                    .temperature()
                    .set_text(&format!("{} °C", temp.round() as i32));
            } else {
                self.infobar_content.box_temp().set_visible(false);
            }
        }

        fn update_pcie(&self, gpu: &Gpu) {
            if let (Some(pcie_gen), Some(pcie_lanes)) = (gpu.pcie_gen, gpu.pcie_lanes) {
                self.infobar_content.set_pcie_info_visible(true);
                self.infobar_content
                    .pcie_speed()
                    .set_text(&format!("PCIe Gen {} x{} ", pcie_gen, pcie_lanes));
                if let (Some(max_pcie_gen), Some(max_pcie_lanes)) =
                    (gpu.max_pcie_gen, gpu.max_pcie_lanes)
                {
                    self.infobar_content.set_max_pcie_info_visible(
                        !(max_pcie_gen == pcie_gen && max_pcie_lanes == pcie_lanes),
                    )
                } else {
                    self.infobar_content.set_max_pcie_info_visible(false);
                }
            } else {
                self.infobar_content.set_pcie_info_visible(false);
                self.infobar_content.set_max_pcie_info_visible(false);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PerformancePageGpu {
        const NAME: &'static str = "PerformancePageGpu";
        type Type = super::PerformancePageGpu;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PerformancePageGpu {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let settings = settings!();

            let mut encode = DatasetGroup::new();
            encode.dataset_settings.fill = false;
            encode.dataset_settings.dashed = true;
            let decode = DatasetGroup::new();

            self.usage_graph_encode_decode.add_dataset(encode);
            self.usage_graph_encode_decode.add_dataset(decode);

            self.usage_graph_encode_decode
                .connect_to_settings(&settings);

            let util = DatasetGroup::new();
            self.graph_utilization.add_dataset(util);
            self.graph_utilization.connect_to_settings(&settings);

            let mem = DatasetGroup::new();
            self.usage_graph_memory.add_dataset(mem);
            let mut idk = DatasetGroup::new();
            idk.dataset_settings.dashed = true;
            idk.dataset_settings.fill = false;
            self.usage_graph_memory.add_dataset(idk);
            self.usage_graph_memory.connect_to_settings(&settings);

            let this = self.obj();

            this.as_ref()
                .bind_property(
                    "encode-decode-available",
                    &self.infobar_content,
                    "encode-decode-available",
                )
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            Self::configure_actions(&this);
            Self::configure_context_menu(&this);
        }
    }

    impl WidgetImpl for PerformancePageGpu {}

    impl BoxImpl for PerformancePageGpu {}
}

glib::wrapper! {
    pub struct PerformancePageGpu(ObjectSubclass<imp::PerformancePageGpu>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl PageExt for PerformancePageGpu {
    fn infobar_collapsed(&self) {
        self.imp().infobar_content.set_collapsed(true);
    }

    fn infobar_uncollapsed(&self) {
        self.imp().infobar_content.set_collapsed(false);
    }
}

impl PerformancePageGpu {
    pub fn new(name: &str) -> Self {
        fn update_refresh_rate_sensitive_labels(
            this: &PerformancePageGpu,
            settings: &gio::Settings,
        ) {
            let this = this.imp();

            let data_points = settings.int("performance-page-data-points") as u32;
            let delay = settings.uint64("app-update-interval-u64");

            let graph_max_duration =
                (((delay as f64) * INTERVAL_STEP) * (data_points as f64)).round() as u32;

            this.graph_max_duration
                .set_text(&to_short_human_readable_time(graph_max_duration));
        }

        let this: Self = glib::Object::builder().property("name", name).build();
        let settings = settings!();
        update_refresh_rate_sensitive_labels(&this, &settings);

        settings.connect_changed(Some("performance-page-data-points"), {
            let this = this.downgrade();
            move |settings, _| {
                if let Some(this) = this.upgrade() {
                    update_refresh_rate_sensitive_labels(&this, settings);
                }
            }
        });

        settings.connect_changed(Some("app-update-interval"), {
            let this = this.downgrade();
            move |settings, _| {
                if let Some(this) = this.upgrade() {
                    update_refresh_rate_sensitive_labels(&this, settings);
                }
            }
        });

        this
    }

    pub fn set_static_information(&self, index: Option<usize>, gpu: &Gpu) -> bool {
        imp::PerformancePageGpu::set_static_information(self, index, gpu)
    }

    pub fn update_readings(&self, gpu: &Gpu, index: Option<usize>) -> bool {
        imp::PerformancePageGpu::update_readings(self, gpu, index)
    }

    pub fn update_animations(&self, new_ticks: f32) -> bool {
        imp::PerformancePageGpu::update_animations(self, new_ticks)
    }
}
