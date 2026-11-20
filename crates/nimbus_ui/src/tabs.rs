/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx,
	MethodCallback, MouseButton, PropertyCtx, PropertyValue, Widget,
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
			hovered_tab: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Tabs {
	fn tab_at_x(&self, bounds: Rect<f32>, x: f32) -> Option<usize> {
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

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let mut x_off = bounds.x() + 16.0;

		for (i, tab) in self.tabs.iter().enumerate() {
			let (tab_w, tab_h) = canvas.measure_text(tab, self.font_size);
			let is_selected = self.tab_id == i as i32;

			if is_selected {
				let tab_rect = Rect::new(
					x_off - 6.0,
					bounds.y() + (bounds.height() * 0.5) - (tab_h * 0.5) - 6.0,
					tab_w + 12.0,
					tab_h + 12.0,
				);
				let tab_radius = tab_rect.height() * 0.5;
				canvas.draw(
					RectShape::new(tab_rect, monet[MonetRole::PrimaryContainer])
						.radius(tab_radius),
				);
			}

			let text_color = if is_selected {
				monet[MonetRole::OnPrimaryContainer]
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
					self.tabs = arr
						.iter()
						.filter_map(|v| v.as_str().map(|s| s.to_string()))
						.collect();
					return true;
				}
			}

			"tab_id" => {
				if let Some(n) = value.as_f32() {
					self.tab_id = n as i32;
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


#[cfg(test)]
mod tests {
	use super::*;
	use nimbus_core::ObjectID;
	use nimbus_theme::Theme;
	use std::sync::atomic::{AtomicBool, Ordering};
	use std::sync::Arc;

	fn make_theme() -> Theme {
		Theme {
			monet: nimbus_theme::Monet::new(
				nimbus_math::Color::WHITE,
				nimbus_math::Color::WHITE,
				nimbus_math::Color::WHITE,
				true,
			),
			radius: nimbus_theme::RadiusTokens::default(),
			elevation: nimbus_theme::ElevationTokens::default(),
			state: nimbus_theme::StateTokens::default(),
			stroke: nimbus_theme::StrokeTokens::default(),
		}
	}

	#[test]
	fn test_tabs_default_and_properties() {
		let mut tabs = Tabs::default();
		assert_eq!(tabs.tab_id, 0);
		assert_eq!(tabs.tabs.len(), 0);

		let theme = make_theme();
		let ctx = PropertyCtx { theme: &theme };

		let arr = PropertyValue::Array(vec![
			PropertyValue::String("Tab1".to_string()),
			PropertyValue::String("Tab2".to_string()),
			PropertyValue::String("Tab3".to_string()),
		]);
		assert!(tabs.set_property("tabs", &arr, &ctx));
		assert_eq!(tabs.tabs, vec!["Tab1", "Tab2", "Tab3"]);

		assert!(tabs.set_property("tab_id", &PropertyValue::Number(2.0), &ctx));
		assert_eq!(tabs.tab_id, 2);
	}

	#[test]
	fn test_tabs_click_changes_selection() {
		let mut tabs = Tabs::default();
		tabs.tabs = vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()];
		tabs.tab_id = 0;
		tabs.last_bounds.set(Rect::new(0.0, 0.0, 200.0, 32.0));

		let changed = Arc::new(AtomicBool::new(false));
		let changed_flag = Arc::clone(&changed);
		tabs.bind_method("on_changed", Box::new(move || {
			changed_flag.store(true, Ordering::SeqCst);
		}));

		let mut redraw = false;
		let mut layout = false;
		let mut ctx = EventCtx::new(ObjectID::default(), &mut redraw, &mut layout);

		// Click roughly around Tab2
		let ev = InputEvent::PointerButton {
			button: MouseButton::Left,
			state: ButtonState::Pressed,
			position: Vec2::new(70.0, 16.0),
		};

		tabs.handle_event(&mut ctx, &ev);
		assert_eq!(tabs.tab_id, 1);
		assert!(changed.load(Ordering::SeqCst));
		assert!(redraw);
	}
}
