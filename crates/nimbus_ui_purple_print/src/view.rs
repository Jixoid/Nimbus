/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::collections::{HashSet, VecDeque};

use nimbus_core::{
	BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, ObjectID, PropertyCtx, PropertyError, PropertyValue,
	UiTree, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};

use crate::node::PurplePrintNode;
use crate::pin::{
	NodeBindKind, NodePin, PinDirection, PinRef, WireConnection, WireState, draw_bezier_wire,
};


const SNAP_RADIUS: f32 = 16.0;


#[derive(Clone, Debug)]
struct CachedPin {
	pin_ref: PinRef,
	kind: NodeBindKind,
	data_type: Option<String>,
	color: Color,
	theme_flow: bool,
	pos: Vec2<f32>, // relative to view origin
}

#[derive(Clone, Debug)]
struct CachedNode {
	id: ObjectID,
	rect: Rect<f32>, // relative to view origin
	header_height: f32,
	graph_pos: Vec2<f32>,
	pins: Vec<CachedPin>,
}

#[derive(Clone, Debug)]
struct RenderWire {
	start: Vec2<f32>, // relative to view origin
	end: Vec2<f32>,   // relative to view origin
	color: Color,
	theme_flow: bool,
}

#[derive(Clone, Debug)]
struct ActiveWire {
	source: PinRef,
	source_kind: NodeBindKind,
	source_data_type: Option<String>,
	source_color: Color,
	theme_flow: bool,
	source_pos: Vec2<f32>, // relative to view origin
	cursor: Vec2<f32>,     // relative to view origin
	snap_target: Option<PinRef>,
	state: WireState,
}


pub struct PurplePrintView {
	pub color: Option<DColor>,
	pub pan: Vec2<f32>,
	pub grid_size: f32,
	pub major_step: u32,
	pub allow_cycles: bool,
	pub connections: Vec<WireConnection>,
	pub on_connect: Option<MethodCallback>,
	pub on_disconnect: Option<MethodCallback>,

	cached_nodes: Vec<CachedNode>,
	render_wires: Vec<RenderWire>,
	selected_node: Option<ObjectID>,
	dragging_node: Option<(ObjectID, Vec2<f32>, Vec2<f32>)>,
	panning: Option<(Vec2<f32>, Vec2<f32>)>,
	wiring: Option<ActiveWire>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for PurplePrintView {
	fn default() -> Self {
		Self {
			color: None,
			pan: Vec2::ZERO,
			grid_size: 16.0,
			major_step: 5,
			allow_cycles: false,
			connections: Vec::new(),
			on_connect: None,
			on_disconnect: None,
			cached_nodes: Vec::new(),
			render_wires: Vec::new(),
			selected_node: None,
			dragging_node: None,
			panning: None,
			wiring: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl PurplePrintView {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> PurplePrintViewBuilder {
		PurplePrintViewBuilder::new()
	}

	/// Returns `true` if adding an edge `from_node -> to_node` would introduce a cycle.
	pub fn would_create_cycle(&self, from_node: ObjectID, to_node: ObjectID) -> bool {
		if from_node == to_node {
			return true;
		}
		let mut visited = HashSet::new();
		let mut queue = VecDeque::new();
		visited.insert(to_node);
		queue.push_back(to_node);

		while let Some(curr) = queue.pop_front() {
			for conn in &self.connections {
				if conn.from_node == curr {
					if conn.to_node == from_node {
						return true;
					}
					if visited.insert(conn.to_node) {
						queue.push_back(conn.to_node);
					}
				}
			}
		}
		false
	}

	/// Connects an output pin on `from_node` to an input pin on `to_node`
	/// Validates cycle prevention (unless `allow_cycles` is true) and pin kind/data_type compatibility
	/// when pin metadata is cached. Replaces any existing wire entering `(to_node, to_pin)`
	pub fn connect(
		&mut self,
		from_node: ObjectID,
		from_pin: usize,
		to_node: ObjectID,
		to_pin: usize,
	) -> bool {
		if from_node == to_node {
			return false;
		}
		if !self.allow_cycles && self.would_create_cycle(from_node, to_node) {
			return false;
		}

		// Validate kind & data_type if cached layout metadata is available
		if let (Some(src), Some(dst)) = (
			self.find_cached_pin(PinRef {
				node_id: from_node,
				direction: PinDirection::Output,
				index: from_pin,
			}),
			self.find_cached_pin(PinRef {
				node_id: to_node,
				direction: PinDirection::Input,
				index: to_pin,
			}),
		) {
			if src.kind != dst.kind
				|| !NodePin::is_compatible_types(src.data_type.as_deref(), dst.data_type.as_deref())
			{
				return false;
			}
		}

		// One incoming wire per input pin
		self.connections
			.retain(|c| !(c.to_node == to_node && c.to_pin == to_pin));

		let conn = WireConnection {
			from_node,
			from_pin,
			to_node,
			to_pin,
		};
		if !self.connections.contains(&conn) {
			self.connections.push(conn);
			if let Some(cb) = &self.on_connect {
				cb.call2(from_pin, to_pin);
			}
			true
		} else {
			false
		}
	}

	/// Removes all wires connected to a specific pin
	pub fn disconnect_pin(&mut self, pin: PinRef) -> bool {
		let before = self.connections.len();
		self.connections.retain(|c| match pin.direction {
			PinDirection::Output => !(c.from_node == pin.node_id && c.from_pin == pin.index),
			PinDirection::Input => !(c.to_node == pin.node_id && c.to_pin == pin.index),
		});
		let removed = self.connections.len() != before;
		if removed && let Some(cb) = &self.on_disconnect {
			cb.call1(pin.index);
		}
		removed
	}

	/// Removes all wires attached to `node_id`
	pub fn disconnect_node(&mut self, node_id: ObjectID) -> bool {
		let before = self.connections.len();
		self.connections
			.retain(|c| c.from_node != node_id && c.to_node != node_id);
		let removed = self.connections.len() != before;
		if removed && let Some(cb) = &self.on_disconnect {
			cb.call0();
		}
		removed
	}

	fn hit_pin(&self, local_pos: Vec2<f32>) -> Option<&CachedPin> {
		let max_d2 = SNAP_RADIUS * SNAP_RADIUS;
		let mut best: Option<(&CachedPin, f32)> = None;
		for node in self.cached_nodes.iter().rev() {
			for pin in &node.pins {
				let dx = local_pos.x - pin.pos.x;
				let dy = local_pos.y - pin.pos.y;
				let d2 = dx * dx + dy * dy;
				if d2 <= max_d2 && best.is_none_or(|b| d2 < b.1) {
					best = Some((pin, d2));
				}
			}
		}
		best.map(|(p, _)| p)
	}

	fn hit_node(&self, local_pos: Vec2<f32>) -> Option<&CachedNode> {
		self.cached_nodes
			.iter()
			.rev()
			.find(|n| n.rect.contains_point(local_pos))
	}

	fn find_cached_pin(&self, target: PinRef) -> Option<&CachedPin> {
		self.cached_nodes
			.iter()
			.find(|n| n.id == target.node_id)
			.and_then(|n| n.pins.iter().find(|p| p.pin_ref == target))
	}

	fn bring_node_to_front(&self, ctx: &mut EventCtx, node_id: ObjectID) {
		let view_id = ctx.node_id();
		ctx.push_task(move |tree: &mut UiTree| {
			if let Some(vnode) = tree.nodes.get_mut(view_id)
				&& let Some(pos) = vnode.children.iter().position(|&c| c == node_id)
			{
				let id = vnode.children.remove(pos);
				vnode.children.push(id);
				tree.needs_layout = true;
				tree.needs_redraw = true;
			}
		});
	}

	fn evaluate_wire_hover(&mut self, local_pos: Vec2<f32>) {
		let Some((src_ref, src_kind, src_dtype)) = self
			.wiring
			.as_ref()
			.map(|w| (w.source, w.source_kind, w.source_data_type.clone()))
		else {
			return;
		};
		let max_d2 = SNAP_RADIUS * SNAP_RADIUS;
		let mut best: Option<(PinRef, Vec2<f32>, WireState, f32)> = None;

		for node in &self.cached_nodes {
			if node.id == src_ref.node_id {
				continue;
			}
			for pin in &node.pins {
				let dx = local_pos.x - pin.pos.x;
				let dy = local_pos.y - pin.pos.y;
				let d2 = dx * dx + dy * dy;
				if d2 <= max_d2 && best.is_none_or(|b| d2 < b.3) {
					let state = if pin.pin_ref.direction == src_ref.direction {
						WireState::Invalid
					} else if pin.kind != src_kind
						|| !NodePin::is_compatible_types(
							src_dtype.as_deref(),
							pin.data_type.as_deref(),
						) {
						WireState::NoSuitable
					} else {
						let (out_node, in_node) = match src_ref.direction {
							PinDirection::Output => (src_ref.node_id, pin.pin_ref.node_id),
							PinDirection::Input => (pin.pin_ref.node_id, src_ref.node_id),
						};
						if !self.allow_cycles && self.would_create_cycle(out_node, in_node) {
							WireState::Cycle
						} else {
							WireState::Valid
						}
					};
					best = Some((pin.pin_ref, pin.pos, state, d2));
				}
			}
		}

		if let Some(wire) = &mut self.wiring {
			if let Some((target_ref, target_pos, state, _)) = best {
				wire.state = state;
				wire.snap_target = Some(target_ref);
				wire.cursor = if state == WireState::Valid {
					target_pos
				} else {
					local_pos
				};
			} else {
				wire.state = WireState::Normal;
				wire.snap_target = None;
				wire.cursor = local_pos;
			}
		}
	}
}

impl Widget for PurplePrintView {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		ctx.set_clip_children(true);

		let size = constraints.constrain(Vec2::new(
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

		let children = ctx.children();

		// Remove stale connections pointing to removed nodes
		self.connections.retain(|c| {
			children.contains(&c.from_node) && children.contains(&c.to_node)
		});

		self.cached_nodes.clear();
		self.render_wires.clear();

		for &child_id in &children {
			// Handle all pointer events centrally in PurplePrintView so dragging & wiring are seamless
			ctx.tree.set_enabled(child_id, false);

			// Update node selection & pin connection indicators before layout
			if let Some(node) = ctx.tree.get_widget_mut::<PurplePrintNode>(child_id) {
				node.selected = self.selected_node == Some(child_id);
				node.connected_inputs.clear();
				node.connected_inputs.resize(node.inputs.len(), false);
				node.connected_outputs.clear();
				node.connected_outputs.resize(node.outputs.len(), false);

				for conn in &self.connections {
					if conn.to_node == child_id && conn.to_pin < node.connected_inputs.len() {
						node.connected_inputs[conn.to_pin] = true;
					}
					if conn.from_node == child_id && conn.from_pin < node.connected_outputs.len() {
						node.connected_outputs[conn.from_pin] = true;
					}
				}
			}

			let child_size = ctx.layout_child(child_id, BoxConstraints::unbounded());
			let (graph_pos, header_height, pins_info) =
				if let Some(node) = ctx.tree.get_widget::<PurplePrintNode>(child_id) {
					let local_origin = node.position + self.pan;
					let mut pins = Vec::with_capacity(node.inputs.len() + node.outputs.len());
					for (i, pin) in node.inputs.iter().enumerate() {
						pins.push(CachedPin {
							pin_ref: PinRef {
								node_id: child_id,
								direction: PinDirection::Input,
								index: i,
							},
							kind: pin.kind,
							data_type: pin.data_type.clone(),
							color: pin.effective_color(),
							theme_flow: pin.uses_theme_flow_color(),
							pos: local_origin + pin.local_pos(),
						});
					}
					for (i, pin) in node.outputs.iter().enumerate() {
						pins.push(CachedPin {
							pin_ref: PinRef {
								node_id: child_id,
								direction: PinDirection::Output,
								index: i,
							},
							kind: pin.kind,
							data_type: pin.data_type.clone(),
							color: pin.effective_color(),
							theme_flow: pin.uses_theme_flow_color(),
							pos: local_origin + pin.local_pos(),
						});
					}
					(node.position, node.header_height(), pins)
				} else {
					(Vec2::ZERO, 30.0, Vec::new())
				};

			let local_origin = graph_pos + self.pan;
			ctx.set_child_position(child_id, local_origin);

			self.cached_nodes.push(CachedNode {
				id: child_id,
				rect: Rect::new(local_origin.x, local_origin.y, child_size.x, child_size.y),
				header_height,
				graph_pos,
				pins: pins_info,
			});
		}

		// Build cached wire endpoints in view-local coordinates
		for conn in &self.connections {
			let start_pin = self.find_cached_pin(PinRef {
				node_id: conn.from_node,
				direction: PinDirection::Output,
				index: conn.from_pin,
			});
			let end_pin = self.find_cached_pin(PinRef {
				node_id: conn.to_node,
				direction: PinDirection::Input,
				index: conn.to_pin,
			});
			if let (Some(sp), Some(ep)) = (start_pin, end_pin) {
				self.render_wires.push(RenderWire {
					start: sp.pos,
					end: ep.pos,
					color: sp.color,
					theme_flow: sp.theme_flow,
				});
			}
		}

		size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;

		// Background
		let bg = self
			.color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);
		canvas.draw(RectShape::new(bounds, bg));

		canvas.push_clip(bounds);

		// Panning minor / major grid
		let cell = self.grid_size.max(8.0);
		let major = self.major_step.max(2) as i32;
		let grid_base = monet[MonetRole::OnSurface];
		let minor_col = Color::new(grid_base.r, grid_base.g, grid_base.b, 0.06);
		let major_col = Color::new(grid_base.r, grid_base.g, grid_base.b, 0.14);

		let start_col = ((-self.pan.x) / cell).floor() as i32;
		let end_col = ((bounds.width() - self.pan.x) / cell).ceil() as i32;
		for col in start_col..=end_col {
			let x = bounds.x() + col as f32 * cell + self.pan.x;
			let is_major = col.rem_euclid(major) == 0;
			let (w, col_rgba) = if is_major { (1.5, major_col) } else { (1.0, minor_col) };
			canvas.draw(RectShape::new(Rect::new(x, bounds.y(), w, bounds.height()), col_rgba));
		}

		let start_row = ((-self.pan.y) / cell).floor() as i32;
		let end_row = ((bounds.height() - self.pan.y) / cell).ceil() as i32;
		for row in start_row..=end_row {
			let y = bounds.y() + row as f32 * cell + self.pan.y;
			let is_major = row.rem_euclid(major) == 0;
			let (h, col_rgba) = if is_major { (1.5, major_col) } else { (1.0, minor_col) };
			canvas.draw(RectShape::new(Rect::new(bounds.x(), y, bounds.width(), h), col_rgba));
		}

		// Connected Bezier wires (drawn behind nodes)
		for wire in &self.render_wires {
			let wire_color = if wire.theme_flow {
				monet[MonetRole::OnSurface]
			} else {
				wire.color
			};
			draw_bezier_wire(
				canvas,
				bounds.min + wire.start,
				bounds.min + wire.end,
				2.8,
				wire_color,
			);
		}

		canvas.pop_clip();
	}

	fn draw_foreground(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		// Active dragging wire preview (drawn on top of all nodes)
		if let Some(active) = &self.wiring {
			let monet = &ctx.theme.monet;
			let bounds = ctx.bounds;
			canvas.push_clip(bounds);

			let wire_col = match active.state {
				WireState::Normal => monet[MonetRole::OnSurfaceVariant],
				WireState::Valid => {
					if active.theme_flow {
						monet[MonetRole::OnSurface]
					} else {
						active.source_color
					}
				}
				WireState::Invalid => Color::new(0.90, 0.20, 0.20, 0.95),
				WireState::NoSuitable => Color::new(0.92, 0.35, 0.20, 0.95),
				WireState::Cycle => Color::new(0.95, 0.55, 0.15, 0.95),
			};

			let (start, end) = match active.source.direction {
				PinDirection::Output => (bounds.min + active.source_pos, bounds.min + active.cursor),
				PinDirection::Input => (bounds.min + active.cursor, bounds.min + active.source_pos),
			};
			draw_bezier_wire(canvas, start, end, 3.0, wire_col);

			canvas.pop_clip();
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		match event {
			InputEvent::PointerButton {
				button,
				state: ButtonState::Pressed,
				position,
			} => {
				let local_pos = *position - bounds.min;

				// Right-click on a pin disconnects all its wires
				if *button == MouseButton::Right {
					if let Some(pin) = self.hit_pin(local_pos).cloned() {
						if self.disconnect_pin(pin.pin_ref) {
							ctx.request_layout();
							ctx.request_redraw();
						}
						return EventResponse::Handled;
					}
					return EventResponse::Ignored;
				}

				// Middle-click always pans the canvas
				if *button == MouseButton::Middle {
					self.panning = Some((*position, self.pan));
					return EventResponse::Handled;
				}

				if *button != MouseButton::Left {
					return EventResponse::Ignored;
				}

				// Check pin hit first (wiring / re-wiring)
				if let Some(clicked_pin) = self.hit_pin(local_pos).cloned() {
					// If clicking an already-connected input pin, detach and re-wire from its source output pin
					if clicked_pin.pin_ref.direction == PinDirection::Input
						&& let Some(pos) = self.connections.iter().position(|c| {
							c.to_node == clicked_pin.pin_ref.node_id && c.to_pin == clicked_pin.pin_ref.index
						})
					{
						let removed = self.connections.remove(pos);
						if let Some(cb) = &self.on_disconnect {
							cb.call1(removed.to_pin);
						}
						let src_ref = PinRef {
							node_id: removed.from_node,
							direction: PinDirection::Output,
							index: removed.from_pin,
						};
						if let Some(src_pin) = self.find_cached_pin(src_ref).cloned() {
							self.wiring = Some(ActiveWire {
								source: src_pin.pin_ref,
								source_kind: src_pin.kind,
								source_data_type: src_pin.data_type,
								source_color: src_pin.color,
								theme_flow: src_pin.theme_flow,
								source_pos: src_pin.pos,
								cursor: local_pos,
								snap_target: None,
								state: WireState::Normal,
							});
							ctx.request_layout();
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}

					self.wiring = Some(ActiveWire {
						source: clicked_pin.pin_ref,
						source_kind: clicked_pin.kind,
						source_data_type: clicked_pin.data_type,
						source_color: clicked_pin.color,
						theme_flow: clicked_pin.theme_flow,
						source_pos: clicked_pin.pos,
						cursor: local_pos,
						snap_target: None,
						state: WireState::Normal,
					});
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				// Check node hit (header drag or select + bring to front)
				if let Some(node) = self.hit_node(local_pos).cloned() {
					self.selected_node = Some(node.id);
					self.bring_node_to_front(ctx, node.id);

					let rel_y = local_pos.y - node.rect.y();
					if rel_y <= node.header_height {
						self.dragging_node = Some((node.id, *position, node.graph_pos));
					}
					ctx.request_layout();
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				// Empty canvas click: deselect & pan
				if self.selected_node.is_some() {
					self.selected_node = None;
					ctx.request_layout();
				}
				self.panning = Some((*position, self.pan));
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::PointerMoved { position } => {
				if let Some((start_ptr, start_pan)) = self.panning {
					self.pan = start_pan + (*position - start_ptr);
					ctx.request_layout();
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				if let Some((node_id, start_ptr, start_graph_pos)) = self.dragging_node {
					let new_pos = start_graph_pos + (*position - start_ptr);
					ctx.push_task(move |tree: &mut UiTree| {
						if let Some(node) = tree.get_widget_mut::<PurplePrintNode>(node_id) {
							node.position = new_pos;
							tree.needs_layout = true;
							tree.needs_redraw = true;
						}
					});
					ctx.request_layout();
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				if self.wiring.is_some() {
					let local_pos = *position - bounds.min;
					self.evaluate_wire_hover(local_pos);
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				state: ButtonState::Released,
				..
			} => {
				let mut handled = false;
				if self.panning.take().is_some() {
					handled = true;
				}
				if self.dragging_node.take().is_some() {
					handled = true;
				}
				if let Some(active) = self.wiring.take() {
					if active.state == WireState::Valid
						&& let Some(target) = active.snap_target
					{
						let (out_pin, in_pin) = match active.source.direction {
							PinDirection::Output => (active.source, target),
							PinDirection::Input => (target, active.source),
						};
						self.connect(out_pin.node_id, out_pin.index, in_pin.node_id, in_pin.index);
					}
					ctx.request_layout();
					ctx.request_redraw();
					handled = true;
				}
				if handled {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::Key {
				key_name,
				pressed: true,
				..
			} => {
				if matches!(key_name.as_str(), "Delete" | "Backspace")
					&& let Some(sel) = self.selected_node
					&& self.disconnect_node(sel)
				{
					ctx.request_layout();
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
			"color" => self.color = Some(value.expect_dcolor()?),
			"grid_size" => self.grid_size = value.expect_f32()?.max(8.0),
			"major_step" => self.major_step = (value.expect_f64()?.max(2.0).round()) as u32,
			"allow_cycles" => self.allow_cycles = value.expect_bool()?,
			"pan" => self.pan = value.expect_vec2()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_connect" | "on_disconnect")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_connect" => {
				self.on_connect = Some(callback);
				true
			}
			"on_disconnect" => {
				self.on_disconnect = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct PurplePrintViewBuilder {
	inner: PurplePrintView,
}

impl PurplePrintViewBuilder {
	pub fn new() -> Self {
		Self {
			inner: PurplePrintView::default(),
		}
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn pan(mut self, x: f32, y: f32) -> Self {
		self.inner.pan = Vec2::new(x, y);
		self
	}

	pub fn grid_size(mut self, size: f32) -> Self {
		self.inner.grid_size = size.max(8.0);
		self
	}

	pub fn major_step(mut self, step: u32) -> Self {
		self.inner.major_step = step.max(2);
		self
	}

	pub fn allow_cycles(mut self, allow: bool) -> Self {
		self.inner.allow_cycles = allow;
		self
	}

	pub fn on_connect<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_connect = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_disconnect<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_disconnect = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> PurplePrintView {
		self.inner
	}
}

impl From<PurplePrintViewBuilder> for PurplePrintView {
	fn from(builder: PurplePrintViewBuilder) -> Self {
		builder.build()
	}
}
