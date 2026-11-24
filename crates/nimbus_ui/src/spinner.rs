/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::f32::consts::{FRAC_PI_2, TAU};
use std::time::Instant;

use nimbus_core::{
	BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, FontId, RectShape};
use nimbus_theme::{DColor, MonetRole};


pub struct Spinner {
	pub size: f32,
	pub thickness: Option<f32>,
	pub speed: f32,
	pub color: Option<DColor>,
	pub track_color: Option<DColor>,
	pub active: bool,
	pub font: Option<FontId>,
	start_time: Instant,
}

impl Default for Spinner {
	fn default() -> Self {
		Self {
			size: 36.0,
			thickness: None,
			speed: 1.0,
			color: None,
			track_color: None,
			active: true,
			font: None,
			start_time: Instant::now(),
		}
	}
}

impl Spinner {
	pub fn new(size: f32) -> Self {
		Self {
			size: size.max(8.0),
			..Default::default()
		}
	}

	pub fn builder() -> SpinnerBuilder {
		SpinnerBuilder::new()
	}

	pub fn effective_thickness(&self) -> f32 {
		self
			.thickness
			.unwrap_or_else(|| (self.size * 0.224).clamp(4.0, self.size * 0.34))
	}
}

impl Widget for Spinner {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let s = self.size.max(8.0);
		constraints.constrain(Vec2::new(s, s))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		if self.active {
			ctx.request_redraw();
		}

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let diameter = bounds.width().min(bounds.height());
		if diameter <= 2.0 {
			return;
		}

		let item_size = self.effective_thickness().min(diameter * 0.42);
		let center = bounds.center();

		if let Some(track_col) = self.track_color {
			let ring_rect = Rect::new(
				center.x - diameter * 0.5,
				center.y - diameter * 0.5,
				diameter,
				diameter,
			);
			canvas.draw(
				RectShape::new(ring_rect, Color::TRANSPARENT)
					.radius(diameter * 0.5)
					.border(track_col.get(monet), (item_size * 0.25).max(1.5)),
			);
		}

		let base_color = self
			.color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::Primary]);

		let elapsed = if self.active {
			self.start_time.elapsed().as_secs_f32() * self.speed
		} else {
			0.0
		};

		let base_orbit = ((diameter - item_size) * 0.5 * 0.88).max(0.0);
		let phase = elapsed * TAU;

		let raw_pulse = phase.cos() * 0.5 + 0.5;
		let smooth_pulse = raw_pulse * raw_pulse * (3.0 - 2.0 * raw_pulse);
		let orbit_radius = base_orbit * (0.36 + 0.52 * smooth_pulse);
		let s = item_size * (0.86 + 0.14 * smooth_pulse);

		let mut col = base_color;
		col.a *= 0.74 + 0.26 * smooth_pulse;

		for i in 0..4usize {
			let angle = phase + (i as f32) * FRAC_PI_2;
			let px = center.x + angle.cos() * orbit_radius;
			let py = center.y + angle.sin() * orbit_radius;

			let dot_rect = Rect::new(px - s * 0.5, py - s * 0.5, s, s);
			canvas.draw(RectShape::new(dot_rect, col).radius(s * 0.5));
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"size" => self.size = value.expect_f32()?.max(8.0),
			"thickness" => self.thickness = Some(value.expect_f32()?.max(1.0)),
			"speed" => self.speed = value.expect_f32()?.max(0.05),
			"color" => self.color = Some(value.expect_dcolor()?),
			"track_color" => self.track_color = Some(value.expect_dcolor()?),
			"active" => self.active = value.expect_bool()?,
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}
}


#[derive(Default)]
pub struct SpinnerBuilder {
	inner: Spinner,
}

impl SpinnerBuilder {
	pub fn new() -> Self {
		Self {
			inner: Spinner::default(),
		}
	}

	pub fn size(mut self, size: f32) -> Self {
		self.inner.size = size.max(8.0);
		self
	}

	pub fn thickness(mut self, thickness: f32) -> Self {
		self.inner.thickness = Some(thickness.max(1.0));
		self
	}

	pub fn speed(mut self, speed: f32) -> Self {
		self.inner.speed = speed.max(0.05);
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn track_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.track_color = Some(color.into());
		self
	}

	pub fn active(mut self, active: bool) -> Self {
		self.inner.active = active;
		self
	}

	pub fn build(self) -> Spinner {
		self.inner
	}
}

impl From<SpinnerBuilder> for Spinner {
	fn from(builder: SpinnerBuilder) -> Self {
		builder.build()
	}
}
