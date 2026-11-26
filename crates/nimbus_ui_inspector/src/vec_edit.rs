/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::time::Instant;

use nimbus_core::{
	BoxConstraints, ButtonState, Cursor, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2, Vec3, Vec4};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub(crate) const AXIS_COLORS: [Color; 4] = [
	Color::new(0.92, 0.34, 0.36, 1.0), // X - Red
	Color::new(0.28, 0.78, 0.44, 1.0), // Y - Green
	Color::new(0.32, 0.60, 0.96, 1.0), // Z - Blue
	Color::new(0.72, 0.46, 0.95, 1.0), // W - Purple
];
pub(crate) const AXIS_LABELS: [&str; 4] = ["X", "Y", "Z", "W"];
const AXIS_LETTER_W: f32 = 22.0;
const DRAG_THRESHOLD: f32 = 4.0;


#[derive(Clone, Debug)]
struct ScrubState {
	axis: usize,
	start_x: f32,
	start_val: f32,
	dragging: bool,
}


pub struct VecEdit<const N: usize> {
	pub values: [f32; N],
	pub default_values: [f32; N],
	pub step: f32,
	pub decimals: usize,
	pub suffix: String,
	pub min: Option<f32>,
	pub max: Option<f32>,
	pub uniform: bool,
	pub row_height: f32,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub on_changed: Option<MethodCallback>,

	hovered_axis: Option<usize>,
	editing_axis: Option<usize>,
	edit_buffer: String,
	cursor: usize,
	selection_anchor: Option<usize>,
	scrub: Option<ScrubState>,
	last_click: Option<(Instant, usize)>,
	last_bounds: Cell<Rect<f32>>,
}

impl<const N: usize> Default for VecEdit<N> {
	fn default() -> Self {
		Self {
			values: [0.0; N],
			default_values: [0.0; N],
			step: 0.05,
			decimals: 2,
			suffix: String::new(),
			min: None,
			max: None,
			uniform: false,
			row_height: 32.0,
			font: None,
			font_size: 13.0,
			on_changed: None,
			hovered_axis: None,
			editing_axis: None,
			edit_buffer: String::new(),
			cursor: 0,
			selection_anchor: None,
			scrub: None,
			last_click: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl<const N: usize> VecEdit<N> {
	pub fn new(values: [f32; N]) -> Self {
		Self {
			values,
			default_values: values,
			..Default::default()
		}
	}

	pub fn set_values(&mut self, values: [f32; N]) {
		for i in 0..N {
			self.values[i] = self.clamp_val(values[i]);
		}
	}

	pub fn is_editing(&self) -> bool {
		self.editing_axis.is_some()
	}

	/// Commits any active text edit and clears editing/hover/scrub state.
	pub fn blur(&mut self) {
		self.commit_edit();
		self.scrub = None;
		self.hovered_axis = None;
	}

	fn clamp_val(&self, v: f32) -> f32 {
		let mut out = v;
		if let Some(lo) = self.min {
			out = out.max(lo);
		}
		if let Some(hi) = self.max {
			out = out.min(hi);
		}
		out
	}

	fn emit_changed(&self) {
		if let Some(cb) = &self.on_changed {
			let args: Vec<PropertyValue> = self
				.values
				.iter()
				.map(|&v| PropertyValue::Number(v as f64))
				.collect();
			cb.call(&args);
		}
	}

	fn segment_rects(&self, bounds: Rect<f32>) -> [Rect<f32>; N] {
		let seg_w = (bounds.width() / N.max(1) as f32).max(24.0);
		let mut rects = [Rect::ZERO; N];
		for (i, r) in rects.iter_mut().enumerate() {
			let x0 = bounds.x() + i as f32 * seg_w;
			let w = if i + 1 == N {
				(bounds.max.x - x0).max(0.0)
			} else {
				seg_w
			};
			*r = Rect::new(x0, bounds.y(), w, bounds.height());
		}
		rects
	}

	fn begin_edit(&mut self, axis: usize) {
		self.editing_axis = Some(axis);
		self.edit_buffer = format!("{:.*}", self.decimals, self.values[axis]);
		let len = self.edit_buffer.chars().count();
		self.cursor = len;
		self.selection_anchor = Some(0);
	}

	fn commit_edit(&mut self) {
		if let Some(axis) = self.editing_axis.take() {
			if let Ok(parsed) = self.edit_buffer.trim().parse::<f32>() {
				let clamped = self.clamp_val(parsed);
				if self.uniform {
					self.values = [clamped; N];
				} else {
					self.values[axis] = clamped;
				}
				self.emit_changed();
			}
			self.edit_buffer.clear();
			self.cursor = 0;
			self.selection_anchor = None;
		}
	}

	fn cancel_edit(&mut self) {
		self.editing_axis = None;
		self.edit_buffer.clear();
		self.cursor = 0;
		self.selection_anchor = None;
	}

	fn char_to_byte(s: &str, char_idx: usize) -> usize {
		s.char_indices()
			.nth(char_idx)
			.map(|(b, _)| b)
			.unwrap_or(s.len())
	}

	fn selection_range(&self) -> Option<(usize, usize)> {
		let anchor = self.selection_anchor?;
		if anchor == self.cursor {
			None
		} else {
			Some((anchor.min(self.cursor), anchor.max(self.cursor)))
		}
	}

	fn delete_selection(&mut self) -> bool {
		if let Some((start, end)) = self.selection_range() {
			let b0 = Self::char_to_byte(&self.edit_buffer, start);
			let b1 = Self::char_to_byte(&self.edit_buffer, end);
			self.edit_buffer.replace_range(b0..b1, "");
			self.cursor = start;
			self.selection_anchor = None;
			true
		} else {
			false
		}
	}

	fn format_value(&self, v: f32) -> String {
		if self.suffix.is_empty() {
			format!("{:.*}", self.decimals, v)
		} else {
			format!("{:.*}{}", self.decimals, v, self.suffix)
		}
	}
}

impl<const N: usize> Widget for VecEdit<N> {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let w = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			240.0
		};
		constraints.constrain(Vec2::new(w, self.row_height))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 4.0 || bounds.height() <= 4.0 {
			return;
		}

		let theme = ctx.theme;
		let monet = &theme.monet;
		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		let axis_font = match self.font {
			Some(id) => Font::with_id(id, fface, 12.0),
			None => Font::new(fface, 12.0),
		};

		let r = theme.radius.small.min(bounds.height() * 0.4);
		let is_active = self.editing_axis.is_some()
			|| self.scrub.as_ref().is_some_and(|s| s.dragging);
		let is_hovered = self.hovered_axis.is_some();

		let border_col = if is_active {
			monet[MonetRole::Primary]
		} else if is_hovered {
			monet[MonetRole::SurfaceOutline]
		} else {
			monet[MonetRole::PrimaryOutline]
		};
		let border_w = if is_active {
			theme.stroke.thick
		} else {
			theme.stroke.thin.max(1.0)
		};

		// Single unified Godot-style card container
		canvas.draw(
			RectShape::new(bounds, monet[MonetRole::SurfaceContainerLow])
				.radius(r)
				.border(border_col, border_w),
		);

		let segments = self.segment_rects(bounds);
		let mut divider_col = monet[MonetRole::SurfaceOutlineVariant];
		divider_col.a = 0.65;

		for (i, seg) in segments.iter().enumerate() {
			let seg_editing = self.editing_axis == Some(i);
			let seg_scrubbing = self.scrub.as_ref().is_some_and(|s| s.axis == i && s.dragging);
			let seg_hovered = self.hovered_axis == Some(i);

			// Subtle per-segment highlight inside the unified card
			if seg_editing || seg_scrubbing || seg_hovered {
				let inner = Rect::new(
					seg.x() + if i == 0 { border_w } else { 0.5 },
					seg.y() + border_w,
					(seg.width()
						- if i == 0 { border_w } else { 0.5 }
						- if i + 1 == N { border_w } else { 0.5 })
					.max(0.0),
					(seg.height() - border_w * 2.0).max(0.0),
				);
				let inner_r = (r - border_w).max(0.0);
				let corners = [
					if i == 0 { inner_r } else { 0.0 },
					if i + 1 == N { inner_r } else { 0.0 },
					if i + 1 == N { inner_r } else { 0.0 },
					if i == 0 { inner_r } else { 0.0 },
				];
				let mut hl = if seg_editing || seg_scrubbing {
					monet[MonetRole::Primary]
				} else {
					monet[MonetRole::OnSurface]
				};
				hl.a = if seg_editing || seg_scrubbing { 0.10 } else { 0.05 };
				canvas.draw(RectShape::new(inner, hl).radius4(corners));
			}

			// Internal vertical divider between segments
			if i > 0 {
				canvas.draw(RectShape::new(
					Rect::new(seg.x() - 0.5, seg.y() + 5.0, 1.0, (seg.height() - 10.0).max(0.0)),
					divider_col,
				));
			}

			// Colored axis letter only (no background block, no accent bar)
			let axis_col = AXIS_COLORS[i % AXIS_COLORS.len()];
			let lbl = AXIS_LABELS[i % AXIS_LABELS.len()];
			let letter_w = AXIS_LETTER_W.min(seg.width() * 0.30);
			let (lw, lh) = canvas.measure_text(lbl, 12.0);
			canvas.draw(TextShape::new(
				lbl,
				axis_font,
				12.0,
				Vec2::new(seg.x() + 6.0 + (letter_w - 6.0 - lw).max(0.0) * 0.5, seg.center().y - lh * 0.5),
				axis_col,
			));

			// Value text + selection + accurate caret
			let text_area = Rect::new(
				seg.x() + letter_w + 2.0,
				seg.y() + 2.0,
				(seg.width() - letter_w - 8.0).max(0.0),
				(seg.height() - 4.0).max(0.0),
			);
			let display = if seg_editing {
				self.edit_buffer.clone()
			} else {
				self.format_value(self.values[i])
			};

			canvas.push_clip(text_area);
			let (_, th) = canvas.measure_text(&display, self.font_size);
			let text_pos = Vec2::new(text_area.x(), text_area.center().y - th * 0.5);

			if seg_editing {
				if let Some((s0, s1)) = self.selection_range() {
					let b0 = Self::char_to_byte(&self.edit_buffer, s0);
					let b1 = Self::char_to_byte(&self.edit_buffer, s1);
					let (x0_off, _) = canvas.measure_text(&self.edit_buffer[..b0], self.font_size);
					let (x1_off, _) = canvas.measure_text(&self.edit_buffer[..b1], self.font_size);
					let sel_h = (th + 4.0).min(text_area.height() - 2.0).max(10.0);
					let sel_rect = Rect::new(
						text_pos.x + x0_off,
						text_area.center().y - sel_h * 0.5,
						(x1_off - x0_off).max(2.0),
						sel_h,
					);
					canvas.draw(
						RectShape::new(sel_rect, monet[MonetRole::PrimaryContainer])
							.radius(theme.radius.xsmall.min(3.0)),
					);
				}
			}

			canvas.draw(TextShape::new(
				&display,
				font,
				self.font_size,
				text_pos,
				monet[MonetRole::OnSurface],
			));

			if seg_editing {
				let cb = Self::char_to_byte(&self.edit_buffer, self.cursor);
				let (cx_off, _) = canvas.measure_text(&self.edit_buffer[..cb], self.font_size);
				let caret_x = (text_pos.x + cx_off).min(text_area.max.x - 1.5);
				let caret_h = (th + 2.0).min(text_area.height() - 4.0).max(10.0);
				canvas.draw(RectShape::new(
					Rect::new(caret_x, text_area.center().y - caret_h * 0.5, 1.5, caret_h),
					monet[MonetRole::Primary],
				));
			}

			canvas.pop_clip();
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let segments = self.segment_rects(bounds);

		match event {
			InputEvent::PointerMoved { position } => {
				if let Some(scrub) = &mut self.scrub {
					let dx = position.x - scrub.start_x;
					let started_drag = if !scrub.dragging && dx.abs() >= DRAG_THRESHOLD {
						scrub.dragging = true;
						true
					} else {
						false
					};
					let dragging = scrub.dragging;
					let axis = scrub.axis;
					let start_val = scrub.start_val;
					if started_drag {
						self.cancel_edit();
					}
					if dragging {
						let sign = dx.signum();
						let mag = dx.abs();
						let accel = 1.0 + (mag / 140.0).powf(1.35);
						let delta = sign * mag * accel * self.step * 0.25;
						let new_val = self.clamp_val(start_val + delta);
						if self.uniform {
							let ratio_delta = new_val - self.values[axis];
							let (lo, hi) = (self.min, self.max);
							for v in &mut self.values {
								let mut nv = *v + ratio_delta;
								if let Some(min_v) = lo {
									nv = nv.max(min_v);
								}
								if let Some(max_v) = hi {
									nv = nv.min(max_v);
								}
								*v = nv;
							}
						} else {
							self.values[axis] = new_val;
						}
						self.emit_changed();
						ctx.request_redraw();
					}
					return EventResponse::Handled;
				}

				let hov = segments.iter().position(|r| r.contains_point(*position));
				if hov != self.hovered_axis {
					self.hovered_axis = hov;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerLeft => {
				if self.hovered_axis.is_some() {
					self.hovered_axis = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				if let Some(axis) = segments.iter().position(|r| r.contains_point(*position)) {
					let seg = segments[axis];
					let letter_w = AXIS_LETTER_W.min(seg.width() * 0.30);

					// Double-click on axis letter resets component to default
					let now = Instant::now();
					let is_double = self.last_click.is_some_and(|(t, prev_ax)| {
						prev_ax == axis && now.duration_since(t).as_millis() < 350
					});
					self.last_click = Some((now, axis));

					if is_double && position.x <= seg.x() + letter_w {
						let def = self.default_values[axis];
						if self.uniform {
							self.values = self.default_values;
						} else {
							self.values[axis] = def;
						}
						self.cancel_edit();
						self.emit_changed();
						ctx.request_redraw();
						return EventResponse::Handled;
					}

					if self.editing_axis.is_some() && self.editing_axis != Some(axis) {
						self.commit_edit();
					}

					self.scrub = Some(ScrubState {
						axis,
						start_x: position.x,
						start_val: self.values[axis],
						dragging: false,
					});
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				if self.editing_axis.is_some() {
					self.commit_edit();
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if let Some(scrub) = self.scrub.take() {
					if !scrub.dragging {
						self.begin_edit(scrub.axis);
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::Key {
				key_name,
				text,
				pressed: true,
			} => {
				let Some(axis) = self.editing_axis else {
					return EventResponse::Ignored;
				};
				let char_len = self.edit_buffer.chars().count();

				match key_name.as_str() {
					"ArrowLeft" => {
						if let Some((s0, _)) = self.selection_range() {
							self.cursor = s0;
							self.selection_anchor = None;
						} else if self.cursor > 0 {
							self.cursor -= 1;
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"ArrowRight" => {
						if let Some((_, s1)) = self.selection_range() {
							self.cursor = s1;
							self.selection_anchor = None;
						} else if self.cursor < char_len {
							self.cursor += 1;
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Home" => {
						self.cursor = 0;
						self.selection_anchor = None;
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"End" => {
						self.cursor = char_len;
						self.selection_anchor = None;
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"ArrowUp" => {
						let base = self.edit_buffer.trim().parse::<f32>().unwrap_or(self.values[axis]);
						let nv = self.clamp_val(base + self.step);
						if self.uniform {
							self.values = [nv; N];
						} else {
							self.values[axis] = nv;
						}
						self.begin_edit(axis);
						self.emit_changed();
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"ArrowDown" => {
						let base = self.edit_buffer.trim().parse::<f32>().unwrap_or(self.values[axis]);
						let nv = self.clamp_val(base - self.step);
						if self.uniform {
							self.values = [nv; N];
						} else {
							self.values[axis] = nv;
						}
						self.begin_edit(axis);
						self.emit_changed();
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Backspace" => {
						if !self.delete_selection() && self.cursor > 0 {
							let b0 = Self::char_to_byte(&self.edit_buffer, self.cursor - 1);
							let b1 = Self::char_to_byte(&self.edit_buffer, self.cursor);
							self.edit_buffer.replace_range(b0..b1, "");
							self.cursor -= 1;
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Delete" => {
						if !self.delete_selection() && self.cursor < char_len {
							let b0 = Self::char_to_byte(&self.edit_buffer, self.cursor);
							let b1 = Self::char_to_byte(&self.edit_buffer, self.cursor + 1);
							self.edit_buffer.replace_range(b0..b1, "");
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Tab" => {
						self.commit_edit();
						let next = (axis + 1) % N;
						self.begin_edit(next);
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Enter" => {
						self.commit_edit();
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					"Escape" => {
						self.cancel_edit();
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					_ => {
						if let Some(txt) = text {
							let mut inserted = false;
							for ch in txt.chars() {
								if ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+') {
									if !inserted {
										self.delete_selection();
										inserted = true;
									}
									let b = Self::char_to_byte(&self.edit_buffer, self.cursor);
									self.edit_buffer.insert(b, ch);
									self.cursor += 1;
								}
							}
							if inserted {
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}
					}
				}
				EventResponse::Ignored
			}

			InputEvent::FocusLost => {
				if self.editing_axis.is_some() || self.scrub.is_some() {
					self.blur();
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}

	fn cursor(&self) -> Cursor {
		if self.scrub.as_ref().is_some_and(|s| s.dragging) {
			Cursor::EwResize
		} else if self.editing_axis.is_some() {
			Cursor::Text
		} else {
			Cursor::Default
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"value" => {
				let arr = value.expect_array()?;
				for (i, item) in arr.iter().take(N).enumerate() {
					if let Some(v) = item.as_f32() {
						self.values[i] = self.clamp_val(v);
					}
				}
			}
			"default" => {
				let arr = value.expect_array()?;
				for (i, item) in arr.iter().take(N).enumerate() {
					if let Some(v) = item.as_f32() {
						self.default_values[i] = v;
					}
				}
			}
			"step" => self.step = value.expect_f32()?.max(0.0001),
			"decimals" => self.decimals = (value.expect_f32()?.max(0.0) as usize).min(6),
			"suffix" => self.suffix = value.expect_str()?.to_string(),
			"min" => self.min = Some(value.expect_f32()?),
			"max" => self.max = Some(value.expect_f32()?),
			"uniform" => self.uniform = value.expect_bool()?,
			"font_size" => self.font_size = value.expect_f32()?.clamp(9.0, 24.0),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if method_name == "on_changed" {
			self.on_changed = Some(callback);
			true
		} else {
			false
		}
	}
}


pub type Vec2Edit = VecEdit<2>;
pub type Vec3Edit = VecEdit<3>;
pub type Vec4Edit = VecEdit<4>;

impl Vec2Edit {
	pub fn vec2(&self) -> Vec2<f32> {
		Vec2::new(self.values[0], self.values[1])
	}
}

impl Vec3Edit {
	pub fn vec3(&self) -> Vec3<f32> {
		Vec3::new(self.values[0], self.values[1], self.values[2])
	}
}

impl Vec4Edit {
	pub fn vec4(&self) -> Vec4<f32> {
		Vec4::new(self.values[0], self.values[1], self.values[2], self.values[3])
	}
}

impl<const N: usize> VecEdit<N> {
	pub fn with_step(mut self, step: f32) -> Self {
		self.step = step.max(0.0001);
		self
	}

	pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
		self.suffix = suffix.into();
		self
	}

	pub fn with_on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.on_changed = Some(MethodCallback::new(callback));
		self
	}
}
