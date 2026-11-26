/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;

use nimbus_core::{
	BoxConstraints, ButtonState, Cursor, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


const STEPPER_WIDTH: f32 = 28.0;
const TEXT_PADDING: f32 = 10.0;
const SCRUB_THRESHOLD_PX: f32 = 4.0;

fn scrub_step_delta(rel_x: f32, widget_width: f32, finite_range_steps: Option<f64>) -> f64 {
	let abs_dx = (rel_x as f64).abs();
	if abs_dx <= f64::EPSILON {
		return 0.0;
	}

	let base_steps = 0.16 * abs_dx.powf(1.55);
	let range_boost = match finite_range_steps {
		Some(total_steps) if total_steps > 100.0 && widget_width > 1.0 => {
			(total_steps / (widget_width as f64 * 1.5)).clamp(1.0, 25.0)
		}
		_ => 1.0,
	};

	(base_steps * range_boost).copysign(rel_x as f64)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum StepperPart {
	None,
	Up,
	Down,
}

fn draw_arrow_icon(
	canvas: &mut Canvas2D,
	center: Vec2<f32>,
	size: f32,
	up: bool,
	thickness: f32,
	color: Color,
) {
	let half_h = size * 0.5;
	let tip_y = if up { -half_h } else { half_h };
	let base_y = if up { half_h } else { -half_h };
	let steps = (size / 0.35).ceil().max(10.0) as usize;
	let r = thickness * 0.5;

	for i in 0..=steps {
		let t = i as f32 / steps as f32;
		let lx = -size * (1.0 - t);
		let rx = size * (1.0 - t);
		let y = base_y + (tip_y - base_y) * t;

		canvas.draw(
			RectShape::new(
				Rect::new(center.x + lx - r, center.y + y - r, thickness, thickness),
				color,
			)
			.radius(r),
		);
		if i < steps {
			canvas.draw(
				RectShape::new(
					Rect::new(center.x + rx - r, center.y + y - r, thickness, thickness),
					color,
				)
				.radius(r),
			);
		}
	}
}

fn box_bounds_for(bounds: Rect<f32>, has_title: bool) -> Rect<f32> {
	if has_title {
		Rect::new(
			bounds.x(),
			bounds.y() + 5.0,
			bounds.width(),
			(bounds.height() - 5.0).max(0.0),
		)
	} else {
		bounds
	}
}

fn stepper_rects(box_bounds: Rect<f32>) -> (Rect<f32>, Rect<f32>) {
	let w = STEPPER_WIDTH.min(box_bounds.width() * 0.4);
	let x = box_bounds.max.x - w;
	let half_h = box_bounds.height() * 0.5;
	let up = Rect::new(x, box_bounds.y(), w, half_h);
	let down = Rect::new(x, box_bounds.y() + half_h, w, (box_bounds.height() - half_h).max(0.0));
	(up, down)
}

fn hit_stepper(box_bounds: Rect<f32>, pos: Vec2<f32>) -> StepperPart {
	let (up, down) = stepper_rects(box_bounds);
	if up.contains_point(pos) {
		StepperPart::Up
	} else if down.contains_point(pos) {
		StepperPart::Down
	} else {
		StepperPart::None
	}
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
	if pos == 0 {
		return 0;
	}
	let mut p = pos.min(s.len()) - 1;
	while p > 0 && !s.is_char_boundary(p) {
		p -= 1;
	}
	p
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
	if pos >= s.len() {
		return s.len();
	}
	let mut p = pos + 1;
	while p < s.len() && !s.is_char_boundary(p) {
		p += 1;
	}
	p
}

fn ensure_char_boundary(s: &str, pos: usize) -> usize {
	let mut p = pos.min(s.len());
	while p > 0 && !s.is_char_boundary(p) {
		p -= 1;
	}
	p
}

fn char_index_at_x(text: &str, font_size: f32, target_x: f32) -> usize {
	if target_x <= 0.0 || text.is_empty() {
		return 0;
	}
	let fface = FontManager::default_font();
	let mut last_w = 0.0;
	let mut last_idx = 0;
	for (idx, _) in text.char_indices() {
		if idx == 0 {
			continue;
		}
		let (w, _) = fface.measure_text(&text[..idx], font_size);
		if target_x <= w {
			let mid = last_w + (w - last_w) * 0.5;
			return if target_x < mid { last_idx } else { idx };
		}
		last_w = w;
		last_idx = idx;
	}
	let (total_w, _) = fface.measure_text(text, font_size);
	if target_x <= total_w {
		let mid = last_w + (total_w - last_w) * 0.5;
		if target_x < mid { last_idx } else { text.len() }
	} else {
		text.len()
	}
}

#[allow(clippy::too_many_arguments)]
fn draw_spinbox_chrome(
	ctx: &DrawCtx,
	canvas: &mut Canvas2D,
	title: &str,
	display_text: &str,
	editing: bool,
	cursor_pos: usize,
	selection_start: Option<usize>,
	scroll_x: &Cell<f32>,
	font: Option<FontId>,
	font_size: f32,
	corner_radius: Option<f32>,
	hovered_part: StepperPart,
	pressed_part: StepperPart,
) {
	let monet = &ctx.theme.monet;
	let bounds = ctx.bounds;
	let has_title = !title.is_empty();
	let box_bounds = box_bounds_for(bounds, has_title);

	let border_color = if ctx.focused || ctx.hovered {
		monet[MonetRole::Primary]
	} else {
		monet[MonetRole::PrimaryOutline]
	};

	let (up_rect, down_rect) = stepper_rects(box_bounds);
	let divider_x = up_rect.x();

	// Outer border
	let stroke_w = ctx.theme.stroke.thick;
	let resolved_radius = corner_radius.unwrap_or(ctx.theme.radius.small);
	canvas.draw(
		RectShape::new(box_bounds, Color::TRANSPARENT)
			.border(border_color, stroke_w)
			.radius(resolved_radius),
	);

	// Minimalist Qt-style stepper arrow icons (color-only hover & press feedback)
	let up_icon_col = if pressed_part == StepperPart::Up {
		monet[MonetRole::Primary]
	} else if hovered_part == StepperPart::Up {
		monet[MonetRole::OnSurface]
	} else {
		monet[MonetRole::OnSurfaceVariant]
	};
	let down_icon_col = if pressed_part == StepperPart::Down {
		monet[MonetRole::Primary]
	} else if hovered_part == StepperPart::Down {
		monet[MonetRole::OnSurface]
	} else {
		monet[MonetRole::OnSurfaceVariant]
	};
	let up_center = Vec2::new(up_rect.center().x, up_rect.center().y + 3.0);
	let down_center = Vec2::new(down_rect.center().x, down_rect.center().y - 3.0);
	draw_arrow_icon(canvas, up_center, 3.6, true, 1.35, up_icon_col);
	draw_arrow_icon(canvas, down_center, 3.6, false, 1.35, down_icon_col);

	let fface = FontManager::default_font();

	// Optional floating title notch (matches Edit)
	if has_title {
		let (title_w, _) = canvas.measure_text(title, 12.0);
		let notch_rect = Rect::new(
			box_bounds.x() + 10.0,
			box_bounds.y() - 1.0,
			title_w + 8.0,
			3.0,
		);
		canvas.draw(RectShape::new(notch_rect, monet[MonetRole::SurfaceContainer]));

		let title_font = match font {
			Some(id) => Font::with_id(id, fface, 12.0),
			None => Font::new(fface, 12.0),
		};
		let title_color = if ctx.focused {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::OnSurface]
		};
		canvas.draw(TextShape::new(
			title,
			title_font,
			12.0,
			Vec2::new(box_bounds.x() + 14.0, bounds.y() - 2.0),
			title_color,
		));
	}

	// Text viewport (left of stepper buttons)
	let text_area_w = (divider_x - box_bounds.x() - TEXT_PADDING * 2.0).max(0.0);
	let view_clip = Rect::new(
		box_bounds.x() + TEXT_PADDING,
		box_bounds.y(),
		text_area_w,
		box_bounds.height(),
	);

	let cursor_idx = ensure_char_boundary(display_text, cursor_pos);
	let (cursor_offset, _) = canvas.measure_text(&display_text[..cursor_idx], font_size);
	let (total_w, text_h) = canvas.measure_text(display_text, font_size);

	let mut scroll = scroll_x.get();
	if editing {
		if cursor_offset < scroll {
			scroll = cursor_offset;
		} else if cursor_offset > scroll + text_area_w {
			scroll = cursor_offset - text_area_w;
		}
	} else {
		scroll = 0.0;
	}
	if total_w <= text_area_w {
		scroll = 0.0;
	}
	scroll_x.set(scroll);

	canvas.push_clip(view_clip);

	let draw_x = box_bounds.x() + TEXT_PADDING - scroll;
	let text_y = box_bounds.y() + (box_bounds.height() - text_h) * 0.5;

	if editing
		&& let Some(sel_start) = selection_start
	{
		let sel_start = ensure_char_boundary(display_text, sel_start);
		if sel_start != cursor_idx {
			let start_pos = sel_start.min(cursor_idx);
			let end_pos = sel_start.max(cursor_idx);
			let (sel_start_x, _) = canvas.measure_text(&display_text[..start_pos], font_size);
			let (sel_end_x, _) = canvas.measure_text(&display_text[..end_pos], font_size);
			let sel_rect = Rect::new(
				draw_x + sel_start_x,
				box_bounds.y() + 6.0,
				sel_end_x - sel_start_x,
				(box_bounds.height() - 12.0).max(0.0),
			);
			canvas.draw(RectShape::new(sel_rect, monet[MonetRole::PrimaryContainer]).radius(2.0));
		}
	}

	let text_font = match font {
		Some(id) => Font::with_id(id, fface, font_size),
		None => Font::new(fface, font_size),
	};
	canvas.draw(TextShape::new(
		display_text,
		text_font,
		font_size,
		Vec2::new(draw_x, text_y),
		monet[MonetRole::OnSurface],
	));

	if ctx.focused && editing {
		let cursor_x = draw_x + cursor_offset;
		let cursor_rect = Rect::new(
			cursor_x,
			box_bounds.y() + 8.0,
			1.5,
			(box_bounds.height() - 16.0).max(0.0),
		);
		canvas.draw(RectShape::new(cursor_rect, monet[MonetRole::Primary]));
	}

	canvas.pop_clip();
}


// SpinBox (int)

pub struct SpinBox {
	pub value: i64,
	pub min: i64,
	pub max: i64,
	pub step: i64,
	pub title: String,
	pub prefix: String,
	pub suffix: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub corner_radius: Option<f32>,
	pub on_changed: Option<MethodCallback>,
	pub on_submit: Option<MethodCallback>,

	buffer: String,
	cursor_pos: usize,
	selection_start: Option<usize>,
	scroll_x: Cell<f32>,
	last_bounds: Cell<Rect<f32>>,
	hovered_part: StepperPart,
	pressed_part: StepperPart,
	is_dragging_text: bool,
	scrub_candidate: bool,
	scrub_active: bool,
	scrub_origin_x: f32,
	scrub_last_x: f32,
	scrub_accum: f64,
	focused: bool,
	editing: bool,
	dirty: bool,
}

impl Default for SpinBox {
	fn default() -> Self {
		Self {
			value: 0,
			min: i64::MIN,
			max: i64::MAX,
			step: 1,
			title: String::new(),
			prefix: String::new(),
			suffix: String::new(),
			font: None,
			font_size: 15.0,
			corner_radius: None,
			on_changed: None,
			on_submit: None,
			buffer: "0".to_string(),
			cursor_pos: 1,
			selection_start: None,
			scroll_x: Cell::new(0.0),
			last_bounds: Cell::new(Rect::ZERO),
			hovered_part: StepperPart::None,
			pressed_part: StepperPart::None,
			is_dragging_text: false,
			scrub_candidate: false,
			scrub_active: false,
			scrub_origin_x: 0.0,
			scrub_last_x: 0.0,
			scrub_accum: 0.0,
			focused: false,
			editing: false,
			dirty: false,
		}
	}
}

impl SpinBox {
	pub fn new(value: i64) -> Self {
		let mut s = Self::default();
		s.set_value(value);
		s
	}

	pub fn builder() -> SpinBoxBuilder {
		SpinBoxBuilder::new()
	}

	pub fn set_value(&mut self, value: i64) {
		let clamped = value.clamp(self.min, self.max);
		self.value = clamped;
		self.buffer = clamped.to_string();
		self.cursor_pos = self.buffer.len();
		self.selection_start = None;
		self.dirty = false;
	}

	fn display_text(&self) -> String {
		if self.focused && self.editing {
			self.buffer.clone()
		} else {
			format!("{}{}{}", self.prefix, self.value, self.suffix)
		}
	}

	fn delete_selection(&mut self) -> bool {
		if let Some(start) = self.selection_start {
			let cursor = ensure_char_boundary(&self.buffer, self.cursor_pos);
			let start = ensure_char_boundary(&self.buffer, start);
			if start != cursor {
				let min = start.min(cursor);
				let max = start.max(cursor);
				self.buffer.drain(min..max);
				self.cursor_pos = min;
				self.selection_start = None;
				return true;
			}
		}
		self.selection_start = None;
		false
	}

	fn apply_step(&mut self, delta_steps: i64) -> bool {
		let step = self.step.max(1);
		let next = self
			.value
			.saturating_add(step.saturating_mul(delta_steps))
			.clamp(self.min, self.max);
		if next != self.value {
			self.value = next;
			self.buffer = next.to_string();
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
			self.dirty = true;
			if let Some(cb) = &mut self.on_changed {
				cb.call1(self.value);
			}
			true
		} else {
			false
		}
	}

	fn commit_buffer(&mut self, fire_submit: bool) {
		let trimmed = self.buffer.trim();
		if let Ok(parsed) = trimmed.parse::<i64>() {
			let clamped = parsed.clamp(self.min, self.max);
			let changed = clamped != self.value;
			self.value = clamped;
			self.buffer = clamped.to_string();
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
			if changed && let Some(cb) = &mut self.on_changed {
				cb.call1(self.value);
			}
		} else {
			self.buffer = self.value.to_string();
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
		}

		if fire_submit {
			self.dirty = false;
			if let Some(cb) = &mut self.on_submit {
				cb.call1(self.value);
			}
		}
	}

	fn sync_live_parse(&mut self) {
		if let Ok(parsed) = self.buffer.trim().parse::<i64>() {
			let clamped = parsed.clamp(self.min, self.max);
			if clamped != self.value {
				self.value = clamped;
				if let Some(cb) = &mut self.on_changed {
					cb.call1(self.value);
				}
			}
		}
	}
}

impl Widget for SpinBox {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(160.0, 40.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let display = self.display_text();
		draw_spinbox_chrome(
			ctx,
			canvas,
			&self.title,
			&display,
			ctx.focused && self.editing,
			self.cursor_pos,
			self.selection_start,
			&self.scroll_x,
			self.font,
			self.font_size,
			self.corner_radius,
			self.hovered_part,
			self.pressed_part,
		);
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let box_bounds = box_bounds_for(bounds, !self.title.is_empty());

		match event {
			InputEvent::FocusGained => {
				self.focused = true;
				self.editing = false;
				self.buffer = self.value.to_string();
				self.cursor_pos = self.buffer.len();
				self.selection_start = None;
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::FocusLost => {
				self.focused = false;
				self.editing = false;
				self.is_dragging_text = false;
				self.scrub_candidate = false;
				self.scrub_active = false;
				self.pressed_part = StepperPart::None;
				self.selection_start = None;
				let should_submit = self.dirty;
				self.commit_buffer(should_submit);
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::PointerLeft => {
				if self.hovered_part != StepperPart::None {
					self.hovered_part = StepperPart::None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerMoved { position } => {
				let part = hit_stepper(box_bounds, *position);
				let mut handled = false;
				if part != self.hovered_part {
					self.hovered_part = part;
					ctx.request_redraw();
					handled = true;
				}
				if self.scrub_candidate {
					let total_dx = position.x - self.scrub_origin_x;
					if !self.scrub_active && total_dx.abs() >= SCRUB_THRESHOLD_PX {
						self.scrub_active = true;
						self.scrub_last_x = position.x;
						self.scrub_accum = 0.0;
					}
					if self.scrub_active {
						let rel_x = position.x - self.scrub_last_x;
						self.scrub_last_x = position.x;
						let step = self.step.max(1);
						let finite_range = if self.min > i64::MIN / 2 && self.max < i64::MAX / 2 {
							Some((self.max - self.min) as f64 / step as f64)
						} else {
							None
						};
						self.scrub_accum += scrub_step_delta(rel_x, box_bounds.width(), finite_range);
						let whole_steps = self.scrub_accum.trunc() as i64;
						if whole_steps != 0 {
							self.scrub_accum -= whole_steps as f64;
							let next = self
								.value
								.saturating_add(step.saturating_mul(whole_steps))
								.clamp(self.min, self.max);
							if next != self.value {
								self.value = next;
								self.buffer = next.to_string();
								self.cursor_pos = self.buffer.len();
								self.dirty = true;
								if let Some(cb) = &mut self.on_changed {
									cb.call1(self.value);
								}
							} else {
								self.scrub_accum = 0.0;
							}
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
				if self.is_dragging_text {
					let local_x = position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
					self.cursor_pos = char_index_at_x(&self.buffer, self.font_size, local_x);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				if handled {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					let part = hit_stepper(box_bounds, *position);
					match part {
						StepperPart::Up => {
							self.pressed_part = StepperPart::Up;
							self.apply_step(1);
							ctx.request_redraw();
							EventResponse::Handled
						}
						StepperPart::Down => {
							self.pressed_part = StepperPart::Down;
							self.apply_step(-1);
							ctx.request_redraw();
							EventResponse::Handled
						}
						StepperPart::None => {
							if !self.editing {
								// Unfocused (or not in text-edit mode): horizontal drag scrubs the value
								self.scrub_candidate = true;
								self.scrub_active = false;
								self.scrub_origin_x = position.x;
								self.scrub_last_x = position.x;
								self.scrub_accum = 0.0;
							} else {
								self.is_dragging_text = true;
								let local_x =
									position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
								let idx = char_index_at_x(&self.buffer, self.font_size, local_x);
								self.cursor_pos = idx;
								self.selection_start = Some(idx);
							}
							ctx.request_redraw();
							EventResponse::Handled
						}
					}
				}
				ButtonState::Released => {
					let mut handled = false;
					if self.pressed_part != StepperPart::None {
						self.pressed_part = StepperPart::None;
						handled = true;
					}
					if self.scrub_candidate {
						let was_scrubbing = self.scrub_active;
						self.scrub_candidate = false;
						self.scrub_active = false;
						if was_scrubbing {
							if self.dirty {
								self.commit_buffer(true);
							}
						} else {
							// Plain click without horizontal drag enters text-editing mode
							self.editing = true;
							self.buffer = self.value.to_string();
							let local_x =
								position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
							self.cursor_pos = char_index_at_x(&self.buffer, self.font_size, local_x);
							self.selection_start = None;
						}
						handled = true;
					}
					if self.is_dragging_text {
						self.is_dragging_text = false;
						if self.selection_start == Some(self.cursor_pos) {
							self.selection_start = None;
						}
						handled = true;
					}
					if handled {
						ctx.request_redraw();
						EventResponse::Handled
					} else {
						EventResponse::Ignored
					}
				}
			},

			InputEvent::Key {
				key_name,
				text,
				pressed,
			} => {
				if !pressed {
					return EventResponse::Ignored;
				}
				self.editing = true;
				match key_name.as_str() {
					"ArrowUp" => {
						self.apply_step(1);
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowDown" => {
						self.apply_step(-1);
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowLeft" => {
						self.cursor_pos = prev_char_boundary(&self.buffer, self.cursor_pos);
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowRight" => {
						self.cursor_pos = next_char_boundary(&self.buffer, self.cursor_pos);
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Home" => {
						self.cursor_pos = 0;
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"End" => {
						self.cursor_pos = self.buffer.len();
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Backspace" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos > 0 {
							let prev = prev_char_boundary(&self.buffer, self.cursor_pos);
							self.buffer.drain(prev..self.cursor_pos);
							self.cursor_pos = prev;
							changed = true;
						}
						if changed {
							self.dirty = true;
							self.sync_live_parse();
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Delete" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos < self.buffer.len() {
							let next = next_char_boundary(&self.buffer, self.cursor_pos);
							self.buffer.drain(self.cursor_pos..next);
							changed = true;
						}
						if changed {
							self.dirty = true;
							self.sync_live_parse();
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Enter" => {
						self.commit_buffer(true);
						ctx.request_redraw();
						EventResponse::Handled
					}
					_ => {
						if let Some(txt) = text {
							let allow_minus = self.min < 0;
							let mut inserted = false;
							for ch in txt.chars() {
								if ch.is_ascii_digit() {
									self.delete_selection();
									self.cursor_pos = ensure_char_boundary(&self.buffer, self.cursor_pos);
									self.buffer.insert(self.cursor_pos, ch);
									self.cursor_pos += ch.len_utf8();
									inserted = true;
								} else if ch == '-' && allow_minus {
									self.delete_selection();
									if self.cursor_pos == 0 && !self.buffer.starts_with('-') {
										self.buffer.insert(0, '-');
										self.cursor_pos = 1;
										inserted = true;
									}
								}
							}
							if inserted {
								self.dirty = true;
								self.sync_live_parse();
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}
						EventResponse::Ignored
					}
				}
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
			"value" => self.set_value(value.expect_f64()?.round() as i64),
			"min" => {
				self.min = value.expect_f64()?.round() as i64;
				if self.max < self.min {
					self.max = self.min;
				}
				self.set_value(self.value);
			}
			"max" => {
				self.max = value.expect_f64()?.round() as i64;
				if self.min > self.max {
					self.min = self.max;
				}
				self.set_value(self.value);
			}
			"step" => self.step = (value.expect_f64()?.round() as i64).max(1),
			"title" => self.title = value.expect_str()?.to_string(),
			"prefix" => self.prefix = value.expect_str()?.to_string(),
			"suffix" => self.suffix = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed" | "on_submit")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_changed" => {
				self.on_changed = Some(callback);
				true
			}
			"on_submit" => {
				self.on_submit = Some(callback);
				true
			}
			_ => false,
		}
	}

	fn cursor(&self) -> Cursor {
		if self.scrub_active {
			Cursor::EwResize
		} else if self.editing {
			Cursor::Text
		} else {
			Cursor::Default
		}
	}
}


#[derive(Default)]
pub struct SpinBoxBuilder {
	inner: SpinBox,
}

impl SpinBoxBuilder {
	pub fn new() -> Self {
		Self {
			inner: SpinBox::default(),
		}
	}

	pub fn value(mut self, value: i64) -> Self {
		self.inner.set_value(value);
		self
	}

	pub fn min(mut self, min: i64) -> Self {
		self.inner.min = min;
		if self.inner.max < min {
			self.inner.max = min;
		}
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn max(mut self, max: i64) -> Self {
		self.inner.max = max;
		if self.inner.min > max {
			self.inner.min = max;
		}
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn range(mut self, min: i64, max: i64) -> Self {
		self.inner.min = min.min(max);
		self.inner.max = min.max(max);
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn step(mut self, step: i64) -> Self {
		self.inner.step = step.max(1);
		self
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
		self.inner.prefix = prefix.into();
		self
	}

	pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
		self.inner.suffix = suffix.into();
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

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_submit<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_submit = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> SpinBox {
		self.inner
	}
}

impl From<SpinBoxBuilder> for SpinBox {
	fn from(builder: SpinBoxBuilder) -> Self {
		builder.build()
	}
}


// FloatSpinBox (float)

pub struct FloatSpinBox {
	pub value: f64,
	pub min: f64,
	pub max: f64,
	pub step: f64,
	pub decimals: usize,
	pub title: String,
	pub prefix: String,
	pub suffix: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub corner_radius: Option<f32>,
	pub on_changed: Option<MethodCallback>,
	pub on_submit: Option<MethodCallback>,

	buffer: String,
	cursor_pos: usize,
	selection_start: Option<usize>,
	scroll_x: Cell<f32>,
	last_bounds: Cell<Rect<f32>>,
	hovered_part: StepperPart,
	pressed_part: StepperPart,
	is_dragging_text: bool,
	scrub_candidate: bool,
	scrub_active: bool,
	scrub_origin_x: f32,
	scrub_last_x: f32,
	scrub_accum: f64,
	focused: bool,
	editing: bool,
	dirty: bool,
}

impl Default for FloatSpinBox {
	fn default() -> Self {
		Self {
			value: 0.0,
			min: f64::NEG_INFINITY,
			max: f64::INFINITY,
			step: 0.1,
			decimals: 2,
			title: String::new(),
			prefix: String::new(),
			suffix: String::new(),
			font: None,
			font_size: 15.0,
			corner_radius: None,
			on_changed: None,
			on_submit: None,
			buffer: "0.00".to_string(),
			cursor_pos: 4,
			selection_start: None,
			scroll_x: Cell::new(0.0),
			last_bounds: Cell::new(Rect::ZERO),
			hovered_part: StepperPart::None,
			pressed_part: StepperPart::None,
			is_dragging_text: false,
			scrub_candidate: false,
			scrub_active: false,
			scrub_origin_x: 0.0,
			scrub_last_x: 0.0,
			scrub_accum: 0.0,
			focused: false,
			editing: false,
			dirty: false,
		}
	}
}

impl FloatSpinBox {
	pub fn new(value: f64) -> Self {
		let mut s = Self::default();
		s.set_value(value);
		s
	}

	pub fn builder() -> FloatSpinBoxBuilder {
		FloatSpinBoxBuilder::new()
	}

	fn round_to_decimals(&self, val: f64) -> f64 {
		let factor = 10.0f64.powi(self.decimals.min(10) as i32);
		let r = (val * factor).round() / factor;
		if r == 0.0 { 0.0 } else { r }
	}

	fn format_value(&self, val: f64) -> String {
		format!("{:.*}", self.decimals.min(10), val)
	}

	pub fn set_value(&mut self, value: f64) {
		let clamped = if value.is_finite() {
			self.round_to_decimals(value.clamp(self.min, self.max))
		} else {
			0.0f64.clamp(self.min, self.max)
		};
		self.value = clamped;
		self.buffer = self.format_value(clamped);
		self.cursor_pos = self.buffer.len();
		self.selection_start = None;
		self.dirty = false;
	}

	fn display_text(&self) -> String {
		if self.focused && self.editing {
			self.buffer.clone()
		} else {
			format!("{}{}{}", self.prefix, self.format_value(self.value), self.suffix)
		}
	}

	fn delete_selection(&mut self) -> bool {
		if let Some(start) = self.selection_start {
			let cursor = ensure_char_boundary(&self.buffer, self.cursor_pos);
			let start = ensure_char_boundary(&self.buffer, start);
			if start != cursor {
				let min = start.min(cursor);
				let max = start.max(cursor);
				self.buffer.drain(min..max);
				self.cursor_pos = min;
				self.selection_start = None;
				return true;
			}
		}
		self.selection_start = None;
		false
	}

	fn apply_step(&mut self, delta_steps: f64) -> bool {
		let step = if self.step > 0.0 && self.step.is_finite() {
			self.step
		} else {
			0.1
		};
		let next = self.round_to_decimals((self.value + step * delta_steps).clamp(self.min, self.max));
		if (next - self.value).abs() > f64::EPSILON {
			self.value = next;
			self.buffer = self.format_value(next);
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
			self.dirty = true;
			if let Some(cb) = &mut self.on_changed {
				cb.call1(self.value);
			}
			true
		} else {
			false
		}
	}

	fn commit_buffer(&mut self, fire_submit: bool) {
		let normalized = self.buffer.trim().replace(',', ".");
		if let Ok(parsed) = normalized.parse::<f64>()
			&& parsed.is_finite()
		{
			let clamped = self.round_to_decimals(parsed.clamp(self.min, self.max));
			let changed = (clamped - self.value).abs() > f64::EPSILON;
			self.value = clamped;
			self.buffer = self.format_value(clamped);
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
			if changed && let Some(cb) = &mut self.on_changed {
				cb.call1(self.value);
			}
		} else {
			self.buffer = self.format_value(self.value);
			self.cursor_pos = self.buffer.len();
			self.selection_start = None;
		}

		if fire_submit {
			self.dirty = false;
			if let Some(cb) = &mut self.on_submit {
				cb.call1(self.value);
			}
		}
	}

	fn sync_live_parse(&mut self) {
		let normalized = self.buffer.trim().replace(',', ".");
		if let Ok(parsed) = normalized.parse::<f64>()
			&& parsed.is_finite()
		{
			let clamped = parsed.clamp(self.min, self.max);
			if (clamped - self.value).abs() > f64::EPSILON {
				self.value = clamped;
				if let Some(cb) = &mut self.on_changed {
					cb.call1(self.value);
				}
			}
		}
	}
}

impl Widget for FloatSpinBox {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(160.0, 40.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let display = self.display_text();
		draw_spinbox_chrome(
			ctx,
			canvas,
			&self.title,
			&display,
			ctx.focused && self.editing,
			self.cursor_pos,
			self.selection_start,
			&self.scroll_x,
			self.font,
			self.font_size,
			self.corner_radius,
			self.hovered_part,
			self.pressed_part,
		);
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let box_bounds = box_bounds_for(bounds, !self.title.is_empty());

		match event {
			InputEvent::FocusGained => {
				self.focused = true;
				self.editing = false;
				self.buffer = self.format_value(self.value);
				self.cursor_pos = self.buffer.len();
				self.selection_start = None;
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::FocusLost => {
				self.focused = false;
				self.editing = false;
				self.is_dragging_text = false;
				self.scrub_candidate = false;
				self.scrub_active = false;
				self.pressed_part = StepperPart::None;
				self.selection_start = None;
				let should_submit = self.dirty;
				self.commit_buffer(should_submit);
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::PointerLeft => {
				if self.hovered_part != StepperPart::None {
					self.hovered_part = StepperPart::None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerMoved { position } => {
				let part = hit_stepper(box_bounds, *position);
				let mut handled = false;
				if part != self.hovered_part {
					self.hovered_part = part;
					ctx.request_redraw();
					handled = true;
				}
				if self.scrub_candidate {
					let total_dx = position.x - self.scrub_origin_x;
					if !self.scrub_active && total_dx.abs() >= SCRUB_THRESHOLD_PX {
						self.scrub_active = true;
						self.scrub_last_x = position.x;
						self.scrub_accum = 0.0;
					}
					if self.scrub_active {
						let rel_x = position.x - self.scrub_last_x;
						self.scrub_last_x = position.x;
						let step = if self.step > 0.0 && self.step.is_finite() {
							self.step
						} else {
							0.1
						};
						let finite_range = if self.min.is_finite() && self.max.is_finite() {
							Some((self.max - self.min) / step)
						} else {
							None
						};
						self.scrub_accum += scrub_step_delta(rel_x, box_bounds.width(), finite_range);
						let whole_steps = self.scrub_accum.trunc();
						if whole_steps.abs() >= 1.0 {
							self.scrub_accum -= whole_steps;
							let next = self
								.round_to_decimals((self.value + step * whole_steps).clamp(self.min, self.max));
							if (next - self.value).abs() > f64::EPSILON {
								self.value = next;
								self.buffer = self.format_value(next);
								self.cursor_pos = self.buffer.len();
								self.dirty = true;
								if let Some(cb) = &mut self.on_changed {
									cb.call1(self.value);
								}
							} else {
								self.scrub_accum = 0.0;
							}
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
				}
				if self.is_dragging_text {
					let local_x = position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
					self.cursor_pos = char_index_at_x(&self.buffer, self.font_size, local_x);
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				if handled {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					let part = hit_stepper(box_bounds, *position);
					match part {
						StepperPart::Up => {
							self.pressed_part = StepperPart::Up;
							self.apply_step(1.0);
							ctx.request_redraw();
							EventResponse::Handled
						}
						StepperPart::Down => {
							self.pressed_part = StepperPart::Down;
							self.apply_step(-1.0);
							ctx.request_redraw();
							EventResponse::Handled
						}
						StepperPart::None => {
							if !self.editing {
								// Unfocused (or not in text-edit mode): horizontal drag scrubs the value
								self.scrub_candidate = true;
								self.scrub_active = false;
								self.scrub_origin_x = position.x;
								self.scrub_last_x = position.x;
								self.scrub_accum = 0.0;
							} else {
								self.is_dragging_text = true;
								let local_x =
									position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
								let idx = char_index_at_x(&self.buffer, self.font_size, local_x);
								self.cursor_pos = idx;
								self.selection_start = Some(idx);
							}
							ctx.request_redraw();
							EventResponse::Handled
						}
					}
				}
				ButtonState::Released => {
					let mut handled = false;
					if self.pressed_part != StepperPart::None {
						self.pressed_part = StepperPart::None;
						handled = true;
					}
					if self.scrub_candidate {
						let was_scrubbing = self.scrub_active;
						self.scrub_candidate = false;
						self.scrub_active = false;
						if was_scrubbing {
							if self.dirty {
								self.commit_buffer(true);
							}
						} else {
							// Plain click without horizontal drag enters text-editing mode
							self.editing = true;
							self.buffer = self.format_value(self.value);
							let local_x =
								position.x - box_bounds.x() - TEXT_PADDING + self.scroll_x.get();
							self.cursor_pos = char_index_at_x(&self.buffer, self.font_size, local_x);
							self.selection_start = None;
						}
						handled = true;
					}
					if self.is_dragging_text {
						self.is_dragging_text = false;
						if self.selection_start == Some(self.cursor_pos) {
							self.selection_start = None;
						}
						handled = true;
					}
					if handled {
						ctx.request_redraw();
						EventResponse::Handled
					} else {
						EventResponse::Ignored
					}
				}
			},

			InputEvent::Key {
				key_name,
				text,
				pressed,
			} => {
				if !pressed {
					return EventResponse::Ignored;
				}
				self.editing = true;
				match key_name.as_str() {
					"ArrowUp" => {
						self.apply_step(1.0);
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowDown" => {
						self.apply_step(-1.0);
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowLeft" => {
						self.cursor_pos = prev_char_boundary(&self.buffer, self.cursor_pos);
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowRight" => {
						self.cursor_pos = next_char_boundary(&self.buffer, self.cursor_pos);
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Home" => {
						self.cursor_pos = 0;
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"End" => {
						self.cursor_pos = self.buffer.len();
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Backspace" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos > 0 {
							let prev = prev_char_boundary(&self.buffer, self.cursor_pos);
							self.buffer.drain(prev..self.cursor_pos);
							self.cursor_pos = prev;
							changed = true;
						}
						if changed {
							self.dirty = true;
							self.sync_live_parse();
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Delete" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos < self.buffer.len() {
							let next = next_char_boundary(&self.buffer, self.cursor_pos);
							self.buffer.drain(self.cursor_pos..next);
							changed = true;
						}
						if changed {
							self.dirty = true;
							self.sync_live_parse();
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Enter" => {
						self.commit_buffer(true);
						ctx.request_redraw();
						EventResponse::Handled
					}
					_ => {
						if let Some(txt) = text {
							let allow_minus = self.min < 0.0;
							let mut inserted = false;
							for ch in txt.chars() {
								if ch.is_ascii_digit() {
									self.delete_selection();
									self.cursor_pos = ensure_char_boundary(&self.buffer, self.cursor_pos);
									self.buffer.insert(self.cursor_pos, ch);
									self.cursor_pos += ch.len_utf8();
									inserted = true;
								} else if (ch == '.' || ch == ',') && self.decimals > 0 {
									self.delete_selection();
									if !self.buffer.contains('.') {
										self.cursor_pos = ensure_char_boundary(&self.buffer, self.cursor_pos);
										self.buffer.insert(self.cursor_pos, '.');
										self.cursor_pos += 1;
										inserted = true;
									}
								} else if ch == '-' && allow_minus {
									self.delete_selection();
									if self.cursor_pos == 0 && !self.buffer.starts_with('-') {
										self.buffer.insert(0, '-');
										self.cursor_pos = 1;
										inserted = true;
									}
								}
							}
							if inserted {
								self.dirty = true;
								self.sync_live_parse();
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}
						EventResponse::Ignored
					}
				}
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
			"value" => self.set_value(value.expect_f64()?),
			"min" => {
				self.min = value.expect_f64()?;
				if self.max < self.min {
					self.max = self.min;
				}
				self.set_value(self.value);
			}
			"max" => {
				self.max = value.expect_f64()?;
				if self.min > self.max {
					self.min = self.max;
				}
				self.set_value(self.value);
			}
			"step" => {
				let s = value.expect_f64()?;
				if s > 0.0 && s.is_finite() {
					self.step = s;
				}
			}
			"decimals" => {
				self.decimals = (value.expect_f64()?.max(0.0).round() as usize).min(10);
				self.set_value(self.value);
			}
			"title" => self.title = value.expect_str()?.to_string(),
			"prefix" => self.prefix = value.expect_str()?.to_string(),
			"suffix" => self.suffix = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?),
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(name, "on_changed" | "on_submit")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_changed" => {
				self.on_changed = Some(callback);
				true
			}
			"on_submit" => {
				self.on_submit = Some(callback);
				true
			}
			_ => false,
		}
	}

	fn cursor(&self) -> Cursor {
		if self.scrub_active {
			Cursor::EwResize
		} else if self.editing {
			Cursor::Text
		} else {
			Cursor::Default
		}
	}
}


#[derive(Default)]
pub struct FloatSpinBoxBuilder {
	inner: FloatSpinBox,
}

impl FloatSpinBoxBuilder {
	pub fn new() -> Self {
		Self {
			inner: FloatSpinBox::default(),
		}
	}

	pub fn value(mut self, value: f64) -> Self {
		self.inner.set_value(value);
		self
	}

	pub fn min(mut self, min: f64) -> Self {
		self.inner.min = min;
		if self.inner.max < min {
			self.inner.max = min;
		}
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn max(mut self, max: f64) -> Self {
		self.inner.max = max;
		if self.inner.min > max {
			self.inner.min = max;
		}
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn range(mut self, min: f64, max: f64) -> Self {
		self.inner.min = min.min(max);
		self.inner.max = min.max(max);
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn step(mut self, step: f64) -> Self {
		if step > 0.0 && step.is_finite() {
			self.inner.step = step;
		}
		self
	}

	pub fn decimals(mut self, decimals: usize) -> Self {
		self.inner.decimals = decimals.min(10);
		self.inner.set_value(self.inner.value);
		self
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
		self
	}

	pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
		self.inner.prefix = prefix.into();
		self
	}

	pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
		self.inner.suffix = suffix.into();
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

	pub fn corner_radius(mut self, radius: f32) -> Self {
		self.inner.corner_radius = Some(radius);
		self
	}

	pub fn on_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_submit<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_submit = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> FloatSpinBox {
		self.inner
	}
}

impl From<FloatSpinBoxBuilder> for FloatSpinBox {
	fn from(builder: FloatSpinBoxBuilder) -> Self {
		builder.build()
	}
}
