/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx, MethodCallback, PropertyCtx, PropertyValue, Widget};
use nimbus_math::Vec2;
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};


pub struct Button {
	pub label: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub padding: Vec2<f32>,
	pub on_click: Option<Box<dyn FnMut()>>,
}

impl Default for Button {
	fn default() -> Self {
		Self {
			label: any::type_name::<Self>().to_string(),
			font: None,
			font_size: 14.0,
			padding: Vec2::new(20.0, 10.0),
			on_click: None,
		}
	}
}


impl Widget for Button {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
		let width = text_w + self.padding.x * 2.0;
		let height = text_h + self.padding.y * 2.0;
		constraints.constrain(Vec2::new(width.ceil(), height.ceil()))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let bg_color = if ctx.pressed {
			theme.primary_active
		} else if ctx.hovered {
			theme.primary_hover
		} else {
			theme.primary
		};

		canvas.draw(RectShape::new(ctx.bounds, bg_color).radius(theme.button_radius));

		let (text_w, text_h) = canvas.measure_text(&self.label, self.font_size);
		let text_x = ctx.bounds.x() + (ctx.bounds.width() - text_w) * 0.5;
		let text_y = ctx.bounds.y() + (ctx.bounds.height() - text_h) * 0.5;

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		canvas.draw(TextShape::new(
			&self.label,
			font,
			self.font_size,
			Vec2::new(text_x, text_y),
			theme.text_primary,
		));
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if let InputEvent::Click { .. } = event
			&& let Some(cb) = &mut self.on_click
		{
			cb();
			ctx.request_redraw();
			return EventResponse::Handled;
		}
		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> bool {
		let _ = ctx;
		match key {
			"text" => {
				if let Some(s) = value.as_str() {
					self.label = s.to_string();
					return true;
				}
			}

			"font_size" => {
				if let Some(n) = value.as_f32() {
					self.font_size = n;
					return true;
				}
			}

			"paddings" => {
				if let Some(n) = value.as_f32() {
					self.padding = Vec2::new(n, n);
					return true;
				} else if let Some(v) = value.as_vec2() {
					self.padding = v;
					return true;
				}
			}
			
			_ => {}
		}
		false
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_click")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_click = Some(callback);
			true
		} else {
			false
		}
	}
}
