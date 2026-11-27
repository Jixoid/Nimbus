/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::time::Instant;

use nimbus_core::{
	BoxConstraints, ButtonState, Cursor, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, CurveShape, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::scroll::ScrollState;


const DRAG_THRESHOLD: f32 = 6.0;


pub type TreePath = Vec<usize>;


#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem {
	pub id: String,
	pub label: String,
	pub expanded: bool,
	pub badge: Option<String>,
	pub color: Option<DColor>,
	pub children: Vec<TreeItem>,
}

impl TreeItem {
	pub fn new(label: impl Into<String>) -> Self {
		let label = label.into();
		Self {
			id: label.clone(),
			label,
			expanded: false,
			badge: None,
			color: None,
			children: Vec::new(),
		}
	}

	pub fn with_id(mut self, id: impl Into<String>) -> Self {
		self.id = id.into();
		self
	}

	pub fn expanded(mut self, expanded: bool) -> Self {
		self.expanded = expanded;
		self
	}

	pub fn badge(mut self, badge: impl Into<String>) -> Self {
		self.badge = Some(badge.into());
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.color = Some(color.into());
		self
	}

	pub fn children(mut self, children: Vec<TreeItem>) -> Self {
		self.children = children;
		self
	}

	pub fn child(mut self, item: TreeItem) -> Self {
		self.children.push(item);
		self
	}

	fn matches_filter(&self, filter_lower: &str) -> bool {
		if filter_lower.is_empty() {
			return true;
		}
		if self.label.to_ascii_lowercase().contains(filter_lower) {
			return true;
		}
		self.children.iter().any(|c| c.matches_filter(filter_lower))
	}
}


#[derive(Clone, Debug)]
struct VisibleRow {
	path: TreePath,
	depth: usize,
	label: String,
	badge: Option<String>,
	color: Option<DColor>,
	has_children: bool,
	expanded: bool,
}

#[derive(Clone, Debug)]
struct DragRow {
	source_path: TreePath,
	label: String,
	start_pos: Vec2<f32>,
	pointer: Vec2<f32>,
	active: bool,
	target_path: Option<TreePath>,
}


pub struct TreeView {
	pub items: Vec<TreeItem>,
	pub row_height: f32,
	pub indent: f32,
	pub padding: f32,
	pub filter: String,
	pub draggable: bool,
	pub show_guides: bool,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
	pub font: Option<FontId>,
	pub font_size: f32,

	pub selected: Option<TreePath>,
	pub scroll: ScrollState,

	pub on_selected: Option<MethodCallback>,
	pub on_reparented: Option<MethodCallback>,

	visible_rows: Vec<VisibleRow>,
	hovered_row: Option<usize>,
	pressed_row: Option<(TreePath, bool)>,
	drag: Option<DragRow>,
	last_click: Option<(Instant, TreePath)>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for TreeView {
	fn default() -> Self {
		Self {
			items: Vec::new(),
			row_height: 26.0,
			indent: 18.0,
			padding: 6.0,
			filter: String::new(),
			draggable: true,
			show_guides: true,
			background: None,
			corner_radius: None,
			border_width: 0.0,
			border_color: None,
			font: None,
			font_size: 13.5,
			selected: None,
			scroll: ScrollState::new(),
			on_selected: None,
			on_reparented: None,
			visible_rows: Vec::new(),
			hovered_row: None,
			pressed_row: None,
			drag: None,
			last_click: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl TreeView {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> TreeViewBuilder {
		TreeViewBuilder::new()
	}

	pub fn item_at(&self, path: &[usize]) -> Option<&TreeItem> {
		if path.is_empty() {
			return None;
		}
		let mut cur = self.items.get(path[0])?;
		for &idx in &path[1..] {
			cur = cur.children.get(idx)?;
		}
		Some(cur)
	}

	pub fn item_at_mut(&mut self, path: &[usize]) -> Option<&mut TreeItem> {
		if path.is_empty() {
			return None;
		}
		let mut cur = self.items.get_mut(path[0])?;
		for &idx in &path[1..] {
			cur = cur.children.get_mut(idx)?;
		}
		Some(cur)
	}

	pub fn toggle_expanded(&mut self, path: &[usize]) -> bool {
		if let Some(item) = self.item_at_mut(path)
			&& !item.children.is_empty()
		{
			item.expanded = !item.expanded;
			return true;
		}
		false
	}

	/// Moves the item at `source` to become a child of `target`
	pub fn reparent(&mut self, source: &[usize], target: &[usize]) -> bool {
		if source.is_empty() || target.is_empty() {
			return false;
		}
		if source == target || target.starts_with(source) {
			return false;
		}

		// Compute adjusted target path after removing `source`
		let mut adjusted_target = target.to_vec();
		let src_parent = &source[..source.len() - 1];
		let src_idx = source[source.len() - 1];
		if adjusted_target.len() > src_parent.len()
			&& adjusted_target[..src_parent.len()] == *src_parent
			&& adjusted_target[src_parent.len()] > src_idx
		{
			adjusted_target[src_parent.len()] -= 1;
		}

		let Some(removed) = self.remove_at(source) else {
			return false;
		};

		if let Some(dest) = self.item_at_mut(&adjusted_target) {
			dest.expanded = true;
			dest.children.push(removed);
			let mut new_path = adjusted_target;
			new_path.push(dest.children.len() - 1);
			self.selected = Some(new_path);
			true
		} else {
			self.items.push(removed);
			false
		}
	}

	fn remove_at(&mut self, path: &[usize]) -> Option<TreeItem> {
		if path.is_empty() {
			return None;
		}
		if path.len() == 1 {
			if path[0] < self.items.len() {
				return Some(self.items.remove(path[0]));
			}
			return None;
		}
		let parent = self.item_at_mut(&path[..path.len() - 1])?;
		let idx = path[path.len() - 1];
		if idx < parent.children.len() {
			Some(parent.children.remove(idx))
		} else {
			None
		}
	}

	fn rebuild_visible_rows(&mut self) {
		self.visible_rows.clear();
		let filter_lower = self.filter.trim().to_ascii_lowercase();
		let mut path = Vec::new();
		Self::collect_rows(&self.items, 0, &filter_lower, &mut path, &mut self.visible_rows);
	}

	fn collect_rows(
		items: &[TreeItem],
		depth: usize,
		filter_lower: &str,
		path: &mut TreePath,
		out: &mut Vec<VisibleRow>,
	) {
		for (i, item) in items.iter().enumerate() {
			if !item.matches_filter(filter_lower) {
				continue;
			}
			path.push(i);
			let has_children = !item.children.is_empty();
			let effective_expanded = item.expanded || !filter_lower.is_empty();
			out.push(VisibleRow {
				path: path.clone(),
				depth,
				label: item.label.clone(),
				badge: item.badge.clone(),
				color: item.color,
				has_children,
				expanded: effective_expanded,
			});
			if has_children && effective_expanded {
				Self::collect_rows(&item.children, depth + 1, filter_lower, path, out);
			}
			path.pop();
		}
	}

	fn row_at_y(&self, bounds: Rect<f32>, y: f32) -> Option<usize> {
		if y < bounds.y() + self.padding || y > bounds.max.y - self.padding {
			return None;
		}
		let rel_y = y - (bounds.y() + self.padding) + self.scroll.offset;
		if rel_y < 0.0 {
			return None;
		}
		let idx = (rel_y / self.row_height).floor() as usize;
		if idx < self.visible_rows.len() {
			Some(idx)
		} else {
			None
		}
	}

	fn chevron_rect(&self, bounds: Rect<f32>, row_idx: usize, depth: usize) -> Rect<f32> {
		let y = bounds.y() + self.padding + row_idx as f32 * self.row_height - self.scroll.offset;
		let x = bounds.x() + self.padding + depth as f32 * self.indent + 2.0;
		Rect::new(x, y, 18.0, self.row_height)
	}
}

fn draw_chevron(canvas: &mut Canvas2D, center: Vec2<f32>, expanded: bool, color: Color) {
	let s = 3.5;
	if expanded {
		// Downward chevron (v)
		let a = Vec2::new(center.x - s, center.y - s * 0.45);
		let b = Vec2::new(center.x, center.y + s * 0.55);
		let c = Vec2::new(center.x + s, center.y - s * 0.45);
		canvas.draw(CurveShape::line(a, b, 1.6, color));
		canvas.draw(CurveShape::line(b, c, 1.6, color));
	} else {
		// Rightward chevron (>)
		let a = Vec2::new(center.x - s * 0.45, center.y - s);
		let b = Vec2::new(center.x + s * 0.55, center.y);
		let c = Vec2::new(center.x - s * 0.45, center.y + s);
		canvas.draw(CurveShape::line(a, b, 1.6, color));
		canvas.draw(CurveShape::line(b, c, 1.6, color));
	}
}

impl Widget for TreeView {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		self.rebuild_visible_rows();
		let content_h = self.visible_rows.len() as f32 * self.row_height + self.padding * 2.0;
		let w = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			240.0
		};
		let h = if constraints.max_height.is_finite() && constraints.min_height >= constraints.max_height {
			constraints.max_height
		} else {
			content_h.max(60.0).min(constraints.max_height)
		};
		let final_size = constraints.constrain(Vec2::new(w, h));
		self.scroll.update(content_h, final_size.y);
		self.scroll.sync();
		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 2.0 || bounds.height() <= 2.0 {
			return;
		}

		let theme = ctx.theme;
		let monet = &theme.monet;
		let radius = self.corner_radius.unwrap_or(theme.radius.small);

		if let Some(bg) = self.background {
			let mut shape = RectShape::new(bounds, bg.get(monet)).radius(radius);
			if self.border_width > 0.0 {
				let bcol = self
					.border_color
					.map(|c| c.get(monet))
					.unwrap_or_else(|| monet[MonetRole::SurfaceOutlineVariant]);
				shape = shape.border(bcol, self.border_width);
			}
			canvas.draw(shape);
		} else if self.border_width > 0.0 {
			let bcol = self
				.border_color
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::SurfaceOutlineVariant]);
			canvas.draw(
				RectShape::new(bounds, Color::TRANSPARENT)
					.radius(radius)
					.border(bcol, self.border_width),
			);
		}

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let badge_font = match self.font {
			Some(id) => Font::with_id(id, fface, 10.5),
			None => Font::new(fface, 10.5),
		};

		let inner_clip = Rect::new(
			bounds.x() + 1.0,
			bounds.y() + 1.0,
			(bounds.width() - 2.0).max(0.0),
			(bounds.height() - 2.0).max(0.0),
		);
		canvas.push_clip(inner_clip);

		let row_r = theme.radius.xsmall;
		let mut guide_col = monet[MonetRole::SurfaceOutlineVariant];
		guide_col.a = 0.45;

		for (i, row) in self.visible_rows.iter().enumerate() {
			let y = bounds.y() + self.padding + i as f32 * self.row_height - self.scroll.offset;
			if y + self.row_height < bounds.y() || y > bounds.max.y {
				continue;
			}

			let row_rect = Rect::new(
				bounds.x() + self.padding,
				y + 1.0,
				(bounds.width() - self.padding * 2.0).max(0.0),
				(self.row_height - 2.0).max(1.0),
			);

			let is_selected = self.selected.as_ref() == Some(&row.path);
			let is_hovered = self.hovered_row == Some(i);
			let is_drop_target = self
				.drag
				.as_ref()
				.is_some_and(|d| d.active && d.target_path.as_ref() == Some(&row.path));

			if is_drop_target {
				let mut fill = monet[MonetRole::Primary];
				fill.a = 0.22;
				canvas.draw(
					RectShape::new(row_rect, fill)
						.radius(row_r)
						.border(monet[MonetRole::Primary], theme.stroke.thick),
				);
			} else if is_selected {
				canvas.draw(
					RectShape::new(row_rect, monet[MonetRole::Primary])
						.radius(row_r),
				);
			} else if is_hovered {
				canvas.draw(
					RectShape::new(row_rect, monet[MonetRole::SurfaceContainerHigh])
						.radius(row_r),
				);
			}

			// Indent guides
			if self.show_guides && row.depth > 0 {
				for d in 0..row.depth {
					let gx = bounds.x() + self.padding + d as f32 * self.indent + 11.0;
					canvas.draw(RectShape::new(
						Rect::new(gx, y, 1.0, self.row_height),
						guide_col,
					));
				}
			}

			let chev_r = self.chevron_rect(bounds, i, row.depth);
			let fg = if is_selected {
				monet[MonetRole::OnPrimary]
			} else {
				monet[MonetRole::OnSurface]
			};

			if row.has_children {
				draw_chevron(canvas, chev_r.center(), row.expanded, fg);
			}

			let mut text_x = chev_r.max.x + 2.0;
			if let Some(dot_col) = row.color {
				canvas.draw(
					RectShape::new(
						Rect::new(text_x, row_rect.center().y - 3.5, 7.0, 7.0),
						dot_col.get(monet),
					)
					.radius(3.5),
				);
				text_x += 12.0;
			}

			let (_, th) = canvas.measure_text(&row.label, self.font_size);
			canvas.draw(TextShape::new(
				&row.label,
				font,
				self.font_size,
				Vec2::new(text_x, row_rect.center().y - th * 0.5),
				fg,
			));

			if let Some(badge) = &row.badge {
				let (bw, bh) = canvas.measure_text(badge, 10.5);
				let bx = row_rect.max.x - bw - 8.0;
				if bx > text_x + 24.0 {
					let bcol = if is_selected {
						monet[MonetRole::OnPrimaryContainer]
					} else {
						monet[MonetRole::OnSurfaceVariant]
					};
					canvas.draw(TextShape::new(
						badge,
						badge_font,
						10.5,
						Vec2::new(bx, row_rect.center().y - bh * 0.5),
						bcol,
					));
				}
			}
		}

		// Floating drag ghost pill
		if let Some(drag) = &self.drag
			&& drag.active
		{
			let (tw, th) = canvas.measure_text(&drag.label, self.font_size);
			let gw = tw + 20.0;
			let gh = 24.0;
			let gr = Rect::new(drag.pointer.x + 10.0, drag.pointer.y - gh * 0.5, gw, gh);
			let mut gbg = monet[MonetRole::SurfaceContainerHighest];
			gbg.a = 0.94;
			canvas.draw(
				RectShape::new(gr, gbg)
					.radius(theme.radius.small)
					.border(monet[MonetRole::Primary], theme.stroke.thin)
					.shadow(monet[MonetRole::Shadow], Vec2::new(0.0, 3.0), 8.0),
			);
			canvas.draw(TextShape::new(
				&drag.label,
				font,
				self.font_size,
				Vec2::new(gr.x() + 10.0, gr.center().y - th * 0.5),
				monet[MonetRole::OnSurface],
			));
		}

		self.scroll.draw(canvas, bounds, true, theme);
		if self.scroll.is_animating() {
			ctx.request_layout();
			ctx.request_redraw();
		}

		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		if let InputEvent::PointerScroll { .. } = event {
			return self.scroll.handle_event(ctx, event, bounds, true);
		}

		match event {
			InputEvent::PointerMoved { position } => {
				if let Some(drag) = &mut self.drag {
					drag.pointer = *position;
					let d = *position - drag.start_pos;
					if !drag.active && (d.x * d.x + d.y * d.y).sqrt() >= DRAG_THRESHOLD {
						drag.active = true;
					}
					if drag.active {
						let src = drag.source_path.clone();
						let target = self.row_at_y(bounds, position.y).and_then(|idx| {
							let p = &self.visible_rows[idx].path;
							if p != &src && !p.starts_with(&src) {
								Some(p.clone())
							} else {
								None
							}
						});
						if let Some(drag) = &mut self.drag {
							drag.target_path = target;
						}
						ctx.request_redraw();
					}
					return EventResponse::Handled;
				}

				let hov = self.row_at_y(bounds, position.y);
				if hov != self.hovered_row {
					self.hovered_row = hov;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerLeft => {
				if self.hovered_row.is_some() {
					self.hovered_row = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				if let Some(row_idx) = self.row_at_y(bounds, position.y) {
					let row = self.visible_rows[row_idx].clone();
					let chev_r = self.chevron_rect(bounds, row_idx, row.depth);
					let hit_chevron = row.has_children && chev_r.contains_point(*position);
					self.pressed_row = Some((row.path.clone(), hit_chevron));

					if self.draggable && !hit_chevron {
						self.drag = Some(DragRow {
							source_path: row.path,
							label: row.label,
							start_pos: *position,
							pointer: *position,
							active: false,
							target_path: None,
						});
					}

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
				if let Some(drag) = self.drag.take()
					&& drag.active
				{
					self.pressed_row = None;
					if let Some(target_path) = drag.target_path {
						let target_label = self
							.item_at(&target_path)
							.map(|i| i.label.clone())
							.unwrap_or_default();
						if self.reparent(&drag.source_path, &target_path) {
							if let Some(cb) = &self.on_reparented {
								cb.call2(drag.label, target_label);
							}
							ctx.request_layout();
						}
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				if let Some((pressed_path, hit_chevron)) = self.pressed_row.take() {
					if bounds.contains_point(*position)
						&& let Some(row_idx) = self.row_at_y(bounds, position.y)
						&& self.visible_rows[row_idx].path == pressed_path
					{
						let row = self.visible_rows[row_idx].clone();
						let chev_r = self.chevron_rect(bounds, row_idx, row.depth);
						if hit_chevron {
							if row.has_children && chev_r.contains_point(*position) && self.toggle_expanded(&row.path) {
								ctx.request_layout();
								ctx.request_redraw();
							}
							return EventResponse::Handled;
						}

						self.selected = Some(row.path.clone());
						if let Some(item) = self.item_at(&row.path)
							&& let Some(cb) = &self.on_selected
						{
							cb.call2(item.id.clone(), item.label.clone());
						}

						let now = Instant::now();
						let is_double = self.last_click.as_ref().is_some_and(|(prev_t, prev_p)| {
							now.duration_since(*prev_t).as_millis() < 350 && prev_p == &row.path
						});
						self.last_click = Some((now, row.path.clone()));
						if is_double && row.has_children {
							self.toggle_expanded(&row.path);
							ctx.request_layout();
						}
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerCancel => {
				if self.pressed_row.take().is_some() || self.drag.take().is_some() {
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			_ => EventResponse::Ignored,
		}
	}

	fn cursor(&self) -> Cursor {
		if self.drag.as_ref().is_some_and(|d| d.active) {
			Cursor::Grabbing
		} else {
			Cursor::Default
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
				if let Some(arr) = value.as_array() {
					self.items = parse_tree_items(arr);
				} else {
					return Err(PropertyError::InvalidValue {
						expected: "array of tree items",
					});
				}
			}
			"row_height" => self.row_height = value.expect_f32()?.clamp(18.0, 64.0),
			"indent" => self.indent = value.expect_f32()?.clamp(8.0, 48.0),
			"paddings" | "padding" => self.padding = value.expect_f32()?.max(0.0),
			"filter" => self.filter = value.expect_str()?.to_string(),
			"draggable" => self.draggable = value.expect_bool()?,
			"show_guides" => self.show_guides = value.expect_bool()?,
			"color" | "background" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" | "radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			"border_width" => self.border_width = value.expect_f32()?.max(0.0),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"font_size" => self.font_size = value.expect_f32()?.clamp(9.0, 28.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_selected" | "on_reparented")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_selected" => {
				self.on_selected = Some(callback);
				true
			}
			"on_reparented" => {
				self.on_reparented = Some(callback);
				true
			}
			_ => false,
		}
	}
}

fn parse_tree_items(arr: &[PropertyValue]) -> Vec<TreeItem> {
	let mut out = Vec::with_capacity(arr.len());
	for val in arr {
		if let Some(s) = val.as_str() {
			out.push(TreeItem::new(s));
		} else if let Some(obj) = val.as_object() {
			let label = obj
				.get("text")
				.or_else(|| obj.get("label"))
				.or_else(|| obj.get("name"))
				.and_then(|v| v.as_str())
				.unwrap_or("Item")
				.to_string();
			let mut item = TreeItem::new(label);
			if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
				item.id = id.to_string();
			}
			if let Some(exp) = obj.get("expanded").and_then(|v| v.as_bool()) {
				item.expanded = exp;
			}
			if let Some(badge) = obj.get("badge").and_then(|v| v.as_str()) {
				item.badge = Some(badge.to_string());
			}
			if let Some(col) = obj.get("color").and_then(|v| v.resolve_dcolor()) {
				item.color = Some(col);
			}
			if let Some(children) = obj
				.get("items")
				.or_else(|| obj.get("children"))
				.and_then(|v| v.as_array())
			{
				item.children = parse_tree_items(children);
			}
			out.push(item);
		}
	}
	out
}


#[derive(Default)]
pub struct TreeViewBuilder {
	inner: TreeView,
}

impl TreeViewBuilder {
	pub fn new() -> Self {
		Self {
			inner: TreeView::default(),
		}
	}

	pub fn items(mut self, items: Vec<TreeItem>) -> Self {
		self.inner.items = items;
		self
	}

	pub fn row_height(mut self, height: f32) -> Self {
		self.inner.row_height = height.clamp(18.0, 64.0);
		self
	}

	pub fn indent(mut self, indent: f32) -> Self {
		self.inner.indent = indent.clamp(8.0, 48.0);
		self
	}

	pub fn draggable(mut self, draggable: bool) -> Self {
		self.inner.draggable = draggable;
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn on_selected<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_selected = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_reparented<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_reparented = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> TreeView {
		self.inner
	}
}

impl From<TreeViewBuilder> for TreeView {
	fn from(builder: TreeViewBuilder) -> Self {
		builder.build()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_treeview_flatten_filter_and_reparent() {
		let mut tv = TreeView::builder()
			.items(vec![
				TreeItem::new("Root")
					.expanded(true)
					.child(TreeItem::new("Camera"))
					.child(TreeItem::new("Player").expanded(true).child(TreeItem::new("Sword"))),
				TreeItem::new("Environment"),
			])
			.build();

		tv.rebuild_visible_rows();
		assert_eq!(tv.visible_rows.len(), 5);

		// Cannot reparent a node into its own descendant
		assert!(!tv.reparent(&[0], &[0, 1]));

		// Move "Camera" ([0, 0]) into "Environment" ([1])
		assert!(tv.reparent(&[0, 0], &[1]));
		assert_eq!(tv.items[0].children.len(), 1);
		assert_eq!(tv.items[1].children.len(), 1);
		assert_eq!(tv.items[1].children[0].label, "Camera");

		// Filter matches ancestor + matching leaf
		tv.filter = "sword".to_string();
		tv.rebuild_visible_rows();
		assert_eq!(tv.visible_rows.len(), 3); // Root -> Player -> Sword
	}
}
