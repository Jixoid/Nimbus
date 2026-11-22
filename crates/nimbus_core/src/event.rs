/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Vec2;
use crate::id::ObjectID;
use crate::property::PropertyValue;
use crate::task::{IntoTask, TaskQueue};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageTarget {
	Siblings,
	Parent,
	Children,
	Root,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
	pub sender: ObjectID,
	pub target: MessageTarget,
	pub name: String,
	pub payload: PropertyValue,
}


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
		text: Option<String>,
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
	pub(crate) messages: Vec<Message>,
	pub(crate) task_queue: TaskQueue,
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
			messages: Vec::new(),
			task_queue: TaskQueue::new(),
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

	pub fn task_queue(&self) -> TaskQueue {
		self.task_queue.clone()
	}

	pub fn push_task<M, F: IntoTask<M>>(&self, task: F) {
		self.task_queue.push(task);
	}

	pub fn dispatch_message(
		&mut self,
		target: MessageTarget,
		name: impl Into<String>,
		payload: PropertyValue,
	) {
		self.messages.push(Message {
			sender: self.node_id,
			target,
			name: name.into(),
			payload,
		});
	}
}
