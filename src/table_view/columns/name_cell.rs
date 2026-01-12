/* table_view/columns/name_cell.rs
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
use std::time::Duration;

use gdk::pango::EllipsizeMode;
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};

use crate::apply_icon_to_image;
use crate::table_view::row_model::{ContentType, RowModel};
use crate::widgets::ListCell;

mod imp {
    use super::*;

    pub struct NameCell {
        icon: gtk::Image,
        name: gtk::Label,

        sig_id: Cell<Option<glib::SignalHandlerId>>,
        sig_icon: Cell<Option<glib::SignalHandlerId>>,
        sig_name: Cell<Option<glib::SignalHandlerId>>,
        sig_content_type: Cell<Option<glib::SignalHandlerId>>,
        sig_children_changed: Cell<Option<glib::SignalHandlerId>>,

        model: Cell<glib::WeakRef<RowModel>>,
        expander: RefCell<glib::WeakRef<gtk::TreeExpander>>,
    }

    impl Default for NameCell {
        fn default() -> Self {
            Self {
                icon: gtk::Image::new(),
                name: gtk::Label::new(None),

                sig_id: Cell::new(None),
                sig_icon: Cell::new(None),
                sig_name: Cell::new(None),
                sig_content_type: Cell::new(None),
                sig_children_changed: Cell::new(None),

                model: Cell::new(glib::WeakRef::default()),
                expander: RefCell::new(glib::WeakRef::default()),
            }
        }
    }

    impl NameCell {
        pub fn bind(&self, model: &RowModel, list_cell: &ListCell, expander: &gtk::TreeExpander) {
            let this = self.obj().downgrade();

            self.model.set(model.downgrade());
            *self.expander.borrow_mut() = expander.downgrade();

            let sig_id = model.connect_id_notify({
                let list_cell = list_cell.downgrade();
                move |model| {
                    let Some(list_cell) = list_cell.upgrade() else {
                        return;
                    };
                    list_cell.set_item_id(model.id())
                }
            });
            self.sig_id.set(Some(sig_id));
            list_cell.set_item_id(model.id());

            let sig_icon = model.connect_icon_changed_notify({
                let this = this.clone();
                move |model| {
                    let Some(this) = this.upgrade() else {
                        return;
                    };
                    let this = this.imp();
                    // todo formalize this sizing.
                    apply_icon_to_image(
                        &this.icon,
                        model.imp().neo_icon(),
                        if model.content_type() == ContentType::Process {
                            16
                        } else {
                            24
                        },
                    );
                }
            });
            self.sig_icon.set(Some(sig_icon));

            apply_icon_to_image(
                &self.icon,
                model.imp().neo_icon(),
                if model.content_type() == ContentType::Process {
                    16
                } else {
                    24
                },
            );

            let sig_name = model.connect_name_notify({
                let this = this.clone();
                move |model| {
                    let Some(this) = this.upgrade() else {
                        return;
                    };
                    let this = this.imp();
                    this.name.set_label(&model.name());
                }
            });
            self.sig_name.set(Some(sig_name));
            self.name.set_label(&model.name());

            let sig_content_type = model.connect_content_type_notify({
                let this = this.clone();
                move |model| {
                    let Some(this) = this.upgrade() else {
                        return;
                    };
                    let this = this.imp();
                    this.set_content_type(model.content_type());
                }
            });
            self.sig_content_type.set(Some(sig_content_type));
            self.set_content_type(model.content_type());

            let sig_children_changed = model.children().connect_items_changed({
                let expander = expander.downgrade();
                move |children, _, _, _| {
                    let Some(expander) = expander.upgrade() else {
                        return;
                    };
                    expander.set_hide_expander(children.n_items() == 0);
                }
            });
            self.sig_children_changed.set(Some(sig_children_changed));
            expander.set_hide_expander(model.children().n_items() == 0);
        }

        pub fn unbind(&self) {
            self.expander.replace(glib::WeakRef::default());
            let Some(model) = self.model.take().upgrade() else {
                return;
            };

            if let Some(sig_id) = self.sig_id.take() {
                model.disconnect(sig_id);
            }

            if let Some(sig_id) = self.sig_icon.take() {
                model.disconnect(sig_id);
            }

            if let Some(sig_id) = self.sig_name.take() {
                model.disconnect(sig_id);
            }

            if let Some(sig_id) = self.sig_content_type.take() {
                model.disconnect(sig_id);
            }

            if let Some(sig_id) = self.sig_children_changed.take() {
                model.children().disconnect(sig_id);
            }
        }

        fn set_content_type(&self, content_type: ContentType) {
            match content_type {
                ContentType::SectionHeader => {
                    self.icon.set_visible(false);
                    self.name.add_css_class("heading");

                    let this = self.obj();
                    this.set_margin_start(6);
                    this.set_margin_top(6);
                    this.set_margin_bottom(6);

                    if let Some(expander) = self.expander.borrow().upgrade() {
                        expander.set_indent_for_icon(false);
                    };
                }
                ContentType::Service => {
                    self.icon.set_visible(true);
                    self.icon.set_margin_end(10);
                    self.icon.set_pixel_size(16);
                    self.name.remove_css_class("heading");

                    let this = self.obj();
                    this.set_margin_start(0);
                    this.set_margin_top(3);
                    this.set_margin_bottom(3);

                    let this = this.downgrade();
                    glib::timeout_add_local_full(
                        Duration::from_millis(0),
                        glib::Priority::HIGH,
                        move || {
                            let Some(this) = this.upgrade() else {
                                return glib::ControlFlow::Break;
                            };
                            let _ = this.activate_action("listitem.collapse", None);

                            glib::ControlFlow::Break
                        },
                    );

                    if let Some(expander) = self.expander.borrow().upgrade() {
                        expander.set_indent_for_icon(true);
                    };
                }
                ContentType::Process => {
                    self.icon.set_visible(true);
                    self.icon.set_margin_end(10);
                    self.icon.set_pixel_size(16);
                    self.name.remove_css_class("heading");

                    let this = self.obj();
                    this.set_margin_start(0);
                    this.set_margin_top(0);
                    this.set_margin_bottom(0);

                    if let Some(expander) = self.expander.borrow().upgrade() {
                        expander.set_indent_for_icon(true);
                    };
                }
                ContentType::App => {
                    self.icon.set_visible(true);
                    self.icon.set_margin_end(10);
                    self.icon.set_pixel_size(24);
                    self.name.remove_css_class("heading");

                    let this = self.obj();
                    this.set_margin_start(0);
                    this.set_margin_top(0);
                    this.set_margin_bottom(0);

                    let this = this.downgrade();
                    glib::timeout_add_local_full(
                        Duration::from_millis(0),
                        glib::Priority::HIGH,
                        move || {
                            let Some(this) = this.upgrade() else {
                                return glib::ControlFlow::Break;
                            };
                            let _ = this.activate_action("listitem.collapse", None);

                            glib::ControlFlow::Break
                        },
                    );

                    if let Some(expander) = self.expander.borrow().upgrade() {
                        expander.set_indent_for_icon(true);
                    };
                }
            };
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NameCell {
        const NAME: &'static str = "NameCell";
        type Type = super::NameCell;
        type ParentType = gtk::Box;

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {}
    }

    impl ObjectImpl for NameCell {
        fn constructed(&self) {
            self.parent_constructed();

            self.name.set_ellipsize(EllipsizeMode::Middle);

            let _ = self.obj().append(&self.icon);
            let _ = self.obj().append(&self.name);
        }
    }

    impl WidgetImpl for NameCell {
        fn realize(&self) {
            self.parent_realize();
        }
    }

    impl BoxImpl for NameCell {}
}

glib::wrapper! {
    pub struct NameCell(ObjectSubclass<imp::NameCell>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NameCell {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn bind(&self, model: &RowModel, list_cell: &ListCell, expander: &gtk::TreeExpander) {
        self.imp().bind(model, list_cell, expander);
    }

    pub fn unbind(&self) {
        self.imp().unbind();
    }
}
