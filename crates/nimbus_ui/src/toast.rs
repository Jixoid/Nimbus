/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::cell::Cell;
use std::time::{Duration, Instant};

use nimbus_core::{
	Anchor, Animated, BoxConstraints, DrawCtx, Easing, EventCtx, EventResponse, InputEvent,
	LayoutCtx, ObjectID, UiTree, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::MonetRole;


const TOAST_MARGIN: f32 = 16.0;
const TOAST_SPACING: f32 = 10.0;
const TOAST_PADDING_X: f32 = 14.0;
const TOAST_PADDING_Y: f32 = 12.0;


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ToastKind {
	#[default]
	Info,
	Success,
	Warning,
	Error,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
	#[default]
	BottomRight,
	BottomLeft,
	BottomCenter,
	TopRight,
	TopLeft,
	TopCenter,
}

impl ToastPosition {
	fn to_anchor(self) -> Anchor {
		match self {
			Self::BottomRight => Anchor::BottomRight,
			Self::BottomLeft => Anchor::BottomLeft,
			Self::BottomCenter => Anchor::BottomCenter,
			Self::TopRight => Anchor::TopRight,
			Self::TopLeft => Anchor::TopLeft,
			Self::TopCenter => Anchor::TopCenter,
		}
	}

	fn is_top(self) -> bool {
		matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
	}

	fn slide_dir_x(self) -> f32 {
		match self {
			Self::TopRight | Self::BottomRight => 1.0,
			Self::TopLeft | Self::BottomLeft => -1.0,
			Self::TopCenter | Self::BottomCenter => 0.0,
		}
	}
}


#[derive(Clone, Debug)]
pub struct Toast {
	pub title: Option<String>,
	pub message: String,
	pub kind: ToastKind,
	pub duration: Duration,
	pub position: ToastPosition,
	pub closable: bool,
	pub width: f32,
	pub font: Option<FontId>,
	pub font_size: f32,
}

impl Default for Toast {
	fn default() -> Self {
		Self {
			title: None,
			message: String::new(),
			kind: ToastKind::Info,
			duration: Duration::from_millis(3500),
			position: ToastPosition::BottomRight,
			closable: true,
			width: 320.0,
			font: None,
			font_size: 14.0,
		}
	}
}

impl Toast {
	pub fn new(message: impl Into<String>) -> Self {
		Self {
			message: message.into(),
			..Default::default()
		}
	}

	pub fn builder() -> ToastBuilder {
		ToastBuilder::new()
	}

	pub fn show(tree: &mut UiTree, message: impl Into<String>) -> ObjectID {
		Self::push(tree, Self::new(message))
	}

	pub fn push(tree: &mut UiTree, toast: Toast) -> ObjectID {
		let position = toast.position;
		let existing_host = tree.roots.iter().copied().find(|&root_id| {
			tree.get_widget::<ToastHost>(root_id)
				.is_some_and(|h| h.position == position)
		});

		let host_id = if let Some(id) = existing_host {
			id
		} else {
			let id = tree.insert_root(ToastHost::new(position));
			tree.set_anchor(id, Some(position.to_anchor()));
			id
		};

		if let Some(host) = tree.get_widget_mut::<ToastHost>(host_id) {
			host.push(toast);
		}

		tree.needs_layout = true;
		tree.needs_redraw = true;
		host_id
	}
}


#[derive(Default)]
pub struct ToastBuilder {
	inner: Toast,
}

impl ToastBuilder {
	pub fn new() -> Self {
		Self {
			inner: Toast::default(),
		}
	}

	pub fn title(mut self, title: impl Into<String>) -> Self {
		let s = title.into();
		self.inner.title = if s.is_empty() { None } else { Some(s) };
		self
	}

	pub fn message(mut self, message: impl Into<String>) -> Self {
		self.inner.message = message.into();
		self
	}

	pub fn kind(mut self, kind: ToastKind) -> Self {
		self.inner.kind = kind;
		self
	}

	pub fn duration(mut self, duration: Duration) -> Self {
		self.inner.duration = duration;
		self
	}

	pub fn position(mut self, position: ToastPosition) -> Self {
		self.inner.position = position;
		self
	}

	pub fn closable(mut self, closable: bool) -> Self {
		self.inner.closable = closable;
		self
	}

	pub fn width(mut self, width: f32) -> Self {
		self.inner.width = width.max(180.0);
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

	pub fn build(self) -> Toast {
		self.inner
	}

	pub fn show(self, tree: &mut UiTree) -> ObjectID {
		Toast::push(tree, self.inner)
	}
}

impl From<ToastBuilder> for Toast {
	fn from(builder: ToastBuilder) -> Self {
		builder.build()
	}
}


struct ToastEntry {
	toast: Toast,
	progress: Animated<f32>,
	elapsed_secs: Cell<f32>,
	last_tick: Cell<Instant>,
	dismissing: Cell<bool>,
	card_height: f32,
}

impl ToastEntry {
	fn new(toast: Toast) -> Self {
		let mut progress = Animated::new(0.0);
		progress.set_target(1.0, Duration::from_millis(200), Easing::EmphasizedDecelerate);
		Self {
			toast,
			progress,
			elapsed_secs: Cell::new(0.0),
			last_tick: Cell::new(Instant::now()),
			dismissing: Cell::new(false),
			card_height: 52.0,
		}
	}

	fn begin_dismiss(&self) {
		if !self.dismissing.get() {
			self.dismissing.set(true);
		}
	}
}

fn draw_close_x(canvas: &mut Canvas2D, center: Vec2<f32>, half_extent: f32, thickness: f32, color: Color) {
	let steps = 10usize;
	let r = thickness * 0.5;
	for i in 0..=steps {
		let t = (i as f32 / steps as f32) * 2.0 - 1.0;
		let offset = t * half_extent;

		canvas.draw(
			RectShape::new(
				Rect::new(center.x + offset - r, center.y + offset - r, thickness, thickness),
				color,
			)
			.radius(r),
		);
		canvas.draw(
			RectShape::new(
				Rect::new(center.x + offset - r, center.y - offset - r, thickness, thickness),
				color,
			)
			.radius(r),
		);
	}
}


pub struct ToastHost {
	pub position: ToastPosition,
	pub max_visible: usize,
	entries: Vec<ToastEntry>,
	hovered_index: Option<usize>,
	hovered_close: Option<usize>,
	last_bounds: Cell<Rect<f32>>,
}

impl ToastHost {
	pub fn new(position: ToastPosition) -> Self {
		Self {
			position,
			max_visible: 5,
			entries: Vec::new(),
			hovered_index: None,
			hovered_close: None,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}

	pub fn push(&mut self, toast: Toast) {
		self.entries.push(ToastEntry::new(toast));
		if self.entries.len() > self.max_visible {
			let excess = self.entries.len() - self.max_visible;
			for entry in self.entries.iter_mut().take(excess) {
				if !entry.dismissing.get() {
					entry.dismissing.set(true);
					entry
						.progress
						.set_target(0.0, Duration::from_millis(160), Easing::Accelerate);
				}
			}
		}
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	fn card_rects(&self, bounds: Rect<f32>) -> Vec<Rect<f32>> {
		let mut rects = Vec::with_capacity(self.entries.len());
		if self.entries.is_empty() {
			return rects;
		}

		let inner_w = (bounds.width() - TOAST_MARGIN * 2.0).max(0.0);
		let mut y = bounds.y() + TOAST_MARGIN;

		if self.position.is_top() {
			for (i, entry) in self.entries.iter().enumerate() {
				if i > 0 {
					y += TOAST_SPACING;
				}
				let w = entry.toast.width.min(inner_w);
				let x = match self.position {
					ToastPosition::TopLeft => bounds.x() + TOAST_MARGIN,
					ToastPosition::TopCenter => bounds.x() + (bounds.width() - w) * 0.5,
					_ => bounds.max.x - TOAST_MARGIN - w,
				};
				rects.push(Rect::new(x, y, w, entry.card_height));
				y += entry.card_height;
			}
		} else {
			let mut cursor_y = bounds.max.y - TOAST_MARGIN;
			rects.resize(self.entries.len(), Rect::ZERO);
			for (i, entry) in self.entries.iter().enumerate().rev() {
				cursor_y -= entry.card_height;
				let w = entry.toast.width.min(inner_w);
				let x = match self.position {
					ToastPosition::BottomLeft => bounds.x() + TOAST_MARGIN,
					ToastPosition::BottomCenter => bounds.x() + (bounds.width() - w) * 0.5,
					_ => bounds.max.x - TOAST_MARGIN - w,
				};
				rects[i] = Rect::new(x, cursor_y, w, entry.card_height);
				cursor_y -= TOAST_SPACING;
			}
		}

		rects
	}

	fn close_rect(card: Rect<f32>) -> Rect<f32> {
		let size = 22.0;
		Rect::new(card.max.x - size - 8.0, card.y() + 8.0, size, size)
	}
}

impl Widget for ToastHost {
	fn layout(&mut self, ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		for entry in &mut self.entries {
			if entry.dismissing.get() && *entry.progress.target() > 0.01 {
				entry
					.progress
					.set_target(0.0, Duration::from_millis(180), Easing::Accelerate);
			}
		}

		self.entries.retain(|e| {
			!(e.dismissing.get() && !e.progress.is_animating() && e.progress.value() <= 0.001)
		});

		if self.entries.is_empty() {
			self.hovered_index = None;
			self.hovered_close = None;
			return Vec2::ZERO;
		}

		let mut max_w = 0.0f32;
		let mut total_h = 0.0f32;

		for (i, entry) in self.entries.iter_mut().enumerate() {
			let (_, msg_h) = ctx.measure_text_with_font(
				&entry.toast.message,
				entry.toast.font,
				entry.toast.font_size,
			);
			let title_h = if let Some(title) = &entry.toast.title {
				let (_, th) = ctx.measure_text_with_font(
					title,
					entry.toast.font,
					entry.toast.font_size + 1.0,
				);
				th + 4.0
			} else {
				0.0
			};

			let h = (TOAST_PADDING_Y * 2.0 + title_h + msg_h + 4.0).max(48.0).ceil();
			entry.card_height = h;
			max_w = max_w.max(entry.toast.width);
			if i > 0 {
				total_h += TOAST_SPACING;
			}
			total_h += h;
		}

		let desired = Vec2::new(max_w + TOAST_MARGIN * 2.0, total_h + TOAST_MARGIN * 2.0);
		constraints.constrain(desired)
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		if self.entries.is_empty() {
			return;
		}

		let now = Instant::now();
		let monet = &ctx.theme.monet;
		let rects = self.card_rects(ctx.bounds);
		let fface = FontManager::default_font();

		let mut needs_redraw = false;
		let mut needs_layout = false;

		for (i, (entry, base_card)) in self.entries.iter().zip(rects.iter()).enumerate() {
			let dt = now.duration_since(entry.last_tick.get()).as_secs_f32().min(0.1);
			entry.last_tick.set(now);

			let is_hovered = self.hovered_index == Some(i);
			let dur_secs = entry.toast.duration.as_secs_f32();

			if !entry.dismissing.get() && dur_secs > 0.0 {
				if !is_hovered {
					let next_elapsed = entry.elapsed_secs.get() + dt;
					entry.elapsed_secs.set(next_elapsed);
					if next_elapsed >= dur_secs {
						entry.begin_dismiss();
						needs_layout = true;
					}
				}
				needs_redraw = true;
			}

			if entry.progress.is_animating() {
				needs_redraw = true;
			} else if entry.dismissing.get() && entry.progress.value() <= 0.001 {
				needs_layout = true;
				continue;
			}

			let t = entry.progress.value().clamp(0.0, 1.0);
			if t <= 0.001 {
				continue;
			}

			let slide_x = self.position.slide_dir_x() * (1.0 - t) * 28.0;
			let slide_y = if self.position.slide_dir_x() == 0.0 {
				let dir_y = if self.position.is_top() { -1.0 } else { 1.0 };
				dir_y * (1.0 - t) * 18.0
			} else {
				0.0
			};

			let card = Rect::new(
				base_card.x() + slide_x,
				base_card.y() + slide_y,
				base_card.width(),
				base_card.height(),
			);

			let accent = match entry.toast.kind {
				ToastKind::Info => monet[MonetRole::Primary],
				ToastKind::Success => Color::from_oklch(0.72, 0.14, 145.0),
				ToastKind::Warning => Color::from_oklch(0.78, 0.14, 75.0),
				ToastKind::Error => monet[MonetRole::Error],
			};

			let mut bg = monet[MonetRole::SurfaceContainerHigh];
			bg.a *= t;
			let mut border = monet[MonetRole::SurfaceOutlineVariant];
			border.a *= t;
			let shadow = Color::new(0.0, 0.0, 0.0, 0.36 * t);

			canvas.draw(
				RectShape::new(card, bg)
					.radius(ctx.theme.radius.medium)
					.border(border, ctx.theme.stroke.thin)
					.shadow(shadow, Vec2::new(0.0, 4.0), 14.0),
			);

			// Left severity accent bar
			let mut bar_col = accent;
			bar_col.a *= t;
			let bar_rect = Rect::new(
				card.x() + 6.0,
				card.y() + 8.0,
				3.5,
				(card.height() - 16.0).max(0.0),
			);
			canvas.draw(RectShape::new(bar_rect, bar_col).radius(2.0));

			// Countdown progress line along bottom edge
			if dur_secs > 0.0 {
				let remaining = (1.0 - entry.elapsed_secs.get() / dur_secs).clamp(0.0, 1.0);
				if remaining > 0.005 {
					let prog_w = (card.width() - 24.0).max(0.0) * remaining;
					let mut prog_col = accent;
					prog_col.a *= 0.65 * t;
					canvas.draw(
						RectShape::new(
							Rect::new(card.x() + 12.0, card.max.y - 3.0, prog_w, 2.0),
							prog_col,
						)
						.radius(1.0),
					);
				}
			}

			// Text area
			let text_x = card.x() + 18.0;
			let right_pad = if entry.toast.closable { 34.0 } else { TOAST_PADDING_X };
			let text_w = (card.width() - 18.0 - right_pad).max(0.0);
			let text_clip = Rect::new(text_x, card.y(), text_w, card.height());

			canvas.push_clip(text_clip);

			let mut text_y = card.y() + TOAST_PADDING_Y;
			if let Some(title) = &entry.toast.title {
				let title_size = entry.toast.font_size + 1.0;
				let title_font = match entry.toast.font {
					Some(id) => Font::with_id(id, fface, title_size),
					None => Font::new(fface, title_size),
				};
				let mut title_col = monet[MonetRole::OnSurface];
				title_col.a *= t;
				let (_, th) = canvas.measure_text(title, title_size);
				canvas.draw(TextShape::new(
					title,
					title_font,
					title_size,
					Vec2::new(text_x, text_y),
					title_col,
				));
				text_y += th + 4.0;
			}

			let msg_font = match entry.toast.font {
				Some(id) => Font::with_id(id, fface, entry.toast.font_size),
				None => Font::new(fface, entry.toast.font_size),
			};
			let mut msg_col = if entry.toast.title.is_some() {
				monet[MonetRole::OnSurfaceVariant]
			} else {
				monet[MonetRole::OnSurface]
			};
			msg_col.a *= t;
			canvas.draw(TextShape::new(
				&entry.toast.message,
				msg_font,
				entry.toast.font_size,
				Vec2::new(text_x, text_y),
				msg_col,
			));

			canvas.pop_clip();

			// Close button
			if entry.toast.closable {
				let close_r = Self::close_rect(card);
				let close_hov = self.hovered_close == Some(i);
				if close_hov {
					let mut hbg = monet[MonetRole::SurfaceContainerHighest];
					hbg.a *= t;
					canvas.draw(RectShape::new(close_r, hbg).radius(close_r.height() * 0.5));
				}
				let mut icon_col = if close_hov {
					monet[MonetRole::OnSurface]
				} else {
					monet[MonetRole::OnSurfaceVariant]
				};
				icon_col.a *= t;
				draw_close_x(canvas, close_r.center(), 3.8, 1.35, icon_col);
			}
		}

		if needs_layout {
			ctx.request_layout();
		}
		if needs_redraw {
			ctx.request_redraw();
		}
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let rects = self.card_rects(bounds);

		match event {
			InputEvent::PointerMoved { position } => {
				let mut new_hov = None;
				let mut new_close = None;
				for (i, &card) in rects.iter().enumerate() {
					if card.contains_point(*position) {
						new_hov = Some(i);
						if self.entries[i].toast.closable && Self::close_rect(card).contains_point(*position) {
							new_close = Some(i);
						}
						break;
					}
				}
				if new_hov != self.hovered_index || new_close != self.hovered_close {
					self.hovered_index = new_hov;
					self.hovered_close = new_close;
					ctx.request_redraw();
				}
				if new_hov.is_some() {
					EventResponse::Handled
				} else {
					EventResponse::Ignored
				}
			}

			InputEvent::PointerLeft => {
				if self.hovered_index.is_some() || self.hovered_close.is_some() {
					self.hovered_index = None;
					self.hovered_close = None;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::Click { position } => {
				for (i, &card) in rects.iter().enumerate() {
					if card.contains_point(*position) {
						if self.entries[i].toast.closable {
							self.entries[i].dismissing.set(true);
							self.entries[i].progress.set_target(
								0.0,
								Duration::from_millis(160),
								Easing::Accelerate,
							);
							ctx.request_layout();
							ctx.request_redraw();
						}
						return EventResponse::Handled;
					}
				}
				EventResponse::Ignored
			}

			_ => EventResponse::Ignored,
		}
	}
}
