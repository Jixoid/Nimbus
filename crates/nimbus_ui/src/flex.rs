/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx, PropertyCtx,
	PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::MonetRole;

use crate::scroll::ScrollState;


// VBox
pub struct VBox {
	pub spacing: f32,
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<Color>,
	pub border_width: f32,
	pub border_color: Option<Color>,
	pub shadow_blur: f32,
	pub scrollable: bool,
	pub scroll: ScrollState,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for VBox {
	fn default() -> Self {
		Self {
			spacing: 12.0,
			padding: 0.0,
			corner_radius: None,
			background: None,
			border_width: 0.0,
			border_color: None,
			shadow_blur: 0.0,
			scrollable: false,
			scroll: ScrollState::new(),
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}


impl Widget for VBox {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		let mut current_y = self.padding;
		let mut max_width = 0.0f32;

		let child_constraints = BoxConstraints::new(
			0.0,
			(constraints.max_width - self.padding * 2.0).max(0.0),
			0.0,
			f32::INFINITY,
		);

		let mut child_sizes = Vec::with_capacity(children.len());
		for (i, child) in children.iter().enumerate() {
			if i > 0 {
				current_y += self.spacing;
			}

			let size = ctx.layout_child(*child, child_constraints);
			current_y += size.y;
			max_width = max_width.max(size.x);
			child_sizes.push(size);
		}

		current_y += self.padding;
		let total_width = max_width + self.padding * 2.0;
		let final_size = constraints.constrain(Vec2::new(total_width, current_y));

		let scroll_offset = if self.scrollable {
			ctx.set_clip_children(true);
			self.scroll.update(current_y, final_size.y);
			self.scroll.sync();
			self.scroll.offset
		} else {
			0.0
		};

		let mut pos_y = self.padding - scroll_offset;
		for (i, child) in children.iter().enumerate() {
			if i > 0 {
				pos_y += self.spacing;
			}
			ctx.set_child_position(*child, Vec2::new(self.padding, pos_y));
			pos_y += child_sizes[i].y;
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		let bg = self
			.background
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);

		if self.border_width > 0.0 {
			let border_col = self
				.border_color
				.unwrap_or_else(|| monet[MonetRole::SurfaceOutline]);
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

		self.last_bounds.set(ctx.bounds);
		if self.scrollable {
			self.scroll.draw(canvas, ctx.bounds, true, ctx.theme);
			if self.scroll.is_animating() {
				ctx.request_layout();
				ctx.request_redraw();
			}
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if self.scrollable {
			self.scroll.handle_event(ctx, event, self.last_bounds.get(), true)
		} else {
			EventResponse::Ignored
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> bool {
		match key {
			"scrollable" => {
				if let Some(b) = value.as_bool() {
					self.scrollable = b;
					return true;
				}
			}
			"spacing" => {
				if let Some(s) = value.as_f32() {
					self.spacing = s;
					return true;
				}
			}

			"paddings" => {
				if let Some(p) = value.as_f32() {
					self.padding = p;
					return true;
				}
			}

			"corner_radius" => {
				if let Some(r) = value.as_f32() {
					self.corner_radius = Some(r);
					return true;
				}
			}

			"color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.background = Some(c);
					return true;
				}
			}

			"border_color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.border_color = Some(c);
					return true;
				}
			}

			"border_width" => {
				if let Some(w) = value.as_f32() {
					self.border_width = w;
					return true;
				}
			}

			"shadow_blur" => {
				if let Some(b) = value.as_f32() {
					self.shadow_blur = b;
					return true;
				}
			}
			
			_ => {}
		}
		false
	}
}


// HBox
pub struct HBox {
	pub spacing: f32,
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<Color>,
	pub border_width: f32,
	pub border_color: Option<Color>,
	pub shadow_blur: f32,
	pub scrollable: bool,
	pub scroll: ScrollState,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for HBox {
	fn default() -> Self {
		Self {
			spacing: 12.0,
			padding: 0.0,
			corner_radius: None,
			background: None,
			border_width: 0.0,
			border_color: None,
			shadow_blur: 0.0,
			scrollable: false,
			scroll: ScrollState::new(),
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}


impl Widget for HBox {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		let mut current_x = self.padding;
		let mut max_height = 0.0f32;

		let child_constraints = BoxConstraints::new(
			0.0,
			f32::INFINITY,
			0.0,
			(constraints.max_height - self.padding * 2.0).max(0.0),
		);

		let mut child_sizes = Vec::with_capacity(children.len());
		for (i, child) in children.iter().enumerate() {
			if i > 0 {
				current_x += self.spacing;
			}

			let size = ctx.layout_child(*child, child_constraints);
			current_x += size.x;
			max_height = max_height.max(size.y);
			child_sizes.push(size);
		}

		current_x += self.padding;
		let total_height = max_height + self.padding * 2.0;
		let final_size = constraints.constrain(Vec2::new(current_x, total_height));

		let scroll_offset = if self.scrollable {
			ctx.set_clip_children(true);
			self.scroll.update(current_x, final_size.x);
			self.scroll.sync();
			self.scroll.offset
		} else {
			0.0
		};

		let mut pos_x = self.padding - scroll_offset;
		for (i, child) in children.iter().enumerate() {
			if i > 0 {
				pos_x += self.spacing;
			}
			ctx.set_child_position(*child, Vec2::new(pos_x, self.padding));
			pos_x += child_sizes[i].x;
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		let bg = self
			.background
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);

		if self.border_width > 0.0 {
			let border_col = self
				.border_color
				.unwrap_or_else(|| monet[MonetRole::SurfaceOutline]);
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

		self.last_bounds.set(ctx.bounds);
		if self.scrollable {
			self.scroll.draw(canvas, ctx.bounds, false, ctx.theme);
			if self.scroll.is_animating() {
				ctx.request_layout();
				ctx.request_redraw();
			}
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if self.scrollable {
			self.scroll.handle_event(ctx, event, self.last_bounds.get(), false)
		} else {
			EventResponse::Ignored
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, ctx: &PropertyCtx) -> bool {
		match key {
			"scrollable" => {
				if let Some(b) = value.as_bool() {
					self.scrollable = b;
					return true;
				}
			}
			"spacing" => {
				if let Some(s) = value.as_f32() {
					self.spacing = s;
					return true;
				}
			}
			
			"paddings" => {
				if let Some(p) = value.as_f32() {
					self.padding = p;
					return true;
				}
			}
			
			"corner_radius" => {
				if let Some(r) = value.as_f32() {
					self.corner_radius = Some(r);
					return true;
				}
			}

			"color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.background = Some(c);
					return true;
				}
			}

			"border_color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.border_color = Some(c);
					return true;
				}
			}

			"border_width" => {
				if let Some(w) = value.as_f32() {
					self.border_width = w;
					return true;
				}
			}

			"shadow_blur" => {
				if let Some(b) = value.as_f32() {
					self.shadow_blur = b;
					return true;
				}
			}

			_ => {}
		}
		false
	}
}
