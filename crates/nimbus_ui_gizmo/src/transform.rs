/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::f32::consts::TAU;
use std::time::Duration;

use nimbus_core::{
	Animated, BoxConstraints, ButtonState, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	IntoMethod, LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue,
	Widget,
};
use nimbus_math::{Color, Plane, Quat, Rad, Rect, Transform, Vec2, Vec3};
use nimbus_render2d::{Canvas2D, CurveShape, Font, FontId, FontManager, RectShape, TextShape};

use crate::camera::{
	GizmoCamera, closest_axis_param, dist_point_to_segment_2d, point_in_quad_2d,
};


const COLOR_X: Color = Color::new(0.95, 0.26, 0.30, 1.0);
const COLOR_Y: Color = Color::new(0.26, 0.85, 0.42, 1.0);
const COLOR_Z: Color = Color::new(0.24, 0.56, 0.98, 1.0);
const COLOR_ACTIVE: Color = Color::new(1.0, 0.90, 0.25, 1.0);
const COLOR_UNIFORM: Color = Color::new(0.92, 0.92, 0.95, 0.95);


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GizmoMode {
	#[default]
	Translate,
	Rotate,
	Scale,
}

impl GizmoMode {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim().to_ascii_lowercase().as_str() {
			"translate" => Some(Self::Translate),
			"rotate" => Some(Self::Rotate),
			"scale" => Some(Self::Scale),
			_ => None,
		}
	}
}


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GizmoSpace {
	#[default]
	World,
	Local,
}

impl GizmoSpace {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim().to_ascii_lowercase().as_str() {
			"world" => Some(Self::World),
			"local" => Some(Self::Local),
			_ => None,
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoSnap {
	pub enabled: bool,
	pub translate: f32,
	pub rotate_deg: f32,
	pub scale: f32,
}

impl Default for GizmoSnap {
	fn default() -> Self {
		Self {
			enabled: false,
			translate: 0.5,
			rotate_deg: 15.0,
			scale: 0.1,
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GizmoHandle {
	AxisX,
	AxisY,
	AxisZ,
	PlaneXY,
	PlaneXZ,
	PlaneYZ,
	Uniform,
}

#[derive(Copy, Clone, Debug)]
struct DragState {
	handle: GizmoHandle,
	start_transform: Transform<f32>,
	start_axis_param: f32,
	start_plane_hit: Vec3<f32>,
	start_pointer: Vec2<f32>,
	start_angle: f32,
	current_angle_delta: f32,
	axis_dir: Vec3<f32>,
	plane: Plane<f32>,
}


pub struct TransformGizmo {
	pub mode: GizmoMode,
	pub space: GizmoSpace,
	pub snap: GizmoSnap,
	pub transform: Transform<f32>,
	camera_anim: Animated<GizmoCamera>,
	pub size_px: f32,
	pub font: Option<FontId>,
	pub on_changed: Option<MethodCallback>,
	pub on_commit: Option<MethodCallback>,

	hovered: Option<GizmoHandle>,
	drag: Option<DragState>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for TransformGizmo {
	fn default() -> Self {
		Self {
			mode: GizmoMode::Translate,
			space: GizmoSpace::World,
			snap: GizmoSnap::default(),
			transform: Transform::IDENTITY,
			camera_anim: Animated::new(GizmoCamera::default()),
			size_px: 88.0,
			font: None,
			on_changed: None,
			on_commit: None,
			hovered: None,
			drag: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl TransformGizmo {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> TransformGizmoBuilder {
		TransformGizmoBuilder::new()
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

	pub fn is_dragging(&self) -> bool {
		self.drag.is_some()
	}

	pub fn hovered_handle(&self) -> Option<GizmoHandle> {
		self.hovered
	}

	fn axes(&self) -> [Vec3<f32>; 3] {
		let base = [Vec3::RIGHT, Vec3::UP, Vec3::BACK];
		match self.space {
			GizmoSpace::World => base,
			GizmoSpace::Local => [
				self.transform.rotation.rotate(base[0]).normalize(),
				self.transform.rotation.rotate(base[1]).normalize(),
				self.transform.rotation.rotate(base[2]).normalize(),
			],
		}
	}

	fn snap_val(val: f32, step: f32, enabled: bool) -> f32 {
		if enabled && step > 1e-5 {
			(val / step).round() * step
		} else {
			val
		}
	}

	fn handle_color(&self, handle: GizmoHandle, base: Color) -> Color {
		if self.drag.is_some_and(|d| d.handle == handle) {
			COLOR_ACTIVE
		} else if self.hovered == Some(handle) {
			Color::new(
				(base.r + 0.25).min(1.0),
				(base.g + 0.25).min(1.0),
				(base.b + 0.25).min(1.0),
				1.0,
			)
		} else {
			base
		}
	}

	fn plane_quad_2d(
		&self,
		bounds: Rect<f32>,
		origin: Vec3<f32>,
		axis_a: Vec3<f32>,
		axis_b: Vec3<f32>,
		world_len: f32,
	) -> Option<[Vec2<f32>; 4]> {
		let d0 = world_len * 0.24;
		let d1 = world_len * 0.50;
		let p00 = origin + axis_a * d0 + axis_b * d0;
		let p10 = origin + axis_a * d1 + axis_b * d0;
		let p11 = origin + axis_a * d1 + axis_b * d1;
		let p01 = origin + axis_a * d0 + axis_b * d1;
		Some([
			self.camera().project(p00, bounds)?.0,
			self.camera().project(p10, bounds)?.0,
			self.camera().project(p11, bounds)?.0,
			self.camera().project(p01, bounds)?.0,
		])
	}

	fn hit_test_handle(&self, bounds: Rect<f32>, pos: Vec2<f32>) -> Option<GizmoHandle> {
		let origin = self.transform.translation;
		let (origin_2d, _) = self.camera().project(origin, bounds)?;
		let world_len = self.camera().world_scale_at(origin, bounds, self.size_px);
		let axes = self.axes();

		match self.mode {
			GizmoMode::Translate => {
				for (handle, a, b) in [
					(GizmoHandle::PlaneXY, axes[0], axes[1]),
					(GizmoHandle::PlaneXZ, axes[0], axes[2]),
					(GizmoHandle::PlaneYZ, axes[1], axes[2]),
				] {
					if let Some(q) = self.plane_quad_2d(bounds, origin, a, b, world_len)
						&& point_in_quad_2d(pos, q)
					{
						return Some(handle);
					}
				}

				let mut best: Option<(GizmoHandle, f32)> = None;
				for (handle, axis) in [
					(GizmoHandle::AxisX, axes[0]),
					(GizmoHandle::AxisY, axes[1]),
					(GizmoHandle::AxisZ, axes[2]),
				] {
					let tip = origin + axis * world_len;
					if let Some((tip_2d, _)) = self.camera().project(tip, bounds) {
						let dist = dist_point_to_segment_2d(pos, origin_2d, tip_2d);
						if dist <= 10.0 && best.is_none_or(|b| dist < b.1) {
							best = Some((handle, dist));
						}
					}
				}
				best.map(|(h, _)| h)
			}

			GizmoMode::Rotate => {
				let radius = world_len * 0.92;
				let mut best: Option<(GizmoHandle, f32)> = None;
				let segments = 48usize;

				for (handle, u_axis, v_axis) in [
					(GizmoHandle::AxisX, axes[1], axes[2]),
					(GizmoHandle::AxisY, axes[2], axes[0]),
					(GizmoHandle::AxisZ, axes[0], axes[1]),
				] {
					let mut prev: Option<Vec2<f32>> = None;
					for s in 0..=segments {
						let (sa, ca) = ((s as f32 / segments as f32) * TAU).sin_cos();
						let wp = origin + u_axis * (ca * radius) + v_axis * (sa * radius);
						if let Some((p2d, _)) = self.camera().project(wp, bounds) {
							if let Some(p_prev) = prev {
								let dist = dist_point_to_segment_2d(pos, p_prev, p2d);
								if dist <= 9.0 && best.is_none_or(|b| dist < b.1) {
									best = Some((handle, dist));
								}
							}
							prev = Some(p2d);
						} else {
							prev = None;
						}
					}
				}
				best.map(|(h, _)| h)
			}

			GizmoMode::Scale => {
				let d_center = pos - origin_2d;
				if (d_center.x * d_center.x + d_center.y * d_center.y).sqrt() <= 13.0 {
					return Some(GizmoHandle::Uniform);
				}

				let mut best: Option<(GizmoHandle, f32)> = None;
				for (handle, axis) in [
					(GizmoHandle::AxisX, axes[0]),
					(GizmoHandle::AxisY, axes[1]),
					(GizmoHandle::AxisZ, axes[2]),
				] {
					let tip = origin + axis * (world_len * 0.92);
					if let Some((tip_2d, _)) = self.camera().project(tip, bounds) {
						let dist = dist_point_to_segment_2d(pos, origin_2d, tip_2d);
						if dist <= 10.0 && best.is_none_or(|b| dist < b.1) {
							best = Some((handle, dist));
						}
					}
				}
				best.map(|(h, _)| h)
			}
		}
	}

	fn draw_translate(&self, canvas: &mut Canvas2D, bounds: Rect<f32>, origin: Vec3<f32>, world_len: f32) {
		let Some((origin_2d, _)) = self.camera().project(origin, bounds) else {
			return;
		};
		let axes = self.axes();

		for (handle, a, b, base_col) in [
			(GizmoHandle::PlaneXY, axes[0], axes[1], COLOR_Z),
			(GizmoHandle::PlaneXZ, axes[0], axes[2], COLOR_Y),
			(GizmoHandle::PlaneYZ, axes[1], axes[2], COLOR_X),
		] {
			if let Some(q) = self.plane_quad_2d(bounds, origin, a, b, world_len) {
				let mut col = self.handle_color(handle, base_col);
				col.a = if self.hovered == Some(handle) || self.drag.is_some_and(|d| d.handle == handle) {
					0.95
				} else {
					0.55
				};
				for i in 0..4 {
					canvas.draw(CurveShape::line(q[i], q[(i + 1) % 4], 2.0, col));
				}
			}
		}

		let mut items = Vec::with_capacity(3);
		for (handle, axis, base_col) in [
			(GizmoHandle::AxisX, axes[0], COLOR_X),
			(GizmoHandle::AxisY, axes[1], COLOR_Y),
			(GizmoHandle::AxisZ, axes[2], COLOR_Z),
		] {
			let tip = origin + axis * world_len;
			if let Some((tip_2d, depth)) = self.camera().project(tip, bounds) {
				items.push((depth, handle, tip_2d, base_col));
			}
		}
		items.sort_by(|a, b| b.0.total_cmp(&a.0));

		for (_, handle, tip_2d, base_col) in items {
			let col = self.handle_color(handle, base_col);
			let is_lit = self.hovered == Some(handle) || self.drag.is_some_and(|d| d.handle == handle);
			let thickness = if is_lit { 3.4 } else { 2.6 };

			canvas.draw(CurveShape::line(origin_2d, tip_2d, thickness, col));

			let dir = tip_2d - origin_2d;
			let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
			if len > 4.0 {
				let ux = dir.x / len;
				let uy = dir.y / len;
				let px = -uy;
				let py = ux;
				let head_len = 11.0;
				let head_w = 5.0;
				let left = Vec2::new(
					tip_2d.x - ux * head_len + px * head_w,
					tip_2d.y - uy * head_len + py * head_w,
				);
				let right = Vec2::new(
					tip_2d.x - ux * head_len - px * head_w,
					tip_2d.y - uy * head_len - py * head_w,
				);
				canvas.draw(CurveShape::line(left, tip_2d, thickness, col));
				canvas.draw(CurveShape::line(right, tip_2d, thickness, col));
				canvas.draw(CurveShape::line(left, right, thickness * 0.85, col));
			}
		}

		canvas.draw(
			RectShape::new(
				Rect::new(origin_2d.x - 4.0, origin_2d.y - 4.0, 8.0, 8.0),
				COLOR_UNIFORM,
			)
			.radius(4.0),
		);
	}

	fn draw_rotate(&self, canvas: &mut Canvas2D, bounds: Rect<f32>, origin: Vec3<f32>, world_len: f32) {
		let Some((origin_2d, _)) = self.camera().project(origin, bounds) else {
			return;
		};
		let axes = self.axes();
		let radius = world_len * 0.92;
		let segments = 56usize;
		let to_cam = (self.camera().position - origin).normalize();

		for (handle, u_axis, v_axis, base_col) in [
			(GizmoHandle::AxisX, axes[1], axes[2], COLOR_X),
			(GizmoHandle::AxisY, axes[2], axes[0], COLOR_Y),
			(GizmoHandle::AxisZ, axes[0], axes[1], COLOR_Z),
		] {
			let base = self.handle_color(handle, base_col);
			let is_lit = self.hovered == Some(handle) || self.drag.is_some_and(|d| d.handle == handle);
			let thickness = if is_lit { 3.2 } else { 2.3 };

			let mut prev: Option<(Vec2<f32>, bool)> = None;
			for s in 0..=segments {
				let (sa, ca) = ((s as f32 / segments as f32) * TAU).sin_cos();
				let offset = u_axis * (ca * radius) + v_axis * (sa * radius);
				let wp = origin + offset;
				let front = offset.dot(&to_cam) >= -0.05 * radius;
				if let Some((p2d, _)) = self.camera().project(wp, bounds) {
					if let Some((p_prev, prev_front)) = prev {
						let mut seg_col = base;
						if !front && !prev_front && !is_lit {
							seg_col.a *= 0.28;
						}
						canvas.draw(CurveShape::line(p_prev, p2d, thickness, seg_col));
					}
					prev = Some((p2d, front));
				} else {
					prev = None;
				}
			}
		}

		if let Some(drag) = self.drag {
			let deg = drag.current_angle_delta.to_degrees();
			let label = format!("{:+.1}°", deg);
			let fface = FontManager::default_font();
			let font = match self.font {
				Some(id) => Font::with_id(id, fface, 13.0),
				None => Font::new(fface, 13.0),
			};
			let (tw, th) = canvas.measure_text(&label, 13.0);
			let pill = Rect::new(
				origin_2d.x - tw * 0.5 - 8.0,
				origin_2d.y - th * 0.5 - 4.0,
				tw + 16.0,
				th + 8.0,
			);
			canvas.draw(
				RectShape::new(pill, Color::new(0.10, 0.10, 0.14, 0.88))
					.radius(6.0)
					.border(COLOR_ACTIVE, 1.0),
			);
			canvas.draw(TextShape::new(
				&label,
				font,
				13.0,
				Vec2::new(pill.x() + 8.0, pill.y() + 4.0),
				COLOR_ACTIVE,
			));
		}
	}

	fn draw_scale(&self, canvas: &mut Canvas2D, bounds: Rect<f32>, origin: Vec3<f32>, world_len: f32) {
		let Some((origin_2d, _)) = self.camera().project(origin, bounds) else {
			return;
		};
		let axes = self.axes();

		let mut items = Vec::with_capacity(3);
		for (handle, axis, base_col) in [
			(GizmoHandle::AxisX, axes[0], COLOR_X),
			(GizmoHandle::AxisY, axes[1], COLOR_Y),
			(GizmoHandle::AxisZ, axes[2], COLOR_Z),
		] {
			let tip = origin + axis * (world_len * 0.92);
			if let Some((tip_2d, depth)) = self.camera().project(tip, bounds) {
				items.push((depth, handle, tip_2d, base_col));
			}
		}
		items.sort_by(|a, b| b.0.total_cmp(&a.0));

		for (_, handle, tip_2d, base_col) in items {
			let col = self.handle_color(handle, base_col);
			let is_lit = self.hovered == Some(handle) || self.drag.is_some_and(|d| d.handle == handle);
			let thickness = if is_lit { 3.4 } else { 2.6 };
			let box_s = if is_lit { 11.0 } else { 9.0 };

			canvas.draw(CurveShape::line(origin_2d, tip_2d, thickness, col));
			canvas.draw(
				RectShape::new(
					Rect::new(
						tip_2d.x - box_s * 0.5,
						tip_2d.y - box_s * 0.5,
						box_s,
						box_s,
					),
					col,
				)
				.radius(2.0),
			);
		}

		let uni_col = self.handle_color(GizmoHandle::Uniform, COLOR_UNIFORM);
		let r = 10.0;
		canvas.draw(
			RectShape::new(
				Rect::new(origin_2d.x - r, origin_2d.y - r, r * 2.0, r * 2.0),
				Color::new(0.15, 0.15, 0.20, 0.45),
			)
			.radius(r)
			.border(uni_col, 2.2),
		);
	}

	fn emit_changed(&self) {
		if let Some(cb) = &self.on_changed {
			let p = self.transform.translation;
			cb.call2(p.x, p.y);
		}
	}

	fn emit_commit(&self) {
		if let Some(cb) = &self.on_commit {
			let p = self.transform.translation;
			cb.call2(p.x, p.y);
		}
	}
}

impl Widget for TransformGizmo {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				400.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				300.0
			},
		))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 2.0 || bounds.height() <= 2.0 {
			return;
		}
		if self.camera_anim.is_animating() {
			ctx.request_redraw();
		}

		let origin = self.transform.translation;
		let world_len = self.camera().world_scale_at(origin, bounds, self.size_px);

		canvas.push_clip(bounds);
		match self.mode {
			GizmoMode::Translate => self.draw_translate(canvas, bounds, origin, world_len),
			GizmoMode::Rotate => self.draw_rotate(canvas, bounds, origin, world_len),
			GizmoMode::Scale => self.draw_scale(canvas, bounds, origin, world_len),
		}
		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		match event {
			InputEvent::PointerMoved { position } => {
				if let Some(mut drag) = self.drag {
					let ray = self.camera().screen_ray(*position, bounds);
					match self.mode {
						GizmoMode::Translate => match drag.handle {
							GizmoHandle::AxisX | GizmoHandle::AxisY | GizmoHandle::AxisZ => {
								if let Some(t) = closest_axis_param(
									&ray,
									drag.start_transform.translation,
									drag.axis_dir,
								) {
									let delta = Self::snap_val(
										t - drag.start_axis_param,
										self.snap.translate,
										self.snap.enabled,
									);
									self.transform.translation =
										drag.start_transform.translation + drag.axis_dir * delta;
									self.emit_changed();
								}
							}
							GizmoHandle::PlaneXY | GizmoHandle::PlaneXZ | GizmoHandle::PlaneYZ => {
								if let Some(t) = ray.intersects_plane(&drag.plane) {
									let hit = ray.point_at(t);
									let mut diff = hit - drag.start_plane_hit;
									if self.snap.enabled && self.snap.translate > 1e-5 {
										let s = self.snap.translate;
										diff = Vec3::new(
											(diff.x / s).round() * s,
											(diff.y / s).round() * s,
											(diff.z / s).round() * s,
										);
									}
									self.transform.translation = drag.start_transform.translation + diff;
									self.emit_changed();
								}
							}
							GizmoHandle::Uniform => {}
						},

						GizmoMode::Rotate => {
							if let Some((origin_2d, _)) =
								self.camera().project(drag.start_transform.translation, bounds)
							{
								let cur_ang =
									(position.y - origin_2d.y).atan2(position.x - origin_2d.x);
								let mut delta = cur_ang - drag.start_angle;
								if delta > std::f32::consts::PI {
									delta -= TAU;
								} else if delta < -std::f32::consts::PI {
									delta += TAU;
								}
								let view_sign = if drag.axis_dir.dot(&self.camera().forward) > 0.0 {
									1.0
								} else {
									-1.0
								};
								let raw_rad = delta * view_sign;
								let snapped_rad = if self.snap.enabled && self.snap.rotate_deg > 1e-4 {
									let step_rad = self.snap.rotate_deg.to_radians();
									(raw_rad / step_rad).round() * step_rad
								} else {
									raw_rad
								};
								drag.current_angle_delta = snapped_rad;
								let q_delta =
									Quat::from_axis_angle(drag.axis_dir, Rad::new(snapped_rad));
								self.transform.rotation =
									(q_delta * drag.start_transform.rotation).normalize();
								self.emit_changed();
							}
						}

						GizmoMode::Scale => {
							let dx = (position.x - drag.start_pointer.x) * 0.01;
							let factor = Self::snap_val(1.0 + dx, self.snap.scale, self.snap.enabled)
								.max(0.05);
							let mut s = drag.start_transform.scale;
							match drag.handle {
								GizmoHandle::AxisX => s.x = (s.x * factor).max(0.01),
								GizmoHandle::AxisY => s.y = (s.y * factor).max(0.01),
								GizmoHandle::AxisZ => s.z = (s.z * factor).max(0.01),
								GizmoHandle::Uniform => {
									s = Vec3::new(
										(s.x * factor).max(0.01),
										(s.y * factor).max(0.01),
										(s.z * factor).max(0.01),
									);
								}
								_ => {}
							}
							self.transform.scale = s;
							self.emit_changed();
						}
					}
					self.drag = Some(drag);
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				let hit = self.hit_test_handle(bounds, *position);
				if hit != self.hovered {
					self.hovered = hit;
					ctx.request_redraw();
				}
				if hit.is_some() {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered.is_some() && self.drag.is_none() {
					self.hovered = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				let Some(handle) = self.hit_test_handle(bounds, *position) else {
					return EventResponse::Ignored;
				};
				let axes = self.axes();
				let axis_dir = match handle {
					GizmoHandle::AxisX => axes[0],
					GizmoHandle::AxisY => axes[1],
					GizmoHandle::AxisZ => axes[2],
					_ => self.camera().forward,
				};
				let plane_normal = match handle {
					GizmoHandle::PlaneXY => axes[2],
					GizmoHandle::PlaneXZ => axes[1],
					GizmoHandle::PlaneYZ => axes[0],
					_ => axis_dir.cross(&self.camera().forward).normalize(),
				};
				let plane = Plane::from_point_normal(self.transform.translation, plane_normal);
				let ray = self.camera().screen_ray(*position, bounds);
				let start_axis_param =
					closest_axis_param(&ray, self.transform.translation, axis_dir).unwrap_or(0.0);
				let start_plane_hit = ray
					.intersects_plane(&plane)
					.map(|t| ray.point_at(t))
					.unwrap_or(self.transform.translation);
				let start_angle = self
					.camera()
					.project(self.transform.translation, bounds)
					.map(|(o2, _)| (position.y - o2.y).atan2(position.x - o2.x))
					.unwrap_or(0.0);

				self.drag = Some(DragState {
					handle,
					start_transform: self.transform,
					start_axis_param,
					start_plane_hit,
					start_pointer: *position,
					start_angle,
					current_angle_delta: 0.0,
					axis_dir,
					plane,
				});
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if self.drag.take().is_some() {
					self.emit_commit();
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
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
			"mode" => {
				let s = value.expect_str()?;
				self.mode = GizmoMode::parse(s).ok_or(PropertyError::InvalidValue {
					expected: "'translate', 'rotate', or 'scale'",
				})?;
			}
			"space" => {
				let s = value.expect_str()?;
				self.space = GizmoSpace::parse(s).ok_or(PropertyError::InvalidValue {
					expected: "'world' or 'local'",
				})?;
			}
			"snap" => self.snap.enabled = value.expect_bool()?,
			"snap_translate" => self.snap.translate = value.expect_f32()?.max(0.001),
			"snap_rotate" => self.snap.rotate_deg = value.expect_f32()?.max(0.1),
			"snap_scale" => self.snap.scale = value.expect_f32()?.max(0.001),
			"size" => self.size_px = value.expect_f32()?.max(32.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed" | "on_commit")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_changed" => {
				self.on_changed = Some(callback);
				true
			}
			"on_commit" => {
				self.on_commit = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct TransformGizmoBuilder {
	inner: TransformGizmo,
}

impl TransformGizmoBuilder {
	pub fn new() -> Self {
		Self {
			inner: TransformGizmo::default(),
		}
	}

	pub fn mode(mut self, mode: GizmoMode) -> Self {
		self.inner.mode = mode;
		self
	}

	pub fn space(mut self, space: GizmoSpace) -> Self {
		self.inner.space = space;
		self
	}

	pub fn snap(mut self, snap: GizmoSnap) -> Self {
		self.inner.snap = snap;
		self
	}

	pub fn transform(mut self, transform: Transform<f32>) -> Self {
		self.inner.transform = transform;
		self
	}

	pub fn camera(mut self, camera: GizmoCamera) -> Self {
		self.inner.set_camera(camera);
		self
	}

	pub fn size_px(mut self, px: f32) -> Self {
		self.inner.size_px = px.max(32.0);
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_commit<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_commit = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> TransformGizmo {
		self.inner
	}
}

impl From<TransformGizmoBuilder> for TransformGizmo {
	fn from(builder: TransformGizmoBuilder) -> Self {
		builder.build()
	}
}
