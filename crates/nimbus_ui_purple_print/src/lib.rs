/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod node;
pub mod pin;
pub mod view;

pub use node::{PurplePrintNode, PurplePrintNodeBuilder};
pub use pin::{NodeBindKind, NodePin, PinDirection, PinRef, WireConnection, WireState};
pub use view::{PurplePrintView, PurplePrintViewBuilder};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("purpleprint::view", || Box::new(PurplePrintView::default()));
	registry.register("purpleprint::node", || Box::new(PurplePrintNode::default()));
}
