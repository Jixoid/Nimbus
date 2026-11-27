/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	Anchor, BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent,
	IntoMethod, LayoutCtx, MethodCallback, PropertyCtx, PropertyError, PropertyValue,
	Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Viewport {
	pub width: Option<f32>,
	pub height: Option<f32>,
	pub texture_id: Option<u32>,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub tint: DColor,
	pub border_width: f32,
	pub border_color: Option<DColor>,
	pub shadow_blur: f32,
	pub clip_children: bool,

	pub on_resize: Option<MethodCallback>,
	pub on_click: Option<MethodCallback>,
	pub on_mouse_move: Option<MethodCallback>,
	pub on_mouse_down: Option<MethodCallback>,
	pub on_mouse_up: Option<MethodCallback>,
	pub on_scroll: Option<MethodCallback>,
	pub on_key: Option<MethodCallback>,
	pub on_focus_lost: Option<MethodCallback>,

	last_size: Vec2<f32>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for Viewport {
	fn default() -> Self {
		Self {
			width: None,
			height: None,
			texture_id: None,
			corner_radius: None,
			background: None,
			tint: DColor::new_color(Color::WHITE),
			border_width: 0.0,
			border_color: None,
			shadow_blur: 0.0,
			clip_children: true,
			on_resize: None,
			on_click: None,
			on_mouse_move: None,
			on_mouse_down: None,
			on_mouse_up: None,
			on_scroll: None,
			on_key: None,
			on_focus_lost: None,
			last_size: Vec2::ZERO,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Viewport {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> ViewportBuilder {
		ViewportBuilder::new()
	}

	pub fn set_texture_id(&mut self, texture_id: Option<u32>) {
		self.texture_id = texture_id;
	}

	pub fn viewport_size(&self) -> Vec2<f32> {
		self.last_size
	}

	fn local_position(&self, screen_pos: Vec2<f32>) -> Vec2<f32> {
		let b = self.last_bounds.get();
		Vec2::new(screen_pos.x - b.x(), screen_pos.y - b.y())
	}
}

impl Widget for Viewport {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		if self.clip_children {
			ctx.set_clip_children(true);
		}

		let desired_w = self.width.unwrap_or_else(|| {
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				240.0
			}
		});

		let desired_h = self.height.unwrap_or_else(|| {
			if constraints.max_height.is_finite() && constraints.min_height >= constraints.max_height {
				constraints.max_height
			} else {
				160.0
			}
		});

		let final_size = constraints.constrain(Vec2::new(desired_w.max(0.0), desired_h.max(0.0)));

		if (final_size.x - self.last_size.x).abs() > 0.5
			|| (final_size.y - self.last_size.y).abs() > 0.5
		{
			self.last_size = final_size;
			if let Some(cb) = &mut self.on_resize {
				cb.call2(final_size.x, final_size.y);
			}
		}

		let children = ctx.children();
		for child in children {
			let anchor = ctx.child_anchor(child).unwrap_or(Anchor::TopLeft);
			let child_constraints = anchor.child_constraints(final_size);
			let child_size = ctx.layout_child(child, child_constraints);
			let pos = anchor.position(final_size, child_size);
			ctx.set_child_position(child, pos);
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);
		let border_col = self
			.border_color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceOutline]);

		if let Some(tex_id) = self.texture_id {
			if let Some(bg) = self.background {
				canvas.draw(RectShape::new(ctx.bounds, bg.get(monet)).radius(radius));
			}
			let mut shape = RectShape::new(ctx.bounds, self.tint.get(monet))
				.radius(radius)
				.texture(tex_id);
			if self.border_width > 0.0 {
				shape = shape.border(border_col, self.border_width);
			}
			if self.shadow_blur > 0.0 {
				shape = shape.shadow(
					monet[MonetRole::Shadow],
					Vec2::new(0.0, 4.0),
					self.shadow_blur,
				);
			}
			canvas.draw(shape);
		} else {
			let bg = self
				.background
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::SurfaceContainerLowest]);
			let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);
			if self.border_width > 0.0 {
				shape = shape.border(border_col, self.border_width);
			}
			if self.shadow_blur > 0.0 {
				shape = shape.shadow(
					monet[MonetRole::Shadow],
					Vec2::new(0.0, 4.0),
					self.shadow_blur,
				);
			}
			canvas.draw(shape);
		}
	}

	fn blocks_touch_scroll(&self, _delta: Vec2<f32>) -> bool {
		true
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerMoved { position } => {
				let local = self.local_position(*position);
				if let Some(cb) = &self.on_mouse_move {
					cb.call2(local.x, local.y);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
			}
			InputEvent::PointerButton { button, state, position } => {
				let local = self.local_position(*position);
				match state {
					ButtonState::Pressed => {
						if let Some(cb) = &self.on_mouse_down {
							cb.call3(local.x, local.y, *button as u32);
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}
					ButtonState::Released => {
						if let Some(cb) = &self.on_mouse_up {
							cb.call3(local.x, local.y, *button as u32);
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}
				}
			}
			InputEvent::Click { position } => {
				let local = self.local_position(*position);
				if let Some(cb) = &self.on_click {
					cb.call2(local.x, local.y);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
			}
			InputEvent::PointerScroll { delta } => {
				if let Some(cb) = &self.on_scroll {
					cb.call2(delta.x, delta.y);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
			}
			InputEvent::Key {
				key_name,
				pressed,
				..
			} => {
				if let Some(cb) = &self.on_key {
					cb.call2(key_name.clone(), *pressed);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
			}
			InputEvent::FocusLost => {
				if let Some(cb) = &self.on_focus_lost {
					cb.call0();
				}
			}
			_ => {}
		}

		EventResponse::Ignored
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"width" => self.width = Some(value.expect_f32()?.max(0.0)),
			"height" => self.height = Some(value.expect_f32()?.max(0.0)),
			"size" => {
				if let Some(v) = value.as_vec2() {
					self.width = Some(v.x.max(0.0));
					self.height = Some(v.y.max(0.0));
				} else {
					let s = value.expect_f32()?.max(0.0);
					self.width = Some(s);
					self.height = Some(s);
				}
			}
			"texture_id" => {
				let id = value.expect_f64()? as i64;
				self.texture_id = if id >= 0 { Some(id as u32) } else { None };
			}
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"background" => self.background = Some(value.expect_dcolor()?),
			"tint" => self.tint = value.expect_dcolor()?,
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			"clip_children" => self.clip_children = value.expect_bool()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(
			name,
			"on_resize"
				| "on_click"
				| "on_mouse_move"
				| "on_mouse_down"
				| "on_mouse_up"
				| "on_scroll"
				| "on_key"
				| "on_focus_lost"
		)
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_resize" => {
				self.on_resize = Some(callback);
				true
			}
			"on_click" => {
				self.on_click = Some(callback);
				true
			}
			"on_mouse_move" => {
				self.on_mouse_move = Some(callback);
				true
			}
			"on_mouse_down" => {
				self.on_mouse_down = Some(callback);
				true
			}
			"on_mouse_up" => {
				self.on_mouse_up = Some(callback);
				true
			}
			"on_scroll" => {
				self.on_scroll = Some(callback);
				true
			}
			"on_key" => {
				self.on_key = Some(callback);
				true
			}
			"on_focus_lost" => {
				self.on_focus_lost = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct ViewportBuilder {
	inner: Viewport,
}

impl ViewportBuilder {
	pub fn new() -> Self {
		Self {
			inner: Viewport::default(),
		}
	}

	pub fn width(mut self, width: f32) -> Self {
		self.inner.width = Some(width.max(0.0));
		self
	}

	pub fn height(mut self, height: f32) -> Self {
		self.inner.height = Some(height.max(0.0));
		self
	}

	pub fn size(mut self, width: f32, height: f32) -> Self {
		self.inner.width = Some(width.max(0.0));
		self.inner.height = Some(height.max(0.0));
		self
	}

	pub fn texture_id(mut self, texture_id: u32) -> Self {
		self.inner.texture_id = Some(texture_id);
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
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

	pub fn tint(mut self, tint: impl Into<DColor>) -> Self {
		self.inner.tint = tint.into();
		self
	}

	pub fn border_width(mut self, width: f32) -> Self {
		self.inner.border_width = width;
		self
	}

	pub fn border_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.border_color = Some(color.into());
		self
	}

	pub fn shadow_blur(mut self, blur: f32) -> Self {
		self.inner.shadow_blur = blur;
		self
	}

	pub fn clip_children(mut self, clip: bool) -> Self {
		self.inner.clip_children = clip;
		self
	}

	pub fn on_resize<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_resize = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_click<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_click = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_mouse_move<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_mouse_move = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_mouse_down<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_mouse_down = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_mouse_up<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_mouse_up = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_scroll<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_scroll = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> Viewport {
		self.inner
	}
}

impl From<ViewportBuilder> for Viewport {
	fn from(builder: ViewportBuilder) -> Self {
		builder.build()
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::{Arc, Mutex};
	use nimbus_core::{MouseButton, UiTree};

	#[test]
	fn test_viewport_builder_layout_resize_and_events() {
		let mut tree = UiTree::new();
		let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

		let ev_resize = events.clone();
		let ev_click = events.clone();
		let ev_scroll = events.clone();

		let vp = Viewport::builder()
			.size(320.0, 200.0)
			.texture_id(7)
			.corner_radius(12.0)
			.on_resize(move |w: f32, h: f32| {
				ev_resize.lock().unwrap().push(format!("resize:{w}x{h}"));
			})
			.on_click(move |x: f32, y: f32| {
				ev_click.lock().unwrap().push(format!("click:{x},{y}"));
			})
			.on_scroll(move |dx: f32, dy: f32| {
				ev_scroll.lock().unwrap().push(format!("scroll:{dx},{dy}"));
			})
			.build();

		assert_eq!(vp.texture_id, Some(7));

		let vp_id = tree.insert_root(vp);
		let size = tree.layout_node(vp_id, BoxConstraints::loose(Vec2::new(500.0, 500.0)));
		assert_eq!(size, Vec2::new(320.0, 200.0));

		let pos = Vec2::new(40.0, 60.0);
		tree.dispatch_pointer_move(pos);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, pos);
		tree.dispatch_pointer_button(MouseButton::Left, ButtonState::Released, pos);
		tree.dispatch_pointer_scroll(pos, Vec2::new(0.0, -32.0));

		let recorded = events.lock().unwrap().clone();
		assert_eq!(
			recorded,
			vec!["resize:320x200", "click:40,60", "scroll:0,-32"]
		);
	}
}
