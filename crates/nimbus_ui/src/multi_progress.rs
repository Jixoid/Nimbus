/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyValue, Widget};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::MonetRole;


#[derive(Clone, Debug, PartialEq)]
pub struct ProgressSegment {
	pub value: f32,
	pub color: Option<Color>,
}

impl ProgressSegment {
	pub fn new(value: f32, color: Option<Color>) -> Self {
		Self { value, color }
	}
}

pub struct MultiProgressBar {
	pub segments: Vec<ProgressSegment>,
	pub height: f32,
	pub corner_radius: Option<f32>,
}

impl Default for MultiProgressBar {
	fn default() -> Self {
		Self {
			segments: Vec::new(),
			height: 16.0,
			corner_radius: None,
		}
	}
}

impl Widget for MultiProgressBar {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let width = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			200.0
		};
		constraints.constrain(Vec2::new(width, self.height))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let radius = self.corner_radius.unwrap_or(ctx.bounds.height() * 0.5);

		// Background track
		canvas.draw(
			RectShape::new(ctx.bounds, theme.monet.get(MonetRole::SurfaceContainerHighest))
				.radius(radius),
		);

		let total: f32 = self.segments.iter().map(|s| s.value.max(0.0)).sum();
		if total <= 0.0001 || self.segments.is_empty() {
			return;
		}

		let default_palette = [
			theme.monet.get(MonetRole::Primary),
			theme.monet.get(MonetRole::Secondary),
			theme.monet.get(MonetRole::Tertiary),
			theme.monet.get(MonetRole::PrimaryContainer),
			theme.monet.get(MonetRole::Outline),
		];

		let gap = 2.0f32;
		let num_gaps = (self.segments.len().saturating_sub(1)) as f32;
		let usable_width = (ctx.bounds.width() - num_gaps * gap).max(0.0);

		let mut current_x = ctx.bounds.x();

		for (i, seg) in self.segments.iter().enumerate() {
			if seg.value <= 0.0 {
				continue;
			}
			let seg_w = (seg.value / total) * usable_width;
			let color = seg
				.color
				.unwrap_or_else(|| default_palette[i % default_palette.len()]);

			let seg_rect = Rect::new(current_x, ctx.bounds.y(), seg_w, ctx.bounds.height());
			canvas.draw(RectShape::new(seg_rect, color).radius(radius.min(seg_w * 0.5)));

			current_x += seg_w + gap;
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> bool {
		match key {
			"values" => {
				if let Some(arr) = value.as_array() {
					let mut segs = Vec::new();
					let mut i = 0;
					while i < arr.len() {
						match &arr[i] {
							PropertyValue::Number(n) => {
								let val = *n as f32;
								// Check if subsequent item is a color
								if i + 1 < arr.len() {
									if let Some(c) = arr[i + 1].resolve_color(&ctx.theme.monet) {
										segs.push(ProgressSegment::new(val, Some(c)));
										i += 2;
										continue;
									}
								}
								segs.push(ProgressSegment::new(val, None));
								i += 1;
							}
							PropertyValue::Object(obj) => {
								let val = obj
									.get("value")
									.or_else(|| obj.get("val"))
									.and_then(|v| v.as_f32())
									.unwrap_or(0.0);
								let color = obj
									.get("color")
									.and_then(|c| c.resolve_color(&ctx.theme.monet));
								segs.push(ProgressSegment::new(val, color));
								i += 1;
							}
							_ => {
								i += 1;
							}
						}
					}
					self.segments = segs;
					return true;
				}
			}
			
			"height" => {
				if let Some(h) = value.as_f32() {
					self.height = h;
					return true;
				}
			}

			"corner_radius" => {
				if let Some(r) = value.as_f32() {
					self.corner_radius = Some(r);
					return true;
				}
			}
			
			_ => {}
		}
		false
	}
}
