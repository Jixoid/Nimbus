/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{any, cell::Cell};
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	LayoutCtx, MethodCallback, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{Monet, MonetRole};


#[derive(Clone, Debug, PartialEq)]
pub struct PopupItem {
	pub text: String,
	pub shortcut: Option<String>,
	pub disabled: bool,
	pub items: Vec<PopupItem>,
}

impl Default for PopupItem {
	fn default() -> Self {
		Self {
			text: any::type_name::<Self>().to_string(),
			shortcut: None,
			disabled: false,
			items: Vec::new(),
		}
	}
}

impl PopupItem {
	pub fn separator() -> Self {
		Self {
			text: "-".to_string(),
			..Self::default()
		}
	}

	pub fn disabled(mut self) -> Self {
		self.disabled = true;
		self
	}

	pub fn has_submenu(&self) -> bool {
		!self.items.is_empty()
	}
}


pub struct Popup {
	pub title: Option<String>,
	pub items: Vec<PopupItem>,
	pub open: bool,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub min_width: f32,
	pub hovered_index: Option<usize>,
	pub hovered_header: bool,
	pub selected_index: Option<usize>,
	pub on_selected: Option<MethodCallback>,
	pub progress: Animated<f32>,

	pub nav_stack: Vec<usize>,
	prev_stack: Vec<usize>,
	slide_progress: Animated<f32>,
	slide_dir: f32,

	last_bounds: Cell<Rect<f32>>,
}

impl Default for Popup {
	fn default() -> Self {
		Self {
			title: None,
			items: Vec::new(),
			open: true,
			font: None,
			font_size: 14.0,
			min_width: 160.0,
			hovered_index: None,
			hovered_header: false,
			selected_index: None,
			on_selected: None,
			progress: Animated::new(1.0),
			nav_stack: Vec::new(),
			prev_stack: Vec::new(),
			slide_progress: Animated::new(1.0),
			slide_dir: 1.0,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Popup {
	pub fn set_open(&mut self, open: bool) {
		self.open = open;
		let target = if open { 1.0 } else { 0.0 };
		self.progress.set_target(
			target,
			Duration::from_millis(150),
			Easing::Emphasized,
		);
	}

	pub fn current_items(&self) -> &[PopupItem] {
		Self::items_at_stack(&self.items, &self.nav_stack)
	}

	pub fn previous_items(&self) -> &[PopupItem] {
		Self::items_at_stack(&self.items, &self.prev_stack)
	}

	pub fn items_at_stack<'a>(root: &'a [PopupItem], stack: &[usize]) -> &'a [PopupItem] {
		let mut curr = root;
		for &idx in stack {
			if let Some(item) = curr.get(idx) {
				curr = &item.items;
			} else {
				break;
			}
		}
		curr
	}

	pub fn current_title(&self) -> Option<&str> {
		Self::title_at_stack(&self.items, &self.nav_stack, self.title.as_deref())
	}

	pub fn previous_title(&self) -> Option<&str> {
		Self::title_at_stack(&self.items, &self.prev_stack, self.title.as_deref())
	}

	pub fn title_at_stack<'a>(
		root: &'a [PopupItem],
		stack: &[usize],
		root_title: Option<&'a str>,
	) -> Option<&'a str> {
		if stack.is_empty() {
			root_title
		} else {
			let mut curr = root;
			let mut last_title = None;
			for &idx in stack {
				if let Some(item) = curr.get(idx) {
					last_title = Some(item.text.as_str());
					curr = &item.items;
				}
			}
			last_title
		}
	}

	pub fn navigate_to(&mut self, index: usize, ctx: &mut EventCtx) {
		let curr = self.current_items();
		if let Some(item) = curr.get(index) {
			if item.has_submenu() {
				self.prev_stack = self.nav_stack.clone();
				self.nav_stack.push(index);
				self.hovered_index = None;
				self.hovered_header = false;
				self.slide_dir = 1.0;
				self.slide_progress.snap_to(0.0);
				self.slide_progress.set_target(
					1.0,
					Duration::from_millis(180),
					Easing::Emphasized,
				);
				ctx.request_redraw();
				ctx.request_layout();
			}
		}
	}

	pub fn navigate_back(&mut self, ctx: &mut EventCtx) {
		if !self.nav_stack.is_empty() {
			self.prev_stack = self.nav_stack.clone();
			self.nav_stack.pop();
			self.hovered_index = None;
			self.hovered_header = false;
			self.slide_dir = -1.0;
			self.slide_progress.snap_to(0.0);
			self.slide_progress.set_target(
				1.0,
				Duration::from_millis(180),
				Easing::Emphasized,
			);
			ctx.request_redraw();
			ctx.request_layout();
		}
	}


	fn item_row_height(&self) -> f32 {
		(self.font_size * 2.2).max(32.0)
	}

	fn header_height(&self, has_title: bool) -> f32 {
		if has_title {
			34.0
		} else {
			0.0
		}
	}

	fn calculate_row_rects(&self, bounds: Rect<f32>, items: &[PopupItem], has_title: bool) -> Vec<(Rect<f32>, bool)> {
		let mut rows = Vec::with_capacity(items.len());
		let item_height = self.item_row_height();
		let mut curr_y = bounds.y() + self.header_height(has_title) + 4.0;

		for item in items {
			if item.text == "-" {
				let sep_rect = Rect::new(bounds.x() + 8.0, curr_y + 4.0, bounds.width() - 16.0, 1.0);
				rows.push((sep_rect, true));
				curr_y += 9.0;
			} else {
				let row_rect = Rect::new(bounds.x() + 4.0, curr_y, bounds.width() - 8.0, item_height);
				rows.push((row_rect, false));
				curr_y += item_height + 2.0;
			}
		}

		rows
	}

	fn item_at_pos(&self, pos: Vec2<f32>) -> Option<usize> {
		let bounds = self.last_bounds.get();
		if !bounds.contains_point(pos) {
			return None;
		}

		let items = self.current_items();
		let has_title = self.current_title().is_some();
		let rows = self.calculate_row_rects(bounds, items, has_title);
		for (i, (rect, is_sep)) in rows.into_iter().enumerate() {
			if !is_sep && rect.contains_point(pos) {
				return Some(i);
			}
		}
		None
	}

	#[allow(clippy::too_many_arguments)]
	fn draw_menu_page(
		&self,
		canvas: &mut Canvas2D,
		monet: &Monet,
		font: Font,
		fface: &nimbus_render2d::FontFace,
		bounds: Rect<f32>,
		offset_x: f32,
		title: Option<&str>,
		is_sub_level: bool,
		items: &[PopupItem],
		hovered_idx: Option<usize>,
		hovered_header: bool,
	) {
		let page_bounds = Rect::new(bounds.x() + offset_x, bounds.y(), bounds.width(), bounds.height());

		// Header Bar
		if let Some(t) = title {
			let header_h = self.header_height(true);
			let header_rect = Rect::new(page_bounds.x()+1.0, page_bounds.y()+1.0, page_bounds.width()-2.0, header_h);

			canvas.draw(
				if is_sub_level && hovered_header {
					RectShape::new(header_rect, monet[MonetRole::Primary])
						.radius4([13.0, 13.0, 0.0, 0.0])
						.noise(0.10)
				} else {
					RectShape::new(header_rect, monet[MonetRole::SurfaceContainer])
						.radius4([13.0, 13.0, 0.0, 0.0])	
				}
			);

			if is_sub_level {
				let back_color = if hovered_header {
					monet[MonetRole::OnPrimary]
				} else {
					monet[MonetRole::OnSurface]
				};
				draw_chevron_left(
					canvas,
					header_rect.x() + 14.0,
					header_rect.center().y,
					4.0,
					1.3,
					back_color,
				);
			}

			// Centered Title
			let title_color = if is_sub_level && hovered_header {
				monet[MonetRole::OnPrimary]
			} else {
				monet[MonetRole::OnSurface]
			};

			let (title_w, _) = fface.measure_text(t, self.font_size);
			let title_pos = Vec2::new(
				header_rect.center().x - title_w * 0.5,
				header_rect.center().y - self.font_size * 0.5,
			);
			canvas.draw(TextShape::new(
				t,
				font,
				self.font_size,
				title_pos,
				title_color,
			));
		}

		// Items
		let rows = self.calculate_row_rects(page_bounds, items, title.is_some());

		for (i, item) in items.iter().enumerate() {
			if i >= rows.len() {
				break;
			}
			let (row_rect, is_sep) = rows[i];

			if is_sep {
				canvas.draw(RectShape::new(row_rect, monet[MonetRole::SurfaceOutlineVariant]));
				continue;
			}

			let is_hovered = Some(i) == hovered_idx && !item.disabled;

			if is_hovered {
				canvas.draw(
					RectShape::new(row_rect, monet[MonetRole::Primary])
						.radius(10.0)
						.noise(0.10),
				);
			}

			// Colors
			let text_color = if item.disabled {
				let mut c = monet[MonetRole::OnSurface];
				c.a = 0.38;
				c
			} else if is_hovered {
				monet[MonetRole::OnPrimary]
			} else {
				monet[MonetRole::OnSurface]
			};

			let secondary_color = if item.disabled {
				let mut c = monet[MonetRole::OnSurface];
				c.a = 0.25;
				c
			} else if is_hovered {
				let mut c = monet[MonetRole::OnPrimary];
				c.a = 0.8;
				c
			} else {
				monet[MonetRole::OnSurfaceVariant]
			};

			// Text
			let text_pos = Vec2::new(
				row_rect.x() + 12.0,
				row_rect.center().y - self.font_size * 0.5,
			);
			canvas.draw(TextShape::new(
				&item.text,
				font,
				self.font_size,
				text_pos,
				text_color,
			));

			// Shortcut
			if let Some(shortcut) = &item.shortcut {
				let sc_size = self.font_size * 0.85;
				let sc_font = Font::new(fface, sc_size);
				let (sc_w, _) = fface.measure_text(shortcut, sc_size);
				let right_margin = if item.has_submenu() { 26.0 } else { 12.0 };
				let sc_pos = Vec2::new(
					row_rect.max.x - right_margin - sc_w,
					row_rect.center().y - sc_size * 0.5,
				);
				canvas.draw(TextShape::new(
					shortcut,
					sc_font,
					sc_size,
					sc_pos,
					secondary_color,
				));
			}

			if item.has_submenu() {
				draw_chevron_right(
					canvas,
					row_rect.max.x - 14.0,
					row_rect.center().y,
					4.0,
					1.3,
					secondary_color,
				);
			}
		}
	}
}


fn draw_chevron_right(canvas: &mut Canvas2D, tip_x: f32, cy: f32, size: f32, thickness: f32, color: Color) {
	let steps = (size / 0.5).ceil() as usize;
	for i in 0..=steps {
		let d = (i as f32) * (size / steps as f32);
		let x = tip_x - size + d;
		let y_top = cy - size + d;
		let y_bot = cy + size - d;
		canvas.draw(RectShape::new(Rect::new(x, y_top - thickness * 0.5, thickness, thickness), color));
		canvas.draw(RectShape::new(Rect::new(x, y_bot - thickness * 0.5, thickness, thickness), color));
	}
}

fn draw_chevron_left(canvas: &mut Canvas2D, tip_x: f32, cy: f32, size: f32, thickness: f32, color: Color) {
	let steps = (size / 0.5).ceil() as usize;
	for i in 0..=steps {
		let d = (i as f32) * (size / steps as f32);
		let x = tip_x + size - d;
		let y_top = cy - size + d;
		let y_bot = cy + size - d;
		canvas.draw(RectShape::new(Rect::new(x, y_top - thickness * 0.5, thickness, thickness), color));
		canvas.draw(RectShape::new(Rect::new(x, y_bot - thickness * 0.5, thickness, thickness), color));
	}
}

impl Widget for Popup {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let fface = FontManager::default_font();
		let mut max_item_w = self.min_width;

		let items = self.current_items();
		let title = self.current_title();

		if let Some(t) = title {
			let (title_w, _) = fface.measure_text(t, self.font_size);
			max_item_w = max_item_w.max(title_w + 54.0);
		}

		for item in items {
			if item.text != "-" {
				let (text_w, _) = fface.measure_text(&item.text, self.font_size);
				let mut w = text_w + 32.0;
				if let Some(sc) = &item.shortcut {
					let (sc_w, _) = fface.measure_text(sc, self.font_size * 0.85);
					w += sc_w + 16.0;
				}
				if item.has_submenu() {
					w += 22.0;
				}
				max_item_w = max_item_w.max(w);
			}
		}

		let item_height = self.item_row_height();
		let mut total_h = self.header_height(title.is_some()) + 6.0;

		for item in items {
			if item.text == "-" {
				total_h += 9.0;
			} else {
				total_h += item_height + 2.0;
			}
		}

		constraints.constrain(Vec2::new(max_item_w.ceil(), total_h.ceil()))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let t = self.progress.value();
		if t < 0.01 {
			return;
		}

		if self.progress.is_animating() || self.slide_progress.is_animating() {
			ctx.request_redraw();
		}

		let bounds = ctx.bounds;
		self.last_bounds.set(bounds);

		let monet = &ctx.theme.monet;
		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		// Shadow & Background Container
		let shadow_color = Color::new(0.0, 0.0, 0.0, 0.35 * t);
		let bg_color = monet[MonetRole::SurfaceContainerHigh];
		let border_color = monet[MonetRole::SurfaceOutlineVariant];

		canvas.draw(
			RectShape::new(bounds, bg_color)
				.radius(13.0)
				.shadow(shadow_color, Vec2::new(0.0, 4.0), 12.0)
				.border(border_color, 1.0)
				.noise(0.03),
		);

		// Animated Sliding Content
		let slide_t = self.slide_progress.value();
		let is_sliding = (self.slide_progress.is_animating() || slide_t < 0.999)
			&& self.prev_stack != self.nav_stack;

		canvas.push_clip(bounds);

		if is_sliding {
			// Outgoing Page
			let out_offset = if self.slide_dir > 0.0 {
				-slide_t * bounds.width() * 0.5
			} else {
				slide_t * bounds.width()
			};

			let prev_items = self.previous_items();
			let prev_title = self.previous_title();
			let prev_sub = !self.prev_stack.is_empty();

			self.draw_menu_page(
				canvas,
				monet,
				font,
				fface,
				bounds,
				out_offset,
				prev_title,
				prev_sub,
				prev_items,
				None,
				false,
			);

			// Incoming Page
			let in_offset = if self.slide_dir > 0.0 {
				(1.0 - slide_t) * bounds.width()
			} else {
				-(1.0 - slide_t) * bounds.width() * 0.5
			};

			let curr_items = self.current_items();
			let curr_title = self.current_title();
			let curr_sub = !self.nav_stack.is_empty();

			self.draw_menu_page(
				canvas,
				monet,
				font,
				fface,
				bounds,
				in_offset,
				curr_title,
				curr_sub,
				curr_items,
				self.hovered_index,
				self.hovered_header,
			);
		} else {
			// Static Page (no active slide animation)
			let curr_items = self.current_items();
			let curr_title = self.current_title();
			let curr_sub = !self.nav_stack.is_empty();

			self.draw_menu_page(
				canvas,
				monet,
				font,
				fface,
				bounds,
				0.0,
				curr_title,
				curr_sub,
				curr_items,
				self.hovered_index,
				self.hovered_header,
			);
		}

		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let is_sub_level = !self.nav_stack.is_empty();
		let header_h = self.header_height(self.current_title().is_some());
		let header_rect = Rect::new(bounds.x(), bounds.y(), bounds.width(), header_h);

		match event {
			InputEvent::PointerMoved { position } => {
				if is_sub_level && header_rect.contains_point(*position) {
					if !self.hovered_header {
						self.hovered_header = true;
						self.hovered_index = None;
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				} else {
					if self.hovered_header {
						self.hovered_header = false;
						ctx.request_redraw();
					}

					let new_hover = self.item_at_pos(*position);
					if new_hover != self.hovered_index {
						self.hovered_index = new_hover;
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered_index.is_some() || self.hovered_header {
					self.hovered_index = None;
					self.hovered_header = false;
					ctx.request_redraw();
					return EventResponse::Handled;
				}
			}

			InputEvent::Click { position } => {
				// Back button in header clicked
				if is_sub_level && header_rect.contains_point(*position) {
					self.navigate_back(ctx);
					return EventResponse::Handled;
				}

				// Item clicked
				if let Some(idx) = self.item_at_pos(*position) {
					let curr = self.current_items();
					if let Some(item) = curr.get(idx) {
						if !item.disabled && item.text != "-" {
							if item.has_submenu() {
								// Drill down into submenu with animation
								self.navigate_to(idx, ctx);
								return EventResponse::Handled;
							} else {
								self.selected_index = Some(idx);
								if let Some(cb) = &mut self.on_selected {
									cb();
								}
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}
					}
				}
			}

			_ => {}
		}

		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> bool {
		match key {
			"title" => {
				if let Some(s) = value.as_str() {
					self.title = Some(s.to_string());
					return true;
				}
			}

			"min_width" => {
				if let Some(n) = value.as_f32() {
					self.min_width = n;
					return true;
				}
			}

			"font_size" => {
				if let Some(n) = value.as_f32() {
					self.font_size = n;
					return true;
				}
			}

			"open" => {
				if let Some(b) = value.as_bool() {
					self.set_open(b);
					return true;
				}
			}

			"items" => {
				if let Some(arr) = value.as_array() {
					self.items = arr.iter().filter_map(parse_popup_item).collect();
					self.nav_stack.clear();
					self.prev_stack.clear();
					return true;
				}
			}

			_ => {}
		}
		false
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_selected")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_selected = Some(callback);
			true
		} else {
			false
		}
	}
}


fn parse_popup_item(val: &PropertyValue) -> Option<PopupItem> {
	if let Some(s) = val.as_str() {
		return Some(PopupItem { text: s.to_string(), ..PopupItem::default()});
	}
	if let Some(obj) = val.as_object() {
		let text = obj.get("text").and_then(|t| t.as_str())?;
		let mut item = PopupItem { text: text.to_string(), ..Default::default()};
		if let Some(sc) = obj.get("shortcut").and_then(|s| s.as_str()) {
			item.shortcut = Some(sc.to_string());
		}
		if let Some(dis) = obj.get("disabled").and_then(|b| b.as_bool()) {
			item.disabled = dis;
		}
		if let Some(sub_arr) = obj.get("items").and_then(|arr| arr.as_array()) {
			item.items = sub_arr.iter().filter_map(parse_popup_item).collect();
		}
		return Some(item);
	}
	None
}
