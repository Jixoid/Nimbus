/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, ButtonState, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	IntoMethod, LayoutCtx, MethodCallback, MouseButton, ObjectID, PropertyCtx, PropertyError,
	PropertyValue, UiTree, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


const TEXT_PADDING: f32 = 12.0;
const CHEVRON_AREA_WIDTH: f32 = 30.0;
const DROPDOWN_GAP: f32 = 4.0;
const DROPDOWN_PADDING: f32 = 4.0;


fn box_bounds_for(bounds: Rect<f32>, has_title: bool) -> Rect<f32> {
	if has_title {
		Rect::new(
			bounds.x(),
			bounds.y() + 5.0,
			bounds.width(),
			(bounds.height() - 5.0).max(0.0),
		)
	} else {
		bounds
	}
}

fn draw_chevron_vertical(
	canvas: &mut Canvas2D,
	center: Vec2<f32>,
	size: f32,
	open_t: f32,
	thickness: f32,
	color: Color,
) {
	let angle = open_t * std::f32::consts::PI;
	let cos_a = angle.cos();
	let sin_a = angle.sin();

	let half_h = size * 0.5;
	let steps = (size / 0.35).ceil().max(10.0) as usize;
	let r = thickness * 0.5;

	for i in 0..=steps {
		let t = i as f32 / steps as f32;
		let lx = -size * (1.0 - t);
		let rx = size * (1.0 - t);
		let y = -half_h + size * t;

		let pts: &[(f32, f32)] = if i == steps {
			&[(lx, y)]
		} else {
			&[(lx, y), (rx, y)]
		};
		for &(px, py) in pts {
			let rxp = px * cos_a - py * sin_a;
			let ryp = px * sin_a + py * cos_a;
			canvas.draw(
				RectShape::new(
					Rect::new(center.x + rxp - r, center.y + ryp - r, thickness, thickness),
					color,
				)
				.radius(r),
			);
		}
	}
}


pub struct ComboBox {
	pub items: Vec<String>,
	pub selected: Option<usize>,
	pub title: String,
	pub placeholder: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub border_radius: f32,
	pub max_dropdown_height: f32,
	pub on_changed: Option<MethodCallback>,

	open: bool,
	dropdown_id: Option<ObjectID>,
	chevron_anim: Animated<f32>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for ComboBox {
	fn default() -> Self {
		Self {
			items: Vec::new(),
			selected: None,
			title: String::new(),
			placeholder: "Select...".to_string(),
			font: None,
			font_size: 15.0,
			border_radius: 8.0,
			max_dropdown_height: 240.0,
			on_changed: None,
			open: false,
			dropdown_id: None,
			chevron_anim: Animated::new(0.0),
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl ComboBox {
	pub fn new(items: Vec<String>) -> Self {
		let selected = if items.is_empty() { None } else { Some(0) };
		Self {
			items,
			selected,
			..Default::default()
		}
	}

	pub fn builder() -> ComboBoxBuilder {
		ComboBoxBuilder::new()
	}

	pub fn selected_text(&self) -> Option<&str> {
		self.selected.and_then(|i| self.items.get(i).map(|s| s.as_str()))
	}

	pub fn set_selected(&mut self, index: Option<usize>) {
		self.selected = index.filter(|&i| i < self.items.len());
	}

	fn select_and_notify(&mut self, idx: usize) {
		if idx >= self.items.len() {
			return;
		}
		if self.selected != Some(idx) {
			self.selected = Some(idx);
			if let Some(cb) = &self.on_changed {
				cb.call2(idx, self.items[idx].clone());
			}
		}
	}

	fn set_open_visual(&mut self, open: bool) {
		self.open = open;
		if !open {
			self.dropdown_id = None;
		}
		self.chevron_anim.set_target(
			if open { 1.0 } else { 0.0 },
			Duration::from_millis(160),
			Easing::EmphasizedDecelerate,
		);
	}

	fn open_dropdown(&mut self, ctx: &mut EventCtx) {
		if self.items.is_empty() {
			return;
		}
		let combo_id = ctx.node_id();
		let anchor_bounds = box_bounds_for(self.last_bounds.get(), !self.title.is_empty());
		let items = self.items.clone();
		let selected = self.selected;
		let font = self.font;
		let font_size = self.font_size;
		let max_height = self.max_dropdown_height;

		self.set_open_visual(true);

		ctx.push_task(move |tree: &mut UiTree| {
			let prev_dropdown = tree
				.get_widget::<ComboBox>(combo_id)
				.and_then(|c| c.dropdown_id);
			if let Some(old_id) = prev_dropdown {
				tree.remove(old_id);
			}

			let dropdown = ComboDropdown::new(
				combo_id,
				anchor_bounds,
				items,
				selected,
				font,
				font_size,
				max_height,
			);
			let dd_id = tree.insert_root(dropdown);
			if let Some(combo) = tree.get_widget_mut::<ComboBox>(combo_id) {
				combo.dropdown_id = Some(dd_id);
				combo.open = true;
			}
			if let Some(node) = tree.nodes.get_mut(dd_id) {
				node.focused = true;
			}
			tree.focused_node = Some(dd_id);
			tree.needs_layout = true;
			tree.needs_redraw = true;
		});
	}

	fn close_dropdown(&mut self, ctx: &mut EventCtx) {
		let dd_id = self.dropdown_id.take();
		self.set_open_visual(false);
		if let Some(id) = dd_id {
			ctx.push_task(move |tree: &mut UiTree| {
				tree.remove(id);
				tree.needs_redraw = true;
			});
		}
	}
}

impl Widget for ComboBox {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(160.0, 40.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		if self.chevron_anim.is_animating() {
			ctx.request_redraw();
		}

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let has_title = !self.title.is_empty();
		let box_bounds = box_bounds_for(bounds, has_title);

		let border_color = if self.open || ctx.focused || ctx.hovered {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::PrimaryOutline]
		};

		canvas.draw(
			RectShape::new(box_bounds, Color::TRANSPARENT)
				.border(border_color, 2.0)
				.radius(self.border_radius),
		);

		let fface = FontManager::default_font();

		// Optional floating title notch (matches Edit & SpinBox)
		if has_title {
			let (title_w, _) = canvas.measure_text(&self.title, 12.0);
			let notch_rect = Rect::new(
				box_bounds.x() + 10.0,
				box_bounds.y() - 1.0,
				title_w + 8.0,
				3.0,
			);
			canvas.draw(RectShape::new(notch_rect, monet[MonetRole::SurfaceContainer]));

			let title_font = match self.font {
				Some(id) => Font::with_id(id, fface, 12.0),
				None => Font::new(fface, 12.0),
			};
			let title_color = if self.open || ctx.focused {
				monet[MonetRole::Primary]
			} else {
				monet[MonetRole::OnSurface]
			};
			canvas.draw(TextShape::new(
				&self.title,
				title_font,
				12.0,
				Vec2::new(box_bounds.x() + 14.0, bounds.y() - 2.0),
				title_color,
			));
		}

		// Selected item or placeholder text
		let (label, text_color) = match self.selected_text() {
			Some(txt) => (txt, monet[MonetRole::OnSurface]),
			None => {
				let mut muted = monet[MonetRole::OnSurfaceVariant];
				muted.a *= 0.7;
				(self.placeholder.as_str(), muted)
			}
		};

		let text_w = (box_bounds.width() - TEXT_PADDING - CHEVRON_AREA_WIDTH).max(0.0);
		let clip_rect = Rect::new(
			box_bounds.x() + TEXT_PADDING,
			box_bounds.y(),
			text_w,
			box_bounds.height(),
		);
		let (_, text_h) = canvas.measure_text(label, self.font_size);
		let text_y = box_bounds.y() + (box_bounds.height() - text_h) * 0.5;
		let text_font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		canvas.push_clip(clip_rect);
		canvas.draw(TextShape::new(
			label,
			text_font,
			self.font_size,
			Vec2::new(box_bounds.x() + TEXT_PADDING, text_y),
			text_color,
		));
		canvas.pop_clip();

		// Rotating chevron on the right
		let chevron_color = if self.open {
			monet[MonetRole::Primary]
		} else if ctx.hovered {
			monet[MonetRole::OnSurface]
		} else {
			monet[MonetRole::OnSurfaceVariant]
		};
		let chevron_center = Vec2::new(
			box_bounds.max.x - CHEVRON_AREA_WIDTH * 0.5,
			box_bounds.center().y,
		);
		draw_chevron_vertical(
			canvas,
			chevron_center,
			4.0,
			self.chevron_anim.value().clamp(0.0, 1.0),
			1.4,
			chevron_color,
		);
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::Click { .. } => {
				if self.open {
					self.close_dropdown(ctx);
				} else {
					self.open_dropdown(ctx);
				}
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::Key {
				key_name,
				pressed,
				..
			} => {
				if !pressed || self.items.is_empty() {
					return EventResponse::Ignored;
				}
				match key_name.as_str() {
					"Enter" | " " | "Space" => {
						if self.open {
							self.close_dropdown(ctx);
						} else {
							self.open_dropdown(ctx);
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowDown" => {
						let next = match self.selected {
							Some(i) => (i + 1).min(self.items.len() - 1),
							None => 0,
						};
						self.select_and_notify(next);
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowUp" => {
						let prev = match self.selected {
							Some(i) => i.saturating_sub(1),
							None => 0,
						};
						self.select_and_notify(prev);
						ctx.request_redraw();
						EventResponse::Handled
					}
					_ => EventResponse::Ignored,
				}
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
				self.items = arr
					.iter()
					.filter_map(|v| v.as_str().map(|s| s.to_string()))
					.collect();
				if let Some(sel) = self.selected {
					if sel >= self.items.len() {
						self.selected = if self.items.is_empty() { None } else { Some(0) };
					}
				} else if !self.items.is_empty() {
					self.selected = Some(0);
				}
			}
			"selected" => {
				let idx = value.expect_f64()?.max(0.0).round() as usize;
				self.selected = if idx < self.items.len() {
					Some(idx)
				} else if self.items.is_empty() {
					None
				} else {
					Some(self.items.len() - 1)
				};
			}
			"title" => self.title = value.expect_str()?.to_string(),
			"placeholder" => self.placeholder = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"corner_radius" => self.border_radius = value.expect_f32()?,
			"max_dropdown_height" => {
				self.max_dropdown_height = value.expect_f32()?.max(48.0);
			}
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed" | "on_selected")
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


#[derive(Default)]
pub struct ComboBoxBuilder {
	inner: ComboBox,
}

impl ComboBoxBuilder {
	pub fn new() -> Self {
		Self {
			inner: ComboBox::default(),
		}
	}

	pub fn items<I, S>(mut self, items: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.inner.items = items.into_iter().map(Into::into).collect();
		if self.inner.selected.is_none() && !self.inner.items.is_empty() {
			self.inner.selected = Some(0);
		}
		self
	}

	pub fn item(mut self, item: impl Into<String>) -> Self {
		self.inner.items.push(item.into());
		if self.inner.selected.is_none() {
			self.inner.selected = Some(0);
		}
		self
	}

	pub fn selected(mut self, index: usize) -> Self {
		self.inner.selected = Some(index);
		self
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
		self.inner.placeholder = placeholder.into();
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

	pub fn border_radius(mut self, radius: f32) -> Self {
		self.inner.border_radius = radius;
		self
	}

	pub fn max_dropdown_height(mut self, height: f32) -> Self {
		self.inner.max_dropdown_height = height.max(48.0);
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(mut self) -> ComboBox {
		if let Some(sel) = self.inner.selected
			&& sel >= self.inner.items.len()
		{
			self.inner.selected = if self.inner.items.is_empty() {
				None
			} else {
				Some(self.inner.items.len() - 1)
			};
		}
		self.inner
	}
}

impl From<ComboBoxBuilder> for ComboBox {
	fn from(builder: ComboBoxBuilder) -> Self {
		builder.build()
	}
}


// Internal Floating Dropdown Overlay Widget

struct ComboDropdown {
	owner_id: ObjectID,
	anchor_bounds: Rect<f32>,
	items: Vec<String>,
	selected: Option<usize>,
	hovered_index: Option<usize>,
	font: Option<FontId>,
	font_size: f32,
	max_height: f32,
	scroll_y: f32,
	progress: Animated<f32>,
	last_window_bounds: Cell<Rect<f32>>,
}

impl ComboDropdown {
	fn new(
		owner_id: ObjectID,
		anchor_bounds: Rect<f32>,
		items: Vec<String>,
		selected: Option<usize>,
		font: Option<FontId>,
		font_size: f32,
		max_height: f32,
	) -> Self {
		let mut progress = Animated::new(0.0);
		progress.set_target(1.0, Duration::from_millis(150), Easing::EmphasizedDecelerate);

		Self {
			owner_id,
			anchor_bounds,
			items,
			selected,
			hovered_index: selected,
			font,
			font_size,
			max_height,
			scroll_y: 0.0,
			progress,
			last_window_bounds: Cell::new(Rect::ZERO),
		}
	}

	fn row_height(&self) -> f32 {
		(self.font_size * 2.1).max(32.0)
	}

	fn content_height(&self) -> f32 {
		if self.items.is_empty() {
			return DROPDOWN_PADDING * 2.0;
		}
		let row_h = self.row_height();
		DROPDOWN_PADDING * 2.0 + self.items.len() as f32 * row_h + (self.items.len().saturating_sub(1)) as f32 * 2.0
	}

	fn menu_rect(&self, window_bounds: Rect<f32>) -> Rect<f32> {
		let w = self.anchor_bounds.width().max(120.0);
		let full_h = self.content_height().min(self.max_height);

		let space_below = (window_bounds.max.y - self.anchor_bounds.max.y - DROPDOWN_GAP - 6.0).max(0.0);
		let space_above = (self.anchor_bounds.y() - window_bounds.y() - DROPDOWN_GAP - 6.0).max(0.0);

		let (y, h) = if space_below >= full_h || space_below >= space_above {
			let h = full_h.min(space_below.max(self.row_height() + DROPDOWN_PADDING * 2.0));
			(self.anchor_bounds.max.y + DROPDOWN_GAP, h)
		} else {
			let h = full_h.min(space_above.max(self.row_height() + DROPDOWN_PADDING * 2.0));
			((self.anchor_bounds.y() - DROPDOWN_GAP - h).max(window_bounds.y() + 4.0), h)
		};

		let x = self
			.anchor_bounds
			.x()
			.min((window_bounds.max.x - w - 4.0).max(window_bounds.x() + 4.0))
			.max(window_bounds.x() + 4.0);

		Rect::new(x, y, w, h)
	}

	fn max_scroll(&self, menu_rect: Rect<f32>) -> f32 {
		(self.content_height() - menu_rect.height()).max(0.0)
	}

	fn item_at_pos(&self, menu_rect: Rect<f32>, pos: Vec2<f32>) -> Option<usize> {
		if !menu_rect.contains_point(pos) {
			return None;
		}
		let row_h = self.row_height();
		let rel_y = pos.y - menu_rect.y() - DROPDOWN_PADDING + self.scroll_y;
		if rel_y < 0.0 {
			return None;
		}
		let stride = row_h + 2.0;
		let idx = (rel_y / stride).floor() as usize;
		if idx < self.items.len() {
			Some(idx)
		} else {
			None
		}
	}

	fn dismiss(&self, ctx: &mut EventCtx, chosen: Option<usize>) {
		let dd_id = ctx.node_id();
		let owner_id = self.owner_id;
		ctx.push_task(move |tree: &mut UiTree| {
			if let Some(combo) = tree.get_widget_mut::<ComboBox>(owner_id) {
				if let Some(idx) = chosen {
					combo.select_and_notify(idx);
				}
				combo.set_open_visual(false);
			}
			tree.remove(dd_id);
			if let Some(node) = tree.nodes.get_mut(owner_id) {
				node.focused = true;
			}
			tree.focused_node = Some(owner_id);
			tree.needs_redraw = true;
		});
	}
}

impl Widget for ComboDropdown {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
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
		if self.progress.is_animating() {
			ctx.request_redraw();
		}

		let t = self.progress.value().clamp(0.0, 1.0);
		if t <= 0.001 {
			return;
		}

		let monet = &ctx.theme.monet;
		let full_rect = self.menu_rect(ctx.bounds);
		let anim_h = (full_rect.height() * (0.65 + 0.35 * t)).max(1.0);
		let anim_rect = Rect::new(full_rect.x(), full_rect.y(), full_rect.width(), anim_h);

		let mut bg = monet[MonetRole::SurfaceContainerHigh];
		bg.a *= t;
		let mut border = monet[MonetRole::SurfaceOutlineVariant];
		border.a *= t;
		let shadow = Color::new(0.0, 0.0, 0.0, 0.35 * t);

		canvas.draw(
			RectShape::new(anim_rect, bg)
				.radius(12.0)
				.border(border, 1.0)
				.shadow(shadow, Vec2::new(0.0, 4.0), 14.0),
		);

		let inner_clip = Rect::new(
			anim_rect.x() + 2.0,
			anim_rect.y() + 2.0,
			(anim_rect.width() - 4.0).max(0.0),
			(anim_rect.height() - 4.0).max(0.0),
		);
		canvas.push_clip(inner_clip);

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let row_h = self.row_height();
		let stride = row_h + 2.0;

		for (i, item) in self.items.iter().enumerate() {
			let ry = full_rect.y() + DROPDOWN_PADDING + i as f32 * stride - self.scroll_y;
			let row_rect = Rect::new(
				full_rect.x() + DROPDOWN_PADDING,
				ry,
				(full_rect.width() - DROPDOWN_PADDING * 2.0).max(0.0),
				row_h,
			);
			if row_rect.max.y < anim_rect.y() || row_rect.y() > anim_rect.max.y {
				continue;
			}

			let is_hovered = self.hovered_index == Some(i);
			let is_selected = self.selected == Some(i);

			if is_hovered {
				let mut hl = monet[MonetRole::Primary];
				hl.a *= t;
				canvas.draw(RectShape::new(row_rect, hl).radius(8.0).noise(0.10));
			} else if is_selected {
				canvas.draw(
					RectShape::new(row_rect, monet[MonetRole::SurfaceContainerHigh])
						.radius(8.0)
						.border(monet[MonetRole::Primary], 2.0),
					);
			}

			let mut text_col = if is_hovered {
				monet[MonetRole::OnPrimary]
			} else if is_selected {
				monet[MonetRole::OnSurface]
			} else {
				monet[MonetRole::OnSurface]
			};
			text_col.a *= t;

			let (_, text_h) = canvas.measure_text(item, self.font_size);
			let text_pos = Vec2::new(
				row_rect.x() + 10.0,
				row_rect.y() + (row_rect.height() - text_h) * 0.5,
			);
			canvas.draw(TextShape::new(item, font, self.font_size, text_pos, text_col));
		}

		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let win = self.last_window_bounds.get();
		let menu = self.menu_rect(win);

		match event {
			InputEvent::PointerMoved { position } => {
				let hit = self.item_at_pos(menu, *position);
				if hit != self.hovered_index {
					self.hovered_index = hit;
					ctx.request_redraw();
				}
				EventResponse::Handled
			}

			InputEvent::PointerScroll { delta } => {
				if menu.contains_point(Vec2::new(menu.center().x, menu.center().y)) {
					let max_s = self.max_scroll(menu);
					if max_s > 0.0 {
						let next = (self.scroll_y - delta.y).clamp(0.0, max_s);
						if (next - self.scroll_y).abs() > 0.1 {
							self.scroll_y = next;
							ctx.request_redraw();
						}
					}
				}
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				if let Some(idx) = self.item_at_pos(menu, *position) {
					self.dismiss(ctx, Some(idx));
				} else {
					self.dismiss(ctx, None);
				}
				EventResponse::Handled
			}

			InputEvent::Key {
				key_name,
				pressed,
				..
			} => {
				if !pressed {
					return EventResponse::Ignored;
				}
				match key_name.as_str() {
					"Escape" => {
						self.dismiss(ctx, None);
						EventResponse::Handled
					}
					"Enter" | " " | "Space" => {
						self.dismiss(ctx, self.hovered_index.or(self.selected));
						EventResponse::Handled
					}
					"ArrowDown" => {
						if !self.items.is_empty() {
							let next = match self.hovered_index {
								Some(i) => (i + 1).min(self.items.len() - 1),
								None => 0,
							};
							self.hovered_index = Some(next);
							ctx.request_redraw();
						}
						EventResponse::Handled
					}
					"ArrowUp" => {
						if !self.items.is_empty() {
							let prev = match self.hovered_index {
								Some(i) => i.saturating_sub(1),
								None => 0,
							};
							self.hovered_index = Some(prev);
							ctx.request_redraw();
						}
						EventResponse::Handled
					}
					_ => EventResponse::Ignored,
				}
			}

			_ => EventResponse::Handled,
		}
	}
}
