/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	BoxConstraints, DrawCtx, LayoutCtx, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::pin::{NodeBindKind, NodePin, PinDirection, draw_pin_icon};


const HEADER_PAD_Y: f32 = 8.0;
const PIN_ROW_HEIGHT: f32 = 26.0;
const PIN_MARGIN_X: f32 = 16.0;
const PIN_LABEL_GAP: f32 = 12.0;
const BOTTOM_PAD: f32 = 10.0;
const PIN_HIT_RADIUS: f32 = 15.0;


pub struct PurplePrintNode {
	pub position: Vec2<f32>,
	pub width: f32,
	pub title: String,
	pub color: Option<DColor>,
	pub back_color: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub inputs: Vec<NodePin>,
	pub outputs: Vec<NodePin>,

	pub(crate) selected: bool,
	pub(crate) connected_inputs: Vec<bool>,
	pub(crate) connected_outputs: Vec<bool>,
	pub(crate) title_height: f32,
	pub(crate) last_bounds: Cell<Rect<f32>>,
}

impl Default for PurplePrintNode {
	fn default() -> Self {
		Self {
			position: Vec2::new(40.0, 40.0),
			width: 180.0,
			title: "PurplePrint Node".to_string(),
			color: Some(DColor::new_color(Color::from_oklch(0.675, 0.138, 65.0))),
			back_color: None,
			corner_radius: None,
			font: None,
			font_size: 14.0,
			inputs: Vec::new(),
			outputs: Vec::new(),
			selected: false,
			connected_inputs: Vec::new(),
			connected_outputs: Vec::new(),
			title_height: 32.0,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl PurplePrintNode {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			..Default::default()
		}
	}

	pub fn builder() -> PurplePrintNodeBuilder {
		PurplePrintNodeBuilder::new()
	}

	pub fn add_input(&mut self, name: impl Into<String>, kind: NodeBindKind) -> usize {
		self.inputs.push(NodePin::new(name, kind));
		self.inputs.len() - 1
	}

	pub fn add_input_typed(
		&mut self,
		name: impl Into<String>,
		kind: NodeBindKind,
		data_type: impl Into<String>,
	) -> usize {
		self.inputs.push(NodePin::new(name, kind).with_data_type(data_type));
		self.inputs.len() - 1
	}

	pub fn add_output(&mut self, name: impl Into<String>, kind: NodeBindKind) -> usize {
		self.outputs.push(NodePin::new(name, kind));
		self.outputs.len() - 1
	}

	pub fn add_output_typed(
		&mut self,
		name: impl Into<String>,
		kind: NodeBindKind,
		data_type: impl Into<String>,
	) -> usize {
		self.outputs.push(NodePin::new(name, kind).with_data_type(data_type));
		self.outputs.len() - 1
	}

	pub fn header_height(&self) -> f32 {
		self.title_height
	}

	pub fn in_header(&self, local_pos: Vec2<f32>) -> bool {
		local_pos.y >= 0.0 && local_pos.y <= self.title_height
	}

	pub fn hit_pin(&self, local_pos: Vec2<f32>) -> Option<(PinDirection, usize)> {
		let max_dist_sq = PIN_HIT_RADIUS * PIN_HIT_RADIUS;
		let mut best: Option<(PinDirection, usize, f32)> = None;

		for (i, pin) in self.inputs.iter().enumerate() {
			let p = pin.local_pos.get();
			let dx = local_pos.x - p.x;
			let dy = local_pos.y - p.y;
			let d2 = dx * dx + dy * dy;
			if d2 <= max_dist_sq && best.is_none_or(|b| d2 < b.2) {
				best = Some((PinDirection::Input, i, d2));
			}
		}

		for (i, pin) in self.outputs.iter().enumerate() {
			let p = pin.local_pos.get();
			let dx = local_pos.x - p.x;
			let dy = local_pos.y - p.y;
			let d2 = dx * dx + dy * dy;
			if d2 <= max_dist_sq && best.is_none_or(|b| d2 < b.2) {
				best = Some((PinDirection::Output, i, d2));
			}
		}

		best.map(|(dir, idx, _)| (dir, idx))
	}

	pub fn pin_global_pos(&self, direction: PinDirection, index: usize) -> Option<Vec2<f32>> {
		let pin = match direction {
			PinDirection::Input => self.inputs.get(index)?,
			PinDirection::Output => self.outputs.get(index)?,
		};
		Some(self.last_bounds.get().min + pin.local_pos.get())
	}

	pub fn pin_kind(&self, direction: PinDirection, index: usize) -> Option<NodeBindKind> {
		let pin = match direction {
			PinDirection::Input => self.inputs.get(index)?,
			PinDirection::Output => self.outputs.get(index)?,
		};
		Some(pin.kind)
	}

	pub fn pin_color(&self, direction: PinDirection, index: usize) -> Option<Color> {
		let pin = match direction {
			PinDirection::Input => self.inputs.get(index)?,
			PinDirection::Output => self.outputs.get(index)?,
		};
		Some(pin.effective_color())
	}

	fn compute_metrics(&mut self, ctx: &LayoutCtx) -> Vec2<f32> {
		let (title_w, title_h) = ctx.measure_text_with_font(&self.title, self.font, self.font_size);
		self.title_height = (title_h + HEADER_PAD_Y * 2.0 + 2.0).max(30.0).ceil();

		let mut max_in_w = 0.0f32;
		for pin in &self.inputs {
			let (w, _) = ctx.measure_text_with_font(&pin.name, self.font, self.font_size);
			max_in_w = max_in_w.max(w);
		}

		let mut max_out_w = 0.0f32;
		for pin in &self.outputs {
			let (w, _) = ctx.measure_text_with_font(&pin.name, self.font, self.font_size);
			max_out_w = max_out_w.max(w);
		}

		let pins_min_w = if !self.inputs.is_empty() && !self.outputs.is_empty() {
			(PIN_MARGIN_X + PIN_LABEL_GAP) * 2.0 + max_in_w + max_out_w + 20.0
		} else if !self.inputs.is_empty() {
			PIN_MARGIN_X * 2.0 + PIN_LABEL_GAP + max_in_w + 12.0
		} else if !self.outputs.is_empty() {
			PIN_MARGIN_X * 2.0 + PIN_LABEL_GAP + max_out_w + 12.0
		} else {
			0.0
		};

		let w = self
			.width
			.max(title_w + 28.0)
			.max(pins_min_w)
			.max(120.0)
			.ceil();

		let rows = self.inputs.len().max(self.outputs.len());
		let body_h = if rows > 0 {
			rows as f32 * PIN_ROW_HEIGHT + BOTTOM_PAD + 6.0
		} else {
			14.0
		};
		let h = (self.title_height + body_h).ceil();

		// Precompute local pin positions so view hit-testing & wire drawing are always exact
		for (i, pin) in self.inputs.iter().enumerate() {
			let py = self.title_height + 6.0 + (i as f32 + 0.5) * PIN_ROW_HEIGHT;
			pin.local_pos.set(Vec2::new(PIN_MARGIN_X, py));
		}
		for (i, pin) in self.outputs.iter().enumerate() {
			let py = self.title_height + 6.0 + (i as f32 + 0.5) * PIN_ROW_HEIGHT;
			pin.local_pos.set(Vec2::new(w - PIN_MARGIN_X, py));
		}

		Vec2::new(w, h)
	}
}

impl Widget for PurplePrintNode {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let size = self.compute_metrics(ctx);
		constraints.constrain(size)
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;

		// Node body card
		let back_col = self.back_color.map(|c| c.get(monet)).unwrap_or_else(|| {
			let mut c = monet[MonetRole::SurfaceContainerLow];
			c.a *= 0.90;
			c
		});
		let border_col = if self.selected {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::SurfaceOutlineVariant]
		};
		let border_w = if self.selected {
			ctx.theme.stroke.thick
		} else {
			ctx.theme.stroke.thin
		};
		let corner_radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);

		let mut shadow_col = monet[MonetRole::Shadow];
		shadow_col.a = (shadow_col.a * 0.40).min(0.36);
		canvas.draw(
			RectShape::new(bounds, back_col)
				.radius(corner_radius)
				.border(border_col, border_w)
				.shadow(shadow_col, Vec2::new(0.0, 4.0), 12.0),
		);

		// Colored title bar
		let header_col = self
			.color
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::Primary]);
		let inset = border_w;
		let header_r = (corner_radius - inset).max(0.0);
		let header_rect = Rect::new(
			bounds.x() + inset,
			bounds.y() + inset,
			(bounds.width() - inset * 2.0).max(0.0),
			(self.title_height - inset).max(0.0),
		);
		canvas.draw(
			RectShape::new(header_rect, header_col)
				.radius4([header_r, header_r, 0.0, 0.0])
				.noise(0.06),
		);

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		// Pick high-contrast title text color based on header luminance
		let lum = 0.2126 * header_col.r + 0.7152 * header_col.g + 0.0722 * header_col.b;
		let title_text_col = if lum > 0.45 {
			Color::new(0.08, 0.08, 0.10, 1.0)
		} else {
			Color::WHITE
		};

		let (_, title_h) = canvas.measure_text(&self.title, self.font_size);
		let title_pos = Vec2::new(
			header_rect.x() + 12.0,
			header_rect.y() + (header_rect.height() - title_h) * 0.5,
		);
		canvas.draw(TextShape::new(
			&self.title,
			font,
			self.font_size,
			title_pos,
			title_text_col,
		));

		// Input pins (left side)
		let label_col = monet[MonetRole::OnSurface];
		for (i, pin) in self.inputs.iter().enumerate() {
			let local = pin.local_pos.get();
			let pos = bounds.min + local;
			let connected = self.connected_inputs.get(i).copied().unwrap_or(false);
			draw_pin_icon(canvas, pos, pin.kind, pin.effective_color_for(monet), connected);

			if !pin.name.is_empty() {
				let (_, lh) = canvas.measure_text(&pin.name, self.font_size);
				canvas.draw(TextShape::new(
					&pin.name,
					font,
					self.font_size,
					Vec2::new(pos.x + PIN_LABEL_GAP, pos.y - lh * 0.5),
					label_col,
				));
			}
		}

		// Output pins (right side)
		for (i, pin) in self.outputs.iter().enumerate() {
			let local = pin.local_pos.get();
			let pos = bounds.min + local;
			let connected = self.connected_outputs.get(i).copied().unwrap_or(false);
			draw_pin_icon(canvas, pos, pin.kind, pin.effective_color_for(monet), connected);

			if !pin.name.is_empty() {
				let (lw, lh) = canvas.measure_text(&pin.name, self.font_size);
				canvas.draw(TextShape::new(
					&pin.name,
					font,
					self.font_size,
					Vec2::new(pos.x - PIN_LABEL_GAP - lw, pos.y - lh * 0.5),
					label_col,
				));
			}
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"title" => self.title = value.expect_str()?.to_string(),
			"color" => self.color = Some(value.expect_dcolor()?),
			"background" => self.back_color = Some(value.expect_dcolor()?),
			"point" => self.position = value.expect_vec2()?,
			"width" => self.width = value.expect_f32()?.max(100.0),
			"size" => {
				if let Some(obj) = value.as_object() {
					if let Some(w) = obj.get("w").or_else(|| obj.get("width")).and_then(|v| v.as_f32()) {
						self.width = w.max(100.0);
					}
				} else if let Some(v) = value.as_vec2() {
					self.width = v.x.max(100.0);
				} else {
					self.width = value.expect_f32()?.max(100.0);
				}
			}
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			"font_size" => self.font_size = value.expect_f32()?,
			"inputs" => {
				let arr = value.expect_array()?;
				self.inputs = arr
					.iter()
					.filter_map(|v| parse_pin_value(v, &ctx.theme.monet))
					.collect();
			}
			"outputs" => {
				let arr = value.expect_array()?;
				self.outputs = arr
					.iter()
					.filter_map(|v| parse_pin_value(v, &ctx.theme.monet))
					.collect();
			}
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}
}

fn parse_pin_value(val: &PropertyValue, monet: &nimbus_theme::Monet) -> Option<NodePin> {
	if let Some(name) = val.as_str() {
		return Some(NodePin::new(name, NodeBindKind::Circle));
	}
	if let Some(obj) = val.as_object() {
		let name = obj
			.get("name")
			.and_then(|v| v.as_str())
			.unwrap_or("");
		let kind = obj
			.get("kind")
			.and_then(|v| v.as_str())
			.and_then(NodeBindKind::parse)
			.unwrap_or(NodeBindKind::Circle);
		let mut pin = NodePin::new(name, kind);
		if let Some(dtype) = obj
			.get("dtype")
			.and_then(|v| v.as_str())
		{
			pin = pin.with_data_type(dtype);
		}
		if let Some(col) = obj.get("color").and_then(|v| v.resolve_color(monet)) {
			pin.color = Some(col);
		}
		return Some(pin);
	}
	None
}


#[derive(Default)]
pub struct PurplePrintNodeBuilder {
	inner: PurplePrintNode,
}

impl PurplePrintNodeBuilder {
	pub fn new() -> Self {
		Self {
			inner: PurplePrintNode::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.title = text.into();
		self
	}

	pub fn position(mut self, x: f32, y: f32) -> Self {
		self.inner.position = Vec2::new(x, y);
		self
	}

	pub fn width(mut self, width: f32) -> Self {
		self.inner.width = width.max(100.0);
		self
	}

	pub fn color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.color = Some(color.into());
		self
	}

	pub fn back_color(mut self, color: impl Into<DColor>) -> Self {
		self.inner.back_color = Some(color.into());
		self
	}

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn font(mut self, font: FontId) -> Self {
		self.inner.font = Some(font);
		self
	}

	pub fn font_size(mut self, size: f32) -> Self {
		self.inner.font_size = size;
		self
	}

	pub fn input(mut self, name: impl Into<String>, kind: NodeBindKind) -> Self {
		self.inner.inputs.push(NodePin::new(name, kind));
		self
	}

	pub fn input_typed(
		mut self,
		name: impl Into<String>,
		kind: NodeBindKind,
		data_type: impl Into<String>,
	) -> Self {
		self.inner.inputs.push(NodePin::new(name, kind).with_data_type(data_type));
		self
	}

	pub fn output(mut self, name: impl Into<String>, kind: NodeBindKind) -> Self {
		self.inner.outputs.push(NodePin::new(name, kind));
		self
	}

	pub fn output_typed(
		mut self,
		name: impl Into<String>,
		kind: NodeBindKind,
		data_type: impl Into<String>,
	) -> Self {
		self.inner.outputs.push(NodePin::new(name, kind).with_data_type(data_type));
		self
	}

	pub fn input_pin(mut self, pin: NodePin) -> Self {
		self.inner.inputs.push(pin);
		self
	}

	pub fn output_pin(mut self, pin: NodePin) -> Self {
		self.inner.outputs.push(pin);
		self
	}

	pub fn build(self) -> PurplePrintNode {
		self.inner
	}
}

impl From<PurplePrintNodeBuilder> for PurplePrintNode {
	fn from(builder: PurplePrintNodeBuilder) -> Self {
		builder.build()
	}
}
