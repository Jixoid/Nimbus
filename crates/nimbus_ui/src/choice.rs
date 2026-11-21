/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::{Cell, RefCell};
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, ButtonState, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct Choice {
	pub tabs: Vec<String>,
	pub tab_id: i32,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_changed: Option<MethodCallback>,

	indicator: RefCell<Animated<Rect<f32>>>,
	hovered_tab: Option<usize>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for Choice {
	fn default() -> Self {
		Self {
			tabs: Vec::new(),
			tab_id: 0,
			font: None,
			font_size: 14.0,
			on_changed: None,
			indicator: RefCell::new(Animated::new(Rect::ZERO)),
			hovered_tab: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Choice {
	fn tab_rect(&self, bounds: Rect<f32>, index: usize) -> Option<Rect<f32>> {
		let fface = FontManager::default_font();
		let mut x_off = bounds.x() - 10.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			x_off += 16.0;
			let (tab_w, _) = fface.measure_text(tab, self.font_size);
			if i == index {
				return Some(Rect::new(
					x_off - 6.0,
					bounds.y(),
					tab_w + 12.0,
					bounds.height(),
				));
			}
			x_off += tab_w;
		}
		None
	}

	fn tab_at_x(&self, bounds: Rect<f32>, x: f32) -> Option<usize> {
		let fface = FontManager::default_font();
		let mut x_off = bounds.x() - 10.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			x_off += 16.0;
			let (tab_w, _) = fface.measure_text(tab, self.font_size);
			let tab_start = x_off - 6.0;
			let tab_end = x_off + tab_w + 6.0;

			if x >= tab_start && x <= tab_end {
				return Some(i);
			}

			x_off += tab_w;
		}
		None
	}
}

impl Widget for Choice {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let mut total_w = -3.0f32;
		for tab in &self.tabs {
			let (w, _) = ctx.measure_text_with_font(tab, self.font, self.font_size);
			total_w += w + 16.0;
		}
		let height = 28.0;
		constraints.constrain(Vec2::new(total_w.max(0.0).ceil(), height))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let theme = ctx.theme;
		let monet = &theme.monet;
		let bounds = ctx.bounds;

		let track_radius = bounds.height() * 0.5;

		canvas.draw(
			RectShape::new(bounds, monet[MonetRole::SurfaceContainerHigh])
				.radius(track_radius),
		);

		// Animate indicator pill sliding/morphing
		let cur_pill = self.indicator.borrow().value();
		if self.indicator.borrow().is_animating() {
			ctx.request_redraw();
		}

		if cur_pill != Rect::ZERO {
			let pill_radius = bounds.height() * 0.5;
			canvas.draw(
				RectShape::new(cur_pill, monet[MonetRole::Primary])
					.radius(pill_radius)
					.noise(0.15),
			);
		} else if let Some(target_rect) = self.tab_rect(bounds, self.tab_id as usize) {
			self.indicator.borrow_mut().snap_to(target_rect);
			let pill_radius = bounds.height() * 0.5;
			canvas.draw(
				RectShape::new(target_rect, monet[MonetRole::Primary])
					.radius(pill_radius)
					.noise(0.15),
			);
		}

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let mut x_off = bounds.x() - 10.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			x_off += 16.0;
			let (tab_w, tab_h) = canvas.measure_text(tab, self.font_size);
			let is_selected = self.tab_id == i as i32;

			let text_color = if is_selected {
				monet[MonetRole::OnPrimary]
			} else {
				monet[MonetRole::OnSurface]
			};

			let text_y = bounds.y() + (bounds.height() - tab_h) * 0.5;
			canvas.draw(TextShape::new(
				tab,
				font,
				self.font_size,
				Vec2::new(x_off, text_y),
				text_color,
			));

			x_off += tab_w;
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				let bounds = self.last_bounds.get();
				if let Some(tab_idx) = self.tab_at_x(bounds, position.x) {
					if self.tab_id != tab_idx as i32 {
						self.tab_id = tab_idx as i32;
						if let Some(target_rect) = self.tab_rect(bounds, tab_idx) {
							let mut ind = self.indicator.borrow_mut();
							if *ind.target() == Rect::ZERO {
								ind.snap_to(target_rect);
							} else {
								ind.set_target(
									target_rect,
									Duration::from_millis(180),
									Easing::Emphasized,
								);
							}
						}
						if let Some(cb) = &mut self.on_changed {
							cb();
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
				EventResponse::Ignored
			}

			InputEvent::PointerMoved { position } => {
				let bounds = self.last_bounds.get();
				let hovered = self.tab_at_x(bounds, position.x);
				if hovered != self.hovered_tab {
					self.hovered_tab = hovered;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(&mut self, name: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> bool {
		match name {
			"tabs" => {
				if let Some(arr) = value.as_array() {
					let mut list = Vec::new();
					for item in arr {
						if let Some(s) = item.as_str() {
							list.push(s.to_string());
						}
					}
					self.tabs = list;
					return true;
				}
			}

			"tab_id" => {
				if let Some(n) = value.as_f32() {
					let new_id = n as i32;
					if self.tab_id != new_id {
						self.tab_id = new_id;
						let bounds = self.last_bounds.get();
						if bounds != Rect::ZERO {
							if let Some(target_rect) = self.tab_rect(bounds, new_id as usize) {
								self.indicator.borrow_mut().snap_to(target_rect);
							}
						}
					}
					return true;
				}
			}

			"font_size" => {
				if let Some(n) = value.as_f32() {
					self.font_size = n;
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
		if method_name == "on_changed" {
			self.on_changed = Some(callback);
			true
		} else {
			false
		}
	}
}
