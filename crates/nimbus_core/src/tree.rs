/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_theme::Theme;
use slotmap::{SecondaryMap, SlotMap};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::Canvas2D;

use crate::draw::DrawCtx;
use crate::event::{ButtonState, EventCtx, EventResponse, InputEvent, MouseButton};
use crate::id::ObjectID;
use crate::layout::{BoxConstraints, LayoutCtx};
use crate::node::NodeData;
use crate::widget::Widget;


pub struct UiTree {
	pub nodes: SlotMap<ObjectID, NodeData>,
	pub widgets: SecondaryMap<ObjectID, Option<Box<dyn Widget>>>,
	pub roots: Vec<ObjectID>,
	pub hovered_node: Option<ObjectID>,
	pub pressed_node: Option<ObjectID>,
	pub focused_node: Option<ObjectID>,
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

	pub fn remove(&mut self, id: ObjectID) {
		let (children, parent) = match self.nodes.get(id) {
			Some(node) => (node.children.clone(), node.parent),
			None => return,
		};

		for child in children {
			self.remove(child);
		}

		if let Some(parent_id) = parent
			&& let Some(parent_node) = self.nodes.get_mut(parent_id)
		{
			parent_node.children.retain(|&c| c != id);
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
		let constraints = BoxConstraints::tight(window_size);
		for root_id in roots {
			self.layout_node(root_id, constraints);
		}
	}

	pub fn layout_node(&mut self, id: ObjectID, constraints: BoxConstraints) -> Vec2<f32> {
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
			for child in children {
				self.offset_subtree(child, delta);
			}
		}
	}


	pub fn set_clip_children(&mut self, id: ObjectID, clip: bool) {
		if let Some(node) = self.nodes.get_mut(id) {
			node.clip_children = clip;
		}
	}

	// Draw Pass

	pub fn draw(&self, canvas: &mut Canvas2D, theme: &Theme) {
		for &root_id in &self.roots {
			self.draw_node(root_id, canvas, theme);
		}
	}

	fn draw_node(&self, id: ObjectID, canvas: &mut Canvas2D, theme: &Theme) {
		let node = match self.nodes.get(id) {
			Some(n) if n.visible => n,
			_ => return,
		};

		if let Some(Some(widget)) = self.widgets.get(id) {
			let ctx = DrawCtx {
				node_id: id,
				bounds: node.bounds,
				theme,
				hovered: node.hovered,
				pressed: node.pressed,
				focused: node.focused,
			};
			widget.draw(&ctx, canvas);
		}

		if node.clip_children {
			canvas.push_clip(node.bounds);
		}

		for &child_id in &node.children {
			self.draw_node(child_id, canvas, theme);
		}

		if node.clip_children {
			canvas.pop_clip();
		}
	}


	// Event & Hit-Testing Pass

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

		redraw_requested
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
				if let Some(target) = hit {
					if let Some(node) = self.nodes.get_mut(target) {
						node.pressed = true;
						node.focused = true;
					}
					self.pressed_node = Some(target);
					self.focused_node = Some(target);

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

		redraw_requested
	}

	fn send_event(
		&mut self,
		id: ObjectID,
		event: &InputEvent,
		redraw_requested: &mut bool,
		layout_requested: &mut bool,
	) -> EventResponse {
		let mut widget = match self.widgets.get_mut(id).and_then(|w| w.take()) {
			Some(w) => w,
			None => return EventResponse::Ignored,
		};

		let mut ctx = EventCtx {
			node_id: id,
			redraw_requested,
			layout_requested,
		};

		let resp = widget.handle_event(&mut ctx, event);
		self.widgets.insert(id, Some(widget));
		resp
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
}
