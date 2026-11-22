/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{
	BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::Vec2;
use nimbus_render2d::Canvas2D;


#[derive(Default)]
pub struct Spacer {
	pub width: f32,
	pub height: f32,
}

impl Spacer {
	pub fn new(width: f32, height: f32) -> Self {
		Self { width, height }
	}
}

impl Widget for Spacer {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(self.width, self.height))
	}

	fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"size" => {
				let s = value.expect_f32()?.max(0.0);
				self.width = s;
				self.height = s;
			}
			"width" => self.width = value.expect_f32()?.max(0.0),
			"height" => self.height = value.expect_f32()?.max(0.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Spacer {
	pub fn builder() -> SpacerBuilder {
		SpacerBuilder::new()
	}
}


#[derive(Default)]
pub struct SpacerBuilder {
	inner: Spacer,
}

impl SpacerBuilder {
	pub fn new() -> Self {
		Self {
			inner: Spacer::default(),
		}
	}

	pub fn width(mut self, width: f32) -> Self {
		self.inner.width = width.max(0.0);
		self
	}

	pub fn height(mut self, height: f32) -> Self {
		self.inner.height = height.max(0.0);
		self
	}

	pub fn size(mut self, size: f32) -> Self {
		let s = size.max(0.0);
		self.inner.width = s;
		self.inner.height = s;
		self
	}

	pub fn build(self) -> Spacer {
		self.inner
	}
}

impl From<SpacerBuilder> for Spacer {
	fn from(builder: SpacerBuilder) -> Self {
		builder.build()
	}
}
