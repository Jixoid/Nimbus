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
	LayoutCtx, MouseButton, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{MonetRole, Theme};


#[derive(Clone, Debug, PartialEq)]
pub struct ScrollState {
	pub offset: f32,
	pub anim: Animated<f32>,
	pub content_size: f32,
	pub viewport_size: f32,
	pub is_dragging: bool,
	pub drag_start_pos: f32,
	pub drag_start_offset: f32,
	pub hovered: bool,
}

impl Default for ScrollState {
	fn default() -> Self {
		Self {
			offset: 0.0,
			anim: Animated::new(0.0),
			content_size: 0.0,
			viewport_size: 0.0,
			is_dragging: false,
			drag_start_pos: 0.0,
			drag_start_offset: 0.0,
			hovered: false,
		}
	}
}

impl ScrollState {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn max_offset(&self) -> f32 {
		(self.content_size - self.viewport_size).max(0.0)
	}

	pub fn is_scrollable(&self) -> bool {
		self.max_offset() > 0.5
	}

	pub fn is_animating(&self) -> bool {
		self.anim.is_animating()
	}

	pub fn target_offset(&self) -> f32 {
		*self.anim.target()
	}

	pub fn sync(&mut self) -> f32 {
		self.offset = self.anim.value().clamp(0.0, self.max_offset());
		self.offset
	}

	pub fn snap_to(&mut self, offset: f32) {
		self.offset = offset.clamp(0.0, self.max_offset());
		self.anim.snap_to(self.offset);
	}

	pub fn update(&mut self, content_size: f32, viewport_size: f32) {
		self.content_size = content_size;
		self.viewport_size = viewport_size;
		let max_off = self.max_offset();
		if self.offset > max_off {
			self.offset = max_off;
			self.anim.snap_to(max_off);
		}
	}

	pub fn apply_scroll(&mut self, delta: f32) -> bool {
		if !self.is_scrollable() {
			return false;
		}
		let current_target = *self.anim.target();
		let new_target = (current_target - delta).clamp(0.0, self.max_offset());
		if (new_target - current_target).abs() < 0.001 {
			return false;
		}
		self.anim.set_target(
			new_target,
			Duration::from_millis(160),
			Easing::Decelerate,
		);
		self.offset = self.anim.value();
		true
	}

	pub fn thumb_rect(&self, bounds: Rect<f32>, is_vertical: bool) -> Option<Rect<f32>> {
		if !self.is_scrollable() {
			return None;
		}

		let thickness = if self.is_dragging || self.hovered {
			6.0
		} else {
			4.0
		};
		let margin = 2.0;

		let cur_offset = self.anim.value().clamp(0.0, self.max_offset());

		if is_vertical {
			let track_len = (bounds.height() - margin * 2.0).max(1.0);
			let ratio = (self.viewport_size / self.content_size).clamp(0.05, 1.0);
			let thumb_len = (track_len * ratio).clamp(20.0, track_len);
			let scrollable_track = track_len - thumb_len;

			let scroll_ratio = if self.max_offset() > 0.0 {
				cur_offset / self.max_offset()
			} else {
				0.0
			};

			let thumb_y = bounds.y() + margin + scroll_ratio * scrollable_track;
			let thumb_x = bounds.x() + bounds.width() - thickness - margin;

			Some(Rect::new(thumb_x, thumb_y, thickness, thumb_len))
		} else {
			let track_len = (bounds.width() - margin * 2.0).max(1.0);
			let ratio = (self.viewport_size / self.content_size).clamp(0.05, 1.0);
			let thumb_len = (track_len * ratio).clamp(20.0, track_len);
			let scrollable_track = track_len - thumb_len;

			let scroll_ratio = if self.max_offset() > 0.0 {
				cur_offset / self.max_offset()
			} else {
				0.0
			};

			let thumb_x = bounds.x() + margin + scroll_ratio * scrollable_track;
			let thumb_y = bounds.y() + bounds.height() - thickness - margin;

			Some(Rect::new(thumb_x, thumb_y, thumb_len, thickness))
		}
	}

	pub fn handle_event(
		&mut self,
		ctx: &mut EventCtx,
		event: &InputEvent,
		bounds: Rect<f32>,
		is_vertical: bool,
	) -> EventResponse {
		if !self.is_scrollable() {
			return EventResponse::Ignored;
		}

		match event {
			InputEvent::PointerScroll { delta } => {
				let scroll_amount = if is_vertical {
					delta.y
				} else if delta.x.abs() > 0.001 {
					delta.x
				} else {
					delta.y
				};

				if self.apply_scroll(scroll_amount) {
					ctx.request_redraw();
					ctx.request_layout();
					return EventResponse::Handled;
				}
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					if let Some(thumb) = self.thumb_rect(bounds, is_vertical) {
						if thumb.contains_point(*position) {
							self.is_dragging = true;
							self.drag_start_pos = if is_vertical { position.y } else { position.x };
							self.drag_start_offset = self.anim.value();
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}
				}
				ButtonState::Released => {
					if self.is_dragging {
						self.is_dragging = false;
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
			},

			InputEvent::PointerMoved { position } => {
				if self.is_dragging {
					let margin = 2.0;
					let track_len = if is_vertical {
						(bounds.height() - margin * 2.0).max(1.0)
					} else {
						(bounds.width() - margin * 2.0).max(1.0)
					};
					let ratio = (self.viewport_size / self.content_size).clamp(0.05, 1.0);
					let thumb_len = (track_len * ratio).clamp(20.0, track_len);
					let scrollable_track = (track_len - thumb_len).max(1.0);

					let current_pos = if is_vertical { position.y } else { position.x };
					let delta_px = current_pos - self.drag_start_pos;
					let offset_delta = (delta_px / scrollable_track) * self.max_offset();

					let new_offset = (self.drag_start_offset + offset_delta).clamp(0.0, self.max_offset());
					let old_offset = self.offset;
					self.snap_to(new_offset);
					if (self.offset - old_offset).abs() > 0.001 {
						ctx.request_redraw();
						ctx.request_layout();
					}
					return EventResponse::Handled;
				} else if let Some(thumb) = self.thumb_rect(bounds, is_vertical) {
					let is_hovered = thumb.contains_point(*position);
					if is_hovered != self.hovered {
						self.hovered = is_hovered;
						ctx.request_redraw();
					}
				}
			}

			_ => {}
		}

		EventResponse::Ignored
	}

	pub fn draw(
		&self,
		canvas: &mut Canvas2D,
		bounds: Rect<f32>,
		is_vertical: bool,
		theme: &Theme,
	) {
		if let Some(thumb) = self.thumb_rect(bounds, is_vertical) {
			let color = if self.is_dragging {
				theme.monet[MonetRole::Primary]
			} else if self.hovered {
				theme.monet[MonetRole::SurfaceOutline]
			} else {
				theme.monet[MonetRole::SurfaceOutlineVariant]
			};

			let radius = if is_vertical {
				thumb.width() * 0.5
			} else {
				thumb.height() * 0.5
			};

			canvas.draw(RectShape::new(thumb, color).radius(radius));
		}
	}
}


pub struct ScrollView {
	pub scrollable_x: bool,
	pub scrollable_y: bool,
	pub scroll_x: ScrollState,
	pub scroll_y: ScrollState,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for ScrollView {
	fn default() -> Self {
		Self {
			scrollable_x: false,
			scrollable_y: true,
			scroll_x: ScrollState::new(),
			scroll_y: ScrollState::new(),
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl Widget for ScrollView {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		if children.is_empty() {
			return constraints.constrain(Vec2::ZERO);
		}

		let child_constraints = BoxConstraints::new(
			0.0,
			if self.scrollable_x {
				f32::INFINITY
			} else {
				constraints.max_width
			},
			0.0,
			if self.scrollable_y {
				f32::INFINITY
			} else {
				constraints.max_height
			},
		);

		let mut content_w = 0.0f32;
		let mut content_h = 0.0f32;

		for child in &children {
			let sz = ctx.layout_child(*child, child_constraints);
			content_w = content_w.max(sz.x);
			content_h = content_h.max(sz.y);
		}

		let final_size = constraints.constrain(Vec2::new(content_w, content_h));

		if self.scrollable_x || self.scrollable_y {
			ctx.set_clip_children(true);
		}

		if self.scrollable_x {
			self.scroll_x.update(content_w, final_size.x);
		}
		if self.scrollable_y {
			self.scroll_y.update(content_h, final_size.y);
		}

		let offset_x = if self.scrollable_x {
			self.scroll_x.sync();
			self.scroll_x.offset
		} else {
			0.0
		};
		let offset_y = if self.scrollable_y {
			self.scroll_y.sync();
			self.scroll_y.offset
		} else {
			0.0
		};

		for child in &children {
			ctx.set_child_position(*child, Vec2::new(-offset_x, -offset_y));
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		if self.scrollable_y {
			self.scroll_y.draw(canvas, ctx.bounds, true, ctx.theme);
		}
		if self.scrollable_x {
			self.scroll_x.draw(canvas, ctx.bounds, false, ctx.theme);
		}

		if (self.scrollable_x && self.scroll_x.is_animating())
			|| (self.scrollable_y && self.scroll_y.is_animating())
		{
			ctx.request_layout();
			ctx.request_redraw();
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let mut resp = EventResponse::Ignored;

		if self.scrollable_y {
			resp = self.scroll_y.handle_event(ctx, event, bounds, true);
		}
		if resp == EventResponse::Ignored && self.scrollable_x {
			resp = self.scroll_x.handle_event(ctx, event, bounds, false);
		}

		resp
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"scrollable_x" => self.scrollable_x = value.expect_bool()?,
			"scrollable_y" => self.scrollable_y = value.expect_bool()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl ScrollView {
	pub fn builder() -> ScrollViewBuilder {
		ScrollViewBuilder::new()
	}
}


#[derive(Default)]
pub struct ScrollViewBuilder {
	inner: ScrollView,
}

impl ScrollViewBuilder {
	pub fn new() -> Self {
		Self {
			inner: ScrollView::default(),
		}
	}

	pub fn scrollable_x(mut self, scrollable: bool) -> Self {
		self.inner.scrollable_x = scrollable;
		self
	}

	pub fn scrollable_y(mut self, scrollable: bool) -> Self {
		self.inner.scrollable_y = scrollable;
		self
	}

	pub fn build(self) -> ScrollView {
		self.inner
	}
}

impl From<ScrollViewBuilder> for ScrollView {
	fn from(builder: ScrollViewBuilder) -> Self {
		builder.build()
	}
}
