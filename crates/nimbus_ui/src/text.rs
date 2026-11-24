/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, Widget};
use nimbus_math::Vec2;
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, TextShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Text {
	pub text: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<DColor>,
}

impl Default for Text {
	fn default() -> Self {
		Self {
			text: any::type_name::<Self>().to_string(),
			font: None,
			font_size: 15.0,
			color: None,
		}
	}
}


impl Widget for Text {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let (width, height) = ctx.measure_text_with_font(&self.text, self.font, self.font_size);
		constraints.constrain(Vec2::new(width.ceil(), height.ceil()))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		if ctx.bounds.width() <= 0.0 || ctx.bounds.height() <= 0.0 || self.text.is_empty() {
			return;
		}
		let color = self
			.color
			.map(|c| c.get(&ctx.theme.monet))
			.unwrap_or_else(|| ctx.theme.monet[MonetRole::OnSurface]);
		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		canvas.draw(TextShape::new(
			&self.text,
			font,
			self.font_size,
			ctx.bounds.min,
			color,
		));
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &nimbus_core::PropertyValue,
		_ctx: &nimbus_core::PropertyCtx,
	) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"text" => self.text = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"color" => self.color = Some(value.expect_dcolor()?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Text {
	pub fn builder() -> TextBuilder {
		TextBuilder::new()
	}
}


#[derive(Default)]
pub struct TextBuilder {
	inner: Text,
}

impl TextBuilder {
	pub fn new() -> Self {
		Self {
			inner: Text::default(),
		}
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.text = text.into();
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn font_size(mut self, size: f32) -> Self {
		self.inner.font_size = size;
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn build(self) -> Text {
		self.inner
	}
}

impl From<TextBuilder> for Text {
	fn from(builder: TextBuilder) -> Self {
		builder.build()
	}
}
