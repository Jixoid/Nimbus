/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, Widget};
use nimbus_math::{Color, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};


pub struct Card {
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<Color>,
	pub border_width: f32,
	pub border_color: Option<Color>,
	pub shadow_blur: f32,
}

impl Default for Card {
	fn default() -> Self {
		Self {
			padding: 16.0,
			corner_radius: None,
			background: None,
			border_width: 1.0,
			border_color: None,
			shadow_blur: 16.0,
		}
	}
}


impl Widget for Card {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		let child_constraints = constraints.deflate(self.padding * 2.0, self.padding * 2.0);

		let mut max_child_w = 0.0f32;
		let mut max_child_h = 0.0f32;

		for child in children {
			let size = ctx.layout_child(child, child_constraints);
			ctx.set_child_position(child, Vec2::new(self.padding, self.padding));
			max_child_w = max_child_w.max(size.x);
			max_child_h = max_child_h.max(size.y);
		}

		let total_size = Vec2::new(
			max_child_w + self.padding * 2.0,
			max_child_h + self.padding * 2.0,
		);

		constraints.constrain(total_size)
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let radius = self.corner_radius.unwrap_or(theme.card_radius);
		let bg = self.background.unwrap_or(theme.surface);
		let border_col = self.border_color.unwrap_or(theme.border);

		canvas.draw(
			RectShape::new(ctx.bounds, bg)
				.radius(radius)
				.border(border_col, self.border_width)
				.shadow(theme.shadow, Vec2::new(0.0, 4.0), self.shadow_blur),
		);
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &nimbus_core::PropertyValue,
		ctx: &nimbus_core::PropertyCtx,
	) -> bool {
		match key {
			"paddings" => {
				if let Some(p) = value.as_f32() {
					self.padding = p;
					return true;
				} else if let Some(obj) = value.as_object() {
					let top = obj
						.get("top")
						.or_else(|| obj.get("left"))
						.and_then(|v| v.as_f32())
						.unwrap_or(8.0);
					self.padding = top;
					return true;
				}
			}
			
			"corner_radius" => {
				if let Some(r) = value.as_f32() {
					self.corner_radius = Some(r);
					return true;
				}
			}

			"color" => {
				if let Some(c) = value.resolve_color(ctx.theme) {
					self.background = Some(c);
					return true;
				}
			}

			"border_color" => {
				if let Some(c) = value.resolve_color(ctx.theme) {
					self.border_color = Some(c);
					return true;
				}
			}

			"border_width" => {
				if let Some(w) = value.as_f32() {
					self.border_width = w;
					return true;
				}
			}

			"shadow_blur" => {
				if let Some(b) = value.as_f32() {
					self.shadow_blur = b;
					return true;
				}
			}
			
			_ => {}
		}
		false
	}
}
