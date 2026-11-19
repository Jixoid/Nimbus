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

		let theme = ctx.theme;
		let bounds = ctx.bounds;
		let track_height = 8.0f32;
		let track_y = bounds.y() + (bounds.height() - track_height) * 0.5;

		let range = (self.max - self.min).max(0.001);
		let ratio1 = ((self.pos1 - self.min) / range).clamp(0.0, 1.0);
		let ratio2 = ((self.pos2 - self.min) / range).clamp(0.0, 1.0);

		let handle_w = 10.0f32;
		let handle_h = 20.0f32;
		let handle_y = bounds.y() + (bounds.height() - handle_h) * 0.5;

		let h1_x = bounds.x() + ratio1 * (bounds.width() - handle_w);
		let h2_x = bounds.x() + ratio2 * (bounds.width() - handle_w);

		// Inactive Background Track
		let full_track = Rect::new(bounds.x(), track_y, bounds.width(), track_height);
		canvas.draw(
			RectShape::new(full_track, theme.monet.get(MonetRole::SurfaceContainerHighest))
				.radius(track_height * 0.5),
		);

		// Middle Active Track
		let seg_start = (h1_x + handle_w * 0.5).min(h2_x + handle_w * 0.5);
		let seg_end = (h1_x + handle_w * 0.5).max(h2_x + handle_w * 0.5);
		let seg_w = seg_end - seg_start;
		if seg_w > 0.0 {
			let middle_track = Rect::new(seg_start, track_y, seg_w, track_height);
			canvas.draw(RectShape::new(middle_track, theme.monet.get(MonetRole::Primary)).radius(4.0));
		}

		// Handles
		let draw_handle = |canvas: &mut Canvas2D, h_x: f32, is_active: bool| {
			let h_rect = Rect::new(h_x, handle_y, handle_w, handle_h);
			let color = if is_active {
				theme.monet.get(MonetRole::PrimaryFixedDim)
			} else if ctx.hovered {
				theme.monet.get(MonetRole::PrimaryContainer)
			} else {
				theme.monet.get(MonetRole::Primary)
			};
			canvas.draw(
				RectShape::new(h_rect, color)
					.radius(theme.radius.xsmall)
					.border(theme.monet.get(MonetRole::Outline), theme.stroke.thin),
			);
		};

		draw_handle(canvas, h1_x, self.active_handle == Some(1));
		draw_handle(canvas, h2_x, self.active_handle == Some(2));
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
						cb();
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
						cb();
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
			"pos1" => {
				if let Some(v) = value.as_f32() {
					self.pos1 = v.clamp(self.min, self.pos2);
					return true;
				}
			}
			"pos2" => {
				if let Some(v) = value.as_f32() {
					self.pos2 = v.clamp(self.pos1, self.max);
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
