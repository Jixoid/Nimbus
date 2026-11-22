/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{
	Animated, BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent,
	Interpolate, LayoutCtx, MethodCallback, PropertyCtx, PropertyValue, Spring, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct Switch {
	pub label: String,
	pub checked: bool,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<Color>,
	pub on_changed: Option<MethodCallback>,
	pub progress: Animated<f32>,
}

impl Default for Switch {
	fn default() -> Self {
		Self {
			label: any::type_name::<Self>().to_string(),
			checked: false,
			font: None,
			font_size: 15.0,
			color: None,
			on_changed: None,
			progress: Animated::new(0.0),
		}
	}
}

impl Widget for Switch {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let switch_w = 51.0f32;
		let switch_h = 26.0f32;

		if self.label.is_empty() {
			constraints.constrain(Vec2::new(switch_w, switch_h))
		} else {
			let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
			let total_w = if constraints.max_width.is_finite() && constraints.max_width > (text_w + 8.0 + switch_w) {
				constraints.max_width
			} else {
				text_w + 8.0 + switch_w
			};
			let total_h = (text_h + 8.0).max(switch_h);
			constraints.constrain(Vec2::new(total_w.ceil(), total_h.ceil()))
		}
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let switch_w = 51.0f32;
		let switch_h = 26.0f32;

		let track_x = if self.label.is_empty() {
			ctx.bounds.x()
		} else {
			ctx.bounds.x() + ctx.bounds.width() - switch_w
		};
		let track_y = ctx.bounds.y() + (ctx.bounds.height() - switch_h) * 0.5;
		let track_rect = Rect::new(track_x, track_y, switch_w, switch_h);

		let t = self.progress.value();
		if self.progress.is_animating() {
			ctx.request_redraw();
		}

		let t_clamped = t.clamp(0.0, 1.0);
		let active_bg = monet[MonetRole::Primary];
		let inactive_bg = Color::TRANSPARENT;
		let track_bg = inactive_bg.interpolate(&active_bg, t_clamped);

		let border_color = if ctx.hovered {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::PrimaryOutline]
		};
		let border_w = (1.0 - t_clamped) * 2.0;

		let mut track_shape = RectShape::new(track_rect, track_bg).radius(13.0);
		if border_w > 0.05 {
			track_shape = track_shape.border(border_color, border_w);
		}
		if t_clamped > 0.02 {
			track_shape = track_shape.noise(0.15 * t_clamped);
		}
		canvas.draw(track_shape);

		let inactive_thumb_x = track_x + 6.0;
		let active_thumb_x = track_x + switch_w - 18.0 - 4.0;
		let thumb_x = inactive_thumb_x.interpolate(&active_thumb_x, t);

		let thumb_size = 14.0f32.interpolate(&18.0f32, t_clamped);
		let thumb_y = track_y + (switch_h - thumb_size) * 0.5;
		let thumb_radius = thumb_size * 0.5;

		let thumb_inactive_color = monet[MonetRole::Primary];
		let thumb_active_color = monet[MonetRole::SurfaceContainerLow];
		let thumb_color = thumb_inactive_color.interpolate(&thumb_active_color, t_clamped);

		canvas.draw(
			RectShape::new(Rect::new(thumb_x, thumb_y, thumb_size, thumb_size), thumb_color)
				.radius(thumb_radius)
				.noise(0.15 * (1.0 - t_clamped)),
		);

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
				.unwrap_or_else(|| monet[MonetRole::OnSurface]);
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
			let target = if self.checked { 1.0 } else { 0.0 };
			self.progress.set_target_spring(
				target,
				Spring::INTERACTIVE,
			);
			if let Some(cb) = &mut self.on_changed {
				cb();
			}
			ctx.request_redraw();
			return EventResponse::Handled;
		}
		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"checked" => {
				let b = value.expect_bool()?;
				self.checked = b;
				self.progress.snap_to(if b { 1.0 } else { 0.0 });
			}
			"text" => self.label = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"color" => self.color = Some(value.expect_color(&ctx.theme.monet)?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
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
