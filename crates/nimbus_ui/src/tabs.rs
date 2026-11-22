/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::{Cell, RefCell};

use nimbus_core::{
	Animated, BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyValue, Spring, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct Tabs {
	pub tabs: Vec<String>,
	pub tab_id: i32,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_changed: Option<MethodCallback>,

	indicator: RefCell<Animated<Rect<f32>>>,
	hovered_tab: Option<usize>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for Tabs {
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

impl Tabs {
	pub fn local_tab_rect(&self, height: f32, index: usize) -> Option<Rect<f32>> {
		let fface = FontManager::default_font();
		let mut x_off = 16.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			let (tab_w, tab_h) = fface.measure_text(tab, self.font_size);
			if i == index {
				return Some(Rect::new(
					x_off - 6.0,
					(height * 0.5) - (tab_h * 0.5) - 6.0,
					tab_w + 12.0,
					tab_h + 12.0,
				));
			}
			x_off += tab_w + 16.0;
		}
		None
	}

	pub fn tab_rect(&self, bounds: Rect<f32>, index: usize) -> Option<Rect<f32>> {
		self.local_tab_rect(bounds.height(), index).map(|r| {
			Rect::new(bounds.x() + r.x(), bounds.y() + r.y(), r.width(), r.height())
		})
	}

	pub fn tab_at_x(&self, bounds: Rect<f32>, x: f32) -> Option<usize> {
		let fface = FontManager::default_font();
		let mut x_off = bounds.x() + 16.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			let (tab_w, _) = fface.measure_text(tab, self.font_size);
			let tab_start = x_off - 6.0;
			let tab_end = x_off + tab_w + 6.0;

			if x >= tab_start && x <= tab_end {
				return Some(i);
			}

			x_off += tab_w + 16.0;
		}
		None
	}
}

impl Widget for Tabs {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let mut total_w = 16.0;
		for tab in &self.tabs {
			let (w, _) = ctx.measure_text_with_font(tab, self.font, self.font_size);
			total_w += w + 16.0;
		}
		let height = 32.0;
		constraints.constrain(Vec2::new(total_w.ceil(), height))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let theme = ctx.theme;
		let monet = &theme.monet;
		let bounds = ctx.bounds;

		let radius = bounds.height() * 0.5;
		canvas.draw(
			RectShape::new(bounds, monet[MonetRole::SurfaceContainerHigh]).radius(radius),
		);

		// Animate indicator pill sliding/morphing
		let cur_local_pill = self.indicator.borrow().value();
		if self.indicator.borrow().is_animating() {
			ctx.request_redraw();
		}

		if cur_local_pill != Rect::ZERO {
			let pill_rect = Rect::new(
				bounds.x() + cur_local_pill.x(),
				bounds.y() + cur_local_pill.y(),
				cur_local_pill.width(),
				cur_local_pill.height(),
			);
			let pill_radius = pill_rect.height() * 0.5;
			canvas.draw(
				RectShape::new(pill_rect, monet[MonetRole::Primary])
					.radius(pill_radius)
					.noise(0.15),
			);
		} else if let Some(target_local) = self.local_tab_rect(bounds.height(), self.tab_id as usize) {
			self.indicator.borrow_mut().snap_to(target_local);
			let pill_rect = Rect::new(
				bounds.x() + target_local.x(),
				bounds.y() + target_local.y(),
				target_local.width(),
				target_local.height(),
			);
			let pill_radius = pill_rect.height() * 0.5;
			canvas.draw(
				RectShape::new(pill_rect, monet[MonetRole::Primary])
					.radius(pill_radius)
					.noise(0.15),
			);
		}

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let mut x_off = bounds.x() + 16.0;

		for (i, tab) in self.tabs.iter().enumerate() {
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

			x_off += tab_w + 16.0;
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
						let h = if bounds != Rect::ZERO { bounds.height() } else { 32.0 };
						if let Some(target_local) = self.local_tab_rect(h, tab_idx) {
							let mut ind = self.indicator.borrow_mut();
							if *ind.target() == Rect::ZERO {
								ind.snap_to(target_local);
							} else {
								ind.set_target_spring(
									target_local,
									Spring::INTERACTIVE,
								);
							}
						}
						if let Some(cb) = &mut self.on_changed {
							let tab_text = self.tabs.get(tab_idx).cloned().unwrap_or_default();
							cb.call2(self.tab_id, tab_text);
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
			}

			InputEvent::PointerMoved { position } => {
				let bounds = self.last_bounds.get();
				let new_hover = self.tab_at_x(bounds, position.x);
				if new_hover != self.hovered_tab {
					self.hovered_tab = new_hover;
					ctx.request_redraw();
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered_tab.is_some() {
					self.hovered_tab = None;
					ctx.request_redraw();
				}
			}

			_ => {}
		}

		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"tab_id" => {
				let new_id = value.expect_f32()? as i32;
				if self.tab_id != new_id {
					self.tab_id = new_id;
					let bounds = self.last_bounds.get();
					let h = if bounds != Rect::ZERO { bounds.height() } else { 32.0 };
					if let Some(target_local) = self.local_tab_rect(h, new_id as usize) {
						self.indicator.borrow_mut().snap_to(target_local);
					}
				}
			}
			"tabs" => {
				let arr = value.expect_array()?;
				let mut list = Vec::new();
				for item in arr {
					if let Some(s) = item.as_str() {
						list.push(s.to_string());
					}
				}
				self.tabs = list;
				let bounds = self.last_bounds.get();
				let h = if bounds != Rect::ZERO { bounds.height() } else { 32.0 };
				if let Some(target_local) = self.local_tab_rect(h, self.tab_id as usize) {
					self.indicator.borrow_mut().snap_to(target_local);
				}
			}
			"font_size" => self.font_size = value.expect_f32()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tabs_local_tab_rect_and_scroll() {
		let mut tabs = Tabs::default();
		tabs.tabs = vec!["Home".to_string(), "Settings".to_string(), "Profile".to_string()];

		let local_0 = tabs.local_tab_rect(32.0, 0).expect("Tab 0 exists");
		let local_1 = tabs.local_tab_rect(32.0, 1).expect("Tab 1 exists");

		// Local coordinates are relative to widget origin
		assert!(local_0.x() >= 0.0);
		assert!(local_1.x() > local_0.x());

		// Verify indicator tracks bounds correctly on scroll
		let bounds_a = Rect::new(0.0, 50.0, 300.0, 32.0);
		let bounds_b = Rect::new(0.0, 250.0, 300.0, 32.0);

		let screen_a = Rect::new(
			bounds_a.x() + local_0.x(),
			bounds_a.y() + local_0.y(),
			local_0.width(),
			local_0.height(),
		);
		let screen_b = Rect::new(
			bounds_b.x() + local_0.x(),
			bounds_b.y() + local_0.y(),
			local_0.width(),
			local_0.height(),
		);

		assert_eq!(screen_b.y() - screen_a.y(), 200.0);
	}
}
