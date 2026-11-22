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
	pub max: Option<f32>,
	pub height: f32,
	pub corner_radius: Option<f32>,
}

impl Default for MultiProgressBar {
	fn default() -> Self {
		Self {
			segments: Vec::new(),
			max: None,
			height: 24.0,
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
		let monet = &ctx.theme.monet;
		let pill_rad = self.corner_radius.unwrap_or(ctx.bounds.height() * 0.5);
		let inner_rad = 5.0f32;

		let default_palette = [
			monet[MonetRole::Primary],
			monet[MonetRole::Secondary],
			monet[MonetRole::Tertiary],
			monet[MonetRole::PrimaryOutline],
		];

		let total: f32 = self.segments.iter().map(|s| s.value.max(0.0)).sum();
		if total <= 0.0001 && self.segments.is_empty() {
			canvas.draw(
				RectShape::new(ctx.bounds, monet[MonetRole::PrimaryOutline]).radius(pill_rad),
			);
			return;
		}

		let scale_total = if let Some(m) = self.max {
			m.max(total)
		} else if total <= 1.0 {
			1.0f32
		} else {
			total.max(100.0)
		};

		let mut items: Vec<(f32, Color)> = Vec::new();
		for (i, seg) in self.segments.iter().enumerate() {
			if seg.value > 0.0 {
				let color = seg
					.color
					.unwrap_or_else(|| default_palette[i % default_palette.len()]);
				items.push((seg.value, color));
			}
		}

		let remaining = scale_total - total;
		if remaining > 0.001 {
			let rem_color = default_palette[items.len() % default_palette.len()];
			items.push((remaining, rem_color));
		}

		if items.is_empty() {
			return;
		}

		let gap = 3.0f32;
		let num_items = items.len();
		let total_gap = (num_items.saturating_sub(1)) as f32 * gap;
		let usable_width = (ctx.bounds.width() - total_gap).max(0.0);

		let mut current_x = ctx.bounds.x();

		for (i, (val, color)) in items.into_iter().enumerate() {
			let seg_w = (val / scale_total) * usable_width;
			if seg_w <= 0.0 {
				continue;
			}

			let seg_rect = Rect::new(current_x, ctx.bounds.y(), seg_w, ctx.bounds.height());

			let shape = if num_items == 1 {
				RectShape::new(seg_rect, color).radius(pill_rad)
			} else if i == 0 {
				RectShape::new(seg_rect, color).radius4([pill_rad, inner_rad, inner_rad, pill_rad])
			} else if i == num_items - 1 {
				RectShape::new(seg_rect, color).radius4([inner_rad, pill_rad, pill_rad, inner_rad])
			} else {
				RectShape::new(seg_rect, color).radius(inner_rad)
			};

			canvas.draw(shape);
			current_x += seg_w + gap;
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"values" => {
				let arr = value.expect_array()?;
				let mut segs = Vec::new();
				let mut i = 0;
				while i < arr.len() {
					match &arr[i] {
						PropertyValue::Number(n) => {
							let val = *n as f32;
							if i + 1 < arr.len()
								&& let Some(c) = arr[i + 1].resolve_color(&ctx.theme.monet)
							{
								segs.push(ProgressSegment::new(val, Some(c)));
								i += 2;
								continue;
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
			}
			"max" => self.max = Some(value.expect_f32()?),
			"height" => self.height = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl MultiProgressBar {
	pub fn builder() -> MultiProgressBarBuilder {
		MultiProgressBarBuilder::new()
	}
}


#[derive(Default)]
pub struct MultiProgressBarBuilder {
	inner: MultiProgressBar,
}

impl MultiProgressBarBuilder {
	pub fn new() -> Self {
		Self {
			inner: MultiProgressBar::default(),
		}
	}

	pub fn segments(mut self, segments: Vec<ProgressSegment>) -> Self {
		self.inner.segments = segments;
		self
	}

	pub fn segment(mut self, value: f32, color: Option<Color>) -> Self {
		self.inner.segments.push(ProgressSegment::new(value, color));
		self
	}

	pub fn values<I>(mut self, values: I) -> Self
	where
		I: IntoIterator<Item = f32>,
	{
		self.inner.segments = values
			.into_iter()
			.map(|v| ProgressSegment::new(v, None))
			.collect();
		self
	}

	pub fn max(mut self, max: f32) -> Self {
		self.inner.max = Some(max);
		self
	}

	pub fn height(mut self, height: f32) -> Self {
		self.inner.height = height;
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn build(self) -> MultiProgressBar {
		self.inner
	}
}

impl From<MultiProgressBarBuilder> for MultiProgressBar {
	fn from(builder: MultiProgressBarBuilder) -> Self {
		builder.build()
	}
}
