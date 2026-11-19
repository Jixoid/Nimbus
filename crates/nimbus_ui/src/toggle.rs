/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx, MethodCallback, PropertyCtx, PropertyValue, Widget};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct Toggle {
	pub label: String,
	pub checked: bool,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<Color>,
	pub on_changed: Option<MethodCallback>,
}

impl Default for Toggle {
	fn default() -> Self {
		Self {
			label: any::type_name::<Self>().to_string(),
			checked: false,
			font: None,
			font_size: 14.0,
			color: None,
			on_changed: None,
		}
	}
}

impl Widget for Toggle {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let switch_w = 48.0f32;
		let switch_h = 24.0f32;

		if self.label.is_empty() {
			constraints.constrain(Vec2::new(switch_w, switch_h))
		} else {
			let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
			let total_w = if constraints.max_width.is_finite() && constraints.max_width > (text_w + switch_w + 12.0) {
				constraints.max_width
			} else {
				text_w + switch_w + 12.0
			};
			let total_h = switch_h.max(text_h);
			constraints.constrain(Vec2::new(total_w.ceil(), total_h.ceil()))
		}
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let switch_w = 48.0f32;
		let switch_h = 24.0f32;
		let thumb_size = 18.0f32;
		let thumb_radius = thumb_size * 0.5;

		let track_x = if self.label.is_empty() {
			ctx.bounds.x()
		} else {
			ctx.bounds.x() + ctx.bounds.width() - switch_w
		};
		let track_y = ctx.bounds.y() + (ctx.bounds.height() - switch_h) * 0.5;
		let track_rect = Rect::new(track_x, track_y, switch_w, switch_h);

		let border_color = if ctx.hovered {
			theme.monet.get(MonetRole::Primary)
		} else {
			theme.monet.get(MonetRole::Outline)
		};

		if self.checked {
			let fill_color = if ctx.hovered {
				theme.monet.get(MonetRole::PrimaryContainer)
			} else {
				theme.monet.get(MonetRole::Primary)
			};
			canvas.draw(RectShape::new(track_rect, fill_color).radius(12.0));

			let thumb_x = track_x + switch_w - thumb_size - 3.0;
			let thumb_y = track_y + (switch_h - thumb_size) * 0.5;
			let thumb_color = theme.monet.get(MonetRole::OnPrimary);
			canvas.draw(
				RectShape::new(Rect::new(thumb_x, thumb_y, thumb_size, thumb_size), thumb_color)
					.radius(thumb_radius),
			);
		} else {
			canvas.draw(
				RectShape::new(track_rect, theme.monet.get(MonetRole::SurfaceContainerHighest))
					.radius(12.0)
					.border(border_color, theme.stroke.thick),
			);

			let thumb_x = track_x + 3.0;
			let thumb_y = track_y + (switch_h - thumb_size) * 0.5;
			canvas.draw(
				RectShape::new(
					Rect::new(thumb_x, thumb_y, thumb_size, thumb_size),
					theme.monet.get(MonetRole::Outline),
				)
				.radius(thumb_radius),
			);
		}

		if !self.label.is_empty() {
			let (_, text_h) = canvas.measure_text(&self.label, self.font_size);
			let text_x = ctx.bounds.x();
			let text_y = ctx.bounds.y() + (ctx.bounds.height() - text_h) * 0.5;

			let fface = FontManager::default_font();
			let font = match self.font {
				Some(id) => Font::with_id(id, fface, self.font_size),
				None => Font::new(fface, self.font_size),
			};

			let text_color = self
				.color
				.unwrap_or_else(|| theme.monet.get(MonetRole::OnSurface));
			canvas.draw(TextShape::new(
				&self.label,
				font,
				self.font_size,
				Vec2::new(text_x, text_y),
				text_color,
			));
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if let InputEvent::Click { .. } = event {
			self.checked = !self.checked;
			if let Some(cb) = &mut self.on_changed {
				cb();
			}
			ctx.request_redraw();
			return EventResponse::Handled;
		}
		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> bool {
		match key {
			"checked" => {
				if let Some(b) = value.as_bool() {
					self.checked = b;
					return true;
				}
			}

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

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_changed = Some(callback);
			true
		} else {
			false
		}
	}
}
