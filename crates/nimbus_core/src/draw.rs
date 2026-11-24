/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use nimbus_math::Rect;
use nimbus_theme::Theme;
use crate::ObjectID;


pub struct DrawCtx<'a> {
	pub node_id: ObjectID,
	pub bounds: Rect<f32>,
	pub theme: &'a Theme,
	pub hovered: bool,
	pub pressed: bool,
	pub focused: bool,
	pub(crate) redraw_requested: &'a Cell<bool>,
	pub(crate) layout_requested: &'a Cell<bool>,
}

impl<'a> DrawCtx<'a> {
	#[inline]
	pub fn request_redraw(&self) {
		self.redraw_requested.set(true);
	}

	#[inline]
	pub fn request_layout(&self) {
		self.layout_requested.set(true);
	}

	#[inline]
	pub fn with_bounds(&self, bounds: Rect<f32>) -> Self {
		Self {
			node_id: self.node_id,
			bounds,
			theme: self.theme,
			hovered: self.hovered,
			pressed: self.pressed,
			focused: self.focused,
			redraw_requested: self.redraw_requested,
			layout_requested: self.layout_requested,
		}
	}
}
