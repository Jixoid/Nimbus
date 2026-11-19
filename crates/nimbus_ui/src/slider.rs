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

		let theme = ctx.theme;
		let bounds = ctx.bounds;
		let track_height = 8.0f32;
		let track_y = bounds.y() + (bounds.height() - track_height) * 0.5;

		let range = (self.max - self.min).max(0.001);
		let ratio = ((self.value - self.min) / range).clamp(0.0, 1.0);

		let handle_w = 12.0f32;
		let handle_h = 20.0f32;
		let handle_x = bounds.x() + ratio * (bounds.width() - handle_w);
		let handle_y = bounds.y() + (bounds.height() - handle_h) * 0.5;

		// Inactive Background Track
		let full_track = Rect::new(bounds.x(), track_y, bounds.width(), track_height);
		canvas.draw(
			RectShape::new(full_track, theme.monet.get(MonetRole::SurfaceContainerHighest))
				.radius(track_height * 0.5),
		);

		// Active Filled Track (Left side)
		let active_w = handle_x - bounds.x() + handle_w * 0.5;
		if active_w > 0.0 {
			let active_track = Rect::new(bounds.x(), track_y, active_w, track_height);
			let fill_color = if self.is_dragging || ctx.hovered {
				theme.monet.get(MonetRole::PrimaryContainer)
			} else {
				theme.monet.get(MonetRole::Primary)
			};
			canvas.draw(RectShape::new(active_track, fill_color).radius(track_height * 0.5));
		}

		// Thumb Handle
		let handle_rect = Rect::new(handle_x, handle_y, handle_w, handle_h);
		let handle_color = if self.is_dragging {
			theme.monet.get(MonetRole::PrimaryFixedDim)
		} else if ctx.hovered {
			theme.monet.get(MonetRole::PrimaryContainer)
		} else {
			theme.monet.get(MonetRole::Primary)
		};
		canvas.draw(
			RectShape::new(handle_rect, handle_color)
				.radius(theme.radius.xsmall)
				.border(theme.monet.get(MonetRole::Outline), theme.stroke.thin),
		);
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
