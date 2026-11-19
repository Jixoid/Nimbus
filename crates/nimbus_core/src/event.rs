/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Vec2;
use crate::id::ObjectID;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Other(u16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonState {
	Pressed,
	Released,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
	PointerMoved {
		position: Vec2<f32>,
	},
	PointerButton {
		button: MouseButton,
		state: ButtonState,
		position: Vec2<f32>,
	},
	PointerScroll {
		delta: Vec2<f32>,
	},
	PointerEntered,
	PointerLeft,
	Click {
		position: Vec2<f32>,
	},
	Key {
		key_name: String,
		pressed: bool,
	},
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EventResponse {
	Ignored,
	Handled,
}

pub struct EventCtx<'a> {
	pub(crate) node_id: ObjectID,
	pub(crate) redraw_requested: &'a mut bool,
	pub(crate) layout_requested: &'a mut bool,
}

impl<'a> EventCtx<'a> {
	pub fn new(
		node_id: ObjectID,
		redraw_requested: &'a mut bool,
		layout_requested: &'a mut bool,
	) -> Self {
		Self {
			node_id,
			redraw_requested,
			layout_requested,
		}
	}

	pub fn node_id(&self) -> ObjectID {
		self.node_id
	}

	pub fn request_redraw(&mut self) {
		*self.redraw_requested = true;
	}

	pub fn request_layout(&mut self) {
		*self.layout_requested = true;
	}
}
