/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, ButtonState, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	IntoMethod, LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue,
	Widget,
};
use nimbus_math::{Color, Rect, Vec2, Vec3};
use nimbus_render2d::{Canvas2D, CurveShape, Font, FontId, FontManager, RectShape, TextShape};

use crate::camera::GizmoCamera;


const COLOR_X: Color = Color::new(0.95, 0.28, 0.32, 1.0);
const COLOR_Y: Color = Color::new(0.28, 0.84, 0.44, 1.0);
const COLOR_Z: Color = Color::new(0.26, 0.56, 0.98, 1.0);


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CameraViewPreset {
	Right,  // +X
	Left,   // -X
	Top,    // +Y
	Bottom, // -Y
	Front,  // +Z
	Back,   // -Z
}

impl CameraViewPreset {
	pub fn name(self) -> &'static str {
		match self {
			Self::Right => "Right",
			Self::Left => "Left",
			Self::Top => "Top",
			Self::Bottom => "Bottom",
			Self::Front => "Front",
			Self::Back => "Back",
		}
	}

	pub fn orbit_angles(self) -> (f32, f32) {
		match self {
			Self::Front => (0.0, 0.0),
			Self::Back => (PI, 0.0),
			Self::Right => (FRAC_PI_2, 0.0),
			Self::Left => (-FRAC_PI_2, 0.0),
			Self::Top => (0.0, 1.54),
			Self::Bottom => (0.0, -1.54),
		}
	}
}


#[derive(Copy, Clone, Debug)]
struct AxisEndpoint {
	preset: CameraViewPreset,
	label: &'static str,
	dir: Vec3<f32>,
	color: Color,
	positive: bool,
}

const ENDPOINTS: [AxisEndpoint; 6] = [
	AxisEndpoint {
		preset: CameraViewPreset::Right,
		label: "X",
		dir: Vec3::new(1.0, 0.0, 0.0),
		color: COLOR_X,
		positive: true,
	},
	AxisEndpoint {
		preset: CameraViewPreset::Left,
		label: "-X",
		dir: Vec3::new(-1.0, 0.0, 0.0),
		color: COLOR_X,
		positive: false,
	},
	AxisEndpoint {
		preset: CameraViewPreset::Top,
		label: "Y",
		dir: Vec3::new(0.0, 1.0, 0.0),
		color: COLOR_Y,
		positive: true,
	},
	AxisEndpoint {
		preset: CameraViewPreset::Bottom,
		label: "-Y",
		dir: Vec3::new(0.0, -1.0, 0.0),
		color: COLOR_Y,
		positive: false,
	},
	AxisEndpoint {
		preset: CameraViewPreset::Front,
		label: "Z",
		dir: Vec3::new(0.0, 0.0, 1.0),
		color: COLOR_Z,
		positive: true,
	},
	AxisEndpoint {
		preset: CameraViewPreset::Back,
		label: "-Z",
		dir: Vec3::new(0.0, 0.0, -1.0),
		color: COLOR_Z,
		positive: false,
	},
];


pub struct OrientationGizmo {
	pub size: f32,
	pub margin: f32,
	camera_anim: Animated<GizmoCamera>,
	pub font: Option<FontId>,
	pub on_snap: Option<MethodCallback>,
	pub on_orbit: Option<MethodCallback>,

	hovered_preset: Option<CameraViewPreset>,
	hovered_bg: bool,
	orbit_drag: Option<Vec2<f32>>,
	did_orbit_drag: bool,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for OrientationGizmo {
	fn default() -> Self {
		Self {
			size: 96.0,
			margin: 12.0,
			camera_anim: Animated::new(GizmoCamera::default()),
			font: None,
			on_snap: None,
			on_orbit: None,
			hovered_preset: None,
			hovered_bg: false,
			orbit_drag: None,
			did_orbit_drag: false,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl OrientationGizmo {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> OrientationGizmoBuilder {
		OrientationGizmoBuilder::new()
	}

	pub fn camera(&self) -> GizmoCamera {
		self.camera_anim.value()
	}

	pub fn set_camera(&mut self, camera: GizmoCamera) {
		self.camera_anim.snap_to(camera);
	}

	pub fn animate_camera(&mut self, camera: GizmoCamera) {
		self.camera_anim
			.set_target(camera, Duration::from_millis(280), Easing::EmphasizedDecelerate);
	}

	pub fn animate_camera_with(&mut self, camera: GizmoCamera, duration: Duration, easing: Easing) {
		self.camera_anim.set_target(camera, duration, easing);
	}

	fn sphere_center_and_radius(&self, bounds: Rect<f32>) -> (Vec2<f32>, f32) {
		let inner = (bounds.width().min(bounds.height()) - self.margin * 2.0).max(24.0);
		(bounds.center(), inner * 0.5)
	}

	fn projected_endpoints(
		&self,
		cam: &GizmoCamera,
		bounds: Rect<f32>,
	) -> Vec<(f32, AxisEndpoint, Vec2<f32>)> {
		let (center, sphere_r) = self.sphere_center_and_radius(bounds);
		let arm_len = sphere_r * 0.68;

		let mut list = Vec::with_capacity(6);
		for ep in ENDPOINTS {
			let vx = ep.dir.dot(&cam.right);
			let vy = ep.dir.dot(&cam.up);
			let vz = -ep.dir.dot(&cam.forward);
			let pos = Vec2::new(center.x + vx * arm_len, center.y - vy * arm_len);
			list.push((vz, ep, pos));
		}

		list.sort_by(|a, b| a.0.total_cmp(&b.0));
		list
	}

	fn hit_endpoint(&self, bounds: Rect<f32>, pos: Vec2<f32>) -> Option<CameraViewPreset> {
		let cam = self.camera_anim.value();
		let list = self.projected_endpoints(&cam, bounds);
		for (_, ep, p2d) in list.into_iter().rev() {
			let r = if ep.positive { 11.5 } else { 9.5 };
			let d = pos - p2d;
			if (d.x * d.x + d.y * d.y).sqrt() <= r {
				return Some(ep.preset);
			}
		}
		None
	}
}

impl Widget for OrientationGizmo {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let s = self.size.max(48.0) + self.margin * 2.0;
		constraints.constrain(Vec2::new(s, s))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		let (center, sphere_r) = self.sphere_center_and_radius(bounds);

		let bg_alpha = if self.hovered_bg || self.orbit_drag.is_some() {
			0.38
		} else {
			0.18
		};
		let sphere_rect = Rect::new(
			center.x - sphere_r,
			center.y - sphere_r,
			sphere_r * 2.0,
			sphere_r * 2.0,
		);
		canvas.draw(
			RectShape::new(sphere_rect, Color::new(0.10, 0.11, 0.14, bg_alpha))
				.radius(sphere_r)
				.border(Color::new(1.0, 1.0, 1.0, bg_alpha * 0.45), 1.0),
		);

		let fface = FontManager::default_font();
		let font_size = 11.0;
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, font_size),
			None => Font::new(fface, font_size),
		};

		let cam = self.camera_anim.value();
		if self.camera_anim.is_animating() {
			ctx.request_redraw();
		}

		let list = self.projected_endpoints(&cam, bounds);

		for (depth, ep, pos) in list {
			let is_hovered = self.hovered_preset == Some(ep.preset);
			let depth_fade = (0.55 + 0.45 * ((depth + 1.0) * 0.5)).clamp(0.45, 1.0);

			if ep.positive {
				let mut arm_col = ep.color;
				arm_col.a *= depth_fade;
				canvas.draw(CurveShape::line(center, pos, 2.2, arm_col));

				let r = if is_hovered { 10.5 } else { 9.0 };
				let col = if is_hovered {
					Color::new(
						(ep.color.r + 0.18).min(1.0),
						(ep.color.g + 0.18).min(1.0),
						(ep.color.b + 0.18).min(1.0),
						1.0,
					)
				} else {
					arm_col
				};
				canvas.draw(
					RectShape::new(Rect::new(pos.x - r, pos.y - r, r * 2.0, r * 2.0), col).radius(r),
				);

				let (tw, th) = canvas.measure_text(ep.label, font_size);
				canvas.draw(TextShape::new(
					ep.label,
					font,
					font_size,
					Vec2::new(pos.x - tw * 0.5, pos.y - th * 0.5),
					Color::new(0.06, 0.06, 0.08, 1.0),
				));
			} else {
				let r = if is_hovered { 9.0 } else { 7.0 };
				let fill = if is_hovered {
					ep.color
				} else {
					Color::new(ep.color.r, ep.color.g, ep.color.b, 0.25 * depth_fade)
				};
				let border = Color::new(ep.color.r, ep.color.g, ep.color.b, 0.75 * depth_fade);
				canvas.draw(
					RectShape::new(Rect::new(pos.x - r, pos.y - r, r * 2.0, r * 2.0), fill)
						.radius(r)
						.border(border, 1.5),
				);

				if is_hovered {
					let (tw, th) = canvas.measure_text(ep.label, 9.5);
					let small_font = match self.font {
						Some(id) => Font::with_id(id, fface, 9.5),
						None => Font::new(fface, 9.5),
					};
					canvas.draw(TextShape::new(
						ep.label,
						small_font,
						9.5,
						Vec2::new(pos.x - tw * 0.5, pos.y - th * 0.5),
						Color::new(0.06, 0.06, 0.08, 1.0),
					));
				}
			}
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let (center, sphere_r) = self.sphere_center_and_radius(bounds);

		match event {
			InputEvent::PointerMoved { position } => {
				if let Some(last) = self.orbit_drag {
					let delta = *position - last;
					if delta.x.abs() > 0.5 || delta.y.abs() > 0.5 {
						self.did_orbit_drag = true;
						self.orbit_drag = Some(*position);
						if let Some(cb) = &self.on_orbit {
							cb.call2(-delta.x * 0.012, delta.y * 0.012);
						}
						ctx.request_redraw();
					}
					return EventResponse::Handled;
				}

				let d = *position - center;
				let in_sphere = (d.x * d.x + d.y * d.y).sqrt() <= sphere_r;
				let hit = self.hit_endpoint(bounds, *position);
				if in_sphere != self.hovered_bg || hit != self.hovered_preset {
					self.hovered_bg = in_sphere;
					self.hovered_preset = hit;
					ctx.request_redraw();
				}
				if in_sphere || hit.is_some() {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered_bg || self.hovered_preset.is_some() {
					self.hovered_bg = false;
					self.hovered_preset = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				let d = *position - center;
				let in_sphere = (d.x * d.x + d.y * d.y).sqrt() <= sphere_r;
				if in_sphere || self.hit_endpoint(bounds, *position).is_some() {
					self.orbit_drag = Some(*position);
					self.did_orbit_drag = false;
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				position,
			} => {
				if self.orbit_drag.take().is_some() {
					if !self.did_orbit_drag
						&& let Some(preset) = self.hit_endpoint(bounds, *position)
						&& let Some(cb) = &self.on_snap
					{
						cb.call1(preset.name());
					}
					self.did_orbit_drag = false;
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"size" => self.size = value.expect_f32()?.max(48.0),
			"margin" => self.margin = value.expect_f32()?.max(0.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_snap" | "on_orbit")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_snap" => {
				self.on_snap = Some(callback);
				true
			}
			"on_orbit" => {
				self.on_orbit = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct OrientationGizmoBuilder {
	inner: OrientationGizmo,
}

impl OrientationGizmoBuilder {
	pub fn new() -> Self {
		Self {
			inner: OrientationGizmo::default(),
		}
	}

	pub fn size(mut self, size: f32) -> Self {
		self.inner.size = size.max(48.0);
		self
	}

	pub fn margin(mut self, margin: f32) -> Self {
		self.inner.margin = margin.max(0.0);
		self
	}

	pub fn camera(mut self, camera: GizmoCamera) -> Self {
		self.inner.set_camera(camera);
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn on_snap<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_snap = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_orbit<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_orbit = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> OrientationGizmo {
		self.inner
	}
}

impl From<OrientationGizmoBuilder> for OrientationGizmo {
	fn from(builder: OrientationGizmoBuilder) -> Self {
		builder.build()
	}
}
