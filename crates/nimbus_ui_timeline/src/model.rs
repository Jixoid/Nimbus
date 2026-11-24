/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::PropertyValue;
use nimbus_math::{Color, Vec2};
use nimbus_theme::{DColor, Monet, MonetRole};


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyframeInterp {
	Step,
	Linear,
	#[default]
	Bezier,
}

impl KeyframeInterp {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim().to_ascii_lowercase().as_str() {
			"step" => Some(Self::Step),
			"linear" => Some(Self::Linear),
			"bezier" => Some(Self::Bezier),
			_ => None,
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Keyframe {
	pub time: f32,
	pub value: f32,
	pub interp: KeyframeInterp,
	/// Relative offset from `(time, value)` for incoming Bézier handle (typically negative x).
	pub in_tangent: Vec2<f32>,
	/// Relative offset from `(time, value)` for outgoing Bézier handle (typically positive x).
	pub out_tangent: Vec2<f32>,
}

impl Keyframe {
	pub fn new(time: f32, value: f32) -> Self {
		Self {
			time: time.max(0.0),
			value,
			interp: KeyframeInterp::Bezier,
			in_tangent: Vec2::new(-0.2, 0.0),
			out_tangent: Vec2::new(0.2, 0.0),
		}
	}

	pub fn with_interp(mut self, interp: KeyframeInterp) -> Self {
		self.interp = interp;
		self
	}

	pub fn with_tangents(mut self, in_tangent: Vec2<f32>, out_tangent: Vec2<f32>) -> Self {
		self.in_tangent = in_tangent;
		self.out_tangent = out_tangent;
		self
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct TimelineClip {
	pub start: f32,
	pub duration: f32,
	pub label: String,
	pub color: Option<DColor>,
}

impl TimelineClip {
	pub fn new(start: f32, duration: f32, label: impl Into<String>) -> Self {
		Self {
			start: start.max(0.0),
			duration: duration.max(0.05),
			label: label.into(),
			color: None,
		}
	}

	pub fn end(&self) -> f32 {
		self.start + self.duration
	}
}


#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
	pub name: String,
	pub color: Option<DColor>,
	pub muted: bool,
	pub locked: bool,
	pub keyframes: Vec<Keyframe>,
	pub clips: Vec<TimelineClip>,
}

impl TimelineTrack {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			color: None,
			muted: false,
			locked: false,
			keyframes: Vec::new(),
			clips: Vec::new(),
		}
	}

	pub fn with_color(mut self, color: impl Into<DColor>) -> Self {
		self.color = Some(color.into());
		self
	}

	pub fn with_keyframes(mut self, mut keyframes: Vec<Keyframe>) -> Self {
		keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
		self.keyframes = keyframes;
		self
	}

	pub fn with_clips(mut self, clips: Vec<TimelineClip>) -> Self {
		self.clips = clips;
		self
	}

	pub fn sort_keyframes(&mut self) {
		self.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
	}

	pub fn resolved_color(&self, index: usize, monet: &Monet) -> Color {
		if let Some(c) = self.color {
			return c.get(monet);
		}
		match index % 4 {
			0 => monet[MonetRole::Primary],
			1 => Color::from_oklch(0.72, 0.14, 150.0),
			2 => Color::from_oklch(0.70, 0.14, 245.0),
			_ => monet[MonetRole::Tertiary],
		}
	}

	/// Samples the track value at time `t` (seconds).
	pub fn sample(&self, t: f32) -> f32 {
		if self.keyframes.is_empty() {
			return 0.0;
		}
		if t <= self.keyframes[0].time {
			return self.keyframes[0].value;
		}
		let last = self.keyframes.len() - 1;
		if t >= self.keyframes[last].time {
			return self.keyframes[last].value;
		}

		for i in 0..last {
			let k0 = &self.keyframes[i];
			let k1 = &self.keyframes[i + 1];
			if t >= k0.time && t < k1.time {
				let span = (k1.time - k0.time).max(1e-5);
				let u = ((t - k0.time) / span).clamp(0.0, 1.0);
				return match k0.interp {
					KeyframeInterp::Step => k0.value,
					KeyframeInterp::Linear => k0.value + (k1.value - k0.value) * u,
					KeyframeInterp::Bezier => sample_cubic_bezier_segment(k0, k1, t),
				};
			}
		}

		self.keyframes[last].value
	}
}

fn eval_cubic(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
	let inv = 1.0 - u;
	inv * inv * inv * p0
		+ 3.0 * inv * inv * u * p1
		+ 3.0 * inv * u * u * p2
		+ u * u * u * p3
}

/// Evaluates `y(t)` on a cubic Bézier segment defined by `k0` and `k1` by solving `x(u) = t`.
pub(crate) fn sample_cubic_bezier_segment(k0: &Keyframe, k1: &Keyframe, t: f32) -> f32 {
	let dt = (k1.time - k0.time).max(1e-5);
	let x0 = k0.time;
	let x3 = k1.time;
	// Clamp control point X inside [x0, x3] so x(u) stays monotonic
	let x1 = (k0.time + k0.out_tangent.x.max(0.0)).clamp(x0, x3);
	let x2 = (k1.time + k1.in_tangent.x.min(0.0)).clamp(x0, x3);

	let y0 = k0.value;
	let y1 = k0.value + k0.out_tangent.y;
	let y2 = k1.value + k1.in_tangent.y;
	let y3 = k1.value;

	let mut lo = 0.0f32;
	let mut hi = 1.0f32;
	let mut u = ((t - x0) / dt).clamp(0.0, 1.0);

	for _ in 0..14 {
		let x = eval_cubic(x0, x1, x2, x3, u);
		if (x - t).abs() < 1e-4 {
			break;
		}
		if x < t {
			lo = u;
		} else {
			hi = u;
		}
		u = 0.5 * (lo + hi);
	}

	eval_cubic(y0, y1, y2, y3, u)
}


pub(crate) fn parse_tracks_from_property(value: &PropertyValue) -> Option<Vec<TimelineTrack>> {
	let arr = value.as_array()?;
	let mut tracks = Vec::with_capacity(arr.len());

	for item in arr {
		let Some(obj) = item.as_object() else {
			continue;
		};
		let name = obj
			.get("name")
			.and_then(|v| v.as_str())
			.unwrap_or("Track")
			.to_string();
		let mut track = TimelineTrack::new(name);
		if let Some(col) = obj.get("color").and_then(|v| v.resolve_dcolor()) {
			track.color = Some(col);
		}
		if let Some(m) = obj.get("muted").and_then(|v| v.as_bool()) {
			track.muted = m;
		}
		if let Some(l) = obj.get("locked").and_then(|v| v.as_bool()) {
			track.locked = l;
		}
		if let Some(kfs) = obj.get("keyframes").and_then(|v| v.as_array()) {
			for kf_val in kfs {
				if let Some(kobj) = kf_val.as_object() {
					let time = kobj.get("time").and_then(|v| v.as_f32()).unwrap_or(0.0);
					let val = kobj.get("value").and_then(|v| v.as_f32()).unwrap_or(0.0);
					let mut kf = Keyframe::new(time, val);
					if let Some(interp) = kobj
						.get("interp")
						.and_then(|v| v.as_str())
						.and_then(KeyframeInterp::parse)
					{
						kf.interp = interp;
					}
					if let Some(v) = kobj.get("in_tangent").and_then(|v| v.as_vec2()) {
						kf.in_tangent = v;
					}
					if let Some(v) = kobj.get("out_tangent").and_then(|v| v.as_vec2()) {
						kf.out_tangent = v;
					}
					track.keyframes.push(kf);
				}
			}
			track.sort_keyframes();
		}
		if let Some(clips) = obj.get("clips").and_then(|v| v.as_array()) {
			for cval in clips {
				if let Some(cobj) = cval.as_object() {
					let start = cobj.get("start").and_then(|v| v.as_f32()).unwrap_or(0.0);
					let dur = cobj.get("duration").and_then(|v| v.as_f32()).unwrap_or(1.0);
					let label = cobj
						.get("label")
						.and_then(|v| v.as_str())
						.unwrap_or("Clip")
						.to_string();
					let mut clip = TimelineClip::new(start, dur, label);
					if let Some(col) = cobj.get("color").and_then(|v| v.resolve_dcolor()) {
						clip.color = Some(col);
					}
					track.clips.push(clip);
				}
			}
		}
		tracks.push(track);
	}

	Some(tracks)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_track_sampling_step_linear_bezier() {
		let track = TimelineTrack::new("Pos X").with_keyframes(vec![
			Keyframe::new(0.0, 0.0).with_interp(KeyframeInterp::Linear),
			Keyframe::new(2.0, 10.0).with_interp(KeyframeInterp::Step),
			Keyframe::new(4.0, 20.0).with_interp(KeyframeInterp::Bezier),
			Keyframe::new(6.0, 0.0),
		]);

		assert!((track.sample(-1.0) - 0.0).abs() < 1e-4);
		assert!((track.sample(1.0) - 5.0).abs() < 1e-4);
		assert!((track.sample(3.5) - 10.0).abs() < 1e-4);
		assert!((track.sample(4.0) - 20.0).abs() < 1e-4);
		let mid_bezier = track.sample(5.0);
		assert!((mid_bezier - 10.0).abs() < 0.25);
		assert!((track.sample(8.0) - 0.0).abs() < 1e-4);
	}
}
