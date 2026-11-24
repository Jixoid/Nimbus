/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::Interpolate;
use nimbus_math::{Mat4, Quat, Rad, Ray, Rect, Vec2, Vec3, Vec4};


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoCamera {
	pub view: Mat4<f32>,
	pub projection: Mat4<f32>,
	pub position: Vec3<f32>,
	pub target: Vec3<f32>,
	pub distance: f32,
	pub orientation: Quat<f32>,
	pub right: Vec3<f32>,
	pub up: Vec3<f32>,
	pub forward: Vec3<f32>,
	pub fov_y: Rad<f32>,
	pub aspect: f32,
}

impl Default for GizmoCamera {
	fn default() -> Self {
		Self::from_orbit(
			Vec3::ZERO,
			5.0,
			0.65,
			0.45,
			Rad::new(std::f32::consts::FRAC_PI_4),
			16.0 / 9.0,
		)
	}
}

impl GizmoCamera {
	pub fn from_orbit(
		target: Vec3<f32>,
		distance: f32,
		yaw: f32,
		pitch: f32,
		fov_y: Rad<f32>,
		aspect: f32,
	) -> Self {
		let dist = distance.max(0.2);
		let clamped_pitch = pitch.clamp(-1.55, 1.55);
		let (sp, cp) = clamped_pitch.sin_cos();
		let (sy, cy) = yaw.sin_cos();

		let offset = Vec3::new(dist * cp * sy, dist * sp, dist * cp * cy);
		Self::look_at(target + offset, target, Vec3::UP, fov_y, aspect)
	}

	pub fn from_orientation(
		target: Vec3<f32>,
		distance: f32,
		orientation: Quat<f32>,
		fov_y: Rad<f32>,
		aspect: f32,
	) -> Self {
		let orientation = orientation.normalize();
		let dist = distance.max(0.2);
		let aspect = aspect.max(0.01);
		let right = orientation.rotate(Vec3::new(1.0, 0.0, 0.0)).normalize();
		let up = orientation.rotate(Vec3::new(0.0, 1.0, 0.0)).normalize();
		let forward = orientation.rotate(Vec3::new(0.0, 0.0, -1.0)).normalize();
		let eye = target - forward * dist;

		let view = Mat4::look_at(eye, target, up);
		let projection = Mat4::perspective_rad(fov_y, aspect, 0.05, 1000.0);

		Self {
			view,
			projection,
			position: eye,
			target,
			distance: dist,
			orientation,
			right,
			up,
			forward,
			fov_y,
			aspect,
		}
	}

	pub fn look_at(
		eye: Vec3<f32>,
		target: Vec3<f32>,
		world_up: Vec3<f32>,
		fov_y: Rad<f32>,
		aspect: f32,
	) -> Self {
		let to_target = target - eye;
		let distance = to_target.length().max(0.2);
		let forward = to_target.normalize();
		let mut right = forward.cross(&world_up);
		if right.length() < 1e-4 {
			right = forward.cross(&Vec3::BACK);
		}
		right = right.normalize();
		let up = right.cross(&forward).normalize();
		let aspect = aspect.max(0.01);
		let orientation = quat_from_basis(right, up, -forward);

		let view = Mat4::look_at(eye, target, up);
		let projection = Mat4::perspective_rad(fov_y, aspect, 0.05, 1000.0);

		Self {
			view,
			projection,
			position: eye,
			target,
			distance,
			orientation,
			right,
			up,
			forward,
			fov_y,
			aspect,
		}
	}

	pub fn project(&self, world_pos: Vec3<f32>, viewport: Rect<f32>) -> Option<(Vec2<f32>, f32)> {
		let v = self.view * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
		if v.z >= -0.01 {
			return None;
		}
		let clip = self.projection * v;
		if clip.w.abs() < 1e-6 {
			return None;
		}
		let ndc_x = clip.x / clip.w;
		let ndc_y = clip.y / clip.w;
		
		let sx = viewport.x() + (ndc_x * 0.5 + 0.5) * viewport.width();
		let sy = viewport.y() + (ndc_y * 0.5 + 0.5) * viewport.height();
		Some((Vec2::new(sx, sy), -v.z))
	}

	pub fn screen_ray(&self, screen_pos: Vec2<f32>, viewport: Rect<f32>) -> Ray<f32> {
		let w = viewport.width().max(1.0);
		let h = viewport.height().max(1.0);
		let aspect = w / h;
		let ndc_x = ((screen_pos.x - viewport.x()) / w) * 2.0 - 1.0;
		let ndc_y = 1.0 - ((screen_pos.y - viewport.y()) / h) * 2.0;
		let tan_half = (self.fov_y * 0.5).tan();

		let dir = self.forward + self.right * (ndc_x * aspect * tan_half) + self.up * (ndc_y * tan_half);
		Ray::new(self.position, dir)
	}

	pub fn world_scale_at(&self, world_pos: Vec3<f32>, viewport: Rect<f32>, desired_px: f32) -> f32 {
		let depth = (world_pos - self.position).dot(&self.forward).max(0.2);
		let h = viewport.height().max(1.0);
		let world_h = 2.0 * depth * (self.fov_y * 0.5).tan();
		(desired_px / h) * world_h
	}
}

pub(crate) fn closest_axis_param(
	ray: &Ray<f32>,
	axis_origin: Vec3<f32>,
	axis_dir: Vec3<f32>,
) -> Option<f32> {
	let w0 = axis_origin - ray.origin;
	let a = axis_dir.dot(&axis_dir);
	let b = axis_dir.dot(&ray.direction);
	let c = ray.direction.dot(&ray.direction);
	let d = axis_dir.dot(&w0);
	let e = ray.direction.dot(&w0);
	let denom = a * c - b * b;
	if denom.abs() < 1e-5 {
		return None;
	}
	Some((b * e - c * d) / denom)
}

pub(crate) fn dist_point_to_segment_2d(p: Vec2<f32>, a: Vec2<f32>, b: Vec2<f32>) -> f32 {
	let ab = b - a;
	let ap = p - a;
	let len_sq = ab.x * ab.x + ab.y * ab.y;
	let t = if len_sq > 1e-6 {
		((ap.x * ab.x + ap.y * ab.y) / len_sq).clamp(0.0, 1.0)
	} else {
		0.0
	};
	let closest = Vec2::new(a.x + ab.x * t, a.y + ab.y * t);
	let d = p - closest;
	(d.x * d.x + d.y * d.y).sqrt()
}

pub(crate) fn point_in_quad_2d(p: Vec2<f32>, q: [Vec2<f32>; 4]) -> bool {
	let mut pos = 0;
	let mut neg = 0;
	for i in 0..4 {
		let a = q[i];
		let b = q[(i + 1) % 4];
		let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
		if cross > 0.0 {
			pos += 1;
		} else if cross < 0.0 {
			neg += 1;
		}
	}
	pos == 0 || neg == 0
}

pub(crate) fn quat_from_basis(right: Vec3<f32>, up: Vec3<f32>, back: Vec3<f32>) -> Quat<f32> {
	let m00 = right.x;
	let m01 = up.x;
	let m02 = back.x;
	let m10 = right.y;
	let m11 = up.y;
	let m12 = back.y;
	let m20 = right.z;
	let m21 = up.z;
	let m22 = back.z;

	let trace = m00 + m11 + m22;
	let q = if trace > 0.0 {
		let s = (trace + 1.0).sqrt() * 2.0;
		Quat::new((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s)
	} else if m00 > m11 && m00 > m22 {
		let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
		Quat::new(0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
	} else if m11 > m22 {
		let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
		Quat::new((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
	} else {
		let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
		Quat::new((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
	};
	q.normalize()
}

impl Interpolate for GizmoCamera {
	fn interpolate(&self, target: &Self, t: f32) -> Self {
		let orientation = self.orientation.interpolate(&target.orientation, t);
		let look_target = self.target.interpolate(&target.target, t);
		let distance = self.distance.interpolate(&target.distance, t);
		let fov_y = Rad::new(self.fov_y.rad.interpolate(&target.fov_y.rad, t));
		let aspect = self.aspect.interpolate(&target.aspect, t);
		Self::from_orientation(look_target, distance, orientation, fov_y, aspect)
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::orientation::CameraViewPreset;

	#[test]
	fn test_camera_orientation_roundtrip_and_slerp() {
		let presets = [
			CameraViewPreset::Front,
			CameraViewPreset::Back,
			CameraViewPreset::Right,
			CameraViewPreset::Left,
			CameraViewPreset::Top,
			CameraViewPreset::Bottom,
		];
		for preset in presets {
			let (yaw, pitch) = preset.orbit_angles();
			let cam = GizmoCamera::from_orbit(
				Vec3::ZERO,
				5.0,
				yaw,
				pitch,
				Rad::new(std::f32::consts::FRAC_PI_4),
				16.0 / 9.0,
			);
			let rebuilt = GizmoCamera::from_orientation(
				cam.target,
				cam.distance,
				cam.orientation,
				cam.fov_y,
				cam.aspect,
			);
			assert!((cam.right - rebuilt.right).length() < 1e-4);
			assert!((cam.up - rebuilt.up).length() < 1e-4);
			assert!((cam.forward - rebuilt.forward).length() < 1e-4);
		}

		let a = GizmoCamera::default();
		let b = GizmoCamera::from_orbit(
			Vec3::ZERO,
			5.0,
			0.0,
			1.54,
			Rad::new(std::f32::consts::FRAC_PI_4),
			16.0 / 9.0,
		);
		let mid = a.interpolate(&b, 0.5);
		assert!(mid.orientation.is_normalized(1e-3));
		assert!((mid.right.dot(&mid.up)).abs() < 1e-4);
		assert!((mid.right.dot(&mid.forward)).abs() < 1e-4);
	}
}
