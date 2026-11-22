/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any;

use nimbus_core::{
	BoxConstraints, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod, LayoutCtx,
	MethodCallback, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
	#[default]
	Filled,
	Tonal,
	Elevated,
	Outlined,
}

pub struct Button {
	pub label: String,
	pub variant: ButtonVariant,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub padding: Vec2<f32>,
	pub on_click: Option<MethodCallback>,
}

impl Default for Button {
	fn default() -> Self {
		Self {
			label: any::type_name::<Self>().to_string(),
			variant: ButtonVariant::default(),
			font: None,
			font_size: 15.0,
			padding: Vec2::new(16.0, 8.0),
			on_click: None,
		}
	}
}


impl Widget for Button {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
		let width = text_w + self.padding.x * 2.0;
		let height = text_h + self.padding.y * 2.0;
		constraints.constrain(Vec2::new(width.ceil(), height.ceil()))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let theme = ctx.theme;
		let monet = &theme.monet;

		let (bg_color, fg_color, border_color, border_width, shadow_blur) = match self.variant {
			ButtonVariant::Filled => {
				if ctx.pressed {
					(
						monet[MonetRole::PrimaryFixedDim],
						monet[MonetRole::OnPrimaryFixedVariant],
						monet[MonetRole::PrimaryFixed], 2.0, 0.0
					)
				} else if ctx.hovered {
					(
						monet[MonetRole::PrimaryFixed],
						monet[MonetRole::OnPrimary],
						Color::TRANSPARENT, 0.0, 0.0
					)
				} else {
					(
						monet[MonetRole::Primary],
						monet[MonetRole::OnPrimary],
						Color::TRANSPARENT, 0.0, 0.0
					)
				}
			}
			ButtonVariant::Tonal => {
				if ctx.pressed {
					(
						monet[MonetRole::SecondaryFixedDim],
						monet[MonetRole::OnSecondaryFixedVariant],
						monet[MonetRole::SecondaryFixed], 2.0, 0.0
					)
				} else if ctx.hovered {
					(
						monet[MonetRole::SecondaryFixed],
						monet[MonetRole::OnSecondary],
						Color::TRANSPARENT, 0.0, 0.0
					)
				} else {
					(
						monet[MonetRole::Secondary],
						monet[MonetRole::OnSecondary],
						Color::TRANSPARENT, 0.0, 0.0
					)
				}
			}
			ButtonVariant::Elevated => {
				let (bg, fg) = if ctx.pressed {
					(
						monet[MonetRole::SurfaceContainerHighest],
						monet[MonetRole::OnSurface],
					)
				} else if ctx.hovered {
					(
						monet[MonetRole::SurfaceContainerHigh],
						monet[MonetRole::OnSurface],
					)
				} else {
					(
						monet[MonetRole::SurfaceContainerLow],
						monet[MonetRole::OnSurface],
					)
				};
				let shadow = if ctx.hovered { 6.0 } else { 3.0 };
				(bg, fg, Color::TRANSPARENT, 0.0, shadow)
			}
			ButtonVariant::Outlined => {
				let (bg, fg, border) = if ctx.pressed {
					(
						monet[MonetRole::SurfaceContainerHigh],
						monet[MonetRole::OnSurface],
						monet[MonetRole::Secondary],
					)
				} else if ctx.hovered {
					(
						monet[MonetRole::SurfaceContainerLow],
						monet[MonetRole::OnSurface],
						monet[MonetRole::Secondary],
					)
				} else {
					(
						Color::TRANSPARENT,
						monet[MonetRole::OnSurface],
						monet[MonetRole::SurfaceOutline],
					)
				};
				(bg, fg, border, 2.0, 0.0)
			}
		};

		let radius = (ctx.bounds.height() * 0.5).min(14.0);
		let mut shape = RectShape::new(ctx.bounds, bg_color).radius(radius);
		if border_width > 0.0 {
			shape = shape.border(border_color, border_width);
		}
		if shadow_blur > 0.0 {
			shape = shape.shadow(monet[MonetRole::Shadow], Vec2::new(0.0, 2.0), shadow_blur);
		}
		canvas.draw(shape);

		let (text_w, text_h) = canvas.measure_text(&self.label, self.font_size);
		let text_x = ctx.bounds.x() + (ctx.bounds.width() - text_w) * 0.5;
		let text_y = ctx.bounds.y() + (ctx.bounds.height() - text_h) * 0.5;

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		canvas.draw(TextShape::new(
			&self.label,
			font,
			self.font_size,
			Vec2::new(text_x, text_y),
			fg_color,
		));
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if let InputEvent::Click { .. } = event
			&& let Some(cb) = &mut self.on_click
		{
			cb.call1(self.label.clone());
			ctx.request_redraw();
			return EventResponse::Handled;
		}
		EventResponse::Ignored
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"text" => self.label = value.expect_str()?.to_string(),
			"variant" => {
				self.variant = match value.expect_str()? {
					"filled" => ButtonVariant::Filled,
					"tonal" => ButtonVariant::Tonal,
					"elevated" => ButtonVariant::Elevated,
					"outlined" => ButtonVariant::Outlined,
					_ => return Err(nimbus_core::PropertyError::InvalidValue { expected: "filled | tonal | elevated | outlined" }),
				};
			}
			"font_size" => self.font_size = value.expect_f32()?,
			"paddings" => {
				if let Some(n) = value.as_f32() {
					self.padding = Vec2::new(n, n);
				} else if let Some(v) = value.as_vec2() {
					self.padding = v;
				} else {
					return Err(nimbus_core::PropertyError::InvalidValue { expected: "number or vec2" });
				}
			}
			_ => return Err(nimbus_core::PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_click")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if self.is_method(method_name) {
			self.on_click = Some(callback);
			true
		} else {
			false
		}
	}
}

impl Button {
	pub fn builder() -> ButtonBuilder {
		ButtonBuilder::new()
	}
}


#[derive(Default)]
pub struct ButtonBuilder {
	inner: Button,
}

impl ButtonBuilder {
	pub fn new() -> Self {
		Self {
			inner: Button::default(),
		}
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.label = text.into();
		self
	}

	pub fn label(mut self, label: impl Into<String>) -> Self {
		self.inner.label = label.into();
		self
	}

	pub fn variant(mut self, variant: ButtonVariant) -> Self {
		self.inner.variant = variant;
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

	pub fn padding(mut self, padding: Vec2<f32>) -> Self {
		self.inner.padding = padding;
		self
	}

	pub fn paddings(mut self, padding: f32) -> Self {
		self.inner.padding = Vec2::new(padding, padding);
		self
	}

	pub fn on_click<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_click = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> Button {
		self.inner
	}
}

impl From<ButtonBuilder> for Button {
	fn from(builder: ButtonBuilder) -> Self {
		builder.build()
	}
}
