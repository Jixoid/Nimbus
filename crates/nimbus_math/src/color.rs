/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::ops::{Add, Mul, Sub};
use crate::{Float, Number, Vec3, Vec4};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color<T: Number = f32> {
	pub r: T,
	pub g: T,
	pub b: T,
	pub a: T,
}


impl<T: Number> Color<T> {
	pub const TRANSPARENT: Self = Self {
		r: T::ZERO,
		g: T::ZERO,
		b: T::ZERO,
		a: T::ZERO,
	};

	pub const BLACK: Self = Self {
		r: T::ZERO,
		g: T::ZERO,
		b: T::ZERO,
		a: T::ONE,
	};
	
	pub const WHITE: Self = Self {
		r: T::ONE,
		g: T::ONE,
		b: T::ONE,
		a: T::ONE,
	};

	pub const RED: Self = Self {
		r: T::ONE,
		g: T::ZERO,
		b: T::ZERO,
		a: T::ONE,
	};

	pub const GREEN: Self = Self {
		r: T::ZERO,
		g: T::ONE,
		b: T::ZERO,
		a: T::ONE,
	};

	pub const BLUE: Self = Self {
		r: T::ZERO,
		g: T::ZERO,
		b: T::ONE,
		a: T::ONE,
	};

	pub const YELLOW: Self = Self {
		r: T::ONE,
		g: T::ONE,
		b: T::ZERO,
		a: T::ONE,
	};

	pub const CYAN: Self = Self {
		r: T::ZERO,
		g: T::ONE,
		b: T::ONE,
		a: T::ONE,
	};

	pub const MAGENTA: Self = Self {
		r: T::ONE,
		g: T::ZERO,
		b: T::ONE,
		a: T::ONE,
	};


	pub const fn new(r: T, g: T, b: T, a: T) -> Self {
		Self { r, g, b, a }
	}

	pub fn rgb(r: T, g: T, b: T) -> Self {
		Self { r, g, b, a: T::ONE }
	}


	pub fn lerp(&self, other: &Self, t: T) -> Self {
		Self {
			r: self.r + (other.r - self.r) * t,
			g: self.g + (other.g - self.g) * t,
			b: self.b + (other.b - self.b) * t,
			a: self.a + (other.a - self.a) * t,
		}
	}


	pub fn to_array(&self) -> [T; 4] {
		[self.r, self.g, self.b, self.a]
	}

	pub fn to_tuple(&self) -> (T, T, T, T) {
		(self.r, self.g, self.b, self.a)
	}
}

impl<T: Float> Color<T> {
	pub fn from_oklch(light: f32, chroma: f32, hue: f32) -> Self {
		// OKLCH -> OKLAB
		let h = hue.to_radians();
		let a = chroma * f32::cos(h);
		let b = chroma * f32::sin(h);

		// OKLAB -> Linear sRGB
		let l_ = light +(0.3963377774 *a) +(0.2158037573 *b);
		let m_ = light -(0.1055613458 *a) -(0.0638541728 *b);
		let s_ = light -(0.0894841775 *a) -(1.2914855480 *b);

		let l = l_ *l_ *l_;
		let m = m_ *m_ *m_;
		let s = s_ *s_ *s_;

		let r_lin = ( 4.0767416621 *l) -(3.3077115913 *m) +(0.2309699292 *s);
		let g_lin = (-1.2684380046 *l) +(2.6097574011 *m) -(0.3413193965 *s);
		let b_lin = (-0.0041960863 *l) -(0.7034186147 *m) +(1.7076147010 *s);

		// Linear sRGB -> sRGB
		let gamma_correct = |c: f32| -> f32 {
			if c <= 0.0031308 {
				12.92 *c
			} else {
				1.055 *c.powf(1.0 / 2.4) -0.055
			}
		};

		Self {
			r: T::from_f32(gamma_correct(r_lin).clamp(0.0, 1.0)),
			g: T::from_f32(gamma_correct(g_lin).clamp(0.0, 1.0)),
			b: T::from_f32(gamma_correct(b_lin).clamp(0.0, 1.0)),
			a: T::ONE,
		}
	}

	pub fn to_oklch(&self) -> (f32, f32, f32) {
		// sRGB -> Linear sRGB
		let to_linear = |c: f32| -> f32 {
			if c <= 0.04045 {
				c / 12.92
			} else {
				f32::powf((c + 0.055) / 1.055, 2.4)
			}
		};

		let r_lin = to_linear(self.r.to_f32());
		let g_lin = to_linear(self.g.to_f32());
		let b_lin = to_linear(self.b.to_f32());

		// Linear sRGB -> LMS
		let l = (0.4122214708 *r_lin) +(0.5363325363 *g_lin) +(0.0514459929 *b_lin);
		let m = (0.2119034982 *r_lin) +(0.6806995451 *g_lin) +(0.1073969566 *b_lin);
		let s = (0.0883024619 *r_lin) +(0.2817188976 *g_lin) +(0.6299787005 *b_lin);

		// LMS -> Non-linear LMS
		let l_ = f32::powf(l, 1.0 / 3.0);
		let m_ = f32::powf(m, 1.0 / 3.0);
		let s_ = f32::powf(s, 1.0 / 3.0);

		// Non-linear LMS -> OKLAB
		let l = (0.2104542553 *l_) +(0.7936177850 *m_) -(0.0040720468 *s_);
		let a = (1.9779984951 *l_) -(2.4285922050 *m_) +(0.4505937099 *s_);
		let b = (0.0259040371 *l_) +(0.7827717662 *m_) -(0.8086757660 *s_);

		// OKLAB -> OKLCH
		let c = f32::sqrt(a * a + b * b);
		let mut h = f32::atan2(b, a).to_degrees();

		if h < 0.0 {
			h += 360.0;
		}
		
		(l, c, h)
	}

	
	pub fn to_rgba_u8(&self) -> [u8; 4] {
		let clamp_u8 = |v: T| -> u8 {
			let clamped = T::clamp(v, T::ZERO, T::ONE);
			(clamped * T::from_f32(255.0)).to_f32() as u8
		};

		[
			clamp_u8(self.r),
			clamp_u8(self.g),
			clamp_u8(self.b),
			clamp_u8(self.a),
		]
	}
}


impl<T: Number> Default for Color<T> {
	fn default() -> Self {
		Self::WHITE
	}
}


impl<T: Number> Add<Self> for Color<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r + rhs.r,
			g: self.g + rhs.g,
			b: self.b + rhs.b,
			a: self.a + rhs.a,
		}
	}
}

impl<T: Number> Sub<Self> for Color<T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r - rhs.r,
			g: self.g - rhs.g,
			b: self.b - rhs.b,
			a: self.a - rhs.a,
		}
	}
}


impl<T: Number> Mul<Self> for Color<T> {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			r: self.r * rhs.r,
			g: self.g * rhs.g,
			b: self.b * rhs.b,
			a: self.a * rhs.a,
		}
	}
}

impl<T: Number> Mul<T> for Color<T> {
	type Output = Self;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			r: self.r * rhs,
			g: self.g * rhs,
			b: self.b * rhs,
			a: self.a * rhs,
		}
	}
}


impl<T: Number> From<Vec4<T>> for Color<T> {
	fn from(v: Vec4<T>) -> Self {
		Self {
			r: v.x,
			g: v.y,
			b: v.z,
			a: v.w,
		}
	}
}

impl<T: Number> From<Color<T>> for Vec4<T> {
	fn from(c: Color<T>) -> Self {
		Vec4::new(c.r, c.g, c.b, c.a)
	}
}


impl<T: Number> From<Vec3<T>> for Color<T> {
	fn from(v: Vec3<T>) -> Self {
		Self {
			r: v.x,
			g: v.y,
			b: v.z,
			a: T::ONE,
		}
	}
}

impl<T: Number> From<Color<T>> for Vec3<T> {
	fn from(c: Color<T>) -> Self {
		Vec3::new(c.r, c.g, c.b)
	}
}

impl<T: Number> From<[T; 4]> for Color<T> {
	fn from(arr: [T; 4]) -> Self {
		Self::new(arr[0], arr[1], arr[2], arr[3])
	}
}

impl<T: Number> From<[T; 3]> for Color<T> {
	fn from(arr: [T; 3]) -> Self {
		Self::rgb(arr[0], arr[1], arr[2])
	}
}

impl<T: Number> From<(T, T, T, T)> for Color<T> {
	fn from((r, g, b, a): (T, T, T, T)) -> Self {
		Self::new(r, g, b, a)
	}
}

impl<T: Number> From<(T, T, T)> for Color<T> {
	fn from((r, g, b): (T, T, T)) -> Self {
		Self::rgb(r, g, b)
	}
}

impl<T: Number> From<Color<T>> for [T; 4] {
	fn from(c: Color<T>) -> Self {
		c.to_array()
	}
}

impl<T: Number> From<Color<T>> for (T, T, T, T) {
	fn from(c: Color<T>) -> Self {
		(c.r, c.g, c.b, c.a)
	}
}


impl Color<f32> {
	pub fn parse(s: &str) -> Option<Self> {
		parse_color(s)
	}

	pub fn parse_color(s: &str) -> Option<Self> {
		parse_color(s)
	}
}

impl core::str::FromStr for Color<f32> {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		parse_color(s).ok_or(())
	}
}


/// Parses a CSS/Nimbus color string into a `Color<f32>`
pub fn parse_color(s: &str) -> Option<Color<f32>> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	// "color(...)" or named colors
	if let Some(inner) = strip_fn_call(s, "color") {
		return parse_named_color(inner.trim());
	}
	if let Some(named) = parse_named_color(s) {
		return Some(named);
	}

	// "rgba(...)" or "rgb(...)"
	if let Some(inner) = strip_fn_call(s, "rgba").or_else(|| strip_fn_call(s, "rgb")) {
		let (parts, count) = split_components(inner.trim())?;
		if count == 3 || count == 4 {
			let r = parse_rgb_channel(parts[0])?;
			let g = parse_rgb_channel(parts[1])?;
			let b = parse_rgb_channel(parts[2])?;
			let a = if count == 4 {
				parse_alpha(parts[3])?
			} else {
				1.0
			};

			return Some(Color::new(
				r.clamp(0.0, 1.0),
				g.clamp(0.0, 1.0),
				b.clamp(0.0, 1.0),
				a.clamp(0.0, 1.0),
			));
		}
		return None;
	}

	// "oklch(l,c,h)" or "oklch(l,c,h,a)"
	if let Some(inner) = strip_fn_call(s, "oklch") {
		let (parts, count) = split_components(inner.trim())?;
		if count == 3 || count == 4 {
			let l_str = parts[0];
			let l = if let Some(pct) = l_str.strip_suffix('%') {
				pct.trim().parse::<f32>().ok()? / 100.0
			} else {
				let val = l_str.parse::<f32>().ok()?;
				if val > 1.0 {
					val / 100.0
				} else {
					val
				}
			};

			let c_str = parts[1];
			let c = if let Some(pct) = c_str.strip_suffix('%') {
				pct.trim().parse::<f32>().ok()? / 100.0 * 0.4
			} else {
				c_str.parse::<f32>().ok()?
			};

			let h_str = parts[2];
			let h = if let Some(deg) = strip_suffix_ignore_ascii_case(h_str, "deg") {
				deg.trim().parse::<f32>().ok()?
			} else if let Some(rad) = strip_suffix_ignore_ascii_case(h_str, "rad") {
				rad.trim().parse::<f32>().ok()?.to_degrees()
			} else if let Some(turn) = strip_suffix_ignore_ascii_case(h_str, "turn") {
				turn.trim().parse::<f32>().ok()? * 360.0
			} else {
				h_str.parse::<f32>().ok()?
			};

			let a = if count == 4 {
				parse_alpha(parts[3])?
			} else {
				1.0
			};

			let mut color = Color::from_oklch(l.clamp(0.0, 1.0), c.max(0.0), h);
			color.a = a.clamp(0.0, 1.0);
			return Some(color);
		}
		return None;
	}

	// Hex: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
	if let Some(hex) = s.strip_prefix('#') {
		return parse_hex_color(hex);
	}

	None
}

fn strip_fn_call<'a>(s: &'a str, name: &str) -> Option<&'a str> {
	let rest = s.strip_suffix(')')?;
	if rest.len() < name.len() + 1 || !rest.is_char_boundary(name.len()) {
		return None;
	}
	let (head, tail) = rest.split_at(name.len());
	if head.eq_ignore_ascii_case(name) {
		tail.strip_prefix('(')
	} else {
		None
	}
}

fn strip_suffix_ignore_ascii_case<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
	if s.len() < suffix.len() {
		return None;
	}
	let split_idx = s.len() - suffix.len();
	if !s.is_char_boundary(split_idx) {
		return None;
	}
	let (head, tail) = s.split_at(split_idx);
	if tail.eq_ignore_ascii_case(suffix) {
		Some(head)
	} else {
		None
	}
}

fn split_components(s: &str) -> Option<([&str; 4], usize)> {
	let mut parts = [""; 4];
	let mut count = 0;

	if s.contains(',') {
		for part in s.split(',') {
			let trimmed = part.trim();
			if trimmed.is_empty() || count >= 4 {
				return None;
			}
			parts[count] = trimmed;
			count += 1;
		}
	} else {
		for part in s.split(|c: char| c.is_whitespace() || c == '/') {
			let trimmed = part.trim();
			if trimmed.is_empty() {
				continue;
			}
			if count >= 4 {
				return None;
			}
			parts[count] = trimmed;
			count += 1;
		}
	}

	Some((parts, count))
}

fn parse_rgb_channel(s: &str) -> Option<f32> {
	let s = s.trim();
	if let Some(pct) = s.strip_suffix('%') {
		pct.trim().parse::<f32>().ok().map(|v| v / 100.0)
	} else {
		s.parse::<f32>().ok().map(|v| v / 255.0)
	}
}

fn parse_alpha(s: &str) -> Option<f32> {
	let s = s.trim();
	if let Some(pct) = s.strip_suffix('%') {
		pct.trim().parse::<f32>().ok().map(|v| v / 100.0)
	} else {
		s.parse::<f32>().ok()
	}
}

fn parse_named_color(name: &str) -> Option<Color<f32>> {
	let name = name.trim();
	if name.eq_ignore_ascii_case("transparent") {
		Some(Color::TRANSPARENT)
	} else if name.eq_ignore_ascii_case("black") {
		Some(Color::BLACK)
	} else if name.eq_ignore_ascii_case("white") {
		Some(Color::WHITE)
	} else if name.eq_ignore_ascii_case("red") {
		Some(Color::RED)
	} else if name.eq_ignore_ascii_case("green") {
		Some(Color::GREEN)
	} else if name.eq_ignore_ascii_case("blue") {
		Some(Color::BLUE)
	} else if name.eq_ignore_ascii_case("yellow") {
		Some(Color::YELLOW)
	} else if name.eq_ignore_ascii_case("cyan") {
		Some(Color::CYAN)
	} else if name.eq_ignore_ascii_case("magenta") {
		Some(Color::MAGENTA)
	} else {
		None
	}
}

fn parse_hex_color(hex: &str) -> Option<Color<f32>> {
	match hex.len() {
		3 => {
			let r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
			let g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
			let b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
			Some(Color::new(r, g, b, 1.0))
		}
		4 => {
			let r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
			let g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
			let b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
			let a = u8::from_str_radix(&hex[3..4], 16).ok()? as f32 / 15.0;
			Some(Color::new(r, g, b, a))
		}
		6 => {
			let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
			let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
			let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
			Some(Color::new(r, g, b, 1.0))
		}
		8 => {
			let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
			let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
			let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
			let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
			Some(Color::new(r, g, b, a))
		}
		_ => None,
	}
}
