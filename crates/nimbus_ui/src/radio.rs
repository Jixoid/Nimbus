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
	Interpolate, IntoMethod, LayoutCtx, MessageTarget, MethodCallback, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Radio {
	pub label: String,
	pub checked: bool,
	pub group: Option<String>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub color: Option<DColor>,
	pub on_changed: Option<MethodCallback>,
	pub progress: Animated<f32>,
}

impl Default for Radio {
	fn default() -> Self {
		Self {
			label: any::type_name::<Self>().to_string(),
			checked: false,
			group: None,
			font: None,
			font_size: 15.0,
			color: None,
			on_changed: None,
			progress: Animated::new(0.0),
		}
	}
}

impl Widget for Radio {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		if self.label.is_empty() {
			constraints.constrain(Vec2::new(22.0, 22.0))
		} else {
			let (text_w, text_h) = ctx.measure_text_with_font(&self.label, self.font, self.font_size);
			let total_w = text_w + 30.0 + 8.0;
			let total_h = (text_h + 8.0).max(22.0);
			constraints.constrain(Vec2::new(total_w.ceil(), total_h.ceil()))
		}
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let circle_size = 22.0f32;
		let circle_x = ctx.bounds.x();
		let circle_y = ctx.bounds.y() + (ctx.bounds.height() - circle_size) * 0.5;
		let outer_rect = Rect::new(circle_x, circle_y, circle_size, circle_size);

		let t = self.progress.value();
		if self.progress.is_animating() {
			ctx.request_redraw();
		}

		let t_clamped = t.clamp(0.0, 1.0);
		let inactive_border = if ctx.hovered {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::PrimaryOutline]
		};
		let border_color = inactive_border.interpolate(&monet[MonetRole::Primary], t_clamped);
		
		// Outer circle
		canvas.draw(
			RectShape::new(outer_rect, Color::TRANSPARENT)
				.radius(circle_size * 0.5)
				.border(border_color, ctx.theme.stroke.thick),
		);

		// Inner circle animated scale
		if t > 0.01 {
			let inner_size = 10.0f32 * t;
			let inner_rect = Rect::new(
				circle_x + (circle_size - inner_size) * 0.5,
				circle_y + (circle_size - inner_size) * 0.5,
				inner_size,
				inner_size,
			);
			canvas.draw(
				RectShape::new(inner_rect, monet[MonetRole::Primary])
					.radius(inner_size * 0.5)
					.noise(0.15),
			);
		}

		// Text label
		if !self.label.is_empty() {
			let (_, text_h) = canvas.measure_text(&self.label, self.font_size);
			let text_x = ctx.bounds.x() + 27.0;
			let text_y = ctx.bounds.y() + (ctx.bounds.height() - text_h) * 0.5;

			let fface = FontManager::default_font();
			let font = match self.font {
				Some(id) => Font::with_id(id, fface, self.font_size),
				None => Font::new(fface, self.font_size),
			};

			let text_color = self
				.color
				.map(|c| c.get(monet))
				.unwrap_or_else(|| monet[MonetRole::OnSurface]);
			canvas.draw(TextShape::new(
				&self.label,
				font,
				self.font_size,
				Vec2::new(text_x, text_y),
				text_color,
			));
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		if let InputEvent::Click { .. } = event {
			if !self.checked {
				self.checked = true;
				self.progress.set_target(
					1.0,
					Duration::from_millis(160),
					Easing::BackOut,
				);
				ctx.dispatch_message(
					MessageTarget::Siblings,
					"radio_selected",
					PropertyValue::String(self.group.clone().unwrap_or_default()),
				);
				if let Some(cb) = &mut self.on_changed {
					cb.call1(self.checked);
				}
				ctx.request_redraw();
				return EventResponse::Handled;
			}
		}
		EventResponse::Ignored
	}

	fn handle_message(&mut self, ctx: &mut EventCtx, name: &str, payload: &PropertyValue) -> bool {
		if name == "radio_selected" {
			let incoming_group = payload.as_str().filter(|s| !s.is_empty());
			let matches = match (incoming_group, &self.group) {
				(Some(g1), Some(g2)) => g1 == g2.as_str(),
				(None, None) => true,
				_ => false,
			};

			if matches && self.checked {
				self.checked = false;
				self.progress.set_target(
					0.0,
					Duration::from_millis(120),
					Easing::EaseIn,
				);
				ctx.request_redraw();
				if let Some(cb) = &mut self.on_changed {
					cb.call1(self.checked);
				}
				return true;
			}
		}

		false
	}

	fn set_property(&mut self, key: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match key {
			"checked" => {
				let b = value.expect_bool()?;
				self.checked = b;
				self.progress.snap_to(if b { 1.0 } else { 0.0 });
			}
			"text" => self.label = value.expect_str()?.to_string(),
			"group" => self.group = Some(value.expect_str()?.to_string()),
			"font_size" => self.font_size = value.expect_f32()?,
			"color" => self.color = Some(value.expect_dcolor()?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
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

impl Radio {
	pub fn builder() -> RadioBuilder {
		RadioBuilder::new()
	}
}


#[derive(Default)]
pub struct RadioBuilder {
	inner: Radio,
}

impl RadioBuilder {
	pub fn new() -> Self {
		Self {
			inner: Radio::default(),
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

	pub fn checked(mut self, checked: bool) -> Self {
		self.inner.checked = checked;
		self.inner.progress.snap_to(if checked { 1.0 } else { 0.0 });
		self
	}

	pub fn group(mut self, group: impl Into<String>) -> Self {
		self.inner.group = Some(group.into());
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

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> Radio {
		self.inner
	}
}

impl From<RadioBuilder> for Radio {
	fn from(builder: RadioBuilder) -> Self {
		builder.build()
	}
}
