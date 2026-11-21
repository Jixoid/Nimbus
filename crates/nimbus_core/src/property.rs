/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use nimbus_except::nimbus_log;
use nimbus_math::{Color, Vec2};
use nimbus_theme::{Monet, MonetRole, Theme};


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
					.get("x")
					.and_then(|v| v.as_f32())
					.unwrap_or(0.0);
				let y = obj
					.get("y")
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

	
	pub fn resolve_color(&self, monet: &Monet) -> Option<Color<f32>> {
		if let Some(c) = self.as_color() {
			return Some(c);
		}
		if let Some(s) = self.as_str() {
			let s_trim = s.trim();
			if s_trim.starts_with("monet(") && s_trim.ends_with(')') {
				let inner = &s_trim[6..s_trim.len() - 1];
				let (target, alpha) = if let Some((role_str, alpha_str)) = inner.split_once(',') {
					(role_str.trim(), alpha_str.trim().parse::<f32>().unwrap_or(1.0))
				} else {
					(inner.trim(), 1.0)
				};

				use MonetRole::*;
				let role = match target {
					// Primary
					"Primary" => Some(Primary),
					"OnPrimary" => Some(OnPrimary),
					"PrimaryContainer" => Some(PrimaryContainer),
					"OnPrimaryContainer" => Some(OnPrimaryContainer),
					"PrimaryFixed" => Some(PrimaryFixed),
					"PrimaryFixedDim" => Some(PrimaryFixedDim),
					"OnPrimaryFixed" => Some(OnPrimaryFixed),
					"OnPrimaryFixedVariant" => Some(OnPrimaryFixedVariant),
					"PrimaryOutline" => Some(PrimaryOutline),

					// Secondary
					"Secondary" => Some(Secondary),
					"OnSecondary" => Some(OnSecondary),
					"SecondaryContainer" => Some(SecondaryContainer),
					"OnSecondaryContainer" => Some(OnSecondaryContainer),
					"SecondaryFixed" => Some(SecondaryFixed),
					"SecondaryFixedDim" => Some(SecondaryFixedDim),
					"OnSecondaryFixed" => Some(OnSecondaryFixed),
					"OnSecondaryFixedVariant" => Some(OnSecondaryFixedVariant),
					"SecondaryOutline" => Some(SecondaryOutline),

					// Tertiary
					"Tertiary" => Some(Tertiary),
					"OnTertiary" => Some(OnTertiary),
					"TertiaryContainer" => Some(TertiaryContainer),
					"OnTertiaryContainer" => Some(OnTertiaryContainer),
					"TertiaryFixed" => Some(TertiaryFixed),
					"TertiaryFixedDim" => Some(TertiaryFixedDim),
					"OnTertiaryFixed" => Some(OnTertiaryFixed),
					"OnTertiaryFixedVariant" => Some(OnTertiaryFixedVariant),
					"TertiaryOutline" => Some(TertiaryOutline),

					// Error
					"Error" => Some(Error),
					"OnError" => Some(OnError),
					"ErrorContainer" => Some(ErrorContainer),
					"OnErrorContainer" => Some(OnErrorContainer),
					"ErrorFixed" => Some(ErrorFixed),
					"ErrorFixedDim" => Some(ErrorFixedDim),
					"OnErrorFixed" => Some(OnErrorFixed),
					"OnErrorFixedVariant" => Some(OnErrorFixedVariant),
					"ErrorOutline" => Some(ErrorOutline),

					// Surface
					"Surface" => Some(Surface),
					"OnSurface" => Some(OnSurface),
					"SurfaceVariant" => Some(SurfaceVariant),
					"OnSurfaceVariant" => Some(OnSurfaceVariant),
					"SurfaceDim" => Some(SurfaceDim),
					"SurfaceBright" => Some(SurfaceBright),

					// Surface Containers
					"SurfaceContainerLowest" => Some(SurfaceContainerLowest),
					"SurfaceContainerLow" => Some(SurfaceContainerLow),
					"SurfaceContainer" => Some(SurfaceContainer),
					"SurfaceContainerHigh" => Some(SurfaceContainerHigh),
					"SurfaceContainerHighest" => Some(SurfaceContainerHighest),
					"SurfaceOutline" => Some(SurfaceOutline),
					"SurfaceOutlineVariant" => Some(SurfaceOutlineVariant),

					// Shadows
					"Shadow" => Some(MonetRole::Shadow),

					// Aliases
					"Background" => Some(MonetRole::SurfaceContainerLowest),

					_ => {
						nimbus_log!("Monet '{}' is not found", target);
						None
					}
				};

				let mut color = match role {
					Some(r) => monet[r],
					None => monet[MonetRole::Primary],
				};

				if alpha < 1.0 {
					color.a *= alpha;
				}
				return Some(color);
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
