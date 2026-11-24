/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub mod curve;
pub mod model;
pub mod view;

pub use curve::{CurveEditor, CurveEditorBuilder};
pub use model::{Keyframe, KeyframeInterp, TimelineClip, TimelineTrack};
pub use view::{TimelineView, TimelineViewBuilder};

use nimbus_core::WidgetRegistry;


pub fn register_widgets(registry: &mut WidgetRegistry) {
	registry.register("timeline::view", || Box::new(TimelineView::default()));
	registry.register("timeline::curve", || Box::new(CurveEditor::default()));
}
