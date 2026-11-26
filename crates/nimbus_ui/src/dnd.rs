/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_core::{
	BoxConstraints, ButtonState, DragPayload, DrawCtx, DropEffect, EventCtx, EventResponse,
	InputEvent, IntoMethod, LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError,
	PropertyValue, Widget,
};
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};


// DragSource

pub struct DragSource {
	pub text: String,
	pub badge: Option<String>,
	pub kind: String,
	pub payload: Option<String>,
	pub accent: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_drag_ended: Option<MethodCallback>,

	is_dragging: bool,
}

impl Default for DragSource {
	fn default() -> Self {
		Self {
			text: "Draggable Item".to_string(),
			badge: None,
			kind: "nimbus/item".to_string(),
			payload: None,
			accent: None,
			corner_radius: None,
			font: None,
			font_size: 13.5,
			on_drag_ended: None,
			is_dragging: false,
		}
	}
}

impl DragSource {
	pub fn new(text: impl Into<String>, kind: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			kind: kind.into(),
			..Default::default()
		}
	}

	pub fn builder() -> DragSourceBuilder {
		DragSourceBuilder::new()
	}
}

impl Widget for DragSource {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let fface = FontManager::default_font();
		let (tw, th) = fface.measure_text(&self.text, self.font_size);
		let badge_w = self
			.badge
			.as_deref()
			.map(|b| fface.measure_text(b, 11.0).0 + 22.0)
			.unwrap_or(0.0);
		let min_w = tw + badge_w + 54.0;
		let w = if constraints.max_width.is_finite() {
			constraints.max_width.max(min_w)
		} else {
			min_w.max(180.0)
		};
		let h = (th + 18.0).max(38.0);
		constraints.constrain(Vec2::new(w, h))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.small);
		let alpha = if self.is_dragging { 0.45 } else { 1.0 };

		let bg = if ctx.hovered && !self.is_dragging {
			monet[MonetRole::SurfaceContainerHighest]
		} else {
			monet[MonetRole::SurfaceContainerHigh]
		} * alpha;

		let border = if ctx.hovered && !self.is_dragging {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::PrimaryOutline] * 0.55
		} * alpha;

		canvas.draw(
			RectShape::new(bounds, bg)
				.radius(radius)
				.border(border, ctx.theme.stroke.thin),
		);

		// 6-dot grip indicator on left
		let dot_color = monet[MonetRole::OnSurfaceVariant] * (0.65 * alpha);
		let gx = bounds.x() + 11.0;
		let gy_mid = bounds.y() + bounds.height() * 0.5;
		for col in 0..2 {
			for row in -1..=1 {
				let dx = gx + col as f32 * 4.5;
				let dy = gy_mid + row as f32 * 4.5 - 1.0;
				canvas.draw(RectShape::new(Rect::new(dx, dy, 2.2, 2.2), dot_color).radius(1.1));
			}
		}

		let mut x = bounds.x() + 26.0;

		// Optional accent color dot
		if let Some(acc) = self.accent {
			let c = acc.get(monet) * alpha;
			canvas.draw(
				RectShape::new(Rect::new(x, gy_mid - 4.0, 8.0, 8.0), c).radius(4.0),
			);
			x += 14.0;
		}

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let (_, th) = canvas.measure_text(&self.text, self.font_size);
		canvas.draw(TextShape::new(
			&self.text,
			font,
			self.font_size,
			Vec2::new(x, bounds.y() + (bounds.height() - th) * 0.5),
			monet[MonetRole::OnSurface] * alpha,
		));

		// Optional badge pill on right
		if let Some(badge) = &self.badge {
			let bsize = 11.0;
			let (bw, bh) = canvas.measure_text(badge, bsize);
			let pill_w = bw + 14.0;
			let pill_h = (bh + 6.0).max(20.0);
			let pill = Rect::new(
				bounds.max.x - pill_w - 10.0,
				bounds.y() + (bounds.height() - pill_h) * 0.5,
				pill_w,
				pill_h,
			);
			let pill_bg = self
				.accent
				.map(|a| a.get(monet) * (0.18 * alpha))
				.unwrap_or_else(|| monet[MonetRole::SecondaryContainer] * alpha);
			let pill_fg = self
				.accent
				.map(|a| a.get(monet) * alpha)
				.unwrap_or_else(|| monet[MonetRole::OnSecondaryContainer] * alpha);
			canvas.draw(RectShape::new(pill, pill_bg).radius(ctx.theme.radius.xsmall));
			canvas.draw(TextShape::new(
				badge,
				Font::new(fface, bsize),
				bsize,
				Vec2::new(pill.x() + 7.0, pill.y() + (pill_h - bh) * 0.5),
				pill_fg,
			));
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				let data_text = self.payload.clone().unwrap_or_else(|| self.text.clone());
				let preview_label = match &self.badge {
					Some(b) => format!("{} [{}]", self.text, b),
					None => self.text.clone(),
				};
				let payload = DragPayload::text(self.kind.clone(), data_text)
					.with_label(preview_label);
				ctx.begin_drag(*position, payload);
				self.is_dragging = true;
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if self.is_dragging {
					self.is_dragging = false;
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::DragEnded { accepted, .. } => {
				self.is_dragging = false;
				if let Some(cb) = &self.on_drag_ended {
					cb.call1(*accepted);
				}
				ctx.request_redraw();
				EventResponse::Handled
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"text" => self.text = value.expect_str()?.to_string(),
			"badge" => self.badge = Some(value.expect_str()?.to_string()),
			"kind" => self.kind = value.expect_str()?.to_string(),
			"payload" => self.payload = Some(value.expect_str()?.to_string()),
			"color" => self.accent = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			"font_size" => self.font_size = value.expect_f32()?.clamp(9.0, 28.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_drag_ended")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if method_name == "on_drag_ended" {
			self.on_drag_ended = Some(callback);
			true
		} else {
			false
		}
	}
}


#[derive(Default)]
pub struct DragSourceBuilder {
	inner: DragSource,
}

impl DragSourceBuilder {
	pub fn new() -> Self {
		Self {
			inner: DragSource::default(),
		}
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.text = text.into();
		self
	}

	pub fn badge(mut self, badge: impl Into<String>) -> Self {
		self.inner.badge = Some(badge.into());
		self
	}

	pub fn kind(mut self, kind: impl Into<String>) -> Self {
		self.inner.kind = kind.into();
		self
	}

	pub fn payload(mut self, payload: impl Into<String>) -> Self {
		self.inner.payload = Some(payload.into());
		self
	}

	pub fn accent(mut self, color: impl Into<DColor>) -> Self {
		self.inner.accent = Some(color.into());
		self
	}

	pub fn on_drag_ended<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_drag_ended = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> DragSource {
		self.inner
	}
}

impl From<DragSourceBuilder> for DragSource {
	fn from(builder: DragSourceBuilder) -> Self {
		builder.build()
	}
}


// DropZone

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum DropHoverState {
	#[default]
	Idle,
	Accepted,
	Rejected,
}

pub struct DropZone {
	pub title: String,
	pub hint: String,
	pub accept: Vec<String>,
	pub effect: DropEffect,
	pub items: Vec<String>,
	pub max_items: usize,
	pub auto_collect: bool,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_drop: Option<MethodCallback>,

	hover_state: DropHoverState,
}

impl Default for DropZone {
	fn default() -> Self {
		Self {
			title: "Drop Zone".to_string(),
			hint: "Drag and drop items here".to_string(),
			accept: vec!["*".to_string()],
			effect: DropEffect::Move,
			items: Vec::new(),
			max_items: 8,
			auto_collect: true,
			background: None,
			corner_radius: None,
			font: None,
			font_size: 13.0,
			on_drop: None,
			hover_state: DropHoverState::Idle,
		}
	}
}

impl DropZone {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
			..Default::default()
		}
	}

	pub fn builder() -> DropZoneBuilder {
		DropZoneBuilder::new()
	}

	pub fn clear(&mut self) {
		self.items.clear();
	}

	pub fn accepts_kind(&self, kind: &str) -> bool {
		if self.accept.is_empty() {
			return true;
		}
		self.accept
			.iter()
			.any(|pattern| pattern == "*" || pattern == kind)
	}

	fn extract_payload_text(payload: &DragPayload) -> String {
		if let Some(val) = &payload.value {
			match val {
				PropertyValue::String(s) => return s.clone(),
				PropertyValue::Array(arr) => {
					let parts: Vec<String> = arr
						.iter()
						.filter_map(|v| v.as_str().map(|s| s.to_string()))
						.collect();
					if !parts.is_empty() {
						return parts.join(", ");
					}
				}
				_ => {}
			}
		}
		payload
			.label()
			.map(|s| s.to_string())
			.unwrap_or_else(|| payload.kind.clone())
	}
}

impl Widget for DropZone {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let w = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			280.0
		};
		let content_h = if self.items.is_empty() {
			96.0
		} else {
			(54.0 + self.items.len() as f32 * 28.0).max(96.0)
		};
		let h = if constraints.max_height.is_finite() && constraints.min_height > 0.0 {
			constraints.max_height.max(content_h)
		} else {
			content_h
		};
		constraints.constrain(Vec2::new(w, h))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		let monet = &ctx.theme.monet;
		let bounds = ctx.bounds;
		let radius = self.corner_radius.unwrap_or(ctx.theme.radius.medium);

		let (bg, border, border_w) = match self.hover_state {
			DropHoverState::Accepted => (
				monet[MonetRole::PrimaryContainer] * 0.35,
				monet[MonetRole::Primary],
				ctx.theme.stroke.thick,
			),
			DropHoverState::Rejected => (
				monet[MonetRole::ErrorContainer] * 0.35,
				monet[MonetRole::Error],
				ctx.theme.stroke.thick,
			),
			DropHoverState::Idle => (
				self.background
					.map(|c| c.get(monet))
					.unwrap_or(monet[MonetRole::SurfaceContainer]),
				monet[MonetRole::PrimaryOutline] * 0.55,
				ctx.theme.stroke.thin,
			),
		};

		canvas.draw(
			RectShape::new(bounds, bg)
				.radius(radius)
				.border(border, border_w),
		);

		let fface = FontManager::default_font();
		let title_font = match self.font {
			Some(id) => Font::with_id(id, fface, 14.0),
			None => Font::new(fface, 14.0),
		};
		let title_color = match self.hover_state {
			DropHoverState::Accepted => monet[MonetRole::Primary],
			DropHoverState::Rejected => monet[MonetRole::Error],
			DropHoverState::Idle => monet[MonetRole::OnSurface],
		};
		canvas.draw(TextShape::new(
			&self.title,
			title_font,
			14.0,
			Vec2::new(bounds.x() + 14.0, bounds.y() + 12.0),
			title_color,
		));

		// Accept filter badge in top-right
		let filter_text = self.accept.join(", ");
		let (fw, fh) = canvas.measure_text(&filter_text, 10.5);
		let badge_w = fw + 12.0;
		let badge_h = (fh + 5.0).max(18.0);
		let badge_rect = Rect::new(
			bounds.max.x - badge_w - 12.0,
			bounds.y() + 11.0,
			badge_w,
			badge_h,
		);
		let (badge_bg, badge_fg) = match self.hover_state {
			DropHoverState::Accepted => (
				monet[MonetRole::Primary] * 0.18,
				monet[MonetRole::Primary],
			),
			DropHoverState::Rejected => (
				monet[MonetRole::Error] * 0.18,
				monet[MonetRole::Error],
			),
			DropHoverState::Idle => (
				monet[MonetRole::SurfaceContainerHighest],
				monet[MonetRole::OnSurfaceVariant],
			),
		};
		canvas.draw(RectShape::new(badge_rect, badge_bg).radius(ctx.theme.radius.xsmall));
		canvas.draw(TextShape::new(
			&filter_text,
			Font::new(fface, 10.5),
			10.5,
			Vec2::new(badge_rect.x() + 6.0, badge_rect.y() + (badge_h - fh) * 0.5),
			badge_fg,
		));

		canvas.push_clip(Rect::new(
			bounds.x() + 10.0,
			bounds.y() + 36.0,
			(bounds.width() - 20.0).max(0.0),
			(bounds.height() - 44.0).max(0.0),
		));

		if self.items.is_empty() {
			let msg = match self.hover_state {
				DropHoverState::Accepted => "Release to drop here",
				DropHoverState::Rejected => "Incompatible item type",
				DropHoverState::Idle => self.hint.as_str(),
			};
			let (mw, mh) = canvas.measure_text(msg, self.font_size);
			let msg_color = match self.hover_state {
				DropHoverState::Accepted => monet[MonetRole::Primary],
				DropHoverState::Rejected => monet[MonetRole::Error],
				DropHoverState::Idle => monet[MonetRole::OnSurfaceVariant],
			};
			canvas.draw(TextShape::new(
				msg,
				Font::new(fface, self.font_size),
				self.font_size,
				Vec2::new(
					bounds.x() + (bounds.width() - mw) * 0.5,
					bounds.y() + 36.0 + ((bounds.height() - 46.0) - mh) * 0.5,
				),
				msg_color,
			));
		} else {
			let mut y = bounds.y() + 40.0;
			let row_w = (bounds.width() - 28.0).max(0.0);
			let row_h = 24.0;
			for item in &self.items {
				let row = Rect::new(bounds.x() + 14.0, y, row_w, row_h);
				canvas.draw(
					RectShape::new(row, monet[MonetRole::SurfaceContainerHigh])
						.radius(ctx.theme.radius.xsmall),
				);
				let (_, th) = canvas.measure_text(item, 12.0);
				canvas.draw(TextShape::new(
					item,
					Font::new(fface, 12.0),
					12.0,
					Vec2::new(row.x() + 10.0, row.y() + (row_h - th) * 0.5),
					monet[MonetRole::OnSurface],
				));
				y += row_h + 4.0;
			}
		}

		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::DragEnter { payload, .. } | InputEvent::DragOver { payload, .. } => {
				let next = if self.accepts_kind(&payload.kind) {
					ctx.set_drop_effect(self.effect);
					DropHoverState::Accepted
				} else {
					ctx.set_drop_effect(DropEffect::None);
					DropHoverState::Rejected
				};
				if self.hover_state != next {
					self.hover_state = next;
					ctx.request_redraw();
				}
				EventResponse::Handled
			}

			InputEvent::DragLeave => {
				if self.hover_state != DropHoverState::Idle {
					self.hover_state = DropHoverState::Idle;
					ctx.request_redraw();
				}
				EventResponse::Handled
			}

			InputEvent::Drop { payload, .. } => {
				self.hover_state = DropHoverState::Idle;
				if !self.accepts_kind(&payload.kind) {
					ctx.request_redraw();
					return EventResponse::Ignored;
				}
				let text = Self::extract_payload_text(payload);
				if self.auto_collect {
					let display = payload
						.label()
						.map(|s| s.to_string())
						.unwrap_or_else(|| text.clone());
					self.items.push(display);
					while self.items.len() > self.max_items {
						self.items.remove(0);
					}
					ctx.request_layout();
				}
				if let Some(cb) = &self.on_drop {
					cb.call2(payload.kind.clone(), text);
				}
				ctx.request_redraw();
				EventResponse::Handled
			}

			_ => EventResponse::Ignored,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"title" => self.title = value.expect_str()?.to_string(),
			"hint" => self.hint = value.expect_str()?.to_string(),
			"accept" => {
				if let Some(s) = value.as_str() {
					self.accept = s
						.split(',')
						.map(|part| part.trim().to_string())
						.filter(|part| !part.is_empty())
						.collect();
				} else if let Some(arr) = value.as_array() {
					self.accept = arr
						.iter()
						.filter_map(|v| v.as_str().map(|s| s.to_string()))
						.collect();
				} else {
					return Err(PropertyError::InvalidValue {
						expected: "string or array of strings",
					});
				}
			}
			"effect" => {
				self.effect = match value.expect_str()?.to_ascii_lowercase().as_str() {
					"copy" => DropEffect::Copy,
					"link" => DropEffect::Link,
					"none" => DropEffect::None,
					_ => DropEffect::Move,
				};
			}
			"max_items" => self.max_items = (value.expect_f64()?.round() as usize).max(1),
			"auto_collect" => self.auto_collect = value.expect_bool()?,
			"color" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			"font_size" => self.font_size = value.expect_f32()?.clamp(9.0, 24.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_drop")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if method_name == "on_drop" {
			self.on_drop = Some(callback);
			true
		} else {
			false
		}
	}
}


#[derive(Default)]
pub struct DropZoneBuilder {
	inner: DropZone,
}

impl DropZoneBuilder {
	pub fn new() -> Self {
		Self {
			inner: DropZone::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn hint(mut self, hint: impl Into<String>) -> Self {
		self.inner.hint = hint.into();
		self
	}

	pub fn accept(mut self, kinds: impl IntoIterator<Item = impl Into<String>>) -> Self {
		self.inner.accept = kinds.into_iter().map(Into::into).collect();
		self
	}

	pub fn effect(mut self, effect: DropEffect) -> Self {
		self.inner.effect = effect;
		self
	}

	pub fn max_items(mut self, max_items: usize) -> Self {
		self.inner.max_items = max_items.max(1);
		self
	}

	pub fn auto_collect(mut self, auto_collect: bool) -> Self {
		self.inner.auto_collect = auto_collect;
		self
	}

	pub fn on_drop<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_drop = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> DropZone {
		self.inner
	}
}

impl From<DropZoneBuilder> for DropZone {
	fn from(builder: DropZoneBuilder) -> Self {
		builder.build()
	}
}
