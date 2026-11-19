/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Rect;
use crate::id::ObjectID;


#[derive(Clone, Debug)]
pub struct NodeData {
	pub parent: Option<ObjectID>,
	pub children: Vec<ObjectID>,
	pub bounds: Rect<f32>,
	pub visible: bool,
	pub enabled: bool,
	pub hovered: bool,
	pub pressed: bool,
	pub focused: bool,
	pub clip_children: bool,
}

impl Default for NodeData {
	fn default() -> Self {
		Self {
			parent: None,
			children: Vec::new(),
			bounds: Rect::ZERO,
			visible: true,
			enabled: true,
			hovered: false,
			pressed: false,
			focused: false,
			clip_children: false,
		}
	}
}

impl NodeData {
	pub fn new() -> Self {
		Self::default()
	}
}
