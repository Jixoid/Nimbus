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


pub struct Check {
	pub label: String,
	pub checked: bool,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<Color>,
	pub on_changed: Option<MethodCallback>,
}

impl Default for Check {
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

impl Widget for Check {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let box_size = 20.0f32;
		let gap = 8.0f32;

		if self.label.is_empty() {
			constraints.constrain(Vec2::new(box_size, box_size))
		} else {
			let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
			let total_w = box_size + gap + text_w;
			let total_h = box_size.max(text_h);
			constraints.constrain(Vec2::new(total_w.ceil(), total_h.ceil()))
		}
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let box_size = 20.0f32;
		let box_x = ctx.bounds.x();
		let box_y = ctx.bounds.y() + (ctx.bounds.height() - box_size) * 0.5;
		let box_rect = Rect::new(box_x, box_y, box_size, box_size);

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
			let check_mark_color = if ctx.hovered {
				theme.monet.get(MonetRole::OnPrimaryContainer)
			} else {
				theme.monet.get(MonetRole::OnPrimary)
			};
			canvas.draw(RectShape::new(box_rect, fill_color).radius(theme.radius.xsmall));

			// Inner check indicator
			let inner_rect = Rect::new(box_x + 5.0, box_y + 5.0, 10.0, 10.0);
			canvas.draw(RectShape::new(inner_rect, check_mark_color).radius(2.0));
		} else {
			canvas.draw(
				RectShape::new(box_rect, Color::TRANSPARENT)
					.radius(theme.radius.xsmall)
					.border(border_color, theme.stroke.thick),
			);
		}

		if !self.label.is_empty() {
			let (text_w, text_h) = canvas.measure_text(&self.label, self.font_size);
			let _ = text_w;
			let text_x = box_x + box_size + 8.0;
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
