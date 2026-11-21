/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, Widget};
use nimbus_math::Vec2;
use nimbus_render2d::Canvas2D;


pub struct Flow {
	pub spacing_x: f32,
	pub spacing_y: f32,
	pub padding: f32,
}

impl Default for Flow {
	fn default() -> Self {
		Self {
			spacing_x: 8.0,
			spacing_y: 8.0,
			padding: 0.0,
		}
	}
}

impl Widget for Flow {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		if children.is_empty() {
			return constraints.constrain(Vec2::new(self.padding * 2.0, self.padding * 2.0));
		}

		let max_width = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			f32::INFINITY
		};

		let child_constraints = BoxConstraints::new(
			0.0,
			(max_width - self.padding * 2.0).max(0.0),
			0.0,
			f32::INFINITY,
		);

		let mut current_x = self.padding;
		let mut current_y = self.padding;
		let mut line_height = 0.0f32;
		let mut max_used_width = 0.0f32;

		for child in &children {
			let size = ctx.layout_child(*child, child_constraints);

			// Wrap to next line if it exceeds max width and we already placed at least one item
			if current_x + size.x + self.padding > max_width && current_x > self.padding {
				current_x = self.padding;
				current_y += line_height + self.spacing_y;
				line_height = 0.0;
			}

			ctx.set_child_position(*child, Vec2::new(current_x, current_y));

			current_x += size.x + self.spacing_x;
			line_height = line_height.max(size.y);
			max_used_width = max_used_width.max(current_x - self.spacing_x + self.padding);
		}

		let total_width = max_used_width.max(self.padding * 2.0);
		let total_height = current_y + line_height + self.padding;

		constraints.constrain(Vec2::new(total_width, total_height))
	}

	fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}

	fn set_property(
		&mut self,
		key: &str,
		value: &nimbus_core::PropertyValue,
		_ctx: &nimbus_core::PropertyCtx,
	) -> bool {
		match key {
			"spacing" => {
				if let Some(s) = value.as_f32() {
					self.spacing_x = s;
					self.spacing_y = s;
					return true;
				}
			}
			
			"horiz_spacing" => {
				if let Some(s) = value.as_f32() {
					self.spacing_x = s;
					return true;
				}
			}
			
			"vert_spacing" => {
				if let Some(s) = value.as_f32() {
					self.spacing_y = s;
					return true;
				}
			}

			"paddings" => {
				if let Some(p) = value.as_f32() {
					self.padding = p;
					return true;
				}
			}
			
			_ => {}
		}
		false
	}
}
