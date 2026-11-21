/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::{Cell, RefCell};
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, DrawCtx, Easing, LayoutCtx, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct ProgressBar {
	pub value: f32,
	pub min: f32,
	pub max: Option<f32>,
	pub height: f32,
	pub corner_radius: Option<f32>,
	pub color: Option<Color>,
	pub track_color: Option<Color>,
	pub text: Option<String>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub show_percentage: bool,

	anim_progress: RefCell<Animated<f32>>,
	has_drawn: Cell<bool>,
}

impl Default for ProgressBar {
	fn default() -> Self {
		Self {
			value: 0.0,
			min: 0.0,
			max: None,
			height: 20.0,
			corner_radius: None,
			color: None,
			track_color: None,
			text: None,
			font: None,
			font_size: 13.0,
			show_percentage: false,
			anim_progress: RefCell::new(Animated::new(0.0)),
			has_drawn: Cell::new(false),
		}
	}
}

impl ProgressBar {
	pub fn ratio(&self) -> f32 {
		let max = match self.max {
			Some(m) => m,
			None => {
				if self.value > 1.0 {
					100.0
				} else {
					1.0
				}
			}
		};
		let range = (max - self.min).max(0.0001);
		((self.value - self.min) / range).clamp(0.0, 1.0)
	}

	fn update_animation(&mut self) {
		let target = self.ratio();
		if self.has_drawn.get() {
			self.anim_progress.borrow_mut().set_target(
				target,
				Duration::from_millis(200),
				Easing::EmphasizedDecelerate,
			);
		} else {
			self.anim_progress.borrow_mut().snap_to(target);
		}
	}
}

impl Widget for ProgressBar {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let width = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			200.0
		};
		constraints.constrain(Vec2::new(width, self.height))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let radius = self.corner_radius.unwrap_or(bounds.height() * 0.5);

		let target_ratio = self.ratio();

		// Snap to target ratio on first draw so there is no unwanted start animation
		if !self.has_drawn.get() {
			self.has_drawn.set(true);
			self.anim_progress.borrow_mut().snap_to(target_ratio);
		}

		if self.anim_progress.borrow().is_animating() {
			ctx.request_redraw();
		}
		let cur_ratio = self.anim_progress.borrow().value().clamp(0.0, 1.0);

		// Track
		let track_color = self
			.track_color
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainerHighest]);
		canvas.draw(
			RectShape::new(bounds, track_color).radius(radius),
		);

		// Fill
		let fill_width = (bounds.width() * cur_ratio).clamp(0.0, bounds.width());
		let fill_color = self
			.color
			.unwrap_or_else(|| monet[MonetRole::Primary]);

		if fill_width > 0.5 {
			let is_full = fill_width >= bounds.width() - 0.5;
			let fill_rect = Rect::new(bounds.x(), bounds.y(), fill_width, bounds.height());
			let inner_rad = 4.0f32.min(radius);

			let shape = if is_full {
				RectShape::new(fill_rect, fill_color)
					.radius(radius)
					.noise(0.15)
			} else {
				let left_rad = radius.min(fill_width * 0.5);
				let right_rad = inner_rad.min(fill_width * 0.5);
				RectShape::new(fill_rect, fill_color)
					.radius4([left_rad, right_rad, right_rad, left_rad])
					.noise(0.15)
			};
			canvas.draw(shape);
		}

		// Centered Text / Percentage label
		let text_content = if let Some(t) = &self.text {
			Some(t.clone())
		} else if self.show_percentage {
			Some(format!("{}%", (cur_ratio * 100.0).round() as i32))
		} else {
			None
		};

		if let Some(t) = text_content {
			let fface = FontManager::default_font();
			let font = match self.font {
				Some(id) => Font::with_id(id, fface, self.font_size),
				None => Font::new(fface, self.font_size),
			};
			let (tw, th) = canvas.measure_text(&t, self.font_size);
			let tx = bounds.center().x - tw * 0.5;
			let ty = bounds.center().y - th * 0.5;

			let text_color = monet[MonetRole::OnSurface];
			canvas.draw(TextShape::new(&t, font, self.font_size, Vec2::new(tx, ty), text_color));
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		ctx: &PropertyCtx,
	) -> bool {
		match key {
			"value" => {
				if let Some(p) = value.as_f32() {
					self.value = p;
					self.update_animation();
					return true;
				}
			}

			"progress" => {
				if let Some(p) = value.as_f32() {
					self.value = p;
					self.max = Some(1.0);
					self.min = 0.0;
					self.update_animation();
					return true;
				}
			}

			"min" => {
				if let Some(m) = value.as_f32() {
					self.min = m;
					self.update_animation();
					return true;
				}
			}

			"max" => {
				if let Some(m) = value.as_f32() {
					self.max = Some(m);
					self.update_animation();
					return true;
				}
			}

			"height" => {
				if let Some(h) = value.as_f32() {
					self.height = h;
					return true;
				}
			}

			"corner_radius" | "radius" => {
				if let Some(r) = value.as_f32() {
					self.corner_radius = Some(r);
					return true;
				}
			}

			"color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.color = Some(c);
					return true;
				}
			}

			"track_color" => {
				if let Some(c) = value.resolve_color(&ctx.theme.monet) {
					self.track_color = Some(c);
					return true;
				}
			}

			"text" => {
				if let Some(s) = value.as_str() {
					self.text = Some(s.to_string());
					return true;
				}
			}

			"show_percentage" => {
				if let Some(b) = value.as_bool() {
					self.show_percentage = b;
					return true;
				}
			}

			"font_size" => {
				if let Some(n) = value.as_f32() {
					self.font_size = n;
					return true;
				}
			}

			_ => {}
		}
		false
	}
}
