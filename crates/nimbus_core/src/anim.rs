/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::time::{Duration, Instant};
use nimbus_math::{Color, Rect, Vec2};


pub trait Interpolate: Clone + Send + Sync + 'static {
	fn interpolate(&self, target: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
	#[inline]
	fn interpolate(&self, target: &Self, t: f32) -> Self {
		*self + (*target - *self) * t
	}
}

impl Interpolate for Vec2<f32> {
	#[inline]
	fn interpolate(&self, target: &Self, t: f32) -> Self {
		Vec2::new(
			self.x.interpolate(&target.x, t),
			self.y.interpolate(&target.y, t),
		)
	}
}

impl Interpolate for Rect<f32> {
	fn interpolate(&self, target: &Self, t: f32) -> Self {
		if t <= 0.0 {
			return *self;
		}
		if (t - 1.0).abs() < 1e-5 {
			return *target;
		}

		// When spring overshoots past 1.0 or under 0.0, apply uniform physical scaling
		if t < 0.0 || t > 1.0 {
			return Rect::from_min_max(
				self.min.interpolate(&target.min, t),
				self.max.interpolate(&target.max, t),
			);
		}

		let dx = (target.min.x + target.max.x) - (self.min.x + self.max.x);
		let dy = (target.min.y + target.max.y) - (self.min.y + self.max.y);

		// Fluid elastic stretch interpolation
		let t_lead = 1.0 - (1.0 - t).powf(1.4);
		let t_trail = t.powf(1.4);

		let (min_x, max_x) = if dx > 0.5 {
			let min_x = self.min.x + (target.min.x - self.min.x) * t_trail;
			let max_x = self.max.x + (target.max.x - self.max.x) * t_lead;
			(min_x, max_x)
		} else if dx < -0.5 {
			let min_x = self.min.x + (target.min.x - self.min.x) * t_lead;
			let max_x = self.max.x + (target.max.x - self.max.x) * t_trail;
			(min_x, max_x)
		} else {
			(
				self.min.x + (target.min.x - self.min.x) * t,
				self.max.x + (target.max.x - self.max.x) * t,
			)
		};

		let (min_y, max_y) = if dy > 0.5 {
			let min_y = self.min.y + (target.min.y - self.min.y) * t_trail;
			let max_y = self.max.y + (target.max.y - self.max.y) * t_lead;
			(min_y, max_y)
		} else if dy < -0.5 {
			let min_y = self.min.y + (target.min.y - self.min.y) * t_lead;
			let max_y = self.max.y + (target.max.y - self.max.y) * t_trail;
			(min_y, max_y)
		} else {
			(
				self.min.y + (target.min.y - self.min.y) * t,
				self.max.y + (target.max.y - self.max.y) * t,
			)
		};

		Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
	}
}

impl Interpolate for Color<f32> {
	#[inline]
	fn interpolate(&self, target: &Self, t: f32) -> Self {
		Color {
			r: self.r.interpolate(&target.r, t),
			g: self.g.interpolate(&target.g, t),
			b: self.b.interpolate(&target.b, t),
			a: self.a.interpolate(&target.a, t),
		}
	}
}


/// Easing curves for duration-based transitions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
	Linear,
	EaseIn,
	EaseOut,
	EaseInOut,
	/// Material Design 3 Emphasized: (0.2, 0.0, 0.0, 1.0)
	Emphasized,
	/// Material Design 3 Standard Decelerate: (0.0, 0.0, 0.2, 1.0)
	Decelerate,
	/// Material Design 3 Standard Accelerate: (0.3, 0.0, 1.0, 1.0)
	Accelerate,
	/// Playful anticipation/overshoot then settle: ideal for popups, tooltips, checkmarks
	BackOut,
	/// Material 3 Emphasized Decelerate: (0.05, 0.7, 0.1, 1.0)
	EmphasizedDecelerate,
	/// Custom Cubic Bézier curve control points: (p1x, p1y, p2x, p2y)
	CubicBezier(f32, f32, f32, f32),
}

impl Easing {
	pub fn apply(&self, t: f32) -> f32 {
		let t = t.clamp(0.0, 1.0);
		match self {
			Easing::Linear => t,
			Easing::EaseIn => t * t * t,
			Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
			Easing::EaseInOut => {
				if t < 0.5 {
					4.0 * t * t * t
				} else {
					1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
				}
			}
			Easing::Emphasized => sample_cubic_bezier(0.2, 0.0, 0.0, 1.0, t),
			Easing::Decelerate => sample_cubic_bezier(0.0, 0.0, 0.2, 1.0, t),
			Easing::Accelerate => sample_cubic_bezier(0.3, 0.0, 1.0, 1.0, t),
			Easing::BackOut => {
				let s = 1.4f32;
				let p = t - 1.0;
				1.0 + (s + 1.0) * p * p * p + s * p * p
			}
			Easing::EmphasizedDecelerate => sample_cubic_bezier(0.05, 0.7, 0.1, 1.0, t),
			Easing::CubicBezier(p1x, p1y, p2x, p2y) => sample_cubic_bezier(*p1x, *p1y, *p2x, *p2y, t),
		}
	}
}

/// Solves a cubic Bézier curve y given x = t using Newton-Raphson iteration
fn sample_cubic_bezier(p1x: f32, p1y: f32, p2x: f32, p2y: f32, t: f32) -> f32 {
	if t <= 0.0 {
		return 0.0;
	}
	if t >= 1.0 {
		return 1.0;
	}

	let mut s = t;
	for _ in 0..8 {
		let x = 3.0 * (1.0 - s) * (1.0 - s) * s * p1x
			+ 3.0 * (1.0 - s) * s * s * p2x
			+ s * s * s;
		let dx = 3.0 * (1.0 - s) * (1.0 - s) * p1x
			+ 6.0 * (1.0 - s) * s * (p2x - p1x)
			+ 3.0 * s * s * (1.0 - p2x);

		let err = x - t;
		if err.abs() < 1e-5 {
			break;
		}
		if dx.abs() > 1e-6 {
			s -= err / dx;
			s = s.clamp(0.0, 1.0);
		} else {
			break;
		}
	}

	3.0 * (1.0 - s) * (1.0 - s) * s * p1y
		+ 3.0 * (1.0 - s) * s * s * p2y
		+ s * s * s
}


/// Physics-based spring parameters (damped harmonic oscillator)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
	pub mass: f32,
	pub stiffness: f32,
	pub damping: f32,
}

impl Spring {
	/// Smooth, critically-damped feel with no overshoot. Ideal for transitions
	pub const SMOOTH: Self = Self {
		mass: 1.0,
		stiffness: 180.0,
		damping: 24.0,
	};

	/// Fast, responsive approach with slight settling. Ideal for scrolling and switches
	pub const SNAPPY: Self = Self {
		mass: 1.0,
		stiffness: 320.0,
		damping: 32.0,
	};

	/// Highly responsive spring with subtle playful overshoot (~4-5%), ideal for tabs, switches, and sliders
	pub const INTERACTIVE: Self = Self {
		mass: 1.0,
		stiffness: 280.0,
		damping: 24.0,
	};

	/// Playful bounce with controlled overshoot. Ideal for alerts and buttons
	pub const BOUNCY: Self = Self {
		mass: 1.0,
		stiffness: 220.0,
		damping: 14.0,
	};

	pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
		Self { mass, stiffness, damping }
	}

	/// Solves the spring position at elapsed time `t`
	pub fn solve(&self, t_sec: f32) -> (f32, bool) {
		if t_sec <= 0.0 {
			return (0.0, false);
		}

		let m = self.mass.max(0.001);
		let k = self.stiffness.max(0.001);
		let c = self.damping.max(0.0);

		let omega0 = (k / m).sqrt();
		let zeta = c / (2.0 * (k * m).sqrt());

		let x0 = -1.0f32; // initial displacement relative to target
		let v0 = 0.0f32;  // initial velocity

		let (x, v) = if zeta < 1.0 {
			// Underdamped (oscillates / bounces)
			let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
			let decay = (-zeta * omega0 * t_sec).exp();
			let cos_val = (omega_d * t_sec).cos();
			let sin_val = (omega_d * t_sec).sin();

			let a = x0;
			let b = (v0 + zeta * omega0 * x0) / omega_d;

			let pos = decay * (a * cos_val + b * sin_val);
			let vel = -zeta * omega0 * pos + decay * (-a * omega_d * sin_val + b * omega_d * cos_val);
			(pos, vel)
		} else if (zeta - 1.0).abs() < 1e-4 {
			// Critically damped
			let decay = (-omega0 * t_sec).exp();
			let pos = decay * (x0 + (v0 + omega0 * x0) * t_sec);
			let vel = decay * (v0 - omega0 * (x0 + (v0 + omega0 * x0) * t_sec) + (v0 + omega0 * x0));
			(pos, vel)
		} else {
			// Overdamped
			let r1 = -omega0 * (zeta - (zeta * zeta - 1.0).sqrt());
			let r2 = -omega0 * (zeta + (zeta * zeta - 1.0).sqrt());

			let c2 = (v0 - r1 * x0) / (r2 - r1);
			let c1 = x0 - c2;

			let pos = c1 * (r1 * t_sec).exp() + c2 * (r2 * t_sec).exp();
			let vel = c1 * r1 * (r1 * t_sec).exp() + c2 * r2 * (r2 * t_sec).exp();
			(pos, vel)
		};

		let factor = 1.0 + x;
		let settled = x.abs() < 0.001 && v.abs() < 0.01;

		if settled {
			(1.0, true)
		} else {
			(factor, false)
		}
	}
}

impl Default for Spring {
	fn default() -> Self {
		Self::SMOOTH
	}
}


#[derive(Clone, Copy, Debug, PartialEq)]
enum AnimMode {
	Idle,
	Curve {
		duration: Duration,
		easing: Easing,
	},
	Spring(Spring),
}


#[derive(Clone, Debug, PartialEq)]
pub struct Animated<T> {
	start: T,
	target: T,
	start_time: Option<Instant>,
	mode: AnimMode,
}

impl<T: Interpolate> Animated<T> {
	pub fn new(initial: T) -> Self {
		Self {
			start: initial.clone(),
			target: initial,
			start_time: None,
			mode: AnimMode::Idle,
		}
	}

	pub fn value(&self) -> T {
		self.value_at(Instant::now())
	}

	pub fn value_at(&self, now: Instant) -> T {
		match self.mode {
			AnimMode::Idle => self.target.clone(),
			AnimMode::Curve { duration, easing } => {
				let start_time = match self.start_time {
					Some(t) => t,
					None => return self.target.clone(),
				};

				if now <= start_time {
					return self.start.clone();
				}

				let elapsed = now.duration_since(start_time);
				if elapsed >= duration {
					return self.target.clone();
				}

				let progress = elapsed.as_secs_f32() / duration.as_secs_f32().max(0.0001);
				let t = easing.apply(progress);
				self.start.interpolate(&self.target, t)
			}
			AnimMode::Spring(spring) => {
				let start_time = match self.start_time {
					Some(t) => t,
					None => return self.target.clone(),
				};

				if now <= start_time {
					return self.start.clone();
				}

				let elapsed_sec = now.duration_since(start_time).as_secs_f32();
				let (factor, settled) = spring.solve(elapsed_sec);

				if settled {
					self.target.clone()
				} else {
					self.start.interpolate(&self.target, factor)
				}
			}
		}
	}

	pub fn is_animating(&self) -> bool {
		self.is_animating_at(Instant::now())
	}

	pub fn is_animating_at(&self, now: Instant) -> bool {
		match self.mode {
			AnimMode::Idle => false,
			AnimMode::Curve { duration, .. } => {
				match self.start_time {
					Some(start) => now < start + duration,
					None => false,
				}
			}
			AnimMode::Spring(spring) => {
				match self.start_time {
					Some(start) => {
						let elapsed = now.saturating_duration_since(start).as_secs_f32();
						let (_, settled) = spring.solve(elapsed);
						!settled
					}
					None => false,
				}
			}
		}
	}

	pub fn set_target(&mut self, target: T, duration: Duration, easing: Easing) {
		let current = self.value();
		self.start = current;
		self.target = target;
		self.start_time = Some(Instant::now());
		self.mode = AnimMode::Curve { duration, easing };
	}

	pub fn set_target_spring(&mut self, target: T, spring: Spring) {
		let current = self.value();
		self.start = current;
		self.target = target;
		self.start_time = Some(Instant::now());
		self.mode = AnimMode::Spring(spring);
	}

	pub fn snap_to(&mut self, value: T) {
		self.start = value.clone();
		self.target = value;
		self.start_time = None;
		self.mode = AnimMode::Idle;
	}

	pub fn target(&self) -> &T {
		&self.target
	}
}

impl<T: Interpolate + Default> Default for Animated<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}
