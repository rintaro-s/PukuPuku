/* performance_page/widgets/graph_widget.rs
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

pub use eject_failure_dialog::EjectFailureDialog;
pub use graph_widget::GraphWidget;
pub use graph_widget_utils::DatasetGroup;
pub use graph_widget_utils::ScalingSettings;
pub use mem_composition_widget::MemoryCompositionWidget;
pub use sidebar_drop_hint::SidebarDropHint;
pub use smart_data_dialog::SmartDataDialog;
pub use smart_failure_dialog::SmartFailureDialog;
pub use smart_nvme_dialog_row::SmartNvmeDialogRow;
pub use smart_sata_dialog_row::SmartSataDialogRow;

const GRAPH_RADIUS: f32 = 7.;

mod eject_failure_dialog;
mod eject_failure_row;
mod graph_widget;
mod graph_widget_utils;
mod mem_composition_widget;
mod sidebar_drop_hint;
mod smart_data_dialog;
mod smart_failure_dialog;
mod smart_nvme_dialog_row;
mod smart_sata_dialog_row;
