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
	) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"spacing" => {
				let s = value.expect_f32()?;
				self.spacing_x = s;
				self.spacing_y = s;
			}
			"horiz_spacing" => self.spacing_x = value.expect_f32()?,
			"vert_spacing" => self.spacing_y = value.expect_f32()?,
			"paddings" => self.padding = value.expect_f32()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Flow {
	pub fn builder() -> FlowBuilder {
		FlowBuilder::new()
	}
}


#[derive(Default)]
pub struct FlowBuilder {
	inner: Flow,
}

impl FlowBuilder {
	pub fn new() -> Self {
		Self {
			inner: Flow::default(),
		}
	}

	pub fn spacing(mut self, spacing: f32) -> Self {
		self.inner.spacing_x = spacing;
		self.inner.spacing_y = spacing;
		self
	}

	pub fn spacing_x(mut self, spacing_x: f32) -> Self {
		self.inner.spacing_x = spacing_x;
		self
	}

	pub fn horiz_spacing(mut self, spacing_x: f32) -> Self {
		self.inner.spacing_x = spacing_x;
		self
	}

	pub fn spacing_y(mut self, spacing_y: f32) -> Self {
		self.inner.spacing_y = spacing_y;
		self
	}

	pub fn vert_spacing(mut self, spacing_y: f32) -> Self {
		self.inner.spacing_y = spacing_y;
		self
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn paddings(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn build(self) -> Flow {
		self.inner
	}
}

impl From<FlowBuilder> for Flow {
	fn from(builder: FlowBuilder) -> Self {
		builder.build()
	}
}
