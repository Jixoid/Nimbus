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
	Anchor, Animated, BoxConstraints, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	IntoMethod, LayoutCtx, MethodCallback, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Collapsible {
	pub title: String,
	pub open: bool,
	pub spacing: f32,
	pub padding: f32,
	pub header_height: f32,
	pub corner_radius: Option<f32>,
	pub background: Option<DColor>,
	pub header_background: Option<DColor>,
	pub color: Option<DColor>,
	pub border_width: f32,
	pub border_color: Option<DColor>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_changed: Option<MethodCallback>,
	pub progress: Animated<f32>,

	header_hovered: bool,
	last_bounds: Cell<Rect<f32>>,
	last_size: Vec2<f32>,
}

impl Default for Collapsible {
	fn default() -> Self {
		Self {
			title: "Collapsible".to_string(),
			open: false,
			spacing: 10.0,
			padding: 12.0,
			header_height: 38.0,
			corner_radius: None,
			background: None,
			header_background: None,
			color: None,
			border_width: 0.0,
			border_color: None,
			font: None,
			font_size: 15.0,
			on_changed: None,
			progress: Animated::new(0.0),
			header_hovered: false,
			last_bounds: Cell::new(Rect::ZERO),
			last_size: Vec2::ZERO,
		}
	}
}

impl Collapsible {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			..Default::default()
		}
	}

	pub fn set_open(&mut self, open: bool, animate: bool) {
		self.open = open;
		let target = if open { 1.0 } else { 0.0 };
		if animate {
			self.progress.set_target(
				target,
				Duration::from_millis(220),
				Easing::EmphasizedDecelerate,
			);
		} else {
			self.progress.snap_to(target);
		}
	}

	fn active_bounds(&self) -> Rect<f32> {
		let b = self.last_bounds.get();
		if b != Rect::ZERO {
			b
		} else {
			Rect::new(0.0, 0.0, self.last_size.x, self.last_size.y)
		}
	}

	fn header_rect(&self, bounds: Rect<f32>) -> Rect<f32> {
		Rect::new(
			bounds.x(),
			bounds.y(),
			bounds.width(),
			self.header_height.min(bounds.height().max(self.header_height)),
		)
	}
}

fn draw_rotating_chevron(
	canvas: &mut Canvas2D,
	cx: f32,
	cy: f32,
	size: f32,
	thickness: f32,
	angle: f32,
	color: Color,
) {
	let cos_a = angle.cos();
	let sin_a = angle.sin();
	let steps = (size / 0.35).ceil().max(12.0) as usize;
	let r = thickness * 0.5;

	let tip = Vec2::new(size * 0.4, 0.0);
	let top = Vec2::new(-size * 0.6, -size);
	let bot = Vec2::new(-size * 0.6, size);

	for i in 0..=steps {
		let t = i as f32 / steps as f32;

		let p1 = Vec2::new(
			top.x + (tip.x - top.x) * t,
			top.y + (tip.y - top.y) * t,
		);
		let p2 = Vec2::new(
			bot.x + (tip.x - bot.x) * t,
			bot.y + (tip.y - bot.y) * t,
		);

		let arms: &[Vec2<f32>] = if i == steps { &[p1] } else { &[p1, p2] };
		for &p in arms {
			let rx = p.x * cos_a - p.y * sin_a;
			let ry = p.x * sin_a + p.y * cos_a;
			canvas.draw(
				RectShape::new(
					Rect::new(cx + rx - r, cy + ry - r, thickness, thickness),
					color,
				)
				.radius(r),
			);
		}
	}
}

impl Widget for Collapsible {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		ctx.set_clip_children(true);

		let (title_w, _) = ctx.measure_text_with_font(&self.title, self.font, self.font_size);
		let header_min_w = title_w + 52.0;

		let children = ctx.children();
		let pad = self.padding * 2.0;
		let avail_w = if constraints.max_width.is_finite() {
			(constraints.max_width - pad).max(0.0)
		} else {
			f32::INFINITY
		};
		let tight_w = constraints.min_width >= constraints.max_width && avail_w.is_finite();
		let total_spacing = if children.len() > 1 {
			self.spacing * (children.len() - 1) as f32
		} else {
			0.0
		};

		let mut total_children_h = 0.0f32;
		let mut max_child_w = 0.0f32;
		let mut items = Vec::with_capacity(children.len());

		for child in children {
			let anchor = ctx.child_anchor(child);
			let is_fill_x = matches!(anchor, Some(Anchor::Fill | Anchor::TopFill | Anchor::BottomFill));
			let child_constraints = if is_fill_x && tight_w {
				BoxConstraints::new(avail_w, avail_w, 0.0, f32::INFINITY)
			} else {
				BoxConstraints::new(0.0, avail_w, 0.0, f32::INFINITY)
			};

			let size = ctx.layout_child(child, child_constraints);
			total_children_h += size.y;
			max_child_w = max_child_w.max(size.x);
			items.push((child, anchor, is_fill_x, size));
		}

		let body_full_h = if items.is_empty() {
			0.0
		} else {
			total_children_h + total_spacing + pad
		};

		let t = self.progress.value().clamp(0.0, 1.0);
		let total_width = (max_child_w + pad).max(header_min_w);
		let total_height = self.header_height + body_full_h * t;
		let final_size = constraints.constrain(Vec2::new(total_width.ceil(), total_height.ceil()));
		self.last_size = final_size;

		let inner_w = (final_size.x - pad).max(0.0);
		let mut pos_y = self.header_height + self.padding;
		for (i, (child, anchor, is_fill_x, mut size)) in items.into_iter().enumerate() {
			if i > 0 {
				pos_y += self.spacing;
			}
			if is_fill_x && (size.x - inner_w).abs() > 0.5 {
				let c = BoxConstraints::new(inner_w, inner_w, 0.0, f32::INFINITY);
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
		self.last_bounds.set(ctx.bounds);
		if self.progress.is_animating() {
			ctx.request_layout();
			ctx.request_redraw();
		}

		let monet = &ctx.theme.monet;
		let t = self.progress.value().clamp(0.0, 1.0);
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);
		let border_col = self
			.border_color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceOutline]);

		let has_expanded_body = t > 0.01 && ctx.bounds.height() > self.header_height + 0.5;
		if has_expanded_body {
			let bg = self
				.background
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);
			let mut body_shape = RectShape::new(ctx.bounds, bg).radius(radius);
			if self.border_width > 0.0 {
				body_shape = body_shape.border(border_col, self.border_width);
			}
			canvas.draw(body_shape);
		}

		let inset = if has_expanded_body && self.border_width > 0.0 {
			self.border_width
		} else {
			0.0
		};
		let header_h = (self.header_height - inset).min(ctx.bounds.height()).max(0.0);
		let header_rect = Rect::new(
			ctx.bounds.x() + inset,
			ctx.bounds.y() + inset,
			(ctx.bounds.width() - inset * 2.0).max(0.0),
			header_h,
		);
		let top_r = (radius - inset).max(0.0);
		let bot_r = if t > 0.05 && has_expanded_body {
			(top_r * (1.0 - t)).max(0.0)
		} else {
			top_r
		};

		let header_bg = if self.header_hovered {
			monet[MonetRole::SurfaceContainerHighest]
		} else {
			self.header_background
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::SurfaceContainerHigh])
		};

		let mut header_shape =
			RectShape::new(header_rect, header_bg).radius4([top_r, top_r, bot_r, bot_r]);
		if !has_expanded_body && self.border_width > 0.0 {
			header_shape = header_shape.border(border_col, self.border_width);
		}
		canvas.draw(header_shape);

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let (_, text_h) = canvas.measure_text(&self.title, self.font_size);
		let text_color = self
			.color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::OnSurface]);
		let text_pos = Vec2::new(
			header_rect.x() + 14.0,
			header_rect.y() + (header_rect.height() - text_h) * 0.5,
		);
		canvas.draw(TextShape::new(
			&self.title,
			font,
			self.font_size,
			text_pos,
			text_color,
		));

		let chevron_color = monet[MonetRole::OnSurfaceVariant];
		let angle = t * std::f32::consts::FRAC_PI_2;
		draw_rotating_chevron(
			canvas,
			header_rect.max.x - 18.0,
			header_rect.center().y,
			4.5,
			1.4,
			angle,
			chevron_color,
		);
	}

	fn should_draw_children(&self) -> bool {
		self.open || self.progress.value() > 0.001
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.active_bounds();
		let header_rect = self.header_rect(bounds);

		match event {
			InputEvent::PointerMoved { position } => {
				let hovered = header_rect.contains_point(*position);
				if hovered != self.header_hovered {
					self.header_hovered = hovered;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}
			InputEvent::PointerLeft => {
				if self.header_hovered {
					self.header_hovered = false;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}
			InputEvent::Click { position } => {
				if header_rect.contains_point(*position) {
					self.set_open(!self.open, true);
					if let Some(cb) = &mut self.on_changed {
						cb.call1(self.open);
					}
					ctx.request_layout();
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}
			_ => EventResponse::Ignored,
		}
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), PropertyError> {
		match key {
			"title" => self.title = value.expect_str()?.to_string(),
			"expanded" => self.set_open(value.expect_bool()?, false),
			"spacing" => self.spacing = value.expect_f32()?,
			"paddings" => self.padding = value.expect_f32()?,
			"header_height" => self.header_height = value.expect_f32()?.max(20.0),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"color" => self.background = Some(value.expect_dcolor()?),
			"header_color" => self.header_background = Some(value.expect_dcolor()?),
			"text_color" => self.color = Some(value.expect_dcolor()?),
			"border_color" => self.border_color = Some(value.expect_dcolor()?),
			"border_width" => self.border_width = value.expect_f32()?,
			"font_size" => self.font_size = value.expect_f32()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed")
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

impl Collapsible {
	pub fn builder() -> CollapsibleBuilder {
		CollapsibleBuilder::new()
	}
}


#[derive(Default)]
pub struct CollapsibleBuilder {
	inner: Collapsible,
}

impl CollapsibleBuilder {
	pub fn new() -> Self {
		Self {
			inner: Collapsible::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn text(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn expanded(mut self, expanded: bool) -> Self {
		self.inner.set_open(expanded, false);
		self
	}

	pub fn open(mut self, open: bool) -> Self {
		self.inner.set_open(open, false);
		self
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

	pub fn header_height(mut self, height: f32) -> Self {
		self.inner.header_height = height.max(20.0);
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

	pub fn header_background(mut self, color: impl Into<DColor>) -> Self {
		self.inner.header_background = Some(color.into());
		self
	}

	pub fn header_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.header_background = Some(color.into());
		self
	}

	pub fn text_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
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

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn font_size(mut self, size: f32) -> Self {
		self.inner.font_size = size;
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> Collapsible {
		self.inner
	}
}

impl From<CollapsibleBuilder> for Collapsible {
	fn from(builder: CollapsibleBuilder) -> Self {
		builder.build()
	}
}
