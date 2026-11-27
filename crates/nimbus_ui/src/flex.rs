/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	Anchor, BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, LayoutCtx, PropertyCtx,
	PropertyValue, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, RectShape};
use nimbus_theme::{DColor, MonetRole};

use crate::scroll::ScrollState;


// VBox
pub struct VBox {
	pub spacing: f32,
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
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
		let pad = self.padding * 2.0;
		let avail_w = if constraints.max_width.is_finite() {
			(constraints.max_width - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let avail_h = if constraints.max_height.is_finite() {
			(constraints.max_height - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let tight_w = constraints.min_width >= constraints.max_width && avail_w.is_finite();
		let can_expand_y = avail_h.is_finite() && !self.scrollable;
		let total_spacing = if children.len() > 1 {
			self.spacing * (children.len() - 1) as f32
		} else {
			0.0
		};

		let mut fixed_height = 0.0f32;
		let mut max_width = 0.0f32;
		let mut expand_count = 0usize;
		let mut items = Vec::with_capacity(children.len());

		// Measure non-expanding children first
		for child in children {
			let anchor = ctx.child_anchor(child);
			let is_expand_y = can_expand_y && matches!(anchor, Some(Anchor::Fill));
			let is_fill_x = matches!(anchor, Some(Anchor::Fill | Anchor::TopFill | Anchor::BottomFill));

			if is_expand_y {
				expand_count += 1;
				items.push((child, anchor, true, is_fill_x, Vec2::ZERO));
			} else {
				let child_constraints = if is_fill_x && tight_w {
					BoxConstraints::new(avail_w, avail_w, 0.0, f32::INFINITY)
				} else {
					BoxConstraints::new(0.0, avail_w, 0.0, f32::INFINITY)
				};
				let size = ctx.layout_child(child, child_constraints);
				fixed_height += size.y;
				max_width = max_width.max(size.x);
				items.push((child, anchor, false, is_fill_x, size));
			}
		}

		// Distribute remaining vertical space to `anchor: "fill"` children
		let mut total_children_h = fixed_height;
		if expand_count > 0 {
			let remaining_h = (avail_h - fixed_height - total_spacing).max(0.0);
			let share_h = remaining_h / expand_count as f32;
			let expand_constraints = if tight_w {
				BoxConstraints::new(avail_w, avail_w, share_h, share_h)
			} else {
				BoxConstraints::new(0.0, avail_w, share_h, share_h)
			};

			for (child, _, is_expand_y, _, size) in &mut items {
				if *is_expand_y {
					*size = ctx.layout_child(*child, expand_constraints);
					total_children_h += size.y;
					max_width = max_width.max(size.x);
				}
			}
		}

		let current_y = total_children_h + total_spacing + pad;
		let total_width = max_width + pad;
		let final_size = constraints.constrain(Vec2::new(total_width, current_y));
		let inner_w = (final_size.x - pad).max(0.0);

		let scroll_offset = if self.scrollable {
			ctx.set_clip_children(true);
			self.scroll.update(current_y, final_size.y);
			self.scroll.sync();
			self.scroll.offset
		} else {
			0.0
		};

		let mut pos_y = self.padding - scroll_offset;
		for (i, (child, anchor, is_expand_y, is_fill_x, mut size)) in items.into_iter().enumerate() {
			if i > 0 {
				pos_y += self.spacing;
			}
			if is_fill_x && (size.x - inner_w).abs() > 0.5 {
				let c = if is_expand_y {
					BoxConstraints::new(inner_w, inner_w, size.y, size.y)
				} else {
					BoxConstraints::new(inner_w, inner_w, 0.0, f32::INFINITY)
				};
				size = ctx.layout_child(child, c);
			}
			let pos_x = match anchor {
				Some(Anchor::Center | Anchor::TopCenter | Anchor::BottomCenter) => {
					self.padding + (inner_w - size.x).max(0.0) * 0.5
				}
				Some(Anchor::CenterRight | Anchor::TopRight | Anchor::BottomRight | Anchor::RightFill) => {
					self.padding + (inner_w - size.x).max(0.0)
				}
				_ => self.padding,
			};
			ctx.set_child_position(child, Vec2::new(pos_x, pos_y));
			pos_y += size.y;
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);

		if self.border_width > 0.0 {
			let border_col = self
				.border_color
				.map(|c| c.get(monet))
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

	fn children_stretch(&self) -> Option<([f32; 2], [f32; 2])> {
		if self.scrollable {
			self.scroll.stretch_transform(self.last_bounds.get(), true)
		} else {
			None
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"scrollable" => self.scrollable = value.expect_bool()?,
			"spacing" => self.spacing = value.expect_f32()?,
			"paddings" => self.padding = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"color" => self.background = Some(value.expect_dcolor()?),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl VBox {
	pub fn builder() -> VBoxBuilder {
		VBoxBuilder::new()
	}
}


#[derive(Default)]
pub struct VBoxBuilder {
	inner: VBox,
}

impl VBoxBuilder {
	pub fn new() -> Self {
		Self {
			inner: VBox::default(),
		}
	}

	pub fn spacing(mut self, spacing: f32) -> Self {
		self.inner.spacing = spacing;
		self
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn paddings(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
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

	pub fn scrollable(mut self, scrollable: bool) -> Self {
		self.inner.scrollable = scrollable;
		self
	}

	pub fn build(self) -> VBox {
		self.inner
	}
}

impl From<VBoxBuilder> for VBox {
	fn from(builder: VBoxBuilder) -> Self {
		builder.build()
	}
}


// HBox
pub struct HBox {
	pub spacing: f32,
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
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
		let pad = self.padding * 2.0;
		let avail_w = if constraints.max_width.is_finite() {
			(constraints.max_width - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let avail_h = if constraints.max_height.is_finite() {
			(constraints.max_height - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let tight_h = constraints.min_height >= constraints.max_height && avail_h.is_finite();
		let can_expand_x = avail_w.is_finite() && !self.scrollable;
		let total_spacing = if children.len() > 1 {
			self.spacing * (children.len() - 1) as f32
		} else {
			0.0
		};

		let mut fixed_width = 0.0f32;
		let mut max_height = 0.0f32;
		let mut expand_count = 0usize;
		let mut items = Vec::with_capacity(children.len());

		// Measure non-expanding children first
		for child in children {
			let anchor = ctx.child_anchor(child);
			let is_expand_x = can_expand_x && matches!(anchor, Some(Anchor::Fill));
			let is_fill_y = matches!(anchor, Some(Anchor::Fill | Anchor::LeftFill | Anchor::RightFill));

			if is_expand_x {
				expand_count += 1;
				items.push((child, anchor, true, is_fill_y, Vec2::ZERO));
			} else {
				let child_constraints = if is_fill_y && tight_h {
					BoxConstraints::new(0.0, f32::INFINITY, avail_h, avail_h)
				} else {
					BoxConstraints::new(0.0, f32::INFINITY, 0.0, avail_h)
				};
				let size = ctx.layout_child(child, child_constraints);
				fixed_width += size.x;
				max_height = max_height.max(size.y);
				items.push((child, anchor, false, is_fill_y, size));
			}
		}

		// Distribute remaining horizontal space to `anchor: "fill"` children
		let mut total_children_w = fixed_width;
		if expand_count > 0 {
			let remaining_w = (avail_w - fixed_width - total_spacing).max(0.0);
			let share_w = remaining_w / expand_count as f32;
			let expand_constraints = if tight_h {
				BoxConstraints::new(share_w, share_w, avail_h, avail_h)
			} else {
				BoxConstraints::new(share_w, share_w, 0.0, avail_h)
			};

			for (child, _, is_expand_x, _, size) in &mut items {
				if *is_expand_x {
					*size = ctx.layout_child(*child, expand_constraints);
					total_children_w += size.x;
					max_height = max_height.max(size.y);
				}
			}
		}

		let current_x = total_children_w + total_spacing + pad;
		let total_height = max_height + pad;
		let final_size = constraints.constrain(Vec2::new(current_x, total_height));
		let inner_h = (final_size.y - pad).max(0.0);

		let scroll_offset = if self.scrollable {
			ctx.set_clip_children(true);
			self.scroll.update(current_x, final_size.x);
			self.scroll.sync();
			self.scroll.offset
		} else {
			0.0
		};

		let mut pos_x = self.padding - scroll_offset;
		for (i, (child, anchor, is_expand_x, is_fill_y, mut size)) in items.into_iter().enumerate() {
			if i > 0 {
				pos_x += self.spacing;
			}
			if is_fill_y && (size.y - inner_h).abs() > 0.5 {
				let c = if is_expand_x {
					BoxConstraints::new(size.x, size.x, inner_h, inner_h)
				} else {
					BoxConstraints::new(0.0, f32::INFINITY, inner_h, inner_h)
				};
				size = ctx.layout_child(child, c);
			}
			let pos_y = match anchor {
				Some(Anchor::Center | Anchor::CenterLeft | Anchor::CenterRight) => {
					self.padding + (inner_h - size.y).max(0.0) * 0.5
				}
				Some(Anchor::BottomCenter | Anchor::BottomLeft | Anchor::BottomRight | Anchor::BottomFill) => {
					self.padding + (inner_h - size.y).max(0.0)
				}
				_ => self.padding,
			};
			ctx.set_child_position(child, Vec2::new(pos_x, pos_y));
			pos_x += size.x;
		}

		final_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.none);
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		let mut shape = RectShape::new(ctx.bounds, bg).radius(radius);

		if self.border_width > 0.0 {
			let border_col = self
				.border_color
				.map(|c| c.get(monet))
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

	fn children_stretch(&self) -> Option<([f32; 2], [f32; 2])> {
		if self.scrollable {
			self.scroll.stretch_transform(self.last_bounds.get(), false)
		} else {
			None
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"scrollable" => self.scrollable = value.expect_bool()?,
			"spacing" => self.spacing = value.expect_f32()?,
			"paddings" => self.padding = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"color" => self.background = Some(value.expect_dcolor()?),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl HBox {
	pub fn builder() -> HBoxBuilder {
		HBoxBuilder::new()
	}
}


#[derive(Default)]
pub struct HBoxBuilder {
	inner: HBox,
}

impl HBoxBuilder {
	pub fn new() -> Self {
		Self {
			inner: HBox::default(),
		}
	}

	pub fn spacing(mut self, spacing: f32) -> Self {
		self.inner.spacing = spacing;
		self
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn paddings(mut self, padding: f32) -> Self {
		self.inner.padding = padding;
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

	pub fn scrollable(mut self, scrollable: bool) -> Self {
		self.inner.scrollable = scrollable;
		self
	}

	pub fn build(self) -> HBox {
		self.inner
	}
}

impl From<HBoxBuilder> for HBox {
	fn from(builder: HBoxBuilder) -> Self {
		builder.build()
	}
}
