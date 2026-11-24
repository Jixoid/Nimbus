/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyValue, Widget};
use nimbus_math::Vec2;
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Card {
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
	pub shadow_blur: f32,
}

impl Default for Card {
	fn default() -> Self {
		Self {
			padding: 16.0,
			corner_radius: None,
			background: None,
			border_width: 0.0,
			border_color: None,
			shadow_blur: 0.0,
		}
	}
}


impl Widget for Card {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		let child_constraints = constraints.deflate(self.padding * 2.0, self.padding * 2.0);

		let mut max_child_w = 0.0f32;
		let mut max_child_h = 0.0f32;
		let mut laid_out = Vec::with_capacity(children.len());

		for child in children {
			let anchor = ctx.child_anchor(child);
			let size = ctx.layout_child(child, child_constraints);
			max_child_w = max_child_w.max(size.x);
			max_child_h = max_child_h.max(size.y);
			laid_out.push((child, anchor, size));
		}

		let final_size = constraints.constrain(Vec2::new(
			max_child_w + self.padding * 2.0,
			max_child_h + self.padding * 2.0,
		));
		let inner_size = Vec2::new(
			(final_size.x - self.padding * 2.0).max(0.0),
			(final_size.y - self.padding * 2.0).max(0.0),
		);

		for (child, anchor, size) in laid_out {
			let pos = if let Some(anchor) = anchor {
				let final_child_size = match anchor {
					nimbus_core::Anchor::Fill
					| nimbus_core::Anchor::TopFill
					| nimbus_core::Anchor::BottomFill
					| nimbus_core::Anchor::LeftFill
					| nimbus_core::Anchor::RightFill => {
						ctx.layout_child(child, anchor.child_constraints(inner_size))
					}
					_ => size,
				};
				Vec2::new(self.padding, self.padding) + anchor.position(inner_size, final_child_size)
			} else {
				Vec2::new(self.padding, self.padding)
			};
			ctx.set_child_position(child, pos);
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);

		if self.border_width > 0.0 {
			let border_col = self
				.border_color
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::SurfaceOutline]);
			shape = shape.border(border_col, self.border_width);
		}

		if self.shadow_blur > 0.0 {
			shape = shape.shadow(
				monet[MonetRole::Shadow],
				Vec2::new(0.0, 4.0),
				self.shadow_blur,
			);
		}

		canvas.draw(shape);
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"paddings" => {
				if let Some(p) = value.as_f32() {
					self.padding = p;
				} else if let Some(obj) = value.as_object() {
					let top = obj
						.get("top")
						.or_else(|| obj.get("left"))
						.and_then(|v| v.as_f32())
						.unwrap_or(8.0);
					self.padding = top;
				} else {
					return Err(nimbus_core::PropertyError::InvalidValue { expected: "number or object" });
				}
			}
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"color" => self.background = Some(value.expect_dcolor()?),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Card {
	pub fn builder() -> CardBuilder {
		CardBuilder::new()
	}
}


#[derive(Default)]
pub struct CardBuilder {
	inner: Card,
}

impl CardBuilder {
	pub fn new() -> Self {
		Self {
			inner: Card::default(),
		}
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn paddings(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn background(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn border_width(mut self, width: f32) -> Self {
		self.inner.border_width = width;
		self
	}

	pub fn border_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.border_color = Some(color.into());
		self
	}

	pub fn shadow_blur(mut self, blur: f32) -> Self {
		self.inner.shadow_blur = blur;
		self
	}

	pub fn build(self) -> Card {
		self.inner
	}
}

impl From<CardBuilder> for Card {
	fn from(builder: CardBuilder) -> Self {
		builder.build()
	}
}
