/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
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

				let role = match target {
					// Primary
					"Primary" => Some(MonetRole::Primary),
					"OnPrimary" => Some(MonetRole::OnPrimary),
					"PrimaryContainer" => Some(MonetRole::PrimaryContainer),
					"OnPrimaryContainer" => Some(MonetRole::OnPrimaryContainer),
					"PrimaryFixed" => Some(MonetRole::PrimaryFixed),
					"PrimaryFixedDim" => Some(MonetRole::PrimaryFixedDim),
					"OnPrimaryFixed" => Some(MonetRole::OnPrimaryFixed),
					"OnPrimaryFixedVariant" => Some(MonetRole::OnPrimaryFixedVariant),
					"PrimaryVariant" => Some(MonetRole::PrimaryFixedDim),
					"OnPrimaryVariant" => Some(MonetRole::OnPrimaryFixedVariant),

					// Secondary
					"Secondary" => Some(MonetRole::Secondary),
					"OnSecondary" => Some(MonetRole::OnSecondary),
					"SecondaryContainer" => Some(MonetRole::SecondaryContainer),
					"OnSecondaryContainer" => Some(MonetRole::OnSecondaryContainer),
					"SecondaryFixed" => Some(MonetRole::SecondaryFixed),
					"SecondaryFixedDim" => Some(MonetRole::SecondaryFixedDim),
					"OnSecondaryFixed" => Some(MonetRole::OnSecondaryFixed),
					"OnSecondaryFixedVariant" => Some(MonetRole::OnSecondaryFixedVariant),
					"SecondaryVariant" => Some(MonetRole::SecondaryFixedDim),
					"OnSecondaryVariant" => Some(MonetRole::OnSecondaryFixedVariant),

					// Tertiary
					"Tertiary" => Some(MonetRole::Tertiary),
					"OnTertiary" => Some(MonetRole::OnTertiary),
					"TertiaryContainer" => Some(MonetRole::TertiaryContainer),
					"OnTertiaryContainer" => Some(MonetRole::OnTertiaryContainer),
					"TertiaryFixed" => Some(MonetRole::TertiaryFixed),
					"TertiaryFixedDim" => Some(MonetRole::TertiaryFixedDim),
					"OnTertiaryFixed" => Some(MonetRole::OnTertiaryFixed),
					"OnTertiaryFixedVariant" => Some(MonetRole::OnTertiaryFixedVariant),
					"TertiaryVariant" => Some(MonetRole::TertiaryFixedDim),
					"OnTertiaryVariant" => Some(MonetRole::OnTertiaryFixedVariant),

					// Error
					"Error" => Some(MonetRole::Error),
					"OnError" => Some(MonetRole::OnError),
					"ErrorContainer" => Some(MonetRole::ErrorContainer),
					"OnErrorContainer" => Some(MonetRole::OnErrorContainer),
					"ErrorFixed" => Some(MonetRole::ErrorFixed),
					"ErrorFixedDim" => Some(MonetRole::ErrorFixedDim),
					"OnErrorFixed" => Some(MonetRole::OnErrorFixed),
					"OnErrorFixedVariant" => Some(MonetRole::OnErrorFixedVariant),
					"ErrorVariant" => Some(MonetRole::ErrorFixedDim),
					"OnErrorVariant" => Some(MonetRole::OnErrorFixedVariant),

					// Surface
					"Surface" => Some(MonetRole::Surface),
					"OnSurface" => Some(MonetRole::OnSurface),
					"SurfaceVariant" => Some(MonetRole::SurfaceVariant),
					"OnSurfaceVariant" => Some(MonetRole::OnSurfaceVariant),
					"SurfaceDim" => Some(MonetRole::SurfaceDim),
					"OnSurfaceDim" => Some(MonetRole::OnSurface),
					"SurfaceBright" => Some(MonetRole::SurfaceBright),
					"OnSurfaceBright" => Some(MonetRole::OnSurface),

					// Surface Containers
					"SurfaceContainerLowest" => Some(MonetRole::SurfaceContainerLowest),
					"OnSurfaceContainerLowest" => Some(MonetRole::OnSurface),
					"SurfaceContainerLow" => Some(MonetRole::SurfaceContainerLow),
					"OnSurfaceContainerLow" => Some(MonetRole::OnSurface),
					"SurfaceContainer" => Some(MonetRole::SurfaceContainer),
					"OnSurfaceContainer" => Some(MonetRole::OnSurface),
					"SurfaceContainerHigh" => Some(MonetRole::SurfaceContainerHigh),
					"OnSurfaceContainerHigh" => Some(MonetRole::OnSurface),
					"SurfaceContainerHighest" => Some(MonetRole::SurfaceContainerHighest),
					"OnSurfaceContainerHighest" => Some(MonetRole::OnSurface),

					// Outlines & Shadows
					"Outline" | "SurfaceOutline" => Some(MonetRole::Outline),
					"OutlineVariant" | "SurfaceOutlineVariant" => Some(MonetRole::OutlineVariant),
					"Shadow" => Some(MonetRole::Shadow),

					// Aliases
					"Background" => Some(MonetRole::SurfaceContainerLowest),
					_ => None,
				};

				let mut color = match role {
					Some(r) => monet.get(r),
					None => monet.get(MonetRole::Primary),
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
