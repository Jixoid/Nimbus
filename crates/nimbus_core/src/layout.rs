/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::Vec2;
use nimbus_render2d::{FontId, FontManager};
use crate::id::ObjectID;
use crate::node::Anchor;
use crate::tree::UiTree;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoxConstraints {
	pub min_width: f32,
	pub max_width: f32,
	pub min_height: f32,
	pub max_height: f32,
}

impl BoxConstraints {
	pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
		Self {
			min_width: min_width.max(0.0),
			max_width: max_width.max(min_width),
			min_height: min_height.max(0.0),
			max_height: max_height.max(min_height),
		}
	}

	pub fn tight(size: Vec2<f32>) -> Self {
		Self {
			min_width: size.x.max(0.0),
			max_width: size.x.max(0.0),
			min_height: size.y.max(0.0),
			max_height: size.y.max(0.0),
		}
	}

	pub fn loose(size: Vec2<f32>) -> Self {
		Self {
			min_width: 0.0,
			max_width: size.x.max(0.0),
			min_height: 0.0,
			max_height: size.y.max(0.0),
		}
	}

	pub fn unbounded() -> Self {
		Self {
			min_width: 0.0,
			max_width: f32::INFINITY,
			min_height: 0.0,
			max_height: f32::INFINITY,
		}
	}

	pub fn constrain(&self, size: Vec2<f32>) -> Vec2<f32> {
		Vec2::new(
			size.x.clamp(self.min_width, self.max_width),
			size.y.clamp(self.min_height, self.max_height),
		)
	}

	pub fn deflate(&self, horizontal: f32, vertical: f32) -> Self {
		let min_w = (self.min_width - horizontal).max(0.0);
		let max_w = if self.max_width.is_finite() {
			(self.max_width - horizontal).max(min_w)
		} else {
			f32::INFINITY
		};

		let min_h = (self.min_height - vertical).max(0.0);
		let max_h = if self.max_height.is_finite() {
			(self.max_height - vertical).max(min_h)
		} else {
			f32::INFINITY
		};

		Self {
			min_width: min_w,
			max_width: max_w,
			min_height: min_h,
			max_height: max_h,
		}
	}

	pub fn is_tight(&self) -> bool {
		self.min_width >= self.max_width && self.min_height >= self.max_height
	}

	pub fn max_size(&self) -> Vec2<f32> {
		Vec2::new(self.max_width, self.max_height)
	}

	pub fn min_size(&self) -> Vec2<f32> {
		Vec2::new(self.min_width, self.min_height)
	}
}


pub struct LayoutCtx<'a> {
	pub tree: &'a mut UiTree,
	pub node_id: ObjectID,
}

impl<'a> LayoutCtx<'a> {
	pub fn children(&self) -> Vec<ObjectID> {
		self
			.tree
			.children(self.node_id)
			.iter()
			.copied()
			.filter(|&id| self.tree.is_visible(id))
			.collect()
	}

	pub fn all_children(&self) -> Vec<ObjectID> {
		self.tree.children(self.node_id).to_vec()
	}

	pub fn is_child_visible(&self, child_id: ObjectID) -> bool {
		self.tree.is_visible(child_id)
	}

	pub fn layout_child(&mut self, child_id: ObjectID, constraints: BoxConstraints) -> Vec2<f32> {
		self.tree.layout_node(child_id, constraints)
	}

	pub fn set_child_position(&mut self, child_id: ObjectID, local_pos: Vec2<f32>) {
		self
			.tree
			.set_child_position(self.node_id, child_id, local_pos);
	}

	pub fn set_clip_children(&mut self, clip: bool) {
		self.tree.set_clip_children(self.node_id, clip);
	}

	pub fn child_anchor(&self, child_id: ObjectID) -> Option<Anchor> {
		self.tree.anchor(child_id)
	}

	pub fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
		FontManager::default_font().measure_text(text, font_size)
	}

	pub fn measure_text_with_font(
		&self,
		text: &str,
		_font_id: Option<FontId>,
		font_size: f32,
	) -> (f32, f32) {
		FontManager::default_font().measure_text(text, font_size)
	}

	pub fn measure_text_with_face(
		&self,
		face: &nimbus_render2d::FontFace,
		text: &str,
		font_size: f32,
	) -> (f32, f32) {
		face.measure_text(text, font_size)
	}
}
