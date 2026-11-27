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
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


pub struct Edit {
	pub text: String,
	pub title: String,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub border_radius: Option<f32>,
	pub cursor_pos: usize,
	pub selection_start: Option<usize>,
	pub on_changed: Option<MethodCallback>,
	pub on_submit: Option<MethodCallback>,

	scroll_x: Cell<f32>,
	last_bounds: Cell<Rect<f32>>,
	is_dragging: bool,
	dirty: bool,
}

impl Default for Edit {
	fn default() -> Self {
		Self {
			text: String::new(),
			title: String::new(),
			font: None,
			font_size: 15.0,
			border_radius: None,
			cursor_pos: 0,
			selection_start: None,
			on_changed: None,
			on_submit: None,
			scroll_x: Cell::new(0.0),
			last_bounds: Cell::new(Rect::ZERO),
			is_dragging: false,
			dirty: false,
		}
	}
}

impl Edit {
	fn ensure_cursor_boundary(&mut self) {
		if self.cursor_pos > self.text.len() {
			self.cursor_pos = self.text.len();
		} else {
			while self.cursor_pos > 0 && !self.text.is_char_boundary(self.cursor_pos) {
				self.cursor_pos -= 1;
			}
		}
	}

	fn prev_char_boundary(&self, pos: usize) -> usize {
		if pos == 0 {
			return 0;
		}
		let mut p = pos - 1;
		while p > 0 && !self.text.is_char_boundary(p) {
			p -= 1;
		}
		p
	}

	fn next_char_boundary(&self, pos: usize) -> usize {
		if pos >= self.text.len() {
			return self.text.len();
		}
		let mut p = pos + 1;
		while p < self.text.len() && !self.text.is_char_boundary(p) {
			p += 1;
		}
		p
	}

	fn delete_selection(&mut self) -> bool {
		if let Some(start) = self.selection_start {
			let cursor = self.cursor_pos;
			if start != cursor {
				let min = start.min(cursor).min(self.text.len());
				let max = start.max(cursor).min(self.text.len());
				if self.text.is_char_boundary(min) && self.text.is_char_boundary(max) {
					self.text.drain(min..max);
					self.cursor_pos = min;
					self.selection_start = None;
					return true;
				}
			}
		}
		self.selection_start = None;
		false
	}

	fn char_index_at_x(&self, target_x: f32) -> usize {
		if target_x <= 0.0 || self.text.is_empty() {
			return 0;
		}
		let fface = FontManager::default_font();
		let mut last_w = 0.0;
		let mut last_idx = 0;
		for (idx, _) in self.text.char_indices() {
			if idx == 0 {
				continue;
			}
			let (w, _) = fface.measure_text(&self.text[..idx], self.font_size);
			if target_x <= w {
				let mid = last_w + (w - last_w) * 0.5;
				return if target_x < mid { last_idx } else { idx };
			}
			last_w = w;
			last_idx = idx;
		}
		let (total_w, _) = fface.measure_text(&self.text, self.font_size);
		if target_x <= total_w {
			let mid = last_w + (total_w - last_w) * 0.5;
			if target_x < mid { last_idx } else { self.text.len() }
		} else {
			self.text.len()
		}
	}
}

impl Widget for Edit {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(160.0, 40.0))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);

		let theme = ctx.theme;
		let monet = &theme.monet;

		let border_color = if ctx.focused || ctx.hovered {
			monet[MonetRole::Primary]
		} else {
			monet[MonetRole::PrimaryOutline]
		};

		let bounds = ctx.bounds;
		let has_title = !self.title.is_empty();

		let box_bounds = if has_title {
			Rect::new(
				bounds.x(),
				bounds.y() + 5.0,
				bounds.width(),
				(bounds.height() - 5.0).max(0.0),
			)
		} else {
			bounds
		};

		let stroke_w = ctx.theme.stroke.thick;
		let border_radius = self.border_radius.unwrap_or(ctx.theme.radius.small);
		canvas.draw(
			RectShape::new(box_bounds, Color::TRANSPARENT)
				.border(border_color, stroke_w)
				.radius(border_radius),
		);

		let fface = FontManager::default_font();

		if has_title {
			let (title_w, _) = canvas.measure_text(&self.title, 12.0);
			let notch_rect = Rect::new(
				box_bounds.x() + 10.0,
				box_bounds.y() - stroke_w * 0.5,
				title_w + 8.0,
				stroke_w + 1.0,
			);
			canvas.draw(RectShape::new(notch_rect, monet[MonetRole::SurfaceContainer]));

			let title_font = match self.font {
				Some(id) => Font::with_id(id, fface, 12.0),
				None => Font::new(fface, 12.0),
			};
			let title_color = if ctx.focused {
				monet[MonetRole::Primary]
			} else {
				monet[MonetRole::OnSurface]
			};
			canvas.draw(TextShape::new(
				&self.title,
				title_font,
				12.0,
				Vec2::new(box_bounds.x() + 14.0, bounds.y() - 2.0),
				title_color,
			));
		}

		let padding = 12.0;
		let view_w = (box_bounds.width() - (padding * 2.0)).max(0.0);
		let view_clip = Rect::new(box_bounds.x() + padding, box_bounds.y(), view_w, box_bounds.height());

		let cursor_idx = self.cursor_pos.min(self.text.len());
		let prefix = &self.text[..cursor_idx];
		let (cursor_offset, _) = canvas.measure_text(prefix, self.font_size);
		let (total_w, text_h) = canvas.measure_text(&self.text, self.font_size);

		let mut scroll = self.scroll_x.get();
		if cursor_offset < scroll {
			scroll = cursor_offset;
		} else if cursor_offset > scroll + view_w {
			scroll = cursor_offset - view_w;
		}
		if total_w <= view_w {
			scroll = 0.0;
		}
		self.scroll_x.set(scroll);

		canvas.push_clip(view_clip);

		let draw_x = box_bounds.x() + padding - scroll;
		let text_y = box_bounds.y() + (box_bounds.height() - text_h) * 0.5;

		if let Some(sel_start) = self.selection_start {
			let sel_start = sel_start.min(self.text.len());
			if sel_start != cursor_idx {
				let start_pos = sel_start.min(cursor_idx);
				let end_pos = sel_start.max(cursor_idx);
				let (sel_start_x, _) = canvas.measure_text(&self.text[..start_pos], self.font_size);
				let (sel_end_x, _) = canvas.measure_text(&self.text[..end_pos], self.font_size);
				let sel_rect = Rect::new(
					draw_x + sel_start_x,
					box_bounds.y() + 6.0,
					sel_end_x - sel_start_x,
					(box_bounds.height() - 12.0).max(0.0),
				);
				canvas.draw(RectShape::new(sel_rect, monet[MonetRole::PrimaryContainer]).radius(2.0));
			}
		}

		let text_font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};
		canvas.draw(TextShape::new(
			&self.text,
			text_font,
			self.font_size,
			Vec2::new(draw_x, text_y),
			monet[MonetRole::OnSurface],
		));

		if ctx.focused {
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

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		match event {
			InputEvent::PointerButton {
				button: MouseButton::Left,
				state,
				position,
			} => match state {
				ButtonState::Pressed => {
					self.is_dragging = true;
					let bounds = self.last_bounds.get();
					let padding = 12.0;
					let local_x = position.x - bounds.x() - padding + self.scroll_x.get();
					let idx = self.char_index_at_x(local_x);
					self.cursor_pos = idx;
					self.selection_start = Some(idx);
					ctx.request_redraw();
					EventResponse::Handled
				}
				ButtonState::Released => {
					if self.is_dragging {
						self.is_dragging = false;
						if self.selection_start == Some(self.cursor_pos) {
							self.selection_start = None;
						}
						ctx.request_redraw();
						EventResponse::Handled
					} else {
						EventResponse::Ignored
					}
				}
			},

			InputEvent::PointerMoved { position } => {
				if self.is_dragging {
					let bounds = self.last_bounds.get();
					let padding = 12.0;
					let local_x = position.x - bounds.x() - padding + self.scroll_x.get();
					let idx = self.char_index_at_x(local_x);
					self.cursor_pos = idx;
					ctx.request_redraw();
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::Click { position } => {
				let bounds = self.last_bounds.get();
				let padding = 12.0;
				let local_x = position.x - bounds.x() - padding + self.scroll_x.get();
				let idx = self.char_index_at_x(local_x);
				self.cursor_pos = idx;
				self.selection_start = None;
				ctx.request_redraw();
				EventResponse::Handled
			}

			InputEvent::Key {
				key_name,
				text,
				pressed,
			} => {
				if !pressed {
					return EventResponse::Ignored;
				}

				match key_name.as_str() {
					"ArrowLeft" => {
						self.cursor_pos = self.prev_char_boundary(self.cursor_pos);
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"ArrowRight" => {
						self.cursor_pos = self.next_char_boundary(self.cursor_pos);
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
						self.cursor_pos = self.text.len();
						self.selection_start = None;
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Backspace" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos > 0 {
							let prev = self.prev_char_boundary(self.cursor_pos);
							self.text.drain(prev..self.cursor_pos);
							self.cursor_pos = prev;
							changed = true;
						}
						if changed {
							self.dirty = true;
							if let Some(cb) = &mut self.on_changed {
								cb.call1(self.text.clone());
							}
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Delete" => {
						let mut changed = self.delete_selection();
						if !changed && self.cursor_pos < self.text.len() {
							let next = self.next_char_boundary(self.cursor_pos);
							self.text.drain(self.cursor_pos..next);
							changed = true;
						}
						if changed {
							self.dirty = true;
							if let Some(cb) = &mut self.on_changed {
								cb.call1(self.text.clone());
							}
						}
						ctx.request_redraw();
						EventResponse::Handled
					}
					"Enter" => {
						self.dirty = false;
						if let Some(cb) = &mut self.on_submit {
							cb.call1(self.text.clone());
						}
						EventResponse::Handled
					}
					_ => {
						if let Some(txt) = text {
							let filtered: String = txt.chars().filter(|c| !c.is_control()).collect();
							if !filtered.is_empty() {
								self.delete_selection();
								self.ensure_cursor_boundary();
								self.text.insert_str(self.cursor_pos, &filtered);
								self.cursor_pos += filtered.len();
								self.dirty = true;
								if let Some(cb) = &mut self.on_changed {
									cb.call1(self.text.clone());
								}
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}
						EventResponse::Ignored
					}
				}
			}

			InputEvent::FocusLost => {
				self.is_dragging = false;
				self.selection_start = None;
				if self.dirty {
					self.dirty = false;
					if let Some(cb) = &mut self.on_submit {
						cb.call1(self.text.clone());
					}
				}
				ctx.request_redraw();
				EventResponse::Handled
			}

			_ => EventResponse::Ignored,
		}
	}

	fn cursor(&self) -> Cursor {
		Cursor::Text
	}

	fn set_property(&mut self, name: &str, value: &PropertyValue, _ctx: &PropertyCtx) -> Result<(), nimbus_core::PropertyError> {
		match name {
			"text" => {
				self.text = value.expect_str()?.to_string();
				self.dirty = false;
				self.ensure_cursor_boundary();
			}
			"title" => self.title = value.expect_str()?.to_string(),
			"font_size" => self.font_size = value.expect_f32()?,
			"border_radius" => self.border_radius = Some(value.expect_f32()?),
			_ => return Err(nimbus_core::PropertyError::NotFound),
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
}

impl Edit {
	pub fn builder() -> EditBuilder {
		EditBuilder::new()
	}
}


#[derive(Default)]
pub struct EditBuilder {
	inner: Edit,
}

impl EditBuilder {
	pub fn new() -> Self {
		Self {
			inner: Edit::default(),
		}
	}

	pub fn text(mut self, text: impl Into<String>) -> Self {
		self.inner.text = text.into();
		self.inner.ensure_cursor_boundary();
		self
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.inner.title = title.into();
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

	pub fn border_radius(mut self, radius: f32) -> Self {
		self.inner.border_radius = Some(radius);
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

	pub fn build(self) -> Edit {
		self.inner
	}
}

impl From<EditBuilder> for Edit {
	fn from(builder: EditBuilder) -> Self {
		builder.build()
	}
}
