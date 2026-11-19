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
use nimbus_theme::MonetRole;


pub struct ProgressBar {
	pub progress: f32, // 0.0 .. 1.0
	pub color: Option<Color>,
}

impl Default for ProgressBar {
	fn default() -> Self {
		Self {
			progress: 0.0,
			color: None,
		}
	}
}


impl Widget for ProgressBar {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let width = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			200.0
		};
		constraints.constrain(Vec2::new(width, 12.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let radius = ctx.bounds.height() * 0.5;

		// Track
		canvas.draw(
			RectShape::new(ctx.bounds, theme.monet.get(MonetRole::SurfaceContainerHighest))
				.radius(radius),
		);

		// Fill
		let fill_width = (ctx.bounds.width() * self.progress).max(radius * 2.0);
		let fill_color = self
			.color
			.unwrap_or_else(|| theme.monet.get(MonetRole::Primary));

		if self.progress > 0.001 {
			canvas.draw(
				RectShape::new(
					nimbus_math::Rect::new(
						ctx.bounds.x(),
						ctx.bounds.y(),
						fill_width.min(ctx.bounds.width()),
						ctx.bounds.height(),
					),
					fill_color,
				)
				.radius(radius),
			);
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &nimbus_core::PropertyValue,
		ctx: &nimbus_core::PropertyCtx,
	) -> bool {
		match key {
			"value" => {
				if let Some(p) = value.as_f32() {
					self.progress = p;
					return true;
				}
			}

			"color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.color = Some(c);
					return true;
				}
			}
			
			_ => {}
		}
		false
	}
}
