/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nimbus_core::{
	BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, ObjectID, PropertyCtx, PropertyError, PropertyValue,
	UiTree, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::popup::{Popup, PopupItem, parse_popup_item};


const BAR_HEIGHT: f32 = 34.0;
const ITEM_PAD_X: f32 = 12.0;
const BAR_PAD_X: f32 = 6.0;
const BAR_PAD_Y: f32 = 3.0;
const ITEM_GAP: f32 = 2.0;
const POPUP_GAP_Y: f32 = 4.0;


pub struct MenuBar {
	pub items: Vec<PopupItem>,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_action: Option<MethodCallback>,

	hovered_index: Option<usize>,
	pressed_index: Option<usize>,
	active_index: Option<usize>,
	overlay_id: Option<ObjectID>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for MenuBar {
	fn default() -> Self {
		Self {
			items: Vec::new(),
			background: None,
			corner_radius: None,
			font: None,
			font_size: 14.0,
			on_action: None,
			hovered_index: None,
			pressed_index: None,
			active_index: None,
			overlay_id: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl MenuBar {
	pub fn new(items: Vec<PopupItem>) -> Self {
		Self {
			items,
			..Default::default()
		}
	}

	pub fn builder() -> MenuBarBuilder {
		MenuBarBuilder::new()
	}

	pub fn add_item(&mut self, item: impl Into<PopupItem>) -> usize {
		self.items.push(item.into());
		self.items.len() - 1
	}

	pub fn add_menu(&mut self, title: impl Into<String>, sub_items: Vec<PopupItem>) -> usize {
		self.add_item(PopupItem {
			text: title.into(),
			items: sub_items,
			..Default::default()
		})
	}

	pub fn merge_menu(&mut self, title: &str, mut sub_items: Vec<PopupItem>) {
		if let Some(existing) = self.items.iter_mut().find(|m| m.text == title) {
			existing.items.append(&mut sub_items);
		} else {
			self.add_menu(title, sub_items);
		}
	}

	fn item_rects(&self, bounds: Rect<f32>) -> Vec<Rect<f32>> {
		let fface = FontManager::default_font();
		let mut rects = Vec::with_capacity(self.items.len());
		let mut x = bounds.x() + BAR_PAD_X;
		let h = (bounds.height() - BAR_PAD_Y * 2.0).max(0.0);
		let y = bounds.y() + BAR_PAD_Y;

		for item in &self.items {
			let (tw, _) = fface.measure_text(&item.text, self.font_size);
			let w = (tw + ITEM_PAD_X * 2.0).ceil();
			rects.push(Rect::new(x, y, w, h));
			x += w + ITEM_GAP;
		}
		rects
	}

	fn item_at_pos(&self, bounds: Rect<f32>, pos: Vec2<f32>) -> Option<usize> {
		self.item_rects(bounds)
			.into_iter()
			.enumerate()
			.find(|(_, r)| r.contains_point(pos))
			.map(|(i, _)| i)
	}

	fn fire_action(&self, action_or_text: String, menu_title: String) {
		if let Some(cb) = &self.on_action {
			cb.call2(action_or_text, menu_title);
		}
	}

	fn open_menu(&mut self, ctx: &mut EventCtx, index: usize) {
		let Some(item) = self.items.get(index) else {
			return;
		};
		if item.disabled {
			return;
		}
		if !item.has_submenu() {
			let act = item.action.clone().unwrap_or_else(|| item.text.clone());
			let title = item.text.clone();
			self.close_menu(ctx);
			self.fire_action(act, title);
			return;
		}

		let bar_id = ctx.node_id();
		let bar_bounds = self.last_bounds.get();
		let rects = self.item_rects(bar_bounds);
		let items = self.items.clone();
		let font = self.font;
		let font_size = self.font_size;

		self.active_index = Some(index);

		ctx.push_task(move |tree: &mut UiTree| {
			let prev_ov = tree.get_widget::<MenuBar>(bar_id).and_then(|b| b.overlay_id);
			if let Some(old_id) = prev_ov {
				tree.remove(old_id);
			}

			let overlay = MenuPopupOverlay::new(bar_id, bar_bounds, rects, items, index, font, font_size);
			let ov_id = tree.insert_root(overlay);
			if let Some(bar) = tree.get_widget_mut::<MenuBar>(bar_id) {
				bar.overlay_id = Some(ov_id);
				bar.active_index = Some(index);
			}
			if let Some(node) = tree.nodes.get_mut(ov_id) {
				node.focused = true;
			}
			tree.focused_node = Some(ov_id);
			tree.needs_layout = true;
			tree.needs_redraw = true;
		});
	}

	fn close_menu(&mut self, ctx: &mut EventCtx) {
		let ov = self.overlay_id.take();
		self.active_index = None;
		if let Some(id) = ov {
			ctx.push_task(move |tree: &mut UiTree| {
				tree.remove(id);
				tree.needs_redraw = true;
			});
		}
	}
}

impl Widget for MenuBar {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let mut total_w = BAR_PAD_X * 2.0;
		for (i, item) in self.items.iter().enumerate() {
			let (tw, _) = ctx.measure_text_with_font(&item.text, self.font, self.font_size);
			if i > 0 {
				total_w += ITEM_GAP;
			}
			total_w += (tw + ITEM_PAD_X * 2.0).ceil();
		}

		let w = if constraints.max_width.is_finite() {
			constraints.max_width.max(total_w)
		} else {
			total_w.max(120.0)
		};
		constraints.constrain(Vec2::new(w, BAR_HEIGHT))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;

		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);
		let bar_radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);
		canvas.draw(RectShape::new(bounds, bg).radius(bar_radius));

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let rects = self.item_rects(bounds);
		let item_radius = ctx.theme.radius.small.min(bar_radius);

		for (i, (item, rect)) in self.items.iter().zip(rects.iter()).enumerate() {
			let is_active = self.active_index == Some(i);
			let is_hovered = self.hovered_index == Some(i) && !item.disabled;

			if is_active {
				canvas.draw(
					RectShape::new(*rect, monet[MonetRole::Primary])
						.radius(item_radius)
						.noise(0.08),
				);
			} else if is_hovered {
				canvas.draw(
					RectShape::new(*rect, monet[MonetRole::SurfaceContainerHighest])
						.radius(item_radius),
				);
			}

			let text_col = if item.disabled {
				let mut c = monet[MonetRole::OnSurface];
				c.a *= 0.38;
				c
			} else if is_active {
				monet[MonetRole::OnPrimary]
			} else {
				monet[MonetRole::OnSurface]
			};

			let (tw, th) = canvas.measure_text(&item.text, self.font_size);
			let pos = Vec2::new(
				rect.x() + (rect.width() - tw) * 0.5,
				rect.y() + (rect.height() - th) * 0.5,
			);
			canvas.draw(TextShape::new(&item.text, font, self.font_size, pos, text_col));
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		match event {
			InputEvent::PointerMoved { position } => {
				let hit = self.item_at_pos(bounds, *position);
				if hit != self.hovered_index {
					self.hovered_index = hit;
					ctx.request_redraw();
				}
				EventResponse::Handled
			}

			InputEvent::PointerLeft => {
				if self.hovered_index.is_some() {
					self.hovered_index = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				self.pressed_index = self.item_at_pos(bounds, *position);
				if self.pressed_index.is_some() {
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				position,
			} => {
				if let Some(pressed) = self.pressed_index.take() {
					if bounds.contains_point(*position)
						&& self.item_at_pos(bounds, *position) == Some(pressed)
					{
						if self.active_index == Some(pressed) {
							self.close_menu(ctx);
						} else {
							self.open_menu(ctx, pressed);
						}
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerCancel => {
				if self.pressed_index.take().is_some() {
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"items" => {
				let arr = value.expect_array()?;
				self.items = arr.iter().filter_map(parse_popup_item).collect();
				self.active_index = None;
				self.hovered_index = None;
			}
			"color" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"font_size" => self.font_size = value.expect_f32()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_action")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_action = Some(callback);
			true
		} else {
			false
		}
	}
}


#[derive(Default)]
pub struct MenuBarBuilder {
	inner: MenuBar,
}

impl MenuBarBuilder {
	pub fn new() -> Self {
		Self {
			inner: MenuBar::default(),
		}
	}

	pub fn items(mut self, items: Vec<PopupItem>) -> Self {
		self.inner.items = items;
		self
	}

	pub fn item(mut self, item: impl Into<PopupItem>) -> Self {
		self.inner.items.push(item.into());
		self
	}

	pub fn menu(mut self, title: impl Into<String>, sub_items: Vec<PopupItem>) -> Self {
		self.inner.add_menu(title, sub_items);
		self
	}

	pub fn background(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn font_size(mut self, size: f32) -> Self {
		self.inner.font_size = size;
		self
	}

	pub fn on_action<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_action = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> MenuBar {
		self.inner
	}
}

impl From<MenuBarBuilder> for MenuBar {
	fn from(builder: MenuBarBuilder) -> Self {
		builder.build()
	}
}


// Internal Floating Popup Overlay (reuses `Popup` + hover-to-switch support)

struct MenuPopupOverlay {
	owner_id: ObjectID,
	bar_bounds: Rect<f32>,
	item_rects: Vec<Rect<f32>>,
	menus: Vec<PopupItem>,
	active_index: usize,
	popup: Popup,
	popup_size: Vec2<f32>,
	pressed_bar_index: Option<usize>,
	selected_leaf: Rc<RefCell<Option<(usize, String)>>>,
	last_window_bounds: Cell<Rect<f32>>,
}

impl MenuPopupOverlay {
	fn new(
		owner_id: ObjectID,
		bar_bounds: Rect<f32>,
		item_rects: Vec<Rect<f32>>,
		menus: Vec<PopupItem>,
		active_index: usize,
		font: Option<FontId>,
		font_size: f32,
	) -> Self {
		let selected_leaf = Rc::new(RefCell::new(None));
		let popup = Self::build_popup(
			&menus[active_index].items,
			font,
			font_size,
			selected_leaf.clone(),
		);

		Self {
			owner_id,
			bar_bounds,
			item_rects,
			menus,
			active_index,
			popup,
			popup_size: Vec2::new(180.0, 120.0),
			pressed_bar_index: None,
			selected_leaf,
			last_window_bounds: Cell::new(Rect::ZERO),
		}
	}

	fn build_popup(
		items: &[PopupItem],
		font: Option<FontId>,
		font_size: f32,
		selected_leaf: Rc<RefCell<Option<(usize, String)>>>,
	) -> Popup {
		let mut builder = Popup::builder()
			.items(items.to_vec())
			.font_size(font_size)
			.open(true)
			.on_selected(move |idx: usize, text: String| {
				*selected_leaf.borrow_mut() = Some((idx, text));
			});
		if let Some(f) = font {
			builder = builder.font(f);
		}
		let mut p = builder.build();
		p.progress.snap_to(0.0);
		p.set_open(true);
		p
	}

	fn switch_menu(&mut self, ctx: &mut EventCtx, new_index: usize) {
		if new_index == self.active_index || new_index >= self.menus.len() {
			return;
		}
		if self.menus[new_index].disabled || !self.menus[new_index].has_submenu() {
			return;
		}
		self.active_index = new_index;
		self.popup = Self::build_popup(
			&self.menus[new_index].items,
			self.popup.font,
			self.popup.font_size,
			self.selected_leaf.clone(),
		);
		let owner_id = self.owner_id;
		ctx.push_task(move |tree: &mut UiTree| {
			if let Some(bar) = tree.get_widget_mut::<MenuBar>(owner_id) {
				bar.active_index = Some(new_index);
				bar.hovered_index = Some(new_index);
			}
			tree.needs_layout = true;
			tree.needs_redraw = true;
		});
		ctx.request_layout();
		ctx.request_redraw();
	}

	fn popup_rect(&self, win_bounds: Rect<f32>) -> Rect<f32> {
		let anchor = self
			.item_rects
			.get(self.active_index)
			.copied()
			.unwrap_or(self.bar_bounds);
		let w = self.popup_size.x;
		let h = self.popup_size.y;

		let x = anchor
			.x()
			.min((win_bounds.max.x - w - 6.0).max(win_bounds.x() + 6.0))
			.max(win_bounds.x() + 6.0);

		let below_y = self.bar_bounds.max.y + POPUP_GAP_Y;
		let y = if below_y + h <= win_bounds.max.y - 4.0 {
			below_y
		} else {
			(self.bar_bounds.y() - POPUP_GAP_Y - h).max(win_bounds.y() + 4.0)
		};

		Rect::new(x, y, w, h)
	}

	fn dismiss(&self, ctx: &mut EventCtx, action: Option<(String, String)>) {
		let ov_id = ctx.node_id();
		let owner_id = self.owner_id;
		ctx.push_task(move |tree: &mut UiTree| {
			if let Some(bar) = tree.get_widget_mut::<MenuBar>(owner_id) {
				bar.active_index = None;
				bar.overlay_id = None;
				if let Some((act, menu_title)) = action {
					bar.fire_action(act, menu_title);
				}
			}
			tree.remove(ov_id);
			tree.needs_redraw = true;
		});
	}

	fn resolve_selected_action(&self, idx: usize, fallback_text: String) -> (String, String) {
		let menu_title = self
			.menus
			.get(self.active_index)
			.map(|m| m.text.clone())
			.unwrap_or_default();
		let current_items = self.popup.current_items();
		let act = current_items
			.get(idx)
			.and_then(|it| it.action.clone())
			.unwrap_or(fallback_text);
		(act, menu_title)
	}
}

impl Widget for MenuPopupOverlay {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		self.popup_size = self.popup.layout(ctx, BoxConstraints::unbounded());
		constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				800.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				600.0
			},
		))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_window_bounds.set(ctx.bounds);
		let rect = self.popup_rect(ctx.bounds);
		let popup_ctx = ctx.with_bounds(rect);
		self.popup.draw(&popup_ctx, canvas);
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let win = self.last_window_bounds.get();
		let prect = self.popup_rect(win);

		match event {
			InputEvent::PointerMoved { position } => {
				if self.bar_bounds.contains_point(*position) {
					if let Some(idx) = self
						.item_rects
						.iter()
						.enumerate()
						.find(|(_, r)| r.contains_point(*position))
						.map(|(i, _)| i)
						&& idx != self.active_index {
							self.switch_menu(ctx, idx);
						}
					return EventResponse::Handled;
				}

				self.popup.handle_event(ctx, event);
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				if prect.contains_point(*position) {
					self.popup.handle_event(ctx, event);
					return EventResponse::Handled;
				}
				if self.bar_bounds.contains_point(*position)
					&& let Some(idx) = self
						.item_rects
						.iter()
						.enumerate()
						.find(|(_, r)| r.contains_point(*position))
						.map(|(i, _)| i)
					{
						if idx == self.active_index {
							self.dismiss(ctx, None);
						} else if self.menus[idx].has_submenu() {
							self.switch_menu(ctx, idx);
						} else {
							self.pressed_bar_index = Some(idx);
						}
						return EventResponse::Handled;
					}
				self.dismiss(ctx, None);
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				position,
			} => {
				if let Some(pressed) = self.pressed_bar_index.take() {
					if self.bar_bounds.contains_point(*position)
						&& self
							.item_rects
							.iter()
							.enumerate()
							.find(|(_, r)| r.contains_point(*position))
							.map(|(i, _)| i)
							== Some(pressed)
						&& !self.menus[pressed].has_submenu()
					{
						let item = &self.menus[pressed];
						let act = item.action.clone().unwrap_or_else(|| item.text.clone());
						self.dismiss(ctx, Some((act, item.text.clone())));
					}
					return EventResponse::Handled;
				}
				self.popup.handle_event(ctx, event);
				EventResponse::Handled
			}

			InputEvent::PointerCancel => {
				self.pressed_bar_index = None;
				self.popup.handle_event(ctx, event);
				EventResponse::Handled
			}

			InputEvent::Click { position } => {
				if prect.contains_point(*position) {
					*self.selected_leaf.borrow_mut() = None;
					self.popup.handle_event(ctx, event);
					if let Some((idx, text)) = self.selected_leaf.borrow_mut().take() {
						let action = self.resolve_selected_action(idx, text);
						self.dismiss(ctx, Some(action));
					}
				}
				EventResponse::Handled
			}

			InputEvent::Key {
				key_name,
				pressed: true,
				..
			} => {
				if key_name == "Escape" {
					self.dismiss(ctx, None);
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			_ => {
				self.popup.handle_event(ctx, event);
				EventResponse::Handled
			}
		}
	}
}
