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
	IntoMethod, LayoutCtx, MethodCallback, ObjectID, PropertyCtx, PropertyError, PropertyValue,
	UiTree, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;

use crate::{HBox, Spinner, Text};


pub struct Dialog {
	pub title: Option<String>,
	pub open: bool,
	pub closable: bool,
	pub close_on_backdrop: bool,
	pub width: Option<f32>,
	pub height: Option<f32>,
	pub min_width: f32,
	pub spacing: f32,
	pub padding: f32,
	pub corner_radius: Option<f32>,
	pub backdrop_radius: Option<f32>,
	pub background: Option<Color>,
	pub backdrop_color: Option<Color>,
	pub title_color: Option<Color>,
	pub border_width: f32,
	pub border_color: Option<Color>,
	pub shadow_blur: f32,
	pub font: Option<FontId>,
	pub title_font_size: f32,
	pub on_close: Option<MethodCallback>,
	pub progress: Animated<f32>,

	close_hovered: bool,
	last_bounds: Cell<Rect<f32>>,
	last_card_rect: Cell<Rect<f32>>,
	was_animating: Cell<bool>,
}

impl Default for Dialog {
	fn default() -> Self {
		Self {
			title: None,
			open: true,
			closable: false,
			close_on_backdrop: false,
			width: None,
			height: None,
			min_width: 240.0,
			spacing: 12.0,
			padding: 20.0,
			corner_radius: None,
			backdrop_radius: None,
			background: None,
			backdrop_color: None,
			title_color: None,
			border_width: 1.0,
			border_color: None,
			shadow_blur: 18.0,
			font: None,
			title_font_size: 17.0,
			on_close: None,
			progress: Animated::new(1.0),
			close_hovered: false,
			last_bounds: Cell::new(Rect::ZERO),
			last_card_rect: Cell::new(Rect::ZERO),
			was_animating: Cell::new(false),
		}
	}
}

impl Dialog {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: Some(title.into()),
			..Default::default()
		}
	}

	pub fn builder() -> DialogBuilder {
		DialogBuilder::new()
	}

	pub fn attach_wait_overlay(
		tree: &mut UiTree,
		target: ObjectID,
		title: impl Into<String>,
		message: impl Into<String>,
	) -> ObjectID {
		let mut dlg = Dialog::builder()
			.title(title)
			.open(true)
			.closable(false)
			.close_on_backdrop(false)
			.build();
		dlg.progress.snap_to(0.0);
		dlg.set_open(true, true);

		let dlg_id = tree.insert_overlay(target, dlg);
		let row_id = tree.insert_child(
			dlg_id,
			HBox::builder()
				.spacing(14.0)
				.color(Color::TRANSPARENT)
				.build(),
		);
		tree.insert_child(row_id, Spinner::builder().size(28.0).build());
		let text_id = tree.insert_child(row_id, Text::builder().text(message).build());
		tree.set_anchor(text_id, Some(Anchor::Center));
		dlg_id
	}

	pub fn set_open(&mut self, open: bool, animate: bool) {
		self.open = open;
		let target = if open { 1.0 } else { 0.0 };
		if animate {
			let (dur, easing) = if open {
				(Duration::from_millis(200), Easing::EmphasizedDecelerate)
			} else {
				(Duration::from_millis(150), Easing::Accelerate)
			};
			self.progress.set_target(target, dur, easing);
		} else {
			self.progress.snap_to(target);
		}
	}

	pub fn is_visible(&self) -> bool {
		self.open || self.progress.value() > 0.001
	}

	fn has_header(&self) -> bool {
		self.title.as_ref().is_some_and(|t| !t.is_empty()) || self.closable
	}

	fn header_height(&self) -> f32 {
		if self.has_header() {
			32.0
		} else {
			0.0
		}
	}

	fn close_button_rect(&self, card_rect: Rect<f32>) -> Rect<f32> {
		let btn_size = 26.0;
		let x = card_rect.max.x - self.padding - btn_size;
		let y = card_rect.y() + self.padding + (self.header_height() - btn_size) * 0.5;
		Rect::new(x, y, btn_size, btn_size)
	}
}

fn draw_close_icon(canvas: &mut Canvas2D, center: Vec2<f32>, half_extent: f32, thickness: f32, color: Color) {
	let steps = 12usize;
	for i in 0..=steps {
		let t = (i as f32 / steps as f32) * 2.0 - 1.0; // -1.0 to 1.0
		let offset = t * half_extent;

		canvas.draw(RectShape::new(
			Rect::new(
				center.x + offset - thickness * 0.5,
				center.y + offset - thickness * 0.5,
				thickness,
				thickness,
			),
			color,
		));
		canvas.draw(RectShape::new(
			Rect::new(
				center.x + offset - thickness * 0.5,
				center.y - offset - thickness * 0.5,
				thickness,
				thickness,
			),
			color,
		));
	}
}

impl Widget for Dialog {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		ctx.set_clip_children(true);

		let t = self.progress.value().clamp(0.0, 1.0);
		if !self.open && !self.progress.is_animating() && t <= 0.001 {
			ctx.tree.set_enabled(ctx.node_id, false);
			for child in ctx.children() {
				ctx.layout_child(child, BoxConstraints::tight(Vec2::ZERO));
			}
			self.last_card_rect.set(Rect::ZERO);
			return Vec2::ZERO;
		}

		ctx.tree.set_enabled(ctx.node_id, true);

		let pad = self.padding * 2.0;
		let has_header = self.has_header();
		let header_h = self.header_height();

		let header_min_w = if let Some(title) = &self.title {
			let (tw, _) = ctx.measure_text_with_font(title, self.font, self.title_font_size);
			let close_extra = if self.closable { 36.0 } else { 0.0 };
			tw + close_extra + pad
		} else if self.closable {
			36.0 + pad
		} else {
			0.0
		};

		let max_overlay_w = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			800.0
		};
		let max_overlay_h = if constraints.max_height.is_finite() {
			constraints.max_height
		} else {
			600.0
		};

		let max_card_w = self
			.width
			.unwrap_or_else(|| (max_overlay_w - 32.0).max(self.min_width));
		let child_avail_w = (max_card_w - pad).max(0.0);
		let tight_card_w = self.width.is_some();

		let children = ctx.children();
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
			let child_constraints = if is_fill_x && tight_card_w {
				BoxConstraints::new(child_avail_w, child_avail_w, 0.0, f32::INFINITY)
			} else {
				BoxConstraints::new(0.0, child_avail_w, 0.0, f32::INFINITY)
			};

			let size = ctx.layout_child(child, child_constraints);
			total_children_h += size.y;
			max_child_w = max_child_w.max(size.x);
			items.push((child, anchor, is_fill_x, size));
		}

		let header_and_gap = if has_header {
			if items.is_empty() {
				header_h
			} else {
				header_h + self.spacing
			}
		} else {
			0.0
		};

		let card_w = self
			.width
			.unwrap_or_else(|| (max_child_w + pad).max(header_min_w).max(self.min_width))
			.min(max_overlay_w)
			.ceil();
		let card_h = self
			.height
			.unwrap_or(header_and_gap + total_children_h + total_spacing + pad)
			.min(max_overlay_h)
			.ceil();

		let overlay_size = constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				card_w + 64.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				card_h + 64.0
			},
		));

		let slide_y = (1.0 - t) * 10.0;
		let card_x = ((overlay_size.x - card_w) * 0.5).max(0.0);
		let card_y = ((overlay_size.y - card_h) * 0.5 + slide_y).max(0.0);
		let local_card_rect = Rect::new(card_x, card_y, card_w, card_h);
		self.last_card_rect.set(local_card_rect);

		let inner_w = (card_w - pad).max(0.0);
		let mut pos_y = card_y + self.padding + header_and_gap;

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
					card_x + self.padding + (inner_w - size.x).max(0.0) * 0.5
				}
				Some(Anchor::CenterRight | Anchor::TopRight | Anchor::BottomRight | Anchor::RightFill) => {
					card_x + self.padding + (inner_w - size.x).max(0.0)
				}
				_ => card_x + self.padding,
			};
			ctx.set_child_position(child, Vec2::new(pos_x, pos_y));
			pos_y += size.y;
		}

		overlay_size
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let animating = self.progress.is_animating();
		if animating || self.was_animating.get() {
			ctx.request_layout();
			ctx.request_redraw();
		}
		self.was_animating.set(animating);

		let t = self.progress.value().clamp(0.0, 1.0);
		if t <= 0.001 {
			return;
		}

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;

		// Backdrop scrim
		let mut backdrop = self
			.backdrop_color
			.unwrap_or_else(|| Color::new(0.0, 0.0, 0.0, 0.48));
		backdrop.a *= t;
		if backdrop.a > 0.001 {
			let mut scrim = RectShape::new(bounds, backdrop);
			if let Some(r) = self.backdrop_radius {
				scrim = scrim.radius(r);
			}
			canvas.draw(scrim);
		}

		// Centered dialog card
		let local_card = self.last_card_rect.get();
		let card_rect = Rect::new(
			bounds.x() + local_card.x(),
			bounds.y() + local_card.y(),
			local_card.width(),
			local_card.height(),
		);

		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.large);
		let mut bg = self
			.background
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainerHigh]);
		bg.a *= t;

		let mut card_shape = RectShape::new(card_rect, bg).radius(radius);

		if self.border_width > 0.0 {
			let mut border_col = self
				.border_color
				.unwrap_or_else(|| monet[MonetRole::SurfaceOutlineVariant]);
			border_col.a *= t;
			card_shape = card_shape.border(border_col, self.border_width);
		}

		if self.shadow_blur > 0.0 {
			let shadow_col = Color::new(0.0, 0.0, 0.0, 0.42 * t);
			card_shape = card_shape.shadow(shadow_col, Vec2::new(0.0, 6.0), self.shadow_blur);
		}

		canvas.draw(card_shape);

		// Header (Title & Close button)
		if self.has_header() {
			let header_h = self.header_height();
			if let Some(title) = &self.title
				&& !title.is_empty()
			{
				let fface = FontManager::default_font();
				let font = match self.font {
					Some(id) => Font::with_id(id, fface, self.title_font_size),
					None => Font::new(fface, self.title_font_size),
				};
				let (_, text_h) = canvas.measure_text(title, self.title_font_size);
				let mut text_col = self
					.title_color
					.unwrap_or_else(|| monet[MonetRole::OnSurface]);
				text_col.a *= t;

				let tx = card_rect.x() + self.padding;
				let ty = card_rect.y() + self.padding + (header_h - text_h) * 0.5;
				canvas.draw(TextShape::new(
					title,
					font,
					self.title_font_size,
					Vec2::new(tx, ty),
					text_col,
				));
			}

			if self.closable {
				let close_rect = self.close_button_rect(card_rect);
				if self.close_hovered {
					let mut hover_bg = monet[MonetRole::SurfaceContainerHighest];
					hover_bg.a *= t;
					canvas.draw(
						RectShape::new(close_rect, hover_bg).radius(close_rect.height() * 0.5),
					);
				}

				let mut icon_col = if self.close_hovered {
					monet[MonetRole::OnSurface]
				} else {
					monet[MonetRole::OnSurfaceVariant]
				};
				icon_col.a *= t;
				draw_close_icon(canvas, close_rect.center(), 4.5, 1.5, icon_col);
			}
		}
	}

	fn should_draw_children(&self) -> bool {
		self.is_visible()
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if !self.is_visible() {
			return EventResponse::Ignored;
		}

		let bounds = self.last_bounds.get();
		let local_card = self.last_card_rect.get();
		let card_rect = Rect::new(
			bounds.x() + local_card.x(),
			bounds.y() + local_card.y(),
			local_card.width(),
			local_card.height(),
		);

		match event {
			InputEvent::PointerMoved { position } => {
				if self.closable {
					let close_rect = self.close_button_rect(card_rect);
					let hovered = close_rect.contains_point(*position);
					if hovered != self.close_hovered {
						self.close_hovered = hovered;
						ctx.request_redraw();
					}
				}
				EventResponse::Handled
			}

			InputEvent::PointerLeft => {
				if self.close_hovered {
					self.close_hovered = false;
					ctx.request_redraw();
				}
				EventResponse::Handled
			}

			InputEvent::Click { position } => {
				if self.closable {
					let close_rect = self.close_button_rect(card_rect);
					if close_rect.contains_point(*position) {
						self.close_hovered = false;
						self.set_open(false, true);
						if let Some(cb) = &self.on_close {
							cb.call0();
						}
						ctx.request_layout();
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}

				if self.close_on_backdrop && !card_rect.contains_point(*position) {
					self.set_open(false, true);
					if let Some(cb) = &self.on_close {
						cb.call0();
					}
					ctx.request_layout();
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				EventResponse::Handled
			}

			// Consume mouse button and scroll events on the modal backdrop so they don't leak through
			InputEvent::PointerButton { .. } | InputEvent::PointerScroll { .. } => {
				EventResponse::Handled
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"title" | "text" => {
				let s = value.expect_str()?;
				self.title = if s.is_empty() { None } else { Some(s.to_string()) };
			}
			"open" | "expanded" => self.set_open(value.expect_bool()?, false),
			"closable" => self.closable = value.expect_bool()?,
			"close_on_backdrop" => self.close_on_backdrop = value.expect_bool()?,
			"width" => self.width = Some(value.expect_f32()?.max(0.0)),
			"height" => self.height = Some(value.expect_f32()?.max(0.0)),
			"min_width" => self.min_width = value.expect_f32()?.max(0.0),
			"spacing" => self.spacing = value.expect_f32()?,
			"paddings" | "padding" => self.padding = value.expect_f32()?,
			"corner_radius" | "radius" => self.corner_radius = Some(value.expect_f32()?),
			"backdrop_radius" => self.backdrop_radius = Some(value.expect_f32()?),
			"color" | "background" => self.background = Some(value.expect_color(&ctx.theme.monet)?),
			"backdrop_color" => self.backdrop_color = Some(value.expect_color(&ctx.theme.monet)?),
			"title_color" | "text_color" => {
				self.title_color = Some(value.expect_color(&ctx.theme.monet)?)
			}
			"border_color" => self.border_color = Some(value.expect_color(&ctx.theme.monet)?),
			"border_width" => self.border_width = value.expect_f32()?,
			"shadow_blur" => self.shadow_blur = value.expect_f32()?,
			"font_size" | "title_font_size" => self.title_font_size = value.expect_f32()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_close" | "on_closed")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_close = Some(callback);
			true
		} else {
			false
		}
	}
}


#[derive(Default)]
pub struct DialogBuilder {
	inner: Dialog,
}

impl DialogBuilder {
	pub fn new() -> Self {
		Self {
			inner: Dialog::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		let s = title.into();
		self.inner.title = if s.is_empty() { None } else { Some(s) };
		self
	}

	pub fn open(mut self, open: bool) -> Self {
		self.inner.set_open(open, false);
		self
	}

	pub fn closable(mut self, closable: bool) -> Self {
		self.inner.closable = closable;
		self
	}

	pub fn close_on_backdrop(mut self, close_on_backdrop: bool) -> Self {
		self.inner.close_on_backdrop = close_on_backdrop;
		self
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

	pub fn min_width(mut self, min_width: f32) -> Self {
		self.inner.min_width = min_width.max(0.0);
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

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn backdrop_radius(mut self, radius: f32) -> Self {
		self.inner.backdrop_radius = Some(radius);
		self
	}

	pub fn background(mut self, color: Color) -> Self {
		self.inner.background = Some(color);
		self
	}

	pub fn color(mut self, color: Color) -> Self {
		self.inner.background = Some(color);
		self
	}

	pub fn backdrop_color(mut self, color: Color) -> Self {
		self.inner.backdrop_color = Some(color);
		self
	}

	pub fn title_color(mut self, color: Color) -> Self {
		self.inner.title_color = Some(color);
		self
	}

	pub fn border_width(mut self, width: f32) -> Self {
		self.inner.border_width = width;
		self
	}

	pub fn border_color(mut self, color: Color) -> Self {
		self.inner.border_color = Some(color);
		self
	}

	pub fn shadow_blur(mut self, blur: f32) -> Self {
		self.inner.shadow_blur = blur;
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn title_font_size(mut self, size: f32) -> Self {
		self.inner.title_font_size = size;
		self
	}

	pub fn on_close<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_close = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> Dialog {
		self.inner
	}
}

impl From<DialogBuilder> for Dialog {
	fn from(builder: DialogBuilder) -> Self {
		builder.build()
	}
}
