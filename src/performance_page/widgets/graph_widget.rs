/* performance_page/widgets/graph_widget.rs
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

use std::cell::Cell;

use glib::{ParamSpec, Properties, Value};
use gtk::gdk;
use gtk::gdk::prelude::*;
use gtk::gio;
use gtk::glib;
use gtk::glib::g_warning;
use gtk::glib::subclass::prelude::*;
use gtk::glib::subclass::Signal;
use gtk::graphene;
use gtk::gsk;
use gtk::gsk::PathBuilder;
use gtk::gsk::Stroke;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Snapshot;
use gtk::TextDirection;

use crate::performance_page::widgets::graph_widget_utils::{DatasetGroup, ScalingSettings};

// no faster than 200 Hz. if everything is going according to plan, we expect two animation frames in quick succession at the start of a new cycle and want to prevent rendering twice
const ANIMATION_LOCKOUT: f32 = 0.005;

mod imp {
    use super::*;
    use crate::performance_page::widgets::GRAPH_RADIUS;

    #[derive(Properties)]
    #[properties(wrapper_type = super::GraphWidget)]
    pub struct GraphWidget {
        #[property(get, set = Self::set_data_points)]
        data_points: Cell<u32>,
        #[property(get, set)]
        base_color: Cell<gdk::RGBA>,
        #[property(get, set = Self::set_horizontal_line_count)]
        horizontal_line_count: Cell<u32>,
        #[property(get, set = Self::set_vertical_line_count)]
        vertical_line_count: Cell<u32>,
        #[property(get, set)]
        animation_ticks: Cell<f32>,
        #[property(get, set = Self::set_do_animation)]
        do_animation: Cell<bool>,
        #[property(get, set = Self::set_smooth_graphs)]
        smooth_graphs: Cell<bool>,
        #[property(get, set)]
        scroll: Cell<bool>,
        #[property(get, set)]
        grid_visible: Cell<bool>,

        pub settings_inited: Cell<bool>,
        scroll_offset: Cell<u32>,
        prev_size: Cell<(i32, i32)>,

        pub(crate) need_redraw: Cell<bool>,
        cached_snapshot: Cell<Option<gsk::RenderNode>>,

        pub data_sets: Cell<Vec<DatasetGroup>>,
    }

    impl Default for GraphWidget {
        fn default() -> Self {
            Self {
                data_points: Cell::new(0),
                base_color: Cell::new(gdk::RGBA::new(0., 0., 0., 1.)),
                horizontal_line_count: Cell::new(9),
                vertical_line_count: Cell::new(6),
                animation_ticks: Cell::new(0.),
                do_animation: Cell::new(false),
                smooth_graphs: Cell::new(false),
                scroll: Cell::new(true),
                grid_visible: Cell::new(true),
                settings_inited: Cell::new(false),
                scroll_offset: Cell::new(0),
                prev_size: Cell::new((0, 0)),
                need_redraw: Cell::new(true),
                cached_snapshot: Cell::new(None),
                data_sets: Cell::new(vec![]),
            }
        }
    }

    impl GraphWidget {
        pub fn set_data_points(&self, count: u32) {
            if self.data_points.get() != count {
                let mut data_sets = self.data_sets.take();
                for values in data_sets.iter_mut() {
                    values.update_data_points(count as usize);
                }
                self.data_sets.set(data_sets);
                self.data_points.set(count);

                self.obj().force_redraw();
            }
        }

        pub fn set_data_points_i32(&self, count: i32) {
            self.set_data_points(count as u32)
        }

        fn set_horizontal_line_count(&self, count: u32) {
            if self.horizontal_line_count.get() != count {
                self.horizontal_line_count.set(count);
                self.obj().force_redraw();
            }
        }

        fn set_vertical_line_count(&self, count: u32) {
            if self.vertical_line_count.get() != count {
                self.vertical_line_count.set(count);
                self.obj().force_redraw();
            }
        }

        pub fn set_smooth_graphs(&self, smooth: bool) {
            if self.smooth_graphs.get() != smooth {
                self.smooth_graphs.set(smooth);
                self.obj().force_redraw();
            }
        }

        pub fn set_do_animation(&self, smooth: bool) {
            if self.do_animation.get() != smooth {
                self.do_animation.set(smooth);
                self.obj().force_redraw();
            }
        }

        pub fn set_animation_ticks(&self, ticks: f32) {
            self.animation_ticks.set(ticks);
        }

        pub fn try_increment_scroll(&self) {
            if !self.scroll.get() {
                return;
            }

            self.scroll_offset
                .set(self.scroll_offset.get().wrapping_add(1));
        }
    }

    impl GraphWidget {
        #[inline]
        fn draw_outline(&self, snapshot: &Snapshot, bounds: &gsk::RoundedRect, color: &gdk::RGBA) {
            let stroke_color = gdk::RGBA::new(color.red(), color.green(), color.blue(), 1.);
            snapshot.append_border(&bounds, &[1.; 4], &[stroke_color.clone(); 4]);
        }

        #[inline]
        fn draw_grid(
            &self,
            snapshot: &Snapshot,
            width: f32,
            height: f32,
            scale_factor: f64,
            color: &gdk::RGBA,
        ) {
            let scale_factor = scale_factor as f32;
            let color = gdk::RGBA::new(color.red(), color.green(), color.blue(), 51. / 256.);

            let stroke = Stroke::new(1.);

            // Draw horizontal lines
            let horizontal_line_count = self.obj().horizontal_line_count() + 1;

            let frame_width = width - scale_factor;
            let frame_height = height / horizontal_line_count as f32;

            let point_spacing = width * self.obj().point_spacing_factor();

            let path_builder = PathBuilder::new();
            for i in 1..horizontal_line_count {
                path_builder.move_to(
                    scale_factor / 2. - 2. * point_spacing,
                    frame_height * i as f32,
                );
                path_builder.line_to(frame_width, frame_height * i as f32);
            }

            // Draw vertical lines
            let vertical_line_count = self.obj().vertical_line_count() + 1;

            let col_width = (width + point_spacing) / vertical_line_count as f32;
            let col_height = height - scale_factor;

            let scroll_offset = if self.obj().scroll() {
                ((point_spacing) * -(self.scroll_offset.get() as f32)).rem_euclid(col_width)
            } else {
                0.
            };

            for i in 0..vertical_line_count + 1 {
                path_builder.move_to(
                    col_width * (i as f32) + scroll_offset - point_spacing,
                    scale_factor / 2.,
                );
                path_builder.line_to(
                    col_width * (i as f32) + scroll_offset - point_spacing,
                    col_height,
                );
            }
            snapshot.append_stroke(&path_builder.to_path(), &stroke, &color);
        }

        fn render(&self, snapshot: &Snapshot, width: f32, height: f32, scale_factor: f64) {
            let base_color = self.base_color.get();

            let radius = graphene::Size::new(GRAPH_RADIUS, GRAPH_RADIUS);
            let bounds = gsk::RoundedRect::new(
                graphene::Rect::new(0., 0., width, height),
                radius,
                radius,
                radius,
                radius,
            );

            let cached_shot = self.cached_snapshot.take();

            let need_redraw =
                !self.do_animation.get() || self.need_redraw.get() || cached_shot.is_none();

            let base_snapshot = if need_redraw {
                self.need_redraw.set(false);

                let snapshot = Snapshot::new();

                if self.obj().grid_visible() {
                    self.draw_grid(&snapshot, width, height, scale_factor, &base_color);
                }

                let mut data_sets = self.data_sets.take();
                let object = self.obj();
                for values in &mut data_sets {
                    values.plot(&snapshot, width, height, &*object);
                }
                self.data_sets.set(data_sets);

                snapshot.to_node()
            } else {
                cached_shot
            };

            let Some(baze) = base_snapshot else {
                g_warning!("MissionCenter", "Drawing was empty");
                return;
            };

            snapshot.push_rounded_clip(&bounds);

            if self.obj().direction() == TextDirection::Rtl {
                snapshot.scale(-1., 1.);
                snapshot.translate(&graphene::Point::new(-width, 0.));
            }

            if self.do_animation.get() {
                snapshot.save();
                let spacing = width / (self.data_points.get() - 2) as f32;
                snapshot.translate(&graphene::Point::new(
                    spacing * (1. - self.animation_ticks.get()),
                    0.,
                ));
                snapshot.append_node(baze.clone());

                snapshot.restore();

                self.cached_snapshot.set(Some(baze));
            } else {
                snapshot.append_node(baze);
            }

            snapshot.pop();

            self.draw_outline(snapshot, &bounds, &base_color);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GraphWidget {
        const NAME: &'static str = "GraphWidget";
        type Type = super::GraphWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for GraphWidget {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn signals() -> &'static [Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| vec![Signal::builder("resize").build()])
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for GraphWidget {
        fn realize(&self) {
            self.parent_realize();

            // connection has to happen slightly later than GraphWidget::new and this works so that the initial resize is not lost in the void
            self.obj().connect_signals();
        }

        fn snapshot(&self, snapshot: &Snapshot) {
            use glib::g_critical;

            let this = self.obj();

            let native = match this.native() {
                Some(native) => native,
                None => {
                    g_critical!("MissionCenter::GraphWidget", "Failed to get native");
                    return;
                }
            };

            let surface = match native.surface() {
                Some(surface) => surface,
                None => {
                    g_critical!("MissionCenter::GraphWidget", "Failed to get surface");
                    return;
                }
            };

            let (prev_width, prev_height) = self.prev_size.get();
            let (width, height) = (this.width(), this.height());

            if prev_width != width || prev_height != height {
                this.emit_by_name::<()>("resize", &[]);
                self.prev_size.set((width, height));
                self.obj().force_redraw();
            }

            self.render(snapshot, width as f32, height as f32, surface.scale());
        }
    }
}

macro_rules! connect_setting {
    ($self: ident, $settings: ident, $setting_key: literal, $settings_method: ident, $set_fn: ident) => {
        $settings.connect_changed(Some($setting_key), {
            let this = $self.downgrade();

            move |settings, _| {
                if let Some(this) = this.upgrade() {
                    this.imp().$set_fn(settings.$settings_method($setting_key));
                    this.force_redraw();
                }
            }
        });

        $self
            .imp()
            .$set_fn($settings.$settings_method($setting_key));
    };
}

glib::wrapper! {
    pub struct GraphWidget(ObjectSubclass<imp::GraphWidget>)
        @extends gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl GraphWidget {
    pub fn new(settings: Option<&gio::Settings>) -> Self {
        let obj: Self = glib::Object::new();

        {
            if let Some(settings) = settings {
                obj.connect_to_settings(settings);
            }
        }

        obj
    }

    pub fn connect_signals(&self) {
        let obj = self;
        obj.connect_local("resize", true, {
            let dcr = obj.downgrade();

            move |_| {
                let Some(obj) = dcr.upgrade() else {
                    return None;
                };

                let width = obj.width() as f32;
                let height = obj.height() as f32;

                obj.set_vertical_line_count((width / 60.).round().max(6.) as u32);
                obj.set_horizontal_line_count((height / 60.).round().max(9.) as u32);

                None
            }
        });

        obj.connect_visible_notify({
            let dcr = obj.downgrade();

            move |_| {
                let Some(obj) = dcr.upgrade() else {
                    return;
                };

                if !obj.is_visible() {
                    return;
                }

                obj.force_redraw();
            }
        });
    }

    pub fn set_dashed(&self, index: usize, dashed: bool) {
        let mut data = self.imp().data_sets.take();
        if index < data.len() {
            data[index].dataset_settings.dashed = dashed;
        }
        self.imp().data_sets.set(data);
    }

    pub fn set_filled(&self, index: usize, filled: bool) {
        let mut data = self.imp().data_sets.take();
        if index < data.len() {
            data[index].dataset_settings.fill = filled;
        }
        self.imp().data_sets.set(data);
    }

    pub fn set_data_visible(&self, index: usize, visible: bool) {
        let mut data = self.imp().data_sets.take();
        if index < data.len() {
            data[index].dataset_settings.visible = visible;
        }
        self.imp().data_sets.set(data);
    }

    pub fn add_data_point(&self, datas: Vec<Vec<f32>>) {
        let this = self.imp();
        let mut data = this.data_sets.take();

        assert_eq!(datas.len(), data.len());

        for (idx, dataset) in data.iter_mut().enumerate() {
            dataset.add_data(&datas[idx]);
        }

        Self::apply_followings(&mut data);

        this.data_sets.set(data);
    }

    pub fn add_single_data_point(&self, idx: usize, datas: Vec<f32>) {
        let mut data = self.imp().data_sets.take();

        data[idx].add_data(&datas);

        Self::apply_followings(&mut data);

        self.imp().data_sets.set(data);
    }

    fn apply_followings(data: &mut Vec<DatasetGroup>) {
        loop {
            let mut breaking = true;

            for i in 0..data.len() {
                let dataset = &data[i];

                let Some(other) = dataset
                    .dataset_settings
                    .following
                    .map(|it| data[it].clone())
                else {
                    continue;
                };

                breaking &= !data[i].apply_following_rules(Some(&other));
            }

            if breaking {
                break;
            }
        }
    }

    pub fn update_animation(&self, new_ticks: f32) -> bool {
        if self.is_visible() {
            if new_ticks == 0. {
                self.set_animation_ticks(new_ticks);
                self.force_redraw();
                self.imp().try_increment_scroll();
            } else if self.do_animation() {
                if new_ticks > self.animation_ticks() + ANIMATION_LOCKOUT {
                    self.set_animation_ticks(new_ticks);
                    self.queue_draw();
                }
            }
        }

        true
    }

    pub fn add_dataset(&self, mut dataset: DatasetGroup) {
        let mut sets = self.imp().data_sets.take();

        dataset.update_data_points(self.data_points() as usize);
        sets.push(dataset);

        self.imp().data_sets.set(sets);
    }

    pub fn set_dataset_scaling(&self, index: usize, scaler: ScalingSettings) {
        let mut sets = self.imp().data_sets.take();

        sets[index].dataset_settings.scaling_settings = scaler;

        self.imp().data_sets.set(sets);

        self.force_redraw();
    }

    pub fn set_all_datasets_scaling(&self, scaler: ScalingSettings) {
        let mut sets = self.imp().data_sets.take();

        for set in sets.iter_mut() {
            set.dataset_settings.scaling_settings = scaler.clone();
        }

        self.imp().data_sets.set(sets);

        self.force_redraw();
    }

    pub fn set_all_datasets_watermarking_multiplier(&self, offset: f32) {
        let mut sets = self.imp().data_sets.take();

        for set in sets.iter_mut() {
            set.dataset_settings.watermarking_multiplier = offset;
        }

        self.imp().data_sets.set(sets);

        self.force_redraw();
    }

    pub fn set_all_datasets_max_scale(&self, max: f32) {
        let mut sets = self.imp().data_sets.take();

        for set in sets.iter_mut() {
            set.dataset_settings.high_watermark = max;
        }

        self.imp().data_sets.set(sets);

        self.force_redraw();
    }

    pub fn set_dataset_max_scale(&self, index: usize, max: f32) {
        let mut sets = self.imp().data_sets.take();

        assert!(index < sets.len());

        sets[index].dataset_settings.high_watermark = max;

        self.imp().data_sets.set(sets);

        self.force_redraw();
    }

    pub fn get_dataset_max_scale(&self, index: usize) -> f32 {
        let sets = self.imp().data_sets.take();

        let it = sets[index].dataset_settings.high_watermark;

        self.imp().data_sets.set(sets);

        self.force_redraw();

        it
    }

    pub fn connect_to_settings(&self, settings: &gio::Settings) {
        if self.imp().settings_inited.get() {
            return;
        }

        self.imp().settings_inited.set(true);

        connect_setting!(
            self,
            settings,
            "performance-page-data-points",
            int,
            set_data_points_i32
        );
        connect_setting!(
            self,
            settings,
            "performance-smooth-graphs",
            boolean,
            set_smooth_graphs
        );
        connect_setting!(
            self,
            settings,
            "performance-sliding-graphs",
            boolean,
            set_do_animation
        );
    }

    /**
     *  "connecting" when used with the Follow scaling settings makes two datasets use the largest high watermark, and the smallest low watermark
     */
    pub fn connect_datasets(&self, idxa: usize, idxb: usize) {
        let this = self.imp();

        let mut sets = this.data_sets.take();

        sets[idxa].dataset_settings.following = Some(idxb);
        sets[idxb].dataset_settings.followed = Some(idxa);

        this.data_sets.set(sets);
    }

    #[inline]
    pub fn force_redraw(&self) {
        self.imp().need_redraw.set(true);
        self.queue_draw();
    }

    pub fn point_spacing_factor(&self) -> f32 {
        1. / (self.data_points() - (if self.do_animation() { 2 } else { 1 })) as f32
    }

    pub fn reset_auto_scaling(&self) {
        let mut datasets = self.imp().data_sets.take();

        for dataset in datasets.iter_mut() {
            dataset.reset_auto_scaling();
        }

        Self::apply_followings(&mut datasets);

        self.imp().data_sets.set(datasets);
    }
}
