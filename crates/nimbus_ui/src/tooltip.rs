/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	LayoutCtx, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};


#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TooltipPosition {
	#[default]
	Top,
	Bottom,
	Left,
	Right,
}

impl TooltipPosition {
	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_ascii_lowercase().as_str() {
			"top" => Some(Self::Top),
			"bottom" => Some(Self::Bottom),
			"left" => Some(Self::Left),
			"right" => Some(Self::Right),
			_ => None,
		}
	}
}

pub struct Tooltip {
	pub text: String,
	pub position: TooltipPosition,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub corner_radius: Option<f32>,
	pub show_arrow: bool,
	pub background: Option<DColor>,
	pub color: Option<DColor>,
	pub open: Option<bool>,
	pub progress: Animated<f32>,
	hovered: bool,
	has_children: bool,
}

pub type ToolTip = Tooltip;

impl Default for Tooltip {
	fn default() -> Self {
		Self {
			text: any::type_name::<Self>().to_string(),
			position: TooltipPosition::Top,
			font: None,
			font_size: 12.0,
			corner_radius: None,
			show_arrow: true,
			background: None,
			color: None,
			open: None,
			progress: Animated::new(0.0),
			hovered: false,
			has_children: false,
		}
	}
}

impl Tooltip {
	pub fn new(text: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			..Default::default()
		}
	}

	
	pub fn is_visible(&self) -> bool {
		if let Some(forced) = self.open {
			forced || self.progress.value() > 0.01
		} else {
			self.progress.value() > 0.01
		}
	}

	fn draw_bubble(
		&self,
		canvas: &mut Canvas2D,
		theme: &nimbus_theme::Theme,
		font: Font,
		bubble_rect: Rect<f32>,
		t: f32,
	) {
		let monet = &theme.monet;
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainerHighest]);
		let text_color = self
			.color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::OnSurface]);
		let border_color = monet[MonetRole::SurfaceOutlineVariant];
		let shadow_color = Color::new(0.0, 0.0, 0.0, 0.3 * t);

		// Bubble background with border & soft shadow
		canvas.draw(
			RectShape::new(bubble_rect, bg)
				.radius(self.corner_radius.unwrap_or(theme.radius.small))
				.border(border_color, theme.stroke.thin)
				.shadow(shadow_color, Vec2::new(0.0, 2.0), 6.0)
				.noise(0.02),
		);

		// Bubble text
		let fface = FontManager::default_font();
		let (tw, th) = fface.measure_text(&self.text, self.font_size);
		let text_pos = Vec2::new(
			bubble_rect.center().x - tw * 0.5,
			bubble_rect.center().y - th * 0.5,
		);
		canvas.draw(TextShape::new(&self.text, font, self.font_size, text_pos, text_color));

		// Arrow / Beak
		if self.show_arrow && self.has_children {
			let arrow_w = 8.0f32;
			let arrow_h = 4.0f32;

			match self.position {
				TooltipPosition::Top => {
					draw_pointer_down(
						canvas,
						bubble_rect.center().x,
						bubble_rect.max.y + arrow_h,
						arrow_w,
						arrow_h,
						bg,
					);
				}
				TooltipPosition::Bottom => {
					draw_pointer_up(
						canvas,
						bubble_rect.center().x,
						bubble_rect.y() - arrow_h,
						arrow_w,
						arrow_h,
						bg,
					);
				}
				TooltipPosition::Left => {
					draw_pointer_right(
						canvas,
						bubble_rect.max.x + arrow_h,
						bubble_rect.center().y,
						arrow_h,
						arrow_w,
						bg,
					);
				}
				TooltipPosition::Right => {
					draw_pointer_left(
						canvas,
						bubble_rect.x() - arrow_h,
						bubble_rect.center().y,
						arrow_h,
						arrow_w,
						bg,
					);
				}
			}
		}
	}
}


fn draw_pointer_down(canvas: &mut Canvas2D, tip_x: f32, tip_y: f32, width: f32, height: f32, color: Color) {
	let steps = (height / 0.5).ceil() as usize;
	for i in 0..steps {
		let t = i as f32 / steps.max(1) as f32; // 0.0 (top) to 1.0 (tip)
		let slice_w = (1.0 - t) * width;
		let slice_x = tip_x - slice_w * 0.5;
		let slice_y = tip_y - height + (i as f32) * 0.5;
		canvas.draw(RectShape::new(Rect::new(slice_x, slice_y, slice_w, 0.6), color));
	}
}

fn draw_pointer_up(canvas: &mut Canvas2D, tip_x: f32, tip_y: f32, width: f32, height: f32, color: Color) {
	let steps = (height / 0.5).ceil() as usize;
	for i in 0..steps {
		let t = i as f32 / steps.max(1) as f32; // 0.0 (tip) to 1.0 (base)
		let slice_w = t * width;
		let slice_x = tip_x - slice_w * 0.5;
		let slice_y = tip_y + (i as f32) * 0.5;
		canvas.draw(RectShape::new(Rect::new(slice_x, slice_y, slice_w, 0.6), color));
	}
}

fn draw_pointer_right(canvas: &mut Canvas2D, tip_x: f32, cy: f32, width: f32, height: f32, color: Color) {
	let steps = (width / 0.5).ceil() as usize;
	for i in 0..steps {
		let t = i as f32 / steps.max(1) as f32;
		let slice_h = (1.0 - t) * height;
		let slice_y = cy - slice_h * 0.5;
		let slice_x = tip_x - width + (i as f32) * 0.5;
		canvas.draw(RectShape::new(Rect::new(slice_x, slice_y, 0.6, slice_h), color));
	}
}

fn draw_pointer_left(canvas: &mut Canvas2D, tip_x: f32, cy: f32, width: f32, height: f32, color: Color) {
	let steps = (width / 0.5).ceil() as usize;
	for i in 0..steps {
		let t = i as f32 / steps.max(1) as f32;
		let slice_h = t * height;
		let slice_y = cy - slice_h * 0.5;
		let slice_x = tip_x + (i as f32) * 0.5;
		canvas.draw(RectShape::new(Rect::new(slice_x, slice_y, 0.6, slice_h), color));
	}
}


impl Widget for Tooltip {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let children = ctx.children();
		self.has_children = !children.is_empty();

		if self.has_children {
			let mut max_w = 0.0f32;
			let mut max_h = 0.0f32;
			for child in children {
				let size = ctx.layout_child(child, constraints);
				ctx.set_child_position(child, Vec2::ZERO);
				max_w = max_w.max(size.x);
				max_h = max_h.max(size.y);
			}
			constraints.constrain(Vec2::new(max_w, max_h))
		} else {
			let fface = FontManager::default_font();
			let (tw, th) = fface.measure_text(&self.text, self.font_size);
			let w = tw + 20.0;
			let h = (th + 10.0).max(24.0);
			constraints.constrain(Vec2::new(w.ceil(), h.ceil()))
		}
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		if self.progress.is_animating() {
			ctx.request_redraw();
		}

		let t = if let Some(forced) = self.open {
			if forced { 1.0 } else { self.progress.value() }
		} else {
			self.progress.value()
		};

		if t < 0.01 {
			return;
		}

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		let (tw, th) = fface.measure_text(&self.text, self.font_size);
		let bubble_w = (tw + 16.0).max(36.0);
		let bubble_h = (th + 8.0).max(22.0);

		let bubble_rect = if self.has_children {
			let bounds = ctx.bounds;
			let offset = (1.0 - t) * 4.0;
			let margin = 6.0;

			match self.position {
				TooltipPosition::Top => {
					let bx = bounds.center().x - bubble_w * 0.5;
					let by = bounds.y() - bubble_h - margin - offset;
					Rect::new(bx, by, bubble_w, bubble_h)
				}
				TooltipPosition::Bottom => {
					let bx = bounds.center().x - bubble_w * 0.5;
					let by = bounds.max.y + margin + offset;
					Rect::new(bx, by, bubble_w, bubble_h)
				}
				TooltipPosition::Left => {
					let bx = bounds.x() - bubble_w - margin - offset;
					let by = bounds.center().y - bubble_h * 0.5;
					Rect::new(bx, by, bubble_w, bubble_h)
				}
				TooltipPosition::Right => {
					let bx = bounds.max.x + margin + offset;
					let by = bounds.center().y - bubble_h * 0.5;
					Rect::new(bx, by, bubble_w, bubble_h)
				}
			}
		} else {
			ctx.bounds
		};

		self.draw_bubble(canvas, ctx.theme, font, bubble_rect, t);
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerEntered | InputEvent::PointerMoved { .. } => {
				if !self.hovered {
					self.hovered = true;
					self.progress.set_target(
						1.0,
						Duration::from_millis(160),
						Easing::BackOut,
					);
					ctx.request_redraw();
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered {
					self.hovered = false;
					self.progress.set_target(
						0.0,
						Duration::from_millis(100),
						Easing::EaseIn,
					);
					ctx.request_redraw();
				}
			}

			_ => {}
		}

		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"text" => self.text = value.expect_str()?.to_string(),
			"position" => {
				let s = value.expect_str()?;
				self.position = TooltipPosition::from_str(s).ok_or(
					nimbus_core::PropertyError::InvalidValue {
						expected: "top | bottom | left | right",
					},
				)?;
			}
			"font_size" => self.font_size = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"show_arrow" => self.show_arrow = value.expect_bool()?,
			"open" => {
				let b = value.expect_bool()?;
				self.open = Some(b);
				self.progress.snap_to(if b { 1.0 } else { 0.0 });
			}
			"background" => self.background = Some(value.expect_dcolor()?),
			"color" => self.color = Some(value.expect_dcolor()?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}
}

impl Tooltip {
	pub fn builder() -> TooltipBuilder {
		TooltipBuilder::new()
	}
}


#[derive(Default)]
pub struct TooltipBuilder {
	inner: Tooltip,
}

pub type ToolTipBuilder = TooltipBuilder;

impl TooltipBuilder {
	pub fn new() -> Self {
		Self {
			inner: Tooltip::default(),
		}
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.text = text.into();
		self
	}

	pub fn position(mut self, position: TooltipPosition) -> Self {
		self.inner.position = position;
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

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn show_arrow(mut self, show: bool) -> Self {
		self.inner.show_arrow = show;
		self
	}

	pub fn background(mut self, color: impl Into<DColor>) -> Self {
		self.inner.background = Some(color.into());
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn open(mut self, open: bool) -> Self {
		self.inner.open = Some(open);
		self.inner.progress.snap_to(if open { 1.0 } else { 0.0 });
		self
	}

	pub fn build(self) -> Tooltip {
		self.inner
	}
}

impl From<TooltipBuilder> for Tooltip {
	fn from(builder: TooltipBuilder) -> Self {
		builder.build()
	}
}
