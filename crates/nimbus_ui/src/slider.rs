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


pub struct Slider {
	pub value: f32,
	pub min: f32,
	pub max: f32,
	pub step: Option<f32>,
	pub is_dragging: bool,
	pub on_changed: Option<MethodCallback>,
	last_bounds: Cell<Rect<f32>>,
}

impl Slider {
	fn calculate_value_from_pos(&self, pos_x: f32) -> f32 {
		let bounds = self.last_bounds.get();
		if bounds.width() <= 0.0 {
			return self.value;
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

impl Default for Slider {
	fn default() -> Self {
		Self {
			value: 30.0,
			min: 0.0,
			max: 100.0,
			step: None,
			is_dragging: false,
			on_changed: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}


impl Widget for Slider {
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

		let range = (self.max - self.min).max(0.001);
		let ratio = ((self.value - self.min) / range).clamp(0.0, 1.0);
		let val_pos = ((w - 4.0) * ratio) + 2.0;

		let is_hover = ctx.hovered || self.is_dragging;
		let gap = if is_hover { 7.0 } else { 6.0 };
		let handle_half_w = if is_hover { 3.0 } else { 2.0 };

		// Filled (Left)
		let left_w = (val_pos - gap - 1.0).max(0.0);
		if left_w > 0.0 {
			let left_rect = Rect::new(bounds.x() + 1.0, bounds.y() + 4.0, left_w, (h - 8.0).max(0.0));
			canvas.draw(
				RectShape::new(left_rect, monet[MonetRole::Secondary])
					.radius4([14.0, 4.0, 4.0, 14.0])
					.noise(0.15),
			);
		}

		// Handle (Middle)
		let handle_color = if self.is_dragging {
			monet[MonetRole::PrimaryFixedDim]
		} else {
			monet[MonetRole::Primary]
		};
		let handle_rect = Rect::new(
			bounds.x() + val_pos - handle_half_w,
			bounds.y() + 1.0,
			handle_half_w * 2.0,
			(h - 2.0).max(0.0),
		);
		canvas.draw(RectShape::new(handle_rect, handle_color).radius(4.0));

		// Empty Part (Right)
		let right_start = val_pos + gap;
		let right_w = (w - 1.0 - right_start).max(0.0);
		if right_w > 0.0 {
			let right_rect = Rect::new(
				bounds.x() + right_start,
				bounds.y() + 4.0,
				right_w,
				(h - 8.0).max(0.0),
			);
			canvas.draw(
				RectShape::new(right_rect, monet[MonetRole::PrimaryOutline] *0.5)
					.radius4([4.0, 14.0, 14.0, 4.0]),
			);
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					self.is_dragging = true;
					self.value = self.calculate_value_from_pos(position.x);
					if let Some(cb) = &mut self.on_changed {
						cb();
					}
					ctx.request_redraw();
					EventResponse::Handled
				}
				ButtonState::Released => {
					if self.is_dragging {
						self.is_dragging = false;
						ctx.request_redraw();
						EventResponse::Handled
					} else {
						EventResponse::Ignored
					}
				}
			},
			InputEvent::PointerMoved { position } => {
				if self.is_dragging {
					self.value = self.calculate_value_from_pos(position.x);
					if let Some(cb) = &mut self.on_changed {
						cb();
					}
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}
			InputEvent::Click { position } => {
				self.value = self.calculate_value_from_pos(position.x);
				if let Some(cb) = &mut self.on_changed {
					cb();
				}
				ctx.request_redraw();
				EventResponse::Handled
			}
			_ => EventResponse::Ignored,
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> bool {
		match key {
			"value" => {
				if let Some(v) = value.as_f32() {
					self.value = v.clamp(self.min, self.max);
					return true;
				}
			}

			"min" => {
				if let Some(v) = value.as_f32() {
					self.min = v;
					return true;
				}
			}

			"max" => {
				if let Some(v) = value.as_f32() {
					self.max = v;
					return true;
				}
			}

			"step" => {
				if let Some(v) = value.as_f32() {
					self.step = Some(v);
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
