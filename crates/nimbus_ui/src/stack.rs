/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{
	Anchor, BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Stack {
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
	pub shadow_blur: f32,
}

impl Default for Stack {
	fn default() -> Self {
		Self {
			padding: 0.0,
			corner_radius: None,
			background: None,
			border_width: 0.0,
			border_color: None,
			shadow_blur: 0.0,
		}
	}
}

impl Widget for Stack {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		let pad = self.padding * 2.0;

		let available_w = if constraints.max_width.is_finite() {
			(constraints.max_width - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let available_h = if constraints.max_height.is_finite() {
			(constraints.max_height - pad).max(0.0)
		} else {
			f32::INFINITY
		};

		let mut max_w = 0.0f32;
		let mut max_h = 0.0f32;
		let mut child_sizes = Vec::with_capacity(children.len());

		// measure non-fill children loosely to determine natural stack size when unconstrained
		for &child in &children {
			let anchor = ctx.child_anchor(child).unwrap_or(Anchor::TopLeft);
			let child_constraints = if available_w.is_finite() && available_h.is_finite() {
				anchor.child_constraints(Vec2::new(available_w, available_h))
			} else {
				BoxConstraints::new(0.0, available_w, 0.0, available_h)
			};

			let size = ctx.layout_child(child, child_constraints);
			max_w = max_w.max(size.x);
			max_h = max_h.max(size.y);
			child_sizes.push((child, anchor, size));
		}

		let final_size = constraints.constrain(Vec2::new(max_w + pad, max_h + pad));
		let inner_size = Vec2::new(
			(final_size.x - pad).max(0.0),
			(final_size.y - pad).max(0.0),
		);

		// re-layout Fill-family children if stack size differed from available, then place all
		for (child, anchor, size) in child_sizes {
			let final_child_size = match anchor {
				Anchor::Fill | Anchor::TopFill | Anchor::BottomFill | Anchor::LeftFill | Anchor::RightFill => {
					ctx.layout_child(child, anchor.child_constraints(inner_size))
				}
				_ => size,
			};
			let pos = Vec2::new(self.padding, self.padding)
				+ anchor.position(inner_size, final_child_size);
			ctx.set_child_position(child, pos);
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		if self.background.is_none() && self.border_width <= 0.0 && self.shadow_blur <= 0.0 {
			return;
		}

		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or(Color::TRANSPARENT);

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

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"paddings" => self.padding = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"color" => self.background = Some(value.expect_dcolor()?),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Stack {
	pub fn builder() -> StackBuilder {
		StackBuilder::new()
	}
}


#[derive(Default)]
pub struct StackBuilder {
	inner: Stack,
}

impl StackBuilder {
	pub fn new() -> Self {
		Self {
			inner: Stack::default(),
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

	pub fn build(self) -> Stack {
		self.inner
	}
}

impl From<StackBuilder> for Stack {
	fn from(builder: StackBuilder) -> Self {
		builder.build()
	}
}
