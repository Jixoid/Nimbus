/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use nimbus_math::{Color, Vec2};
use crate::theme::Theme;


#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
	String(String),
	Number(f64),
	Bool(bool),
	Color(Color<f32>),
	Vec2(Vec2<f32>),
	Array(Vec<PropertyValue>),
	Object(HashMap<String, PropertyValue>),
	Ident(String),
}

impl PropertyValue {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			PropertyValue::String(s) | PropertyValue::Ident(s) => Some(s.as_str()),
			_ => None,
		}
	}

	pub fn as_f32(&self) -> Option<f32> {
		match self {
			PropertyValue::Number(n) => Some(*n as f32),
			_ => None,
		}
	}

	pub fn as_f64(&self) -> Option<f64> {
		match self {
			PropertyValue::Number(n) => Some(*n),
			_ => None,
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			PropertyValue::Bool(b) => Some(*b),
			_ => None,
		}
	}

	pub fn as_color(&self) -> Option<Color<f32>> {
		match self {
			PropertyValue::Color(c) => Some(*c),
			_ => None,
		}
	}

	pub fn as_vec2(&self) -> Option<Vec2<f32>> {
		match self {
			PropertyValue::Vec2(v) => Some(*v),
			PropertyValue::Object(obj) => {
				let x = obj
					.get("w")
					.or_else(|| obj.get("x"))
					.and_then(|v| v.as_f32())
					.unwrap_or(0.0);
				let y = obj
					.get("h")
					.or_else(|| obj.get("y"))
					.and_then(|v| v.as_f32())
					.unwrap_or(0.0);
				Some(Vec2::new(x, y))
			}
			_ => None,
		}
	}

	pub fn as_array(&self) -> Option<&[PropertyValue]> {
		match self {
			PropertyValue::Array(arr) => Some(arr.as_slice()),
			_ => None,
		}
	}

	pub fn as_object(&self) -> Option<&HashMap<String, PropertyValue>> {
		match self {
			PropertyValue::Object(obj) => Some(obj),
			_ => None,
		}
	}

	/// Resolves monet
	pub fn resolve_color(&self, theme: &Theme) -> Option<Color<f32>> {
		if let Some(c) = self.as_color() {
			return Some(c);
		}
		if let Some(s) = self.as_str() {
			let s_trim = s.trim();
			if s_trim.starts_with("monet(") && s_trim.ends_with(')') {
				let inner = &s_trim[6..s_trim.len() - 1];
				return match inner {
					"Primary" => Some(theme.primary),
					"PrimaryContainer" => Some(theme.primary_hover),
					"Secondary" => Some(theme.primary_active),
					"OnSecondary" => Some(theme.text_primary),
					"Surface" => Some(theme.surface),
					"SurfaceContainer" => Some(theme.surface_elevated),
					"SurfaceContainerLow" => Some(theme.background),
					"Background" => Some(theme.background),
					_ => Some(theme.primary),
				};
			} else if let Some(hex) = s_trim.strip_prefix('#') {
				if hex.len() == 6 {
					let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
					let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
					let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
					return Some(Color::new(r, g, b, 1.0));
				} else if hex.len() == 8 {
					let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
					let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
					let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
					let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0;
					return Some(Color::new(r, g, b, a));
				}
			}
		}
		None
	}
}

pub struct PropertyCtx<'a> {
	pub theme: &'a Theme,
}
