/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;
use std::sync::Arc;

use nimbus_theme::Theme;
use slotmap::{SecondaryMap, SlotMap};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::Canvas2D;

use crate::dnd::{DRAG_THRESHOLD_PX, DragPayload, DragSession, DropEffect};
use crate::draw::DrawCtx;
use crate::event::{
	ButtonState, Cursor, EventCtx, EventResponse, InputEvent, Message, MessageTarget, MouseButton,
};
use crate::id::ObjectID;
use crate::layout::{BoxConstraints, LayoutCtx};
use crate::node::NodeData;
use crate::task::{IntoTask, Task, TaskQueue};
use crate::widget::Widget;


pub struct UiTree {
	pub nodes: SlotMap<ObjectID, NodeData>,
	pub widgets: SecondaryMap<ObjectID, Option<Box<dyn Widget>>>,
	pub roots: Vec<ObjectID>,
	pub hovered_node: Option<ObjectID>,
	pub pressed_node: Option<ObjectID>,
	pub focused_node: Option<ObjectID>,
	pub drag_session: Option<DragSession>,
	pub needs_layout: bool,
	pub needs_redraw: bool,
	pub task_queue: TaskQueue,
}

impl UiTree {
	pub fn new() -> Self {
		Self {
			nodes: SlotMap::with_key(),
			widgets: SecondaryMap::new(),
			roots: Vec::new(),
			hovered_node: None,
			pressed_node: None,
			focused_node: None,
			drag_session: None,
			needs_layout: true,
			needs_redraw: true,
			task_queue: TaskQueue::new(),
		}
	}


	pub fn insert_root<W: Widget>(&mut self, widget: W) -> ObjectID {
		let id = self.nodes.insert(NodeData::new());
		self.widgets.insert(id, Some(Box::new(widget)));
		self.roots.push(id);
		id
	}

	pub fn insert_root_boxed(&mut self, widget: Box<dyn Widget>) -> ObjectID {
		let id = self.nodes.insert(NodeData::new());
		self.widgets.insert(id, Some(widget));
		self.roots.push(id);
		id
	}

	pub fn insert_child<W: Widget>(&mut self, parent: ObjectID, widget: W) -> ObjectID {
		let mut node = NodeData::new();
		node.parent = Some(parent);
		let child_id = self.nodes.insert(node);
		self.widgets.insert(child_id, Some(Box::new(widget)));

		if let Some(parent_node) = self.nodes.get_mut(parent) {
			parent_node.children.push(child_id);
		}

		child_id
	}

	pub fn insert_child_boxed(&mut self, parent: ObjectID, widget: Box<dyn Widget>) -> ObjectID {
		let mut node = NodeData::new();
		node.parent = Some(parent);
		let child_id = self.nodes.insert(node);
		self.widgets.insert(child_id, Some(widget));

		if let Some(parent_node) = self.nodes.get_mut(parent) {
			parent_node.children.push(child_id);
		}

		child_id
	}

	pub fn insert_overlay<W: Widget>(&mut self, target: ObjectID, widget: W) -> ObjectID {
		self.insert_overlay_boxed(target, Box::new(widget))
	}

	pub fn insert_overlay_boxed(&mut self, target: ObjectID, widget: Box<dyn Widget>) -> ObjectID {
		let old_overlay = self.nodes.get(target).and_then(|n| n.overlay);
		if let Some(old_id) = old_overlay {
			self.remove(old_id);
		}

		let mut node = NodeData::new();
		node.parent = Some(target);
		let overlay_id = self.nodes.insert(node);
		self.widgets.insert(overlay_id, Some(widget));

		if let Some(target_node) = self.nodes.get_mut(target) {
			target_node.overlay = Some(overlay_id);
		}

		self.needs_layout = true;
		self.needs_redraw = true;
		overlay_id
	}

	pub fn set_overlay(&mut self, target: ObjectID, overlay: Option<ObjectID>) {
		if let Some(ov_id) = overlay {
			if ov_id == target || !self.nodes.contains_key(ov_id) || !self.nodes.contains_key(target) {
				return;
			}

			// Detach overlay from roots or any previous parent
			self.roots.retain(|&r| r != ov_id);
			if let Some(prev_parent) = self.nodes.get(ov_id).and_then(|n| n.parent)
				&& let Some(pnode) = self.nodes.get_mut(prev_parent)
			{
				pnode.children.retain(|&c| c != ov_id);
				if pnode.overlay == Some(ov_id) {
					pnode.overlay = None;
				}
			}

			if let Some(ov_node) = self.nodes.get_mut(ov_id) {
				ov_node.parent = Some(target);
			}
			if let Some(target_node) = self.nodes.get_mut(target) {
				target_node.overlay = Some(ov_id);
			}
		} else if let Some(target_node) = self.nodes.get_mut(target) {
			target_node.overlay = None;
		}

		self.needs_layout = true;
		self.needs_redraw = true;
	}

	pub fn clear_overlay(&mut self, target: ObjectID) {
		self.set_overlay(target, None);
	}

	pub fn overlay(&self, target: ObjectID) -> Option<ObjectID> {
		self.nodes.get(target).and_then(|n| n.overlay)
	}

	pub fn remove(&mut self, id: ObjectID) {
		let (children, overlay, parent) = match self.nodes.get(id) {
			Some(node) => (node.children.clone(), node.overlay, node.parent),
			None => return,
		};

		for child in children {
			self.remove(child);
		}

		if let Some(ov_id) = overlay {
			self.remove(ov_id);
		}

		if let Some(parent_id) = parent
			&& let Some(parent_node) = self.nodes.get_mut(parent_id)
		{
			parent_node.children.retain(|&c| c != id);
			if parent_node.overlay == Some(id) {
				parent_node.overlay = None;
			}
		}

		self.roots.retain(|&r| r != id);
		self.nodes.remove(id);
		self.widgets.remove(id);

		if self.hovered_node == Some(id) {
			self.hovered_node = None;
		}
		if self.pressed_node == Some(id) {
			self.pressed_node = None;
		}
		if self.focused_node == Some(id) {
			self.focused_node = None;
		}
	}


	pub fn node(&self, id: ObjectID) -> Option<&NodeData> {
		self.nodes.get(id)
	}

	pub fn node_mut(&mut self, id: ObjectID) -> Option<&mut NodeData> {
		self.nodes.get_mut(id)
	}


	pub fn children(&self, id: ObjectID) -> &[ObjectID] {
		self
			.nodes
			.get(id)
			.map(|n| n.children.as_slice())
			.unwrap_or(&[])
	}

	pub fn parent(&self, id: ObjectID) -> Option<ObjectID> {
		self.nodes.get(id).and_then(|n| n.parent)
	}

	pub fn bounds(&self, id: ObjectID) -> Rect<f32> {
		self.nodes.get(id).map(|n| n.bounds).unwrap_or(Rect::ZERO)
	}

	pub fn set_bounds(&mut self, id: ObjectID, bounds: Rect<f32>) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.bounds = bounds;
		}
	}

	pub fn is_visible(&self, id: ObjectID) -> bool {
		self.nodes.get(id).map(|n| n.visible).unwrap_or(false)
	}

	pub fn set_visible(&mut self, id: ObjectID, visible: bool) {
		let changed = if let Some(node) = self.nodes.get_mut(id) {
			if node.visible != visible {
				node.visible = visible;
				true
			} else {
				false
			}
		} else {
			false
		};

		if changed {
			if !visible {
				self.clear_interaction_in_subtree(id);
			}
			self.needs_layout = true;
			self.needs_redraw = true;
		}
	}

	pub fn is_enabled(&self, id: ObjectID) -> bool {
		self.nodes.get(id).map(|n| n.enabled).unwrap_or(false)
	}

	pub fn set_enabled(&mut self, id: ObjectID, enabled: bool) {
		let changed = if let Some(node) = self.nodes.get_mut(id) {
			if node.enabled != enabled {
				node.enabled = enabled;
				true
			} else {
				false
			}
		} else {
			false
		};

		if changed {
			if !enabled {
				self.clear_interaction_in_subtree(id);
			}
			self.needs_redraw = true;
		}
	}

	fn clear_interaction_in_subtree(&mut self, id: ObjectID) {
		let (children, overlay) = if let Some(node) = self.nodes.get_mut(id) {
			node.hovered = false;
			node.pressed = false;
			node.focused = false;
			(node.children.clone(), node.overlay)
		} else {
			return;
		};

		if self.hovered_node == Some(id) {
			self.hovered_node = None;
		}
		if self.pressed_node == Some(id) {
			self.pressed_node = None;
		}
		if self.focused_node == Some(id) {
			self.focused_node = None;
		}

		for child in children {
			self.clear_interaction_in_subtree(child);
		}
		if let Some(ov_id) = overlay {
			self.clear_interaction_in_subtree(ov_id);
		}
	}

	pub fn task_queue(&self) -> TaskQueue {
		self.task_queue.clone()
	}

	pub fn push_task<M, F: IntoTask<M>>(&self, task: F) {
		self.task_queue.push(task);
	}

	pub fn process_tasks(&mut self) -> bool {
		let mut executed = false;
		let mut deferred_window_tasks = Vec::new();

		for _ in 0..64 {
			let tasks = self.task_queue.drain_all();
			if tasks.is_empty() {
				break;
			}
			for task in tasks {
				match task {
					Task::Plain(f) => {
						f();
						executed = true;
					}
					Task::Tree(f) => {
						f(self);
						self.needs_redraw = true;
						executed = true;
					}
					other @ (Task::Window(_) | Task::Theme(_)) => {
						deferred_window_tasks.push(other);
					}
				}
			}
		}

		if !deferred_window_tasks.is_empty() {
			self.task_queue.push_front_all(deferred_window_tasks);
		}

		executed
	}


	pub fn get_widget<W: Widget>(&self, id: ObjectID) -> Option<&W> {
		self
			.widgets
			.get(id)
			.and_then(|w| w.as_ref())
			.and_then(|w| w.as_any().downcast_ref::<W>())
	}

	pub fn get_widget_mut<W: Widget>(&mut self, id: ObjectID) -> Option<&mut W> {
		self
			.widgets
			.get_mut(id)
			.and_then(|w| w.as_mut())
			.and_then(|w| w.as_any_mut().downcast_mut::<W>())
	}


	// Layout Pass

	pub fn layout_all(&mut self, window_size: Vec2<f32>) {
		let roots = self.roots.clone();
		let default_constraints = BoxConstraints::tight(window_size);
		for root_id in roots {
			if let Some(anchor) = self.anchor(root_id) {
				let constraints = anchor.child_constraints(window_size);
				let size = self.layout_node(root_id, constraints);
				let pos = anchor.position(window_size, size);
				let old_origin = self
					.nodes
					.get(root_id)
					.map(|n| n.bounds.min)
					.unwrap_or(Vec2::ZERO);
				let delta = pos - old_origin;
				if delta.x.abs() > f32::EPSILON || delta.y.abs() > f32::EPSILON {
					self.offset_subtree(root_id, delta);
				}
			} else {
				self.layout_node(root_id, default_constraints);
			}
		}
	}

	pub fn layout_node(&mut self, id: ObjectID, constraints: BoxConstraints) -> Vec2<f32> {
		if !self.is_visible(id) {
			if let Some(node) = self.nodes.get_mut(id) {
				node.bounds = Rect::new(node.bounds.x(), node.bounds.y(), 0.0, 0.0);
			}
			return Vec2::ZERO;
		}

		let all_children = self.children(id).to_vec();
		for child_id in all_children {
			if !self.is_visible(child_id)
				&& let Some(child_node) = self.nodes.get_mut(child_id)
			{
				child_node.bounds = Rect::new(child_node.bounds.x(), child_node.bounds.y(), 0.0, 0.0);
			}
		}

		let mut widget = match self.widgets.get_mut(id).and_then(|w| w.take()) {
			Some(w) => w,
			None => return Vec2::ZERO,
		};

		let mut ctx = LayoutCtx {
			tree: self,
			node_id: id,
		};

		let desired_size = widget.layout(&mut ctx, constraints);
		let final_size = constraints.constrain(desired_size);

		if let Some(node) = self.nodes.get_mut(id) {
			node.bounds = Rect::new(node.bounds.x(), node.bounds.y(), final_size.x, final_size.y);
		}

		self.widgets.insert(id, Some(widget));

		if let Some(ov_id) = self.nodes.get(id).and_then(|n| n.overlay) {
			if self.is_visible(ov_id) {
				if let Some(anchor) = self.anchor(ov_id) {
					let ov_constraints = anchor.child_constraints(final_size);
					let ov_size = self.layout_node(ov_id, ov_constraints);
					let ov_pos = anchor.position(final_size, ov_size);
					self.set_child_position(id, ov_id, ov_pos);
				} else {
					let ov_constraints = BoxConstraints::tight(final_size);
					self.layout_node(ov_id, ov_constraints);
					self.set_child_position(id, ov_id, Vec2::ZERO);
				}
			} else if let Some(ov_node) = self.nodes.get_mut(ov_id) {
				ov_node.bounds = Rect::new(ov_node.bounds.x(), ov_node.bounds.y(), 0.0, 0.0);
			}
		}

		final_size
	}

	pub fn set_child_position(
		&mut self,
		parent_id: ObjectID,
		child_id: ObjectID,
		local_pos: Vec2<f32>,
	) {
		let parent_origin = self
			.nodes
			.get(parent_id)
			.map(|p| p.bounds.min)
			.unwrap_or(Vec2::ZERO);

		let new_origin = parent_origin + local_pos;
		let old_origin = self
			.nodes
			.get(child_id)
			.map(|c| c.bounds.min)
			.unwrap_or(Vec2::ZERO);

		let delta = new_origin - old_origin;
		self.offset_subtree(child_id, delta);
	}

	fn offset_subtree(&mut self, id: ObjectID, delta: Vec2<f32>) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.bounds.min += delta;
			node.bounds.max += delta;
			let children = node.children.clone();
			let overlay = node.overlay;
			for child in children {
				self.offset_subtree(child, delta);
			}
			if let Some(ov_id) = overlay {
				self.offset_subtree(ov_id, delta);
			}
		}
	}


	pub fn set_clip_children(&mut self, id: ObjectID, clip: bool) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.clip_children = clip;
		}
	}

	pub fn anchor(&self, id: ObjectID) -> Option<crate::node::Anchor> {
		self.nodes.get(id).and_then(|n| n.anchor)
	}

	pub fn set_anchor(&mut self, id: ObjectID, anchor: Option<crate::node::Anchor>) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.anchor = anchor;
		}
	}

	// Draw Pass

	pub fn draw(&self, canvas: &mut Canvas2D, theme: &Theme) -> (bool, bool) {
		let redraw_requested = std::cell::Cell::new(false);
		let layout_requested = std::cell::Cell::new(false);
		for &root_id in &self.roots {
			self.draw_node(root_id, canvas, theme, &redraw_requested, &layout_requested);
		}
		if let Some(session) = &self.drag_session {
			session.draw_preview(canvas, theme);
		}
		(redraw_requested.get(), layout_requested.get())
	}

	fn draw_node(
		&self,
		id: ObjectID,
		canvas: &mut Canvas2D,
		theme: &Theme,
		redraw_requested: &std::cell::Cell<bool>,
		layout_requested: &std::cell::Cell<bool>,
	) {
		let node = match self.nodes.get(id) {
			Some(n) if n.visible => n,
			_ => return,
		};

		let draw_children = if let Some(Some(widget)) = self.widgets.get(id) {
			let ctx = DrawCtx {
				node_id: id,
				bounds: node.bounds,
				theme,
				hovered: node.hovered,
				pressed: node.pressed,
				focused: node.focused,
				redraw_requested,
				layout_requested,
			};
			widget.draw(&ctx, canvas);
			widget.should_draw_children()
		} else {
			true
		};

		if draw_children {
			if node.clip_children {
				canvas.push_clip(node.bounds);
			}

			for &child_id in &node.children {
				self.draw_node(child_id, canvas, theme, redraw_requested, layout_requested);
			}

			if node.clip_children {
				canvas.pop_clip();
			}
		}

		if let Some(Some(widget)) = self.widgets.get(id) {
			let ctx = DrawCtx {
				node_id: id,
				bounds: node.bounds,
				theme,
				hovered: node.hovered,
				pressed: node.pressed,
				focused: node.focused,
				redraw_requested,
				layout_requested,
			};
			widget.draw_foreground(&ctx, canvas);
		}

		if let Some(overlay_id) = node.overlay {
			canvas.push_clip(node.bounds);
			self.draw_node(overlay_id, canvas, theme, redraw_requested, layout_requested);
			canvas.pop_clip();
		}
	}


	// Event & Hit-Testing Pass

	pub fn active_cursor(&self) -> Cursor {
		if let Some(session) = &self.drag_session
			&& session.active
		{
			return if session.effect != DropEffect::None {
				Cursor::Grabbing
			} else {
				Cursor::NotAllowed
			};
		}

		if let Some(pressed_id) = self.pressed_node
			&& let Some(Some(widget)) = self.widgets.get(pressed_id)
		{
			let c = widget.cursor();
			if c != Cursor::Default {
				return c;
			}
		}

		let mut current = self.hovered_node;
		while let Some(id) = current {
			if let Some(Some(widget)) = self.widgets.get(id) {
				let c = widget.cursor();
				if c != Cursor::Default {
					return c;
				}
			}
			current = self.nodes.get(id).and_then(|n| n.parent);
		}

		Cursor::Default
	}

	pub fn hit_test(&self, pos: Vec2<f32>) -> Option<ObjectID> {
		for &root_id in self.roots.iter().rev() {
			if let Some(hit) = self.hit_test_node(root_id, pos) {
				return Some(hit);
			}
		}
		None
	}

	fn hit_test_node(&self, id: ObjectID, pos: Vec2<f32>) -> Option<ObjectID> {
		let node = self.nodes.get(id)?;
		if !node.visible || !node.enabled {
			return None;
		}

		if let Some(overlay_id) = node.overlay
			&& let Some(ov_node) = self.nodes.get(overlay_id)
			&& ov_node.visible
			&& ov_node.enabled
		{
			if !node.bounds.contains_point(pos) {
				return None;
			}
			return self.hit_test_node(overlay_id, pos).or(Some(overlay_id));
		}

		if node.clip_children && !node.bounds.contains_point(pos) {
			return None;
		}

		for &child_id in node.children.iter().rev() {
			if let Some(hit) = self.hit_test_node(child_id, pos) {
				return Some(hit);
			}
		}

		if node.bounds.contains_point(pos) {
			return Some(id);
		}

		None
	}

	pub fn dispatch_pointer_move(&mut self, pos: Vec2<f32>) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		if let Some(session) = &mut self.drag_session {
			session.pointer = pos;
			if !session.active {
				let d = pos - session.origin;
				if (d.x * d.x + d.y * d.y).sqrt() >= DRAG_THRESHOLD_PX {
					session.active = true;
					redraw_requested = true;
				}
			}
			if session.active {
				self.update_drag_hover(pos, &mut redraw_requested, &mut layout_requested);
				redraw_requested = true;
				if !self.task_queue.is_empty() && self.process_tasks() {
					redraw_requested = true;
				}
				if layout_requested {
					self.needs_layout = true;
				}
				if redraw_requested {
					self.needs_redraw = true;
				}
				return true;
			}
		}

		let hit_target = self.hit_test(pos);

		if hit_target != self.hovered_node {
			if let Some(old_id) = self.hovered_node {
				if let Some(node) = self.nodes.get_mut(old_id) {
					node.hovered = false;
				}
				self.send_event(
					old_id,
					&InputEvent::PointerLeft,
					&mut redraw_requested,
					&mut layout_requested,
				);
			}

			if let Some(new_id) = hit_target {
				if let Some(node) = self.nodes.get_mut(new_id) {
					node.hovered = true;
				}
				self.send_event(
					new_id,
					&InputEvent::PointerEntered,
					&mut redraw_requested,
					&mut layout_requested,
				);
			}

			self.hovered_node = hit_target;
			redraw_requested = true;
		}

		if let Some(target) = self.hovered_node {
			self.send_event(
				target,
				&InputEvent::PointerMoved { position: pos },
				&mut redraw_requested,
				&mut layout_requested,
			);
		}

		if let Some(pressed) = self.pressed_node {
			if Some(pressed) != self.hovered_node {
				self.send_event(
					pressed,
					&InputEvent::PointerMoved { position: pos },
					&mut redraw_requested,
					&mut layout_requested,
				);
			}
		}

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}

		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}

		redraw_requested || layout_requested
	}

	pub fn dispatch_pointer_button(
		&mut self,
		button: MouseButton,
		state: ButtonState,
		pos: Vec2<f32>,
	) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		match state {
			ButtonState::Pressed => {
				let hit = self.hit_test(pos);
				let prev_focused = self.focused_node;
				if let Some(prev) = prev_focused {
					if hit != Some(prev) {
						if let Some(node) = self.nodes.get_mut(prev) {
							node.focused = false;
						}
						self.focused_node = None;
						self.send_event(
							prev,
							&InputEvent::FocusLost,
							&mut redraw_requested,
							&mut layout_requested,
						);
						redraw_requested = true;
					}
				}
				if let Some(target) = hit {
					if let Some(node) = self.nodes.get_mut(target) {
						node.pressed = true;
						node.focused = true;
					}
					self.pressed_node = Some(target);
					self.focused_node = Some(target);

					if prev_focused != Some(target) {
						self.send_event(
							target,
							&InputEvent::FocusGained,
							&mut redraw_requested,
							&mut layout_requested,
						);
					}

					self.send_event(
						target,
						&InputEvent::PointerButton {
							button,
							state,
							position: pos,
						},
						&mut redraw_requested,
						&mut layout_requested,
					);
					redraw_requested = true;
				}
			}

			ButtonState::Released => {
				if button == MouseButton::Left
					&& let Some(session) = self.drag_session.take()
				{
					if session.active {
						let mut accepted = false;
						if let Some(target) = session.current_target {
							if session.effect != DropEffect::None {
								self.send_event(
									target,
									&InputEvent::Drop {
										position: pos,
										payload: session.payload.clone(),
									},
									&mut redraw_requested,
									&mut layout_requested,
								);
								accepted = true;
							} else {
								self.send_event(
									target,
									&InputEvent::DragLeave,
									&mut redraw_requested,
									&mut layout_requested,
								);
							}
						}
						if let Some(source) = session.source {
							self.send_event(
								source,
								&InputEvent::DragEnded {
									accepted,
									effect: if accepted {
										session.effect
									} else {
										DropEffect::None
									},
								},
								&mut redraw_requested,
								&mut layout_requested,
							);
						}
						if let Some(pressed) = self.pressed_node.take()
							&& let Some(node) = self.nodes.get_mut(pressed)
						{
							node.pressed = false;
						}
						redraw_requested = true;

						if !self.task_queue.is_empty() && self.process_tasks() {
							redraw_requested = true;
						}
						if layout_requested {
							self.needs_layout = true;
						}
						if redraw_requested {
							self.needs_redraw = true;
						}
						return true;
					}
				}

				if let Some(target) = self.pressed_node.take() {
					if let Some(node) = self.nodes.get_mut(target) {
						node.pressed = false;
					}

					self.send_event(
						target,
						&InputEvent::PointerButton {
							button,
							state,
							position: pos,
						},
						&mut redraw_requested,
						&mut layout_requested,
					);

					if self.hit_test(pos) == Some(target) {
						self.send_event(
							target,
							&InputEvent::Click { position: pos },
							&mut redraw_requested,
							&mut layout_requested,
						);
					}

					redraw_requested = true;
				}
			}
		}

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}

		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}

		redraw_requested || layout_requested
	}

	pub fn dispatch_key(&mut self, event: &InputEvent) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		if self.drag_session.as_ref().is_some_and(|s| s.active)
			&& matches!(event, InputEvent::Key { key_name, pressed: true, .. } if key_name == "Escape")
		{
			self.cancel_drag(&mut redraw_requested, &mut layout_requested);
			if !self.task_queue.is_empty() && self.process_tasks() {
				redraw_requested = true;
			}
			if layout_requested {
				self.needs_layout = true;
			}
			if redraw_requested {
				self.needs_redraw = true;
			}
			return true;
		}

		if let Some(target) = self.focused_node {
			self.send_event(
				target,
				event,
				&mut redraw_requested,
				&mut layout_requested,
			);
		}

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}

		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}

		redraw_requested || layout_requested
	}

	pub fn dispatch_pointer_scroll(&mut self, pos: Vec2<f32>, delta: Vec2<f32>) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		let hit = self.hit_test(pos);
		let mut curr = hit;

		while let Some(target) = curr {
			let resp = self.send_event(
				target,
				&InputEvent::PointerScroll { delta },
				&mut redraw_requested,
				&mut layout_requested,
			);

			if resp == EventResponse::Handled {
				redraw_requested = true;
				break;
			}

			curr = self.nodes.get(target).and_then(|n| n.parent);
		}

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}

		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}

		redraw_requested || layout_requested
	}

	fn update_drag_hover(
		&mut self,
		pos: Vec2<f32>,
		redraw_requested: &mut bool,
		layout_requested: &mut bool,
	) {
		let (payload, prev_target) = match &self.drag_session {
			Some(s) => (s.payload.clone(), s.current_target),
			None => return,
		};

		let mut new_target = None;
		let mut new_effect = DropEffect::None;
		let mut curr = self.hit_test(pos);

		while let Some(id) = curr {
			let (resp, effect) = self.send_event_ex(
				id,
				&InputEvent::DragOver {
					position: pos,
					payload: payload.clone(),
				},
				redraw_requested,
				layout_requested,
			);
			if let Some(eff) = effect
				&& eff != DropEffect::None
			{
				new_target = Some(id);
				new_effect = eff;
				break;
			}
			if resp == EventResponse::Handled {
				new_target = Some(id);
				new_effect = effect.unwrap_or(DropEffect::None);
				break;
			}
			curr = self.nodes.get(id).and_then(|n| n.parent);
		}

		if new_target != prev_target {
			if let Some(old) = prev_target {
				self.send_event(old, &InputEvent::DragLeave, redraw_requested, layout_requested);
			}
			if let Some(new_id) = new_target {
				self.send_event(
					new_id,
					&InputEvent::DragEnter {
						position: pos,
						payload,
					},
					redraw_requested,
					layout_requested,
				);
			}
			*redraw_requested = true;
		}

		if let Some(session) = &mut self.drag_session {
			session.current_target = new_target;
			session.effect = new_effect;
		}
	}

	fn cancel_drag(&mut self, redraw_requested: &mut bool, layout_requested: &mut bool) {
		if let Some(session) = self.drag_session.take() {
			if let Some(target) = session.current_target {
				self.send_event(target, &InputEvent::DragLeave, redraw_requested, layout_requested);
			}
			if let Some(source) = session.source {
				self.send_event(
					source,
					&InputEvent::DragEnded {
						accepted: false,
						effect: DropEffect::None,
					},
					redraw_requested,
					layout_requested,
				);
			}
			if let Some(pressed) = self.pressed_node.take()
				&& let Some(node) = self.nodes.get_mut(pressed)
			{
				node.pressed = false;
			}
			*redraw_requested = true;
		}
	}

	fn find_fallback_drop_target(
		&mut self,
		pos: Vec2<f32>,
		payload: &Arc<DragPayload>,
		redraw_requested: &mut bool,
		layout_requested: &mut bool,
	) -> Option<(ObjectID, DropEffect)> {
		let mut visible_ids = Vec::new();
		for &root_id in self.roots.iter().rev() {
			self.collect_visible_postorder(root_id, &mut visible_ids);
		}
		for id in visible_ids {
			let (_, effect) = self.send_event_ex(
				id,
				&InputEvent::DragOver {
					position: pos,
					payload: payload.clone(),
				},
				redraw_requested,
				layout_requested,
			);
			if let Some(eff) = effect
				&& eff != DropEffect::None
			{
				return Some((id, eff));
			}
			self.send_event(id, &InputEvent::DragLeave, redraw_requested, layout_requested);
		}
		None
	}

	fn collect_visible_postorder(&self, id: ObjectID, out: &mut Vec<ObjectID>) {
		let Some(node) = self.nodes.get(id) else {
			return;
		};
		if !node.visible || !node.enabled {
			return;
		}
		if let Some(ov) = node.overlay {
			self.collect_visible_postorder(ov, out);
		}
		for &child in node.children.iter().rev() {
			self.collect_visible_postorder(child, out);
		}
		out.push(id);
	}

	pub fn dispatch_os_file_hover(&mut self, path: PathBuf, pos: Vec2<f32>) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		let payload = Arc::new(DragPayload::files(vec![path]));
		let prev_target = self.drag_session.as_ref().and_then(|s| s.current_target);
		self.drag_session = Some(DragSession {
			source: None,
			origin: pos,
			pointer: pos,
			payload: payload.clone(),
			active: true,
			current_target: prev_target,
			effect: DropEffect::None,
		});
		self.update_drag_hover(pos, &mut redraw_requested, &mut layout_requested);

		if self
			.drag_session
			.as_ref()
			.is_some_and(|s| s.effect == DropEffect::None)
			&& let Some((fallback_id, eff)) = self.find_fallback_drop_target(
				pos,
				&payload,
				&mut redraw_requested,
				&mut layout_requested,
			)
		{
			if let Some(session) = &mut self.drag_session {
				session.current_target = Some(fallback_id);
				session.effect = eff;
			}
			self.send_event(
				fallback_id,
				&InputEvent::DragEnter {
					position: pos,
					payload,
				},
				&mut redraw_requested,
				&mut layout_requested,
			);
		}
		redraw_requested = true;

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}
		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}
		true
	}

	pub fn dispatch_os_file_drop(&mut self, path: PathBuf, pos: Vec2<f32>) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;

		let payload = Arc::new(DragPayload::files(vec![path]));
		let prev_target = self.drag_session.as_ref().and_then(|s| s.current_target);
		self.drag_session = Some(DragSession {
			source: None,
			origin: pos,
			pointer: pos,
			payload: payload.clone(),
			active: true,
			current_target: prev_target,
			effect: DropEffect::None,
		});
		self.update_drag_hover(pos, &mut redraw_requested, &mut layout_requested);

		if self
			.drag_session
			.as_ref()
			.is_some_and(|s| s.effect == DropEffect::None)
			&& let Some((fallback_id, eff)) = self.find_fallback_drop_target(
				pos,
				&payload,
				&mut redraw_requested,
				&mut layout_requested,
			)
			&& let Some(session) = &mut self.drag_session
		{
			session.current_target = Some(fallback_id);
			session.effect = eff;
		}

		if let Some(session) = self.drag_session.take()
			&& let Some(target) = session.current_target
		{
			if session.effect != DropEffect::None {
				self.send_event(
					target,
					&InputEvent::Drop {
						position: pos,
						payload,
					},
					&mut redraw_requested,
					&mut layout_requested,
				);
			} else {
				self.send_event(
					target,
					&InputEvent::DragLeave,
					&mut redraw_requested,
					&mut layout_requested,
				);
			}
		}
		redraw_requested = true;

		if !self.task_queue.is_empty() && self.process_tasks() {
			redraw_requested = true;
		}
		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}
		true
	}

	pub fn dispatch_os_file_cancel(&mut self) -> bool {
		let mut redraw_requested = false;
		let mut layout_requested = false;
		if self.drag_session.as_ref().is_some_and(|s| s.source.is_none()) {
			self.cancel_drag(&mut redraw_requested, &mut layout_requested);
		}
		if layout_requested {
			self.needs_layout = true;
		}
		if redraw_requested {
			self.needs_redraw = true;
		}
		redraw_requested || layout_requested
	}

	fn send_event(
		&mut self,
		id: ObjectID,
		event: &InputEvent,
		redraw_requested: &mut bool,
		layout_requested: &mut bool,
	) -> EventResponse {
		self.send_event_ex(id, event, redraw_requested, layout_requested).0
	}

	fn send_event_ex(
		&mut self,
		id: ObjectID,
		event: &InputEvent,
		redraw_requested: &mut bool,
		layout_requested: &mut bool,
	) -> (EventResponse, Option<DropEffect>) {
		let mut widget = match self.widgets.get_mut(id).and_then(|w| w.take()) {
			Some(w) => w,
			None => return (EventResponse::Ignored, None),
		};

		let mut ctx = EventCtx {
			node_id: id,
			redraw_requested,
			layout_requested,
			messages: Vec::new(),
			task_queue: self.task_queue.clone(),
			pending_drag: None,
			drop_effect: None,
		};

		let resp = widget.handle_event(&mut ctx, event);
		self.widgets.insert(id, Some(widget));

		if let Some((origin, payload)) = ctx.pending_drag {
			self.drag_session = Some(DragSession {
				source: Some(id),
				origin,
				pointer: origin,
				payload: Arc::new(payload),
				active: false,
				current_target: None,
				effect: DropEffect::None,
			});
		}

		let drop_effect = ctx.drop_effect;

		if !ctx.messages.is_empty() {
			self.process_messages(ctx.messages);
		}

		if !self.task_queue.is_empty() && self.process_tasks() {
			*redraw_requested = true;
		}

		if *layout_requested {
			self.needs_layout = true;
		}
		if *redraw_requested {
			self.needs_redraw = true;
		}
		(resp, drop_effect)
	}

	pub fn process_messages(&mut self, initial_messages: Vec<Message>) {
		let mut queue = std::collections::VecDeque::from(initial_messages);
		let mut redraw = false;
		let mut layout = false;

		while let Some(msg) = queue.pop_front() {
			let targets: Vec<ObjectID> = match msg.target {
				MessageTarget::Siblings => {
					if let Some(parent_id) = self.nodes.get(msg.sender).and_then(|n| n.parent) {
						if let Some(parent_node) = self.nodes.get(parent_id) {
							parent_node
								.children
								.iter()
								.copied()
								.filter(|&id| id != msg.sender)
								.collect()
						} else {
							Vec::new()
						}
					} else {
						Vec::new()
					}
				}
				MessageTarget::Parent => {
					self.nodes
						.get(msg.sender)
						.and_then(|n| n.parent)
						.into_iter()
						.collect()
				}
				MessageTarget::Children => {
					self.nodes
						.get(msg.sender)
						.map(|n| n.children.clone())
						.unwrap_or_default()
				}
				MessageTarget::Root => self.roots.clone(),
			};

			for target_id in targets {
				if let Some(mut target_widget) = self.widgets.get_mut(target_id).and_then(|w| w.take()) {
					let mut target_ctx = EventCtx {
						node_id: target_id,
						redraw_requested: &mut redraw,
						layout_requested: &mut layout,
						messages: Vec::new(),
						task_queue: self.task_queue.clone(),
						pending_drag: None,
						drop_effect: None,
					};

					let _ = target_widget.handle_message(&mut target_ctx, &msg.name, &msg.payload);
					self.widgets.insert(target_id, Some(target_widget));

					for child_msg in target_ctx.messages {
						queue.push_back(child_msg);
					}
				}
			}
		}

		if redraw {
			self.needs_redraw = true;
		}
		if layout {
			self.needs_layout = true;
		}
	}
}

impl Default for UiTree {
	fn default() -> Self {
		Self::new()
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::sync::Arc;

	struct TestWidget {
		size: Vec2<f32>,
		on_click: Option<Box<dyn FnMut()>>,
	}

	impl TestWidget {
		fn new(w: f32, h: f32) -> Self {
			Self {
				size: Vec2::new(w, h),
				on_click: None,
			}
		}

		fn with_on_click(mut self, cb: impl FnMut() + 'static) -> Self {
			self.on_click = Some(Box::new(cb));
			self
		}
	}

	impl Widget for TestWidget {
		fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
			let children = ctx.children();
			let mut child_y = 0.0f32;
			let child_constraints = BoxConstraints::loose(constraints.max_size());
			for child in children {
				let sz = ctx.layout_child(child, child_constraints);
				ctx.set_child_position(child, Vec2::new(0.0, child_y));
				child_y += sz.y;
			}
			constraints.constrain(self.size)
		}

		fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}

		fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
			if let InputEvent::Click { .. } = event
				&& let Some(cb) = &mut self.on_click
			{
				cb();
				ctx.request_redraw();
				return EventResponse::Handled;
			}
			EventResponse::Ignored
		}
	}

	#[test]
	fn test_tree_hierarchy_and_lifecycle() {
		let mut tree = UiTree::new();

		let root = tree.insert_root(TestWidget::new(200.0, 100.0));
		let child1 = tree.insert_child(root, TestWidget::new(50.0, 20.0));
		let child2 = tree.insert_child(root, TestWidget::new(50.0, 20.0));

		assert_eq!(tree.roots.len(), 1);
		assert_eq!(tree.roots[0], root);
		assert_eq!(tree.children(root), &[child1, child2]);
		assert_eq!(tree.parent(child1), Some(root));
		assert_eq!(tree.parent(child2), Some(root));

		// Layout (tight window constraints fill the root)
		tree.layout_all(Vec2::new(800.0, 600.0));
		let root_bounds = tree.bounds(root);
		assert_eq!(root_bounds.size(), Vec2::new(800.0, 600.0));
		let child_bounds = tree.bounds(child1);
		assert_eq!(child_bounds.size(), Vec2::new(50.0, 20.0));

		// Recursive removal
		tree.remove(root);
		assert_eq!(tree.roots.len(), 0);
		assert!(tree.node(root).is_none());
		assert!(tree.node(child1).is_none());
		assert!(tree.node(child2).is_none());
	}

	#[test]
	fn test_widget_click_event() {
		let mut tree = UiTree::new();
		let counter = Arc::new(AtomicUsize::new(0));

		let counter_clone = counter.clone();
		let widget_id = tree.insert_root(TestWidget::new(100.0, 40.0).with_on_click(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
		}));

		tree.layout_all(Vec2::new(400.0, 300.0));
		let bounds = tree.bounds(widget_id);
		let center = bounds.center();

		// Move pointer to widget
		let redraw = tree.dispatch_pointer_move(center);
		assert!(redraw);
		assert_eq!(tree.hovered_node, Some(widget_id));

		// Press mouse button
		let redraw = tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, center);
		assert!(redraw);
		assert_eq!(tree.pressed_node, Some(widget_id));

		// Release mouse button to trigger click
		let redraw = tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Released, center);
		assert!(redraw);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	#[test]
	fn test_message_dispatch_siblings() {
		use crate::property::PropertyValue;

		struct MockItem {
			active: bool,
		}
		impl Widget for MockItem {
			fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
				constraints.constrain(Vec2::new(20.0, 20.0))
			}
			fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}
			fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
				if let InputEvent::Click { .. } = event {
					if !self.active {
						self.active = true;
						ctx.dispatch_message(
							MessageTarget::Siblings,
							"deactivate",
							PropertyValue::Bool(false),
						);
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
				EventResponse::Ignored
			}
			fn handle_message(
				&mut self,
				ctx: &mut EventCtx,
				name: &str,
				_payload: &PropertyValue,
			) -> bool {
				if name == "deactivate" && self.active {
					self.active = false;
					ctx.request_redraw();
					return true;
				}
				false
			}
		}

		let mut tree = UiTree::new();
		let root = tree.insert_root(TestWidget::new(200.0, 100.0));
		let i1 = tree.insert_child(root, MockItem { active: true });
		let i2 = tree.insert_child(root, MockItem { active: false });

		tree.layout_all(Vec2::new(200.0, 100.0));
		let i2_pos = tree.bounds(i2).center();

		// Click i2
		tree.dispatch_pointer_move(i2_pos);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, i2_pos);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Released, i2_pos);

		// Now i1 should be inactive and i2 should be active!
		let i1_widget = tree.widgets.get(i1).unwrap().as_ref().unwrap();
		let i2_widget = tree.widgets.get(i2).unwrap().as_ref().unwrap();
		let i1_item = i1_widget.as_any().downcast_ref::<MockItem>().unwrap();
		let i2_item = i2_widget.as_any().downcast_ref::<MockItem>().unwrap();
		assert!(!i1_item.active);
		assert!(i2_item.active);
	}

	#[test]
	fn test_visible_excludes_hidden_children_from_layout() {
		let mut tree = UiTree::new();
		let root = tree.insert_root(TestWidget::new(200.0, 200.0));
		let c1 = tree.insert_child(root, TestWidget::new(80.0, 30.0));
		let c2 = tree.insert_child(root, TestWidget::new(80.0, 40.0));

		tree.layout_all(Vec2::new(200.0, 200.0));
		assert_eq!(tree.bounds(c1).size(), Vec2::new(80.0, 30.0));
		assert_eq!(tree.bounds(c2).min.y, 30.0);
		assert_eq!(tree.bounds(c2).size(), Vec2::new(80.0, 40.0));

		tree.needs_layout = false;
		tree.needs_redraw = false;
		tree.set_visible(c1, false);
		assert!(tree.needs_layout);
		assert!(tree.needs_redraw);

		tree.layout_all(Vec2::new(200.0, 200.0));
		assert_eq!(tree.bounds(c1).size(), Vec2::ZERO);
		assert_eq!(tree.bounds(c2).min.y, 0.0);
		assert_eq!(tree.bounds(c2).size(), Vec2::new(80.0, 40.0));

		tree.needs_layout = false;
		tree.needs_redraw = false;
		tree.set_visible(c1, true);
		assert!(tree.needs_layout);
		assert!(tree.needs_redraw);

		tree.layout_all(Vec2::new(200.0, 200.0));
		assert_eq!(tree.bounds(c1).size(), Vec2::new(80.0, 30.0));
		assert_eq!(tree.bounds(c2).min.y, 30.0);
	}

	#[test]
	fn test_task_queue_cross_thread_and_plain_tasks() {
		let mut tree = UiTree::new();
		let root = tree.insert_root(TestWidget::new(100.0, 100.0));
		let child = tree.insert_child(root, TestWidget::new(50.0, 25.0));

		let ran = Arc::new(AtomicUsize::new(0));
		let ran_clone = ran.clone();

		let sender = tree.task_queue();
		sender.push(move || {
			ran_clone.fetch_add(1, Ordering::SeqCst);
		});

		let sender_thread = tree.task_queue();
		let handle = std::thread::spawn(move || {
			sender_thread.push(move |tree: &mut UiTree| {
				tree.set_visible(child, false);
			});
		});
		handle.join().unwrap();

		assert!(tree.process_tasks());
		assert_eq!(ran.load(Ordering::SeqCst), 1);
		assert!(!tree.is_visible(child));
		assert!(tree.needs_layout);
		assert!(tree.needs_redraw);
	}

	struct MockDragSource {
		ended: Option<bool>,
	}

	impl Widget for MockDragSource {
		fn layout(&mut self, _ctx: &mut LayoutCtx, c: BoxConstraints) -> Vec2<f32> {
			c.constrain(Vec2::new(80.0, 40.0))
		}
		fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}
		fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
			match event {
				InputEvent::PointerButton {
					button: MouseButton::Left,
					state: ButtonState::Pressed,
					position,
				} => {
					ctx.begin_drag(*position, DragPayload::text("test/item", "hello"));
					EventResponse::Handled
				}
				InputEvent::DragEnded { accepted, .. } => {
					self.ended = Some(*accepted);
					EventResponse::Handled
				}
				_ => EventResponse::Ignored,
			}
		}
	}

	struct MockDropTarget {
		accept_kind: &'static str,
		dropped: Option<String>,
	}

	impl Widget for MockDropTarget {
		fn layout(&mut self, _ctx: &mut LayoutCtx, c: BoxConstraints) -> Vec2<f32> {
			c.constrain(Vec2::new(80.0, 40.0))
		}
		fn draw(&self, _ctx: &DrawCtx, _canvas: &mut Canvas2D) {}
		fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
			match event {
				InputEvent::DragOver { payload, .. } => {
					if payload.kind == self.accept_kind {
						ctx.set_drop_effect(DropEffect::Copy);
					} else {
						ctx.set_drop_effect(DropEffect::None);
					}
					EventResponse::Handled
				}
				InputEvent::Drop { payload, .. } => {
					self.dropped = payload.downcast_ref::<String>().cloned();
					EventResponse::Handled
				}
				_ => EventResponse::Ignored,
			}
		}
	}

	#[test]
	fn test_drag_and_drop_lifecycle_and_cancel() {
		let mut tree = UiTree::new();
		let root = tree.insert_root(TestWidget::new(200.0, 200.0));
		let src = tree.insert_child(root, MockDragSource { ended: None });
		let dst_ok = tree.insert_child(
			root,
			MockDropTarget {
				accept_kind: "test/item",
				dropped: None,
			},
		);
		let dst_no = tree.insert_child(
			root,
			MockDropTarget {
				accept_kind: "other/kind",
				dropped: None,
			},
		);
		tree.layout_all(Vec2::new(200.0, 200.0));

		// Press on source (y: 0..40)
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, Vec2::new(10.0, 10.0));
		assert!(tree.drag_session.is_some());
		assert!(!tree.drag_session.as_ref().unwrap().active);

		// Move over rejecting target (y: 80..120) -> activates drag, cursor is NotAllowed
		tree.dispatch_pointer_move(Vec2::new(10.0, 90.0));
		assert!(tree.drag_session.as_ref().unwrap().active);
		assert_eq!(tree.active_cursor(), Cursor::NotAllowed);

		// Move over accepting target (y: 40..80) -> cursor is Grabbing
		tree.dispatch_pointer_move(Vec2::new(10.0, 50.0));
		assert_eq!(tree.active_cursor(), Cursor::Grabbing);

		// Release -> drops on dst_ok, notifies src
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Released, Vec2::new(10.0, 50.0));
		assert!(tree.drag_session.is_none());
		assert_eq!(
			tree.get_widget::<MockDropTarget>(dst_ok).unwrap().dropped.as_deref(),
			Some("hello")
		);
		assert_eq!(tree.get_widget::<MockDropTarget>(dst_no).unwrap().dropped, None);
		assert_eq!(tree.get_widget::<MockDragSource>(src).unwrap().ended, Some(true));

		// Start another drag and cancel with Escape
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, Vec2::new(10.0, 10.0));
		tree.dispatch_pointer_move(Vec2::new(10.0, 50.0));
		tree.dispatch_key(&InputEvent::Key {
			key_name: "Escape".to_string(),
			text: None,
			pressed: true,
		});
		assert!(tree.drag_session.is_none());
		assert_eq!(tree.get_widget::<MockDragSource>(src).unwrap().ended, Some(false));
	}
}
