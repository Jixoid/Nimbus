/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::any::Any;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontManager, RectShape, TextShape};
use nimbus_theme::{MonetRole, Theme};

use crate::id::ObjectID;
use crate::property::PropertyValue;


pub const DRAG_THRESHOLD_PX: f32 = 6.0;


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum DropEffect {
	#[default]
	None,
	Move,
	Copy,
	Link,
}


#[derive(Clone)]
pub enum DragPreview {
	Label(String),
	Custom(Arc<dyn Fn(&mut Canvas2D, &Theme, Vec2<f32>) + Send + Sync>),
	None,
}

impl fmt::Debug for DragPreview {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Label(s) => f.debug_tuple("Label").field(s).finish(),
			Self::Custom(_) => f.write_str("Custom(<fn>)"),
			Self::None => f.write_str("None"),
		}
	}
}


#[derive(Clone)]
pub struct DragPayload {
	pub kind: String,
	pub data: Arc<dyn Any + Send + Sync>,
	pub value: Option<PropertyValue>,
	pub preview: DragPreview,
}

impl fmt::Debug for DragPayload {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DragPayload")
			.field("kind", &self.kind)
			.field("value", &self.value)
			.field("preview", &self.preview)
			.finish()
	}
}

impl PartialEq for DragPayload {
	fn eq(&self, other: &Self) -> bool {
		self.kind == other.kind && Arc::ptr_eq(&self.data, &other.data)
	}
}

impl DragPayload {
	pub fn new<T: Any + Send + Sync>(kind: impl Into<String>, data: T) -> Self {
		Self {
			kind: kind.into(),
			data: Arc::new(data),
			value: None,
			preview: DragPreview::None,
		}
	}

	pub fn text(kind: impl Into<String>, text: impl Into<String>) -> Self {
		let s = text.into();
		Self {
			kind: kind.into(),
			data: Arc::new(s.clone()),
			value: Some(PropertyValue::String(s.clone())),
			preview: DragPreview::Label(s),
		}
	}

	pub fn files(paths: Vec<PathBuf>) -> Self {
		let label = if paths.len() == 1 {
			paths[0]
				.file_name()
				.map(|n| n.to_string_lossy().into_owned())
				.unwrap_or_else(|| paths[0].to_string_lossy().into_owned())
		} else {
			format!("{} files", paths.len())
		};
		let value = PropertyValue::Array(
			paths
				.iter()
				.map(|p| PropertyValue::String(p.to_string_lossy().into_owned()))
				.collect(),
		);
		Self {
			kind: "os/files".to_string(),
			data: Arc::new(paths),
			value: Some(value),
			preview: DragPreview::Label(label),
		}
	}

	pub fn with_value(mut self, value: PropertyValue) -> Self {
		self.value = Some(value);
		self
	}

	pub fn with_label(mut self, label: impl Into<String>) -> Self {
		self.preview = DragPreview::Label(label.into());
		self
	}

	pub fn with_preview(mut self, preview: DragPreview) -> Self {
		self.preview = preview;
		self
	}

	pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
		self.data.downcast_ref::<T>()
	}

	pub fn label(&self) -> Option<&str> {
		match &self.preview {
			DragPreview::Label(s) => Some(s.as_str()),
			_ => None,
		}
	}
}


#[derive(Clone, Debug)]
pub struct DragSession {
	pub source: Option<ObjectID>,
	pub origin: Vec2<f32>,
	pub pointer: Vec2<f32>,
	pub payload: Arc<DragPayload>,
	pub active: bool,
	pub current_target: Option<ObjectID>,
	pub effect: DropEffect,
}

impl DragSession {
	pub fn draw_preview(&self, canvas: &mut Canvas2D, theme: &Theme) {
		if !self.active {
			return;
		}
		match &self.payload.preview {
			DragPreview::None => {}
			DragPreview::Custom(f) => f(canvas, theme, self.pointer),
			DragPreview::Label(label) => {
				let monet = &theme.monet;
				let font_size = 12.5;
				let fface = FontManager::default_font();
				let (tw, th) = canvas.measure_text(label, font_size);
				let badge_text = match self.effect {
					DropEffect::Copy => Some("+"),
					DropEffect::Link => Some("->"),
					_ => None,
				};
				let extra_w = badge_text
					.map(|b| canvas.measure_text(b, 11.0).0 + 10.0)
					.unwrap_or(0.0);

				let pad_x = 12.0;
				let pad_y = 6.0;
				let pill_w = (tw + pad_x * 2.0 + extra_w).max(56.0);
				let pill_h = (th + pad_y * 2.0).max(26.0);
				let origin = Vec2::new(self.pointer.x + 14.0, self.pointer.y + 14.0);
				let rect = Rect::new(origin.x, origin.y, pill_w, pill_h);

				let (bg, border, fg) = if self.effect != DropEffect::None {
					(
						monet[MonetRole::PrimaryContainer] * 0.95,
						monet[MonetRole::Primary],
						monet[MonetRole::OnPrimaryContainer],
					)
				} else {
					(
						monet[MonetRole::SurfaceContainerHigh] * 0.95,
						monet[MonetRole::PrimaryOutline],
						monet[MonetRole::OnSurface],
					)
				};

				let radius = theme.radius.small;
				canvas.draw(
					RectShape::new(rect, bg)
						.radius(radius)
						.border(border, theme.stroke.thin),
				);
				canvas.draw(TextShape::new(
					label,
					Font::new(fface, font_size),
					font_size,
					Vec2::new(rect.x() + pad_x, rect.y() + (pill_h - th) * 0.5),
					fg,
				));

				if let Some(badge) = badge_text {
					let (bw, bh) = canvas.measure_text(badge, 11.0);
					canvas.draw(TextShape::new(
						badge,
						Font::new(fface, 11.0),
						11.0,
						Vec2::new(rect.max.x - pad_x - bw, rect.y() + (pill_h - bh) * 0.5),
						monet[MonetRole::Primary],
					));
				}
			}
		}
	}
}
