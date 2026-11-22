/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{BoxConstraints, DrawCtx, LayoutCtx, Widget};
use nimbus_math::{Color, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, TextShape};
use nimbus_theme::MonetRole;


pub struct Text {
	pub text: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<Color>,
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
		let color = self
			.color
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
		ctx: &nimbus_core::PropertyCtx,
	) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"text" => self.text = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"color" => self.color = Some(value.expect_color(&ctx.theme.monet)?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}
