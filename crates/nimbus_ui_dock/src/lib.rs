/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod area;
pub mod pane;
pub mod tree;

pub use area::{DockArea, DockAreaBuilder};
pub use pane::{DockPane, DockPaneBuilder};
pub use tree::{DockNode, DockPath, DockSplitAxis, DockZone};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("dock::area", || Box::new(DockArea::default()));
	registry.register("dock::pane", || Box::new(DockPane::default()));
}
