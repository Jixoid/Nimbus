/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	BoxConstraints, ButtonState, Cursor, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, ObjectID, PropertyCtx, PropertyError, PropertyValue,
	Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::pane::DockPane;
use crate::tree::{DockNode, DockPath, DockSplitAxis, DockZone};


const TAB_BAR_HEIGHT: f32 = 30.0;
const TAB_PADDING_X: f32 = 12.0;
const TAB_GAP: f32 = 4.0;
const CLOSE_BTN_SIZE: f32 = 14.0;
const DRAG_THRESHOLD: f32 = 6.0;


#[derive(Clone, Debug)]
struct CachedTab {
	pane_id: ObjectID,
	title: String,
	rect: Rect<f32>,
	close_rect: Option<Rect<f32>>,
	active: bool,
}

#[derive(Clone, Debug)]
struct CachedLeaf {
	path: DockPath,
	anchor_pane: ObjectID,
	full_rect: Rect<f32>,
	tab_bar_rect: Rect<f32>,
	tabs: Vec<CachedTab>,
}

#[derive(Clone, Debug)]
struct CachedSplitter {
	path: DockPath,
	axis: DockSplitAxis,
	rect: Rect<f32>,
	parent_rect: Rect<f32>,
}

#[derive(Clone, Debug)]
struct SplitterDrag {
	path: DockPath,
	axis: DockSplitAxis,
	parent_rect: Rect<f32>,
}

#[derive(Clone, Debug)]
struct TabDrag {
	pane_id: ObjectID,
	title: String,
	start_pos: Vec2<f32>,
	pointer: Vec2<f32>,
	active: bool,
	drop_target: Option<DropTarget>,
}

#[derive(Clone, Debug)]
struct DropTarget {
	anchor_pane: ObjectID,
	zone: DockZone,
	preview_rect: Rect<f32>,
}


pub struct DockArea {
	pub root: DockNode,
	pub splitter_size: f32,
	pub tab_bar_height: f32,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_pane_closed: Option<MethodCallback>,
	pub on_layout_changed: Option<MethodCallback>,

	leaves: Vec<CachedLeaf>,
	splitters: Vec<CachedSplitter>,
	closed_panes: Vec<ObjectID>,
	hovered_splitter: Option<DockPath>,
	hovered_tab: Option<(DockPath, usize)>,
	hovered_close: Option<(DockPath, usize)>,
	splitter_drag: Option<SplitterDrag>,
	tab_drag: Option<TabDrag>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for DockArea {
	fn default() -> Self {
		Self {
			root: DockNode::Empty,
			splitter_size: 6.0,
			tab_bar_height: TAB_BAR_HEIGHT,
			background: None,
			corner_radius: None,
			font: None,
			font_size: 13.0,
			on_pane_closed: None,
			on_layout_changed: None,
			leaves: Vec::new(),
			splitters: Vec::new(),
			closed_panes: Vec::new(),
			hovered_splitter: None,
			hovered_tab: None,
			hovered_close: None,
			splitter_drag: None,
			tab_drag: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl DockArea {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> DockAreaBuilder {
		DockAreaBuilder::new()
	}

	pub fn dock_pane(&mut self, pane_id: ObjectID, zone: DockZone) {
		self.closed_panes.retain(|&id| id != pane_id);
		if self.root.contains_pane(pane_id) {
			self.root.remove_pane(pane_id);
		}
		self.root.dock_here(pane_id, zone);
	}

	pub fn close_pane(&mut self, pane_id: ObjectID) -> bool {
		if self.root.remove_pane(pane_id) {
			if !self.closed_panes.contains(&pane_id) {
				self.closed_panes.push(pane_id);
			}
			true
		} else {
			false
		}
	}

	pub fn restore_pane(&mut self, pane_id: ObjectID, zone: DockZone) {
		self.closed_panes.retain(|&id| id != pane_id);
		if !self.root.contains_pane(pane_id) {
			self.root.dock_here(pane_id, zone);
		}
	}

	fn compute_drop_target(&self, local_pos: Vec2<f32>, dragged_pane: ObjectID) -> Option<DropTarget> {
		for leaf in &self.leaves {
			if !leaf.full_rect.contains_point(local_pos) {
				continue;
			}
			// If dragging the only tab in this leaf, don't offer a self-drop
			if leaf.tabs.len() == 1 && leaf.tabs[0].pane_id == dragged_pane {
				return None;
			}

			let r = leaf.full_rect;
			let w = r.width().max(1.0);
			let h = r.height().max(1.0);
			let rx = (local_pos.x - r.x()) / w;
			let ry = (local_pos.y - r.y()) / h;

			let zone = if leaf.tab_bar_rect.contains_point(local_pos) {
				DockZone::Center
			} else if rx < 0.24 {
				DockZone::Left
			} else if rx > 0.76 {
				DockZone::Right
			} else if ry < 0.24 {
				DockZone::Top
			} else if ry > 0.76 {
				DockZone::Bottom
			} else {
				DockZone::Center
			};

			let preview_rect = match zone {
				DockZone::Center => r,
				DockZone::Left => Rect::new(r.x(), r.y(), w * 0.5, h),
				DockZone::Right => Rect::new(r.x() + w * 0.5, r.y(), w * 0.5, h),
				DockZone::Top => Rect::new(r.x(), r.y(), w, h * 0.5),
				DockZone::Bottom => Rect::new(r.x(), r.y() + h * 0.5, w, h * 0.5),
			};

			// Pick an anchor pane in this leaf that isn't the dragged pane if possible
			let anchor_pane = leaf
				.tabs
				.iter()
				.find(|t| t.pane_id != dragged_pane)
				.map(|t| t.pane_id)
				.unwrap_or(leaf.anchor_pane);

			return Some(DropTarget {
				anchor_pane,
				zone,
				preview_rect,
			});
		}
		None
	}

	fn collect_layout_recursive(
		node: &DockNode,
		rect: Rect<f32>,
		path: &mut DockPath,
		splitter_size: f32,
		tab_bar_h: f32,
		font: Option<FontId>,
		font_size: f32,
		ctx: &mut LayoutCtx,
		leaves: &mut Vec<CachedLeaf>,
		splitters: &mut Vec<CachedSplitter>,
	) {
		match node {
			DockNode::Empty => {}
			DockNode::Tabs { panes, active } => {
				if panes.is_empty() {
					return;
				}
				let active_idx = (*active).min(panes.len() - 1);
				let bar_h = tab_bar_h.min(rect.height()).max(0.0);
				let tab_bar_rect = Rect::new(rect.x(), rect.y(), rect.width(), bar_h);
				let content_rect = Rect::new(
					rect.x(),
					rect.y() + bar_h,
					rect.width(),
					(rect.height() - bar_h).max(0.0),
				);

				let mut tabs = Vec::with_capacity(panes.len());
				let mut cursor_x = tab_bar_rect.x() + 4.0;

				for (i, &pane_id) in panes.iter().enumerate() {
					let (title, closable) = ctx
						.tree
						.get_widget::<DockPane>(pane_id)
						.map(|p| (p.title.clone(), p.closable))
						.unwrap_or_else(|| ("Pane".to_string(), true));

					let (text_w, _) = ctx.measure_text_with_font(&title, font, font_size);
					let close_extra = if closable { CLOSE_BTN_SIZE + 6.0 } else { 0.0 };
					let tab_w = (text_w + TAB_PADDING_X * 2.0 + close_extra).max(60.0);
					let tab_h = (bar_h - 4.0).max(16.0);
					let tab_rect = Rect::new(cursor_x, tab_bar_rect.y() + 4.0, tab_w, tab_h);

					let close_rect = if closable {
						Some(Rect::new(
							tab_rect.max.x - CLOSE_BTN_SIZE - 6.0,
							tab_rect.center().y - CLOSE_BTN_SIZE * 0.5,
							CLOSE_BTN_SIZE,
							CLOSE_BTN_SIZE,
						))
					} else {
						None
					};

					let is_active = i == active_idx;
					ctx.tree.set_visible(pane_id, is_active);
					if is_active {
						let c = BoxConstraints::tight(Vec2::new(
							content_rect.width().max(0.0),
							content_rect.height().max(0.0),
						));
						ctx.layout_child(pane_id, c);
						ctx.set_child_position(pane_id, content_rect.min);
					}

					tabs.push(CachedTab {
						pane_id,
						title,
						rect: tab_rect,
						close_rect,
						active: is_active,
					});

					cursor_x += tab_w + TAB_GAP;
				}

				leaves.push(CachedLeaf {
					path: path.clone(),
					anchor_pane: panes[active_idx],
					full_rect: rect,
					tab_bar_rect,
					tabs,
				});
			}
			DockNode::Split {
				axis,
				ratio,
				first,
				second,
			} => {
				let gap = splitter_size.max(4.0);
				let (r1, split_rect, r2) = match axis {
					DockSplitAxis::Horizontal => {
						let usable = (rect.width() - gap).max(0.0);
						let w1 = (usable * ratio.clamp(0.12, 0.88)).round();
						let w2 = (usable - w1).max(0.0);
						(
							Rect::new(rect.x(), rect.y(), w1, rect.height()),
							Rect::new(rect.x() + w1, rect.y(), gap, rect.height()),
							Rect::new(rect.x() + w1 + gap, rect.y(), w2, rect.height()),
						)
					}
					DockSplitAxis::Vertical => {
						let usable = (rect.height() - gap).max(0.0);
						let h1 = (usable * ratio.clamp(0.12, 0.88)).round();
						let h2 = (usable - h1).max(0.0);
						(
							Rect::new(rect.x(), rect.y(), rect.width(), h1),
							Rect::new(rect.x(), rect.y() + h1, rect.width(), gap),
							Rect::new(rect.x(), rect.y() + h1 + gap, rect.width(), h2),
						)
					}
				};

				splitters.push(CachedSplitter {
					path: path.clone(),
					axis: *axis,
					rect: split_rect,
					parent_rect: rect,
				});

				path.push(0);
				Self::collect_layout_recursive(
					first,
					r1,
					path,
					splitter_size,
					tab_bar_h,
					font,
					font_size,
					ctx,
					leaves,
					splitters,
				);
				path.pop();

				path.push(1);
				Self::collect_layout_recursive(
					second,
					r2,
					path,
					splitter_size,
					tab_bar_h,
					font,
					font_size,
					ctx,
					leaves,
					splitters,
				);
				path.pop();
			}
		}
	}
}

fn draw_close_x(canvas: &mut Canvas2D, center: Vec2<f32>, half: f32, thickness: f32, color: Color) {
	let steps = 8usize;
	for i in 0..=steps {
		let t = (i as f32 / steps as f32) * 2.0 - 1.0;
		let o = t * half;
		canvas.draw(RectShape::new(
			Rect::new(
				center.x + o - thickness * 0.5,
				center.y + o - thickness * 0.5,
				thickness,
				thickness,
			),
			color,
		));
		canvas.draw(RectShape::new(
			Rect::new(
				center.x + o - thickness * 0.5,
				center.y - o - thickness * 0.5,
				thickness,
				thickness,
			),
			color,
		));
	}
}

impl Widget for DockArea {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		ctx.set_clip_children(true);

		let children = ctx.children();
		// Auto-dock any newly attached DockPane children that aren't in the tree or explicitly closed
		for &child in &children {
			if self.closed_panes.contains(&child) {
				ctx.tree.set_visible(child, false);
				continue;
			}
			if !self.root.contains_pane(child) {
				let slot = ctx
					.tree
					.get_widget::<DockPane>(child)
					.map(|p| p.initial_slot)
					.unwrap_or(DockZone::Center);
				self.root.dock_here(child, slot);
			}
		}

		let final_size = constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				640.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				420.0
			},
		));

		self.leaves.clear();
		self.splitters.clear();

		let local_bounds = Rect::new(0.0, 0.0, final_size.x, final_size.y);
		let mut path = Vec::new();
		Self::collect_layout_recursive(
			&self.root,
			local_bounds,
			&mut path,
			self.splitter_size,
			self.tab_bar_height,
			self.font,
			self.font_size,
			ctx,
			&mut self.leaves,
			&mut self.splitters,
		);

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 1.0 || bounds.height() <= 1.0 {
			return;
		}

		let theme = ctx.theme;
		let monet = &theme.monet;
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainerLow]);
		let outer_r = self.corner_radius.unwrap_or(theme.radius.medium);
		canvas.draw(RectShape::new(bounds, bg).radius(outer_r));

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let pane_r = theme.radius.small;
		let tab_r = theme.radius.small.min(self.tab_bar_height * 0.4);

		// Draw each leaf frame + tab bar
		for leaf in &self.leaves {
			let full_abs = Rect::new(
				bounds.x() + leaf.full_rect.x(),
				bounds.y() + leaf.full_rect.y(),
				leaf.full_rect.width(),
				leaf.full_rect.height(),
			);
			let bar_abs = Rect::new(
				bounds.x() + leaf.tab_bar_rect.x(),
				bounds.y() + leaf.tab_bar_rect.y(),
				leaf.tab_bar_rect.width(),
				leaf.tab_bar_rect.height(),
			);

			// Leaf container & tab bar background
			canvas.draw(
				RectShape::new(full_abs, monet[MonetRole::SurfaceContainer])
					.radius(pane_r)
					.border(monet[MonetRole::SurfaceOutlineVariant], theme.stroke.thin),
			);
			canvas.draw(
				RectShape::new(bar_abs, monet[MonetRole::SurfaceContainerHigh])
					.radius4([pane_r, pane_r, 0.0, 0.0]),
			);

			canvas.push_clip(bar_abs);
			for (i, tab) in leaf.tabs.iter().enumerate() {
				let tab_abs = Rect::new(
					bounds.x() + tab.rect.x(),
					bounds.y() + tab.rect.y(),
					tab.rect.width(),
					tab.rect.height(),
				);
				let is_hovered = self.hovered_tab.as_ref() == Some(&(leaf.path.clone(), i));
				let tab_bg = if tab.active {
					monet[MonetRole::SurfaceContainer]
				} else if is_hovered {
					monet[MonetRole::SurfaceContainerHighest]
				} else {
					Color::TRANSPARENT
				};

				if tab.active || is_hovered {
					canvas.draw(
						RectShape::new(tab_abs, tab_bg).radius4([tab_r, tab_r, 0.0, 0.0]),
					);
				}

				if tab.active {
					let accent_h = theme.stroke.thick.clamp(1.5, 3.0);
					canvas.draw(RectShape::new(
						Rect::new(tab_abs.x() + 6.0, tab_abs.y(), (tab_abs.width() - 12.0).max(0.0), accent_h),
						monet[MonetRole::Primary],
					));
				}

				let (_, th) = canvas.measure_text(&tab.title, self.font_size);
				let text_col = if tab.active {
					monet[MonetRole::OnSurface]
				} else {
					monet[MonetRole::OnSurfaceVariant]
				};
				canvas.draw(TextShape::new(
					&tab.title,
					font,
					self.font_size,
					Vec2::new(tab_abs.x() + TAB_PADDING_X, tab_abs.center().y - th * 0.5),
					text_col,
				));

				if let Some(cr) = tab.close_rect {
					let cr_abs = Rect::new(
						bounds.x() + cr.x(),
						bounds.y() + cr.y(),
						cr.width(),
						cr.height(),
					);
					let close_hov = self.hovered_close.as_ref() == Some(&(leaf.path.clone(), i));
					if close_hov {
						canvas.draw(
							RectShape::new(cr_abs, monet[MonetRole::SurfaceContainerHighest])
								.radius(cr_abs.height() * 0.5),
						);
					}
					let x_col = if close_hov {
						monet[MonetRole::Error]
					} else {
						monet[MonetRole::OnSurfaceVariant]
					};
					draw_close_x(canvas, cr_abs.center(), 3.2, 1.3, x_col);
				}
			}
			canvas.pop_clip();
		}

		// Draw splitters
		for sp in &self.splitters {
			let sp_abs = Rect::new(
				bounds.x() + sp.rect.x(),
				bounds.y() + sp.rect.y(),
				sp.rect.width(),
				sp.rect.height(),
			);
			let active = self.splitter_drag.as_ref().is_some_and(|d| d.path == sp.path)
				|| self.hovered_splitter.as_ref() == Some(&sp.path);

			if active {
				let pill = match sp.axis {
					DockSplitAxis::Horizontal => Rect::new(
						sp_abs.center().x - 1.5,
						sp_abs.y() + 6.0,
						3.0,
						(sp_abs.height() - 12.0).max(0.0),
					),
					DockSplitAxis::Vertical => Rect::new(
						sp_abs.x() + 6.0,
						sp_abs.center().y - 1.5,
						(sp_abs.width() - 12.0).max(0.0),
						3.0,
					),
				};
				canvas.draw(RectShape::new(pill, monet[MonetRole::Primary]).radius(1.5));
			}
		}
	}

	fn draw_foreground(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let Some(drag) = &self.tab_drag else {
			return;
		};
		if !drag.active {
			return;
		}

		let theme = ctx.theme;
		let monet = &theme.monet;
		let bounds = ctx.bounds;

		// Drop-zone preview overlay on top of child panes
		if let Some(target) = &drag.drop_target {
			let pr = Rect::new(
				bounds.x() + target.preview_rect.x() + 2.0,
				bounds.y() + target.preview_rect.y() + 2.0,
				(target.preview_rect.width() - 4.0).max(0.0),
				(target.preview_rect.height() - 4.0).max(0.0),
			);
			let mut fill = monet[MonetRole::Primary];
			fill.a = 0.20;
			canvas.draw(
				RectShape::new(pr, fill)
					.radius(theme.radius.small)
					.border(monet[MonetRole::Primary], theme.stroke.thick),
			);
		}

		// Floating ghost tab following cursor
		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let (tw, th) = canvas.measure_text(&drag.title, self.font_size);
		let ghost_w = tw + TAB_PADDING_X * 2.0;
		let ghost_h = 26.0;
		let ghost_rect = Rect::new(
			drag.pointer.x - ghost_w * 0.5,
			drag.pointer.y - ghost_h * 0.5,
			ghost_w,
			ghost_h,
		);
		let mut ghost_bg = monet[MonetRole::SurfaceContainerHigh];
		ghost_bg.a = 0.94;
		canvas.draw(
			RectShape::new(ghost_rect, ghost_bg)
				.radius(theme.radius.small)
				.border(monet[MonetRole::Primary], theme.stroke.thick)
				.shadow(monet[MonetRole::Shadow], Vec2::new(0.0, 4.0), 10.0),
		);
		canvas.draw(TextShape::new(
			&drag.title,
			font,
			self.font_size,
			Vec2::new(ghost_rect.x() + TAB_PADDING_X, ghost_rect.center().y - th * 0.5),
			monet[MonetRole::OnSurface],
		));
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		match event {
			InputEvent::PointerMoved { position } => {
				let local = *position - bounds.min;

				if let Some(drag) = &self.splitter_drag {
					let new_ratio = match drag.axis {
						DockSplitAxis::Horizontal => {
							(local.x - drag.parent_rect.x()) / drag.parent_rect.width().max(1.0)
						}
						DockSplitAxis::Vertical => {
							(local.y - drag.parent_rect.y()) / drag.parent_rect.height().max(1.0)
						}
					};
					if self.root.set_ratio_at(&drag.path, new_ratio) {
						ctx.request_layout();
						ctx.request_redraw();
					}
					return EventResponse::Handled;
				}

				if let Some(drag) = &mut self.tab_drag {
					drag.pointer = *position;
					let d = *position - drag.start_pos;
					if !drag.active && (d.x * d.x + d.y * d.y).sqrt() >= DRAG_THRESHOLD {
						drag.active = true;
					}
					if drag.active {
						let dragged_id = drag.pane_id;
						let target = self.compute_drop_target(local, dragged_id);
						if let Some(drag) = &mut self.tab_drag {
							drag.drop_target = target;
						}
						ctx.request_redraw();
					}
					return EventResponse::Handled;
				}

				// Hover detection for splitters, tabs, and close buttons
				let new_sp = self
					.splitters
					.iter()
					.find(|s| s.rect.contains_point(local))
					.map(|s| s.path.clone());

				let mut new_tab = None;
				let mut new_close = None;
				if new_sp.is_none() {
					'outer: for leaf in &self.leaves {
						if !leaf.tab_bar_rect.contains_point(local) {
							continue;
						}
						for (i, tab) in leaf.tabs.iter().enumerate() {
							if tab.rect.contains_point(local) {
								new_tab = Some((leaf.path.clone(), i));
								if tab.close_rect.is_some_and(|cr| cr.contains_point(local)) {
									new_close = Some((leaf.path.clone(), i));
								}
								break 'outer;
							}
						}
					}
				}

				if new_sp != self.hovered_splitter
					|| new_tab != self.hovered_tab
					|| new_close != self.hovered_close
				{
					self.hovered_splitter = new_sp;
					self.hovered_tab = new_tab;
					self.hovered_close = new_close;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerLeft => {
				if self.hovered_splitter.is_some()
					|| self.hovered_tab.is_some()
					|| self.hovered_close.is_some()
				{
					self.hovered_splitter = None;
					self.hovered_tab = None;
					self.hovered_close = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				let local = *position - bounds.min;

				// Check splitters
				if let Some(sp) = self.splitters.iter().find(|s| s.rect.contains_point(local)) {
					self.splitter_drag = Some(SplitterDrag {
						path: sp.path.clone(),
						axis: sp.axis,
						parent_rect: sp.parent_rect,
					});
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				// Check tab close buttons & tabs
				for leaf in &self.leaves {
					if !leaf.tab_bar_rect.contains_point(local) {
						continue;
					}
					for (i, tab) in leaf.tabs.iter().enumerate() {
						if !tab.rect.contains_point(local) {
							continue;
						}
						if tab.close_rect.is_some_and(|cr| cr.contains_point(local)) {
							let pane_id = tab.pane_id;
							let title = tab.title.clone();
							if self.close_pane(pane_id) {
								if let Some(cb) = &self.on_pane_closed {
									cb.call1(title);
								}
								if let Some(cb) = &self.on_layout_changed {
									cb.call0();
								}
								ctx.request_layout();
								ctx.request_redraw();
							}
							return EventResponse::Handled;
						}

						// Activate tab immediately and arm potential drag
						let path = leaf.path.clone();
						let pane_id = tab.pane_id;
						let title = tab.title.clone();
						if self.root.set_active_at(&path, i) {
							ctx.request_layout();
							ctx.request_redraw();
						}
						self.tab_drag = Some(TabDrag {
							pane_id,
							title,
							start_pos: *position,
							pointer: *position,
							active: false,
							drop_target: None,
						});
						return EventResponse::Handled;
					}
				}

				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if self.splitter_drag.take().is_some() {
					if let Some(cb) = &self.on_layout_changed {
						cb.call0();
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				if let Some(drag) = self.tab_drag.take() {
					if drag.active
						&& let Some(target) = drag.drop_target
						&& self
							.root
							.move_pane_to(drag.pane_id, target.anchor_pane, target.zone)
					{
						if let Some(cb) = &self.on_layout_changed {
							cb.call0();
						}
						ctx.request_layout();
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}

	fn cursor(&self) -> Cursor {
		if let Some(drag) = &self.splitter_drag {
			return match drag.axis {
				DockSplitAxis::Horizontal => Cursor::EwResize,
				DockSplitAxis::Vertical => Cursor::NsResize,
			};
		}
		if self.tab_drag.as_ref().is_some_and(|d| d.active) {
			return Cursor::Grabbing;
		}
		if let Some(path) = &self.hovered_splitter
			&& let Some(splitter) = self.splitters.iter().find(|s| s.path == *path)
		{
			return match splitter.axis {
				DockSplitAxis::Horizontal => Cursor::EwResize,
				DockSplitAxis::Vertical => Cursor::NsResize,
			};
		}
		Cursor::Default
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"splitter_size" => self.splitter_size = value.expect_f32()?.clamp(4.0, 16.0),
			"tab_bar_height" => self.tab_bar_height = value.expect_f32()?.clamp(22.0, 48.0),
			"color" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			"font_size" => self.font_size = value.expect_f32()?.clamp(9.0, 24.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_pane_closed" | "on_layout_changed")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_pane_closed" => {
				self.on_pane_closed = Some(callback);
				true
			}
			"on_layout_changed" => {
				self.on_layout_changed = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct DockAreaBuilder {
	inner: DockArea,
}

impl DockAreaBuilder {
	pub fn new() -> Self {
		Self {
			inner: DockArea::default(),
		}
	}

	pub fn splitter_size(mut self, size: f32) -> Self {
		self.inner.splitter_size = size.clamp(4.0, 16.0);
		self
	}

	pub fn tab_bar_height(mut self, height: f32) -> Self {
		self.inner.tab_bar_height = height.clamp(22.0, 48.0);
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius.max(0.0));
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn font_size(mut self, size: f32) -> Self {
		self.inner.font_size = size.clamp(9.0, 24.0);
		self
	}

	pub fn on_pane_closed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_pane_closed = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_layout_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_layout_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> DockArea {
		self.inner
	}
}

impl From<DockAreaBuilder> for DockArea {
	fn from(builder: DockAreaBuilder) -> Self {
		builder.build()
	}
}
