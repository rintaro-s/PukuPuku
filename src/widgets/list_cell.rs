/* widgets/list_cell.rs
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
use std::rc::Rc;

use adw::{self, subclass::prelude::*};
use glib::{gobject_ffi, ParamSpec, Properties, Value, Variant};
use gtk::glib;
use gtk::prelude::*;

#[allow(unreachable_code)]
mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::ListCell)]
    pub struct ListCell {
        #[property(set = Self::set_item_id, type = glib::GString)]
        item_id: RefCell<Rc<str>>,
        #[property(set = Self::set_action_name, type = glib::GString)]
        action_name: RefCell<Rc<str>>,
        #[property(set)]
        is_tree_view: Cell<bool>,
    }

    impl Default for ListCell {
        fn default() -> Self {
            let empty_str = Rc::<str>::from("");
            Self {
                item_id: RefCell::new(empty_str.clone()),
                action_name: RefCell::new(empty_str),
                is_tree_view: Cell::new(false),
            }
        }
    }

    impl ListCell {
        fn set_item_id(&self, item_name: &str) {
            *self.item_id.borrow_mut() = Rc::<str>::from(item_name);
        }

        fn set_action_name(&self, action_name: &str) {
            *self.action_name.borrow_mut() = Rc::<str>::from(action_name);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ListCell {
        const NAME: &'static str = "ListCell";
        type Type = super::ListCell;
        type ParentType = adw::Bin;

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {}
    }

    impl ObjectImpl for ListCell {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for ListCell {
        fn realize(&self) {
            self.parent_realize();

            let this = self.obj();
            if let Some(mut row_widget) = this.parent().and_then(|p| p.parent()) {
                if self.is_tree_view.get() {
                    if let Some(rw) = row_widget.parent() {
                        row_widget = rw;
                    }
                }

                let gesture_handler = {
                    let weak_row_widget = unsafe {
                        let weak_ref =
                            Box::into_raw(Box::<gobject_ffi::GWeakRef>::new(core::mem::zeroed()));
                        gobject_ffi::g_weak_ref_init(weak_ref, row_widget.as_ptr() as *mut _);

                        weak_ref as u64
                    };
                    let this = this.downgrade();
                    move |x, y| {
                        let Some(this) = this.upgrade() else {
                            return;
                        };
                        let this = this.imp();

                        let item_name = this.item_id.borrow().as_ref().to_owned();
                        let _ = this.obj().activate_action(
                            this.action_name.borrow().as_ref(),
                            Some(&Variant::from((item_name, weak_row_widget, x, y))),
                        );
                    }
                };

                let gesture_click = gtk::GestureClick::new();
                gesture_click.set_button(3);
                gesture_click.connect_released({
                    let gesture_handler = gesture_handler.clone();
                    move |_, _, x, y| {
                        gesture_handler(x, y);
                    }
                });

                let gesture_touch = gtk::GestureLongPress::new();
                gesture_touch.set_button(1);
                gesture_touch.set_touch_only(true);
                gesture_touch.connect_pressed(move |_, x, y| {
                    gesture_handler(x, y);
                });

                row_widget.add_controller(gesture_click);
                row_widget.add_controller(gesture_touch);
            }
        }
    }

    impl BinImpl for ListCell {}
}

glib::wrapper! {
    pub struct ListCell(ObjectSubclass<imp::ListCell>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ListCell {
    pub fn new(action_name: &str) -> Self {
        glib::Object::builder()
            .property("action-name", action_name)
            .build()
    }
}
