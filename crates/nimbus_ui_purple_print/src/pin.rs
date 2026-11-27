/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::ObjectID;
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, CurveShape, RectShape};
use nimbus_theme::{Monet, MonetRole};


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeBindKind {
	#[default]
	Flow,
	Circle,
	Square,
	Diamond,
	Array,
}

impl NodeBindKind {
	pub fn parse(s: &str) -> Option<Self> {
		match s.trim().to_ascii_lowercase().as_str() {
			"flow" => Some(Self::Flow),
			"circle" => Some(Self::Circle),
			"square" => Some(Self::Square),
			"diamond" => Some(Self::Diamond),
			"array" => Some(Self::Array),
			_ => None,
		}
	}

	pub fn default_color(self) -> Color {
		match self {
			Self::Flow => Color::WHITE,
			Self::Circle => Color::from_oklch(0.76, 0.14, 150.0),
			Self::Square => Color::from_oklch(0.74, 0.14, 240.0),
			Self::Diamond => Color::from_oklch(0.74, 0.15, 310.0),
			Self::Array => Color::from_oklch(0.78, 0.14, 70.0),
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PinDirection {
	Input,
	Output,
}


#[derive(Clone, Debug)]
pub struct NodePin {
	pub name: String,
	pub kind: NodeBindKind,
	pub data_type: Option<String>,
	pub color: Option<Color>,
	pub(crate) local_pos: Cell<Vec2<f32>>,
}

impl NodePin {
	pub fn new(name: impl Into<String>, kind: NodeBindKind) -> Self {
		Self {
			name: name.into(),
			kind,
			data_type: None,
			color: None,
			local_pos: Cell::new(Vec2::ZERO),
		}
	}

	pub fn with_data_type(mut self, data_type: impl Into<String>) -> Self {
		let s = data_type.into();
		self.data_type = if s.is_empty() { None } else { Some(s) };
		self
	}

	pub fn with_color(mut self, color: Color) -> Self {
		self.color = Some(color);
		self
	}

	pub fn effective_color(&self) -> Color {
		self.color.unwrap_or_else(|| self.kind.default_color())
	}

	pub fn effective_color_for(&self, monet: &Monet) -> Color {
		if let Some(c) = self.color {
			return c;
		}
		match self.kind {
			NodeBindKind::Flow => monet[MonetRole::OnSurface],
			_ => self.kind.default_color(),
		}
	}

	pub fn uses_theme_flow_color(&self) -> bool {
		self.color.is_none() && self.kind == NodeBindKind::Flow
	}

	pub fn local_pos(&self) -> Vec2<f32> {
		self.local_pos.get()
	}

	/// Checks whether two optional data type tags are compatible.
	/// `None`, `"any"`, or `"*"` acts as a wildcard matching any type.
	pub fn is_compatible_types(a: Option<&str>, b: Option<&str>) -> bool {
		match (a, b) {
			(None, _) | (_, None) => true,
			(Some(ta), Some(tb)) => {
				let ta = ta.trim();
				let tb = tb.trim();
				ta.is_empty()
					|| tb.is_empty()
					|| ta == "*"
					|| tb == "*"
					|| ta.eq_ignore_ascii_case("any")
					|| tb.eq_ignore_ascii_case("any")
					|| ta.eq_ignore_ascii_case(tb)
			}
		}
	}
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinRef {
	pub node_id: ObjectID,
	pub direction: PinDirection,
	pub index: usize,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WireConnection {
	pub from_node: ObjectID,
	pub from_pin: usize,
	pub to_node: ObjectID,
	pub to_pin: usize,
}


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum WireState {
	#[default]
	Normal,
	Invalid,
	NoSuitable,
	Cycle,
	Valid,
}


pub(crate) fn draw_line_segment(
	canvas: &mut Canvas2D,
	p0: Vec2<f32>,
	p1: Vec2<f32>,
	thickness: f32,
	color: Color,
) {
	canvas.draw(CurveShape::line(p0, p1, thickness, color));
}

pub(crate) fn draw_pin_icon(
	canvas: &mut Canvas2D,
	center: Vec2<f32>,
	kind: NodeBindKind,
	color: Color,
	connected: bool,
) {
	let stroke = 1.8;
	match kind {
		NodeBindKind::Flow => {
			// Blueprint execution pentagon arrow pointing right
			let pts = [
				Vec2::new(center.x - 5.5, center.y - 6.0),
				Vec2::new(center.x + 1.0, center.y - 6.0),
				Vec2::new(center.x + 6.0, center.y),
				Vec2::new(center.x + 1.0, center.y + 6.0),
				Vec2::new(center.x - 5.5, center.y + 6.0),
			];
			if connected {
				// Fill interior with horizontal strips
				for row in -5..=5 {
					let dy = row as f32;
					let right_x = center.x + 1.0 + (1.0 - dy.abs() / 6.0) * 4.8;
					let left_x = center.x - 5.0;
					canvas.draw(RectShape::new(
						Rect::new(left_x, center.y + dy - 0.6, (right_x - left_x).max(0.0), 1.3),
						color,
					));
				}
			}
			for i in 0..pts.len() {
				let a = pts[i];
				let b = pts[(i + 1) % pts.len()];
				draw_line_segment(canvas, a, b, stroke, color);
			}
		}

		NodeBindKind::Circle => {
			let d = 12.0;
			let rect = Rect::new(center.x - d * 0.5, center.y - d * 0.5, d, d);
			if connected {
				canvas.draw(RectShape::new(rect, color).radius(d * 0.5));
			} else {
				canvas.draw(
					RectShape::new(rect, Color::TRANSPARENT)
						.radius(d * 0.5)
						.border(color, stroke),
				);
			}
		}

		NodeBindKind::Square => {
			let d = 11.0;
			let rect = Rect::new(center.x - d * 0.5, center.y - d * 0.5, d, d);
			if connected {
				canvas.draw(RectShape::new(rect, color).radius(2.5));
			} else {
				canvas.draw(
					RectShape::new(rect, Color::TRANSPARENT)
						.radius(2.5)
						.border(color, stroke),
				);
			}
		}

		NodeBindKind::Diamond => {
			let h = 6.0;
			let pts = [
				Vec2::new(center.x, center.y - h),
				Vec2::new(center.x + h, center.y),
				Vec2::new(center.x, center.y + h),
				Vec2::new(center.x - h, center.y),
			];
			if connected {
				for row in -5..=5 {
					let dy = row as f32;
					let half_w = (h - dy.abs()).max(0.0);
					canvas.draw(RectShape::new(
						Rect::new(center.x - half_w, center.y + dy - 0.6, half_w * 2.0, 1.3),
						color,
					));
				}
			}
			for i in 0..4 {
				draw_line_segment(canvas, pts[i], pts[(i + 1) % 4], stroke, color);
			}
		}

		NodeBindKind::Array => {
			let dot = if connected { 3.0 } else { 2.4 };
			let r = dot * 0.5;
			for gy in [-4.5f32, 0.0, 4.5] {
				for gx in [-4.5f32, 0.0, 4.5] {
					canvas.draw(
						RectShape::new(
							Rect::new(center.x + gx - r, center.y + gy - r, dot, dot),
							color,
						)
						.radius(r),
					);
				}
			}
		}
	}
}

pub(crate) fn draw_bezier_wire(
	canvas: &mut Canvas2D,
	start: Vec2<f32>,
	end: Vec2<f32>,
	thickness: f32,
	color: Color,
) {
	let dx = (end.x - start.x).abs();
	let tangent = (dx * 0.5).clamp(40.0, 240.0);

	let p0 = start;
	let p1 = Vec2::new(start.x + tangent, start.y);
	let p2 = Vec2::new(end.x - tangent, end.y);
	let p3 = end;

	canvas.draw(CurveShape::cubic(p0, p1, p2, p3, thickness, color));
}
