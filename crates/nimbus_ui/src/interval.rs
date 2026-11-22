/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::event::{ButtonState, MouseButton};
use nimbus_core::{BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx, MethodCallback, PropertyCtx, PropertyValue, Widget};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::MonetRole;


pub struct Interval {
	pub pos1: f32,
	pub pos2: f32,
	pub min: f32,
	pub max: f32,
	pub step: Option<f32>,
	pub active_handle: Option<u8>,
	pub on_changed: Option<MethodCallback>,
	last_bounds: Cell<Rect<f32>>,
}

impl Interval {
	fn calculate_value_from_pos(&self, pos_x: f32) -> f32 {
		let bounds = self.last_bounds.get();
		if bounds.width() <= 0.0 {
			return self.min;
		}

		let ratio = ((pos_x - bounds.x()) / bounds.width()).clamp(0.0, 1.0);
		let mut raw = self.min + ratio * (self.max - self.min);

		if let Some(s) = self.step {
			if s > 0.0 {
				raw = (raw / s).round() * s;
			}
		}

		raw.clamp(self.min, self.max)
	}
}

impl Default for Interval {
	fn default() -> Self {
		Self {
			pos1: 30.0,
			pos2: 70.0,
			min: 0.0,
			max: 100.0,
			step: None,
			active_handle: None,
			on_changed: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Widget for Interval {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let width = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			160.0
		};
		constraints.constrain(Vec2::new(width, 28.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let w = bounds.width();
		let h = bounds.height();

		let is_hover = ctx.hovered || self.active_handle.is_some();
		let handle_half_w = if is_hover { 3.0 } else { 2.0 };
		let gap = if is_hover { 7.0 } else { 6.0 };
		let track_padding = 4.0f32;

		let range = (self.max - self.min).max(0.001);
		let usable_w = w - 4.0;
		let p1 = self.pos1.min(self.pos2);
		let p2 = self.pos1.max(self.pos2);
		let pos1 = (usable_w / range * (p1 - self.min)) + 2.0;
		let pos2 = (usable_w / range * (p2 - self.min)) + 2.0;

		let track_h = (h - track_padding * 2.0).max(0.0);
		let empty_color = monet[MonetRole::SurfaceContainerHighest];

		// Sol Boş Kısım (Start -> Pos1)
		let left_w = (pos1 - gap - 1.0).max(0.0);
		if left_w > 0.0 {
			let left_rect = Rect::new(bounds.x() + 1.0, bounds.y() + track_padding, left_w, track_h);
			canvas.draw(RectShape::new(left_rect, empty_color).radius4([14.0, 4.0, 4.0, 14.0]));
		}

		// Orta Dolu Kısım (Pos1 -> Pos2)
		let mid_start = pos1 + gap;
		let mid_w = (pos2 - gap - mid_start).max(0.0);
		if mid_w > 0.0 {
			let mid_rect = Rect::new(bounds.x() + mid_start, bounds.y() + track_padding, mid_w, track_h);
			canvas.draw(
				RectShape::new(mid_rect, monet[MonetRole::Secondary])
					.radius(4.0)
					.noise(0.15),
			);
		}

		// Sağ Boş Kısım (Pos2 -> End)
		let right_start = pos2 + gap;
		let right_w = (w - 1.0 - right_start).max(0.0);
		if right_w > 0.0 {
			let right_rect = Rect::new(bounds.x() + right_start, bounds.y() + track_padding, right_w, track_h);
			canvas.draw(RectShape::new(right_rect, empty_color).radius4([4.0, 14.0, 14.0, 4.0]));
		}

		// Handles (Pos1 ve Pos2)
		let handle_h = (h - 2.0).max(0.0);
		let h1_color = if self.active_handle == Some(1) {
			monet[MonetRole::PrimaryFixedDim]
		} else {
			monet[MonetRole::Primary]
		};
		let h2_color = if self.active_handle == Some(2) {
			monet[MonetRole::PrimaryFixedDim]
		} else {
			monet[MonetRole::Primary]
		};

		let h1_rect = Rect::new(
			bounds.x() + pos1 - handle_half_w,
			bounds.y() + 1.0,
			handle_half_w * 2.0,
			handle_h,
		);
		canvas.draw(RectShape::new(h1_rect, h1_color).radius(4.0));

		let h2_rect = Rect::new(
			bounds.x() + pos2 - handle_half_w,
			bounds.y() + 1.0,
			handle_half_w * 2.0,
			handle_h,
		);
		canvas.draw(RectShape::new(h2_rect, h2_color).radius(4.0));
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					let val = self.calculate_value_from_pos(position.x);
					let dist1 = (val - self.pos1).abs();
					let dist2 = (val - self.pos2).abs();

					if dist1 <= dist2 {
						self.active_handle = Some(1);
						self.pos1 = val.min(self.pos2);
					} else {
						self.active_handle = Some(2);
						self.pos2 = val.max(self.pos1);
					}

					if let Some(cb) = &mut self.on_changed {
						cb.call2(self.pos1, self.pos2);
					}
					ctx.request_redraw();
					EventResponse::Handled
				}
				ButtonState::Released => {
					if self.active_handle.is_some() {
						self.active_handle = None;
						ctx.request_redraw();
						EventResponse::Handled
					} else {
						EventResponse::Ignored
					}
				}
			},
			InputEvent::PointerMoved { position } => {
				if let Some(handle) = self.active_handle {
					let val = self.calculate_value_from_pos(position.x);
					if handle == 1 {
						self.pos1 = val.min(self.pos2);
					} else {
						self.pos2 = val.max(self.pos1);
					}
					if let Some(cb) = &mut self.on_changed {
						cb.call2(self.pos1, self.pos2);
					}
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}
			InputEvent::Click { position } => {
				let val = self.calculate_value_from_pos(position.x);
				let dist1 = (val - self.pos1).abs();
				let dist2 = (val - self.pos2).abs();

				if dist1 <= dist2 {
					self.pos1 = val.min(self.pos2);
				} else {
					self.pos2 = val.max(self.pos1);
				}

				if let Some(cb) = &mut self.on_changed {
					cb.call2(self.pos1, self.pos2);
				}
				ctx.request_redraw();
				EventResponse::Handled
			}
			_ => EventResponse::Ignored,
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"pos1" => self.pos1 = value.expect_f32()?.clamp(self.min, self.pos2),
			"pos2" => self.pos2 = value.expect_f32()?.clamp(self.pos1, self.max),
			"min" => self.min = value.expect_f32()?,
			"max" => self.max = value.expect_f32()?,
			"step" => self.step = Some(value.expect_f32()?),
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
