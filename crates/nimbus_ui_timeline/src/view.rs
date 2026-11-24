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
	BoxConstraints, ButtonState, DrawCtx, EventCtx, EventResponse, InputEvent, IntoMethod,
	LayoutCtx, MethodCallback, MouseButton, PropertyCtx, PropertyError, PropertyValue, Widget,
};
use nimbus_math::{Color, Rect, Vec2};
use nimbus_render2d::{Canvas2D, CurveShape, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::model::{Keyframe, TimelineTrack, parse_tracks_from_property};


const RULER_HEIGHT: f32 = 30.0;
const ROW_HEIGHT: f32 = 32.0;
const HEADER_WIDTH: f32 = 160.0;
const KF_HIT_RADIUS: f32 = 13.0;
const CLIP_EDGE_WIDTH: f32 = 9.0;


#[derive(Copy, Clone, Debug, PartialEq)]
enum DragAction {
	ScrubPlayhead,
	Pan {
		start_x: f32,
		start_scroll: f32,
	},
	MoveKeyframe {
		track_idx: usize,
		kf_idx: usize,
	},
	MoveClip {
		track_idx: usize,
		clip_idx: usize,
		grab_offset_sec: f32,
	},
	ResizeClipLeft {
		track_idx: usize,
		clip_idx: usize,
		orig_end: f32,
	},
	ResizeClipRight {
		track_idx: usize,
		clip_idx: usize,
	},
}


pub struct TimelineView {
	pub tracks: Vec<TimelineTrack>,
	pub duration: f32,
	pub fps: f32,
	pub playhead: f32,
	pub playing: bool,
	pub looping: bool,
	pub snap: bool,
	pub pixels_per_sec: f32,
	pub scroll_x: f32,
	pub header_width: f32,
	pub row_height: f32,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,

	pub selected_track: Option<usize>,
	pub selected_keyframe: Option<(usize, usize)>,

	pub on_time_changed: Option<MethodCallback>,
	pub on_keyframe_changed: Option<MethodCallback>,
	pub on_track_selected: Option<MethodCallback>,

	drag: Option<DragAction>,
	hovered_kf: Option<(usize, usize)>,
	last_pointer: Vec2<f32>,
	last_click: Option<(Instant, Vec2<f32>)>,
	last_tick: Cell<Option<Instant>>,
	playhead_cell: Cell<f32>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for TimelineView {
	fn default() -> Self {
		Self {
			tracks: Vec::new(),
			duration: 6.0,
			fps: 30.0,
			playhead: 0.0,
			playing: false,
			looping: true,
			snap: true,
			pixels_per_sec: 110.0,
			scroll_x: 0.0,
			header_width: HEADER_WIDTH,
			row_height: ROW_HEIGHT,
			background: None,
			corner_radius: None,
			font: None,
			font_size: 12.0,
			selected_track: Some(0),
			selected_keyframe: None,
			on_time_changed: None,
			on_keyframe_changed: None,
			on_track_selected: None,
			drag: None,
			hovered_kf: None,
			last_pointer: Vec2::ZERO,
			last_click: None,
			last_tick: Cell::new(None),
			playhead_cell: Cell::new(0.0),
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl TimelineView {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> TimelineViewBuilder {
		TimelineViewBuilder::new()
	}

	pub fn current_time(&self) -> f32 {
		if self.playing {
			self.playhead_cell.get()
		} else {
			self.playhead
		}
	}

	pub fn set_playhead(&mut self, t: f32) {
		let clamped = t.clamp(0.0, self.duration.max(0.1));
		self.playhead = clamped;
		self.playhead_cell.set(clamped);
	}

	pub fn set_playing(&mut self, playing: bool) {
		if self.playing && !playing {
			self.playhead = self.playhead_cell.get();
			self.last_tick.set(None);
		} else if !self.playing && playing {
			self.playhead_cell.set(self.playhead);
			self.last_tick.set(Some(Instant::now()));
		}
		self.playing = playing;
	}

	fn time_to_x(&self, bounds: Rect<f32>, t: f32) -> f32 {
		bounds.x() + self.header_width + t * self.pixels_per_sec - self.scroll_x
	}

	fn x_to_time(&self, bounds: Rect<f32>, x: f32) -> f32 {
		((x - bounds.x() - self.header_width + self.scroll_x) / self.pixels_per_sec.max(1.0))
			.clamp(0.0, self.duration.max(0.1))
	}

	fn snap_time(&self, t: f32) -> f32 {
		let clamped = t.clamp(0.0, self.duration.max(0.1));
		if !self.snap || self.fps <= 0.0 {
			return clamped;
		}
		let step = 1.0 / self.fps;
		(clamped / step).round() * step
	}

	fn track_y(&self, bounds: Rect<f32>, idx: usize) -> f32 {
		bounds.y() + RULER_HEIGHT + idx as f32 * self.row_height
	}

	fn track_at_y(&self, bounds: Rect<f32>, y: f32) -> Option<usize> {
		if y < bounds.y() + RULER_HEIGHT {
			return None;
		}
		let idx = ((y - bounds.y() - RULER_HEIGHT) / self.row_height).floor() as usize;
		if idx < self.tracks.len() {
			Some(idx)
		} else {
			None
		}
	}

	fn hit_keyframe(&self, bounds: Rect<f32>, pos: Vec2<f32>) -> Option<(usize, usize)> {
		for (ti, track) in self.tracks.iter().enumerate() {
			let cy = self.track_y(bounds, ti) + self.row_height * 0.5;
			if (pos.y - cy).abs() > KF_HIT_RADIUS + 2.0 {
				continue;
			}
			for (ki, kf) in track.keyframes.iter().enumerate() {
				let cx = self.time_to_x(bounds, kf.time);
				let d = pos - Vec2::new(cx, cy);
				if (d.x * d.x + d.y * d.y).sqrt() <= KF_HIT_RADIUS {
					return Some((ti, ki));
				}
			}
		}
		None
	}

	fn header_btn_rects(&self, bounds: Rect<f32>, ti: usize) -> (Rect<f32>, Rect<f32>) {
		let y = self.track_y(bounds, ti);
		let btn_s = 18.0;
		let cy = y + (self.row_height - btn_s) * 0.5;
		let right = bounds.x() + self.header_width - 8.0;
		let lock_r = Rect::new(right - btn_s, cy, btn_s, btn_s);
		let mute_r = Rect::new(right - btn_s * 2.0 - 4.0, cy, btn_s, btn_s);
		(mute_r, lock_r)
	}
}

fn draw_diamond(canvas: &mut Canvas2D, center: Vec2<f32>, r: f32, fill: Color, border: Color) {
	// Crisp rotated-square diamond via horizontal scanlines + border outline
	let steps = (r * 2.0).ceil().max(6.0) as i32;
	for i in 0..=steps {
		let dy = -r + (i as f32 / steps as f32) * (r * 2.0);
		let half_w = (r - dy.abs()).max(0.8);
		canvas.draw(RectShape::new(
			Rect::new(center.x - half_w, center.y + dy - 0.5, half_w * 2.0, 1.2),
			fill,
		));
	}
	let pts = [
		Vec2::new(center.x, center.y - r),
		Vec2::new(center.x + r, center.y),
		Vec2::new(center.x, center.y + r),
		Vec2::new(center.x - r, center.y),
	];
	for i in 0..4 {
		canvas.draw(CurveShape::line(pts[i], pts[(i + 1) % 4], 1.2, border));
	}
}

impl Widget for TimelineView {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		let min_h = RULER_HEIGHT + (self.tracks.len().max(3) as f32) * self.row_height + 4.0;
		let w = if constraints.max_width.is_finite() {
			constraints.max_width
		} else {
			640.0
		};
		let h = if constraints.max_height.is_finite() && constraints.min_height >= constraints.max_height {
			constraints.max_height
		} else {
			min_h.min(constraints.max_height)
		};
		constraints.constrain(Vec2::new(w, h))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 4.0 || bounds.height() <= 4.0 {
			return;
		}

		// Advance playhead smoothly when playing
		let cur_time = if self.playing {
			let now = Instant::now();
			let prev = self.last_tick.get().unwrap_or(now);
			let dt = now.duration_since(prev).as_secs_f32().min(0.1);
			self.last_tick.set(Some(now));
			let mut t = self.playhead_cell.get() + dt;
			let dur = self.duration.max(0.1);
			if t > dur {
				t = if self.looping { t % dur } else { dur };
			}
			self.playhead_cell.set(t);
			if let Some(cb) = &self.on_time_changed {
				cb.call1(t);
			}
			ctx.request_redraw();
			t
		} else {
			self.playhead_cell.set(self.playhead);
			self.playhead
		};

		let theme = ctx.theme;
		let monet = &theme.monet;
		let radius = self.corner_radius.unwrap_or(theme.radius.medium);
		let bg = self
			.background
			.map(|c| c.get(monet))
			.unwrap_or_else(|| monet[MonetRole::SurfaceContainer]);

		canvas.draw(
			RectShape::new(bounds, bg)
				.radius(radius)
				.border(monet[MonetRole::SurfaceOutlineVariant], theme.stroke.thin),
		);

		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		canvas.push_clip(bounds);

		// Top ruler bar & corner header
		let ruler_rect = Rect::new(
			bounds.x() + self.header_width,
			bounds.y(),
			(bounds.width() - self.header_width).max(0.0),
			RULER_HEIGHT,
		);
		let corner_rect = Rect::new(bounds.x(), bounds.y(), self.header_width, RULER_HEIGHT);
		canvas.draw(RectShape::new(
			Rect::new(bounds.x(), bounds.y(), bounds.width(), RULER_HEIGHT),
			monet[MonetRole::SurfaceContainerHigh],
		));

		// Corner time readout
		let frame_num = (cur_time * self.fps).round() as i32;
		let time_label = format!("{:.2}s  (f{})", cur_time, frame_num);
		let (_, th) = canvas.measure_text(&time_label, self.font_size);
		canvas.draw(TextShape::new(
			&time_label,
			font,
			self.font_size,
			Vec2::new(corner_rect.x() + 10.0, corner_rect.center().y - th * 0.5),
			monet[MonetRole::Primary],
		));

		// Left header column background
		let header_body = Rect::new(
			bounds.x(),
			bounds.y() + RULER_HEIGHT,
			self.header_width,
			(bounds.height() - RULER_HEIGHT).max(0.0),
		);
		canvas.draw(RectShape::new(header_body, monet[MonetRole::SurfaceContainerLow]));

		// Track rows (left header + alternating lane tint)
		let mut grid_tint = monet[MonetRole::OnSurface];
		grid_tint.a = 0.035;
		let mut sep_col = monet[MonetRole::SurfaceOutlineVariant];
		sep_col.a = 0.55;

		for (ti, track) in self.tracks.iter().enumerate() {
			let y = self.track_y(bounds, ti);
			if y > bounds.max.y {
				break;
			}
			let is_selected = self.selected_track == Some(ti);
			if is_selected {
				let mut sel = monet[MonetRole::Primary];
				sel.a = 0.12;
				canvas.draw(RectShape::new(
					Rect::new(bounds.x(), y, bounds.width(), self.row_height),
					sel,
				));
			} else if ti % 2 == 1 {
				canvas.draw(RectShape::new(
					Rect::new(bounds.x() + self.header_width, y, ruler_rect.width(), self.row_height),
					grid_tint,
				));
			}

			// Bottom divider
			canvas.draw(RectShape::new(
				Rect::new(bounds.x(), y + self.row_height - 1.0, bounds.width(), 1.0),
				sep_col,
			));

			// Track color dot + name
			let tcol = track.resolved_color(ti, monet);
			canvas.draw(
				RectShape::new(Rect::new(bounds.x() + 10.0, y + self.row_height * 0.5 - 4.0, 8.0, 8.0), tcol)
					.radius(4.0),
			);
			let name_col = if track.muted {
				monet[MonetRole::OnSurfaceVariant]
			} else {
				monet[MonetRole::OnSurface]
			};
			let (_, nth) = canvas.measure_text(&track.name, self.font_size);
			canvas.draw(TextShape::new(
				&track.name,
				font,
				self.font_size,
				Vec2::new(bounds.x() + 24.0, y + (self.row_height - nth) * 0.5),
				name_col,
			));

			// Mute (M) & Lock (L) buttons
			let (mute_r, lock_r) = self.header_btn_rects(bounds, ti);
			for (btn_r, active, lbl) in [(mute_r, track.muted, "M"), (lock_r, track.locked, "L")] {
				let bcol = if active {
					monet[MonetRole::Primary]
				} else {
					monet[MonetRole::SurfaceContainerHigh]
				};
				let lcol = if active {
					monet[MonetRole::OnPrimary]
				} else {
					monet[MonetRole::OnSurfaceVariant]
				};
				canvas.draw(RectShape::new(btn_r, bcol).radius(theme.radius.xsmall));
				let (lw, lh) = canvas.measure_text(lbl, 10.0);
				let small_font = match self.font {
					Some(id) => Font::with_id(id, fface, 10.0),
					None => Font::new(fface, 10.0),
				};
				canvas.draw(TextShape::new(
					lbl,
					small_font,
					10.0,
					Vec2::new(btn_r.center().x - lw * 0.5, btn_r.center().y - lh * 0.5),
					lcol,
				));
			}
		}

		// Clip right timeline area
		let timeline_area = Rect::new(
			bounds.x() + self.header_width,
			bounds.y(),
			(bounds.width() - self.header_width).max(0.0),
			bounds.height(),
		);
		canvas.push_clip(timeline_area);

		// Ruler ticks + vertical grid lines
		let major_step = if self.pixels_per_sec >= 140.0 {
			0.5
		} else if self.pixels_per_sec >= 60.0 {
			1.0
		} else {
			2.0
		};
		let minor_step = major_step * 0.25;
		let start_t = (self.scroll_x / self.pixels_per_sec).floor().max(0.0);
		let end_t = ((self.scroll_x + timeline_area.width()) / self.pixels_per_sec)
			.ceil()
			.min(self.duration + major_step);

		let mut major_line = monet[MonetRole::OnSurface];
		major_line.a = 0.14;
		let mut minor_line = monet[MonetRole::OnSurface];
		minor_line.a = 0.05;

		let num_steps = ((end_t - start_t) / minor_step).ceil().max(0.0) as i32;
		for i in 0..=num_steps {
			let t = start_t + i as f32 * minor_step;
			if t > self.duration + 1e-3 {
				break;
			}
			let x = self.time_to_x(bounds, t);
			let is_major = i % 4 == 0;
			let tick_h = if is_major { 11.0 } else { 5.0 };
			canvas.draw(RectShape::new(
				Rect::new(x, bounds.y() + RULER_HEIGHT - tick_h, 1.0, tick_h),
				monet[MonetRole::OnSurfaceVariant],
			));
			canvas.draw(RectShape::new(
				Rect::new(
					x,
					bounds.y() + RULER_HEIGHT,
					1.0,
					(bounds.height() - RULER_HEIGHT).max(0.0),
				),
				if is_major { major_line } else { minor_line },
			));
			if is_major {
				let lbl = format!("{:.1}s", t);
				canvas.draw(TextShape::new(
					&lbl,
					font,
					10.5,
					Vec2::new(x + 4.0, bounds.y() + 4.0),
					monet[MonetRole::OnSurfaceVariant],
				));
			}
		}

		// Duration end marker
		let end_x = self.time_to_x(bounds, self.duration);
		if end_x < bounds.max.x {
			let mut dim = monet[MonetRole::Shadow];
			dim.a = 0.14;
			canvas.draw(RectShape::new(
				Rect::new(
					end_x,
					bounds.y() + RULER_HEIGHT,
					(bounds.max.x - end_x).max(0.0),
					(bounds.height() - RULER_HEIGHT).max(0.0),
				),
				dim,
			));
		}

		// Clips & Keyframes
		for (ti, track) in self.tracks.iter().enumerate() {
			let y = self.track_y(bounds, ti);
			let cy = y + self.row_height * 0.5;
			let tcol = track.resolved_color(ti, monet);

			for clip in &track.clips {
				let x0 = self.time_to_x(bounds, clip.start);
				let x1 = self.time_to_x(bounds, clip.end());
				let cr = Rect::new(x0, y + 4.0, (x1 - x0).max(6.0), (self.row_height - 8.0).max(8.0));
				let mut cfill = clip.color.map(|c| c.get(monet)).unwrap_or(tcol);
				cfill.a = if track.muted { 0.22 } else { 0.38 };
				canvas.draw(
					RectShape::new(cr, cfill)
						.radius(theme.radius.xsmall)
						.border(tcol, theme.stroke.thin),
				);
				if cr.width() > 28.0 {
					let (_, lh) = canvas.measure_text(&clip.label, 11.0);
					canvas.draw(TextShape::new(
						&clip.label,
						font,
						11.0,
						Vec2::new(cr.x() + 6.0, cr.center().y - lh * 0.5),
						monet[MonetRole::OnSurface],
					));
				}
			}

			for (ki, kf) in track.keyframes.iter().enumerate() {
				let cx = self.time_to_x(bounds, kf.time);
				let is_sel = self.selected_keyframe == Some((ti, ki));
				let is_hov = self.hovered_kf == Some((ti, ki));
				let r = if is_sel || is_hov { 6.5 } else { 5.2 };
				let fill = if is_sel {
					monet[MonetRole::Primary]
				} else if track.muted {
					monet[MonetRole::SurfaceOutline]
				} else {
					tcol
				};
				let border = if is_sel {
					monet[MonetRole::OnPrimary]
				} else {
					monet[MonetRole::SurfaceContainerLowest]
				};
				draw_diamond(canvas, Vec2::new(cx, cy), r, fill, border);
			}
		}

		// Playhead line + top handle
		let px = self.time_to_x(bounds, cur_time);
		let ph_col = monet[MonetRole::Error];
		canvas.draw(RectShape::new(
			Rect::new(px - 7.0, bounds.y() + 2.0, 14.0, 15.0),
			ph_col,
		).radius(theme.radius.xsmall));
		canvas.draw(RectShape::new(
			Rect::new(px - 1.0, bounds.y() + 15.0, 2.0, (bounds.height() - 15.0).max(0.0)),
			ph_col,
		));

		canvas.pop_clip();

		// Vertical separator between header & timeline
		canvas.draw(RectShape::new(
			Rect::new(bounds.x() + self.header_width - 1.0, bounds.y(), 1.0, bounds.height()),
			monet[MonetRole::SurfaceOutlineVariant],
		));
		canvas.draw(RectShape::new(
			Rect::new(bounds.x(), bounds.y() + RULER_HEIGHT - 1.0, bounds.width(), 1.0),
			monet[MonetRole::SurfaceOutlineVariant],
		));

		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();

		match event {
			InputEvent::PointerMoved { position } => {
				self.last_pointer = *position;
				if let Some(action) = self.drag {
					match action {
						DragAction::ScrubPlayhead => {
							let t = self.snap_time(self.x_to_time(bounds, position.x));
							self.set_playhead(t);
							if let Some(cb) = &self.on_time_changed {
								cb.call1(t);
							}
							ctx.request_redraw();
						}
						DragAction::Pan { start_x, start_scroll } => {
							self.scroll_x = (start_scroll - (position.x - start_x)).max(0.0);
							ctx.request_redraw();
						}
						DragAction::MoveKeyframe { track_idx, kf_idx } => {
							let t = self.snap_time(self.x_to_time(bounds, position.x));
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(kf) = track.keyframes.get_mut(kf_idx)
							{
								kf.time = t;
								if let Some(cb) = &self.on_keyframe_changed {
									cb.call2(track_idx as i32, t);
								}
								ctx.request_redraw();
							}
						}
						DragAction::MoveClip {
							track_idx,
							clip_idx,
							grab_offset_sec,
						} => {
							let t = self.snap_time(self.x_to_time(bounds, position.x) - grab_offset_sec);
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(clip) = track.clips.get_mut(clip_idx)
							{
								clip.start = t.clamp(0.0, (self.duration - clip.duration).max(0.0));
								ctx.request_redraw();
							}
						}
						DragAction::ResizeClipLeft {
							track_idx,
							clip_idx,
							orig_end,
						} => {
							let t = self.snap_time(self.x_to_time(bounds, position.x));
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(clip) = track.clips.get_mut(clip_idx)
							{
								let new_start = t.clamp(0.0, (orig_end - 0.1).max(0.0));
								clip.start = new_start;
								clip.duration = (orig_end - new_start).max(0.1);
								ctx.request_redraw();
							}
						}
						DragAction::ResizeClipRight { track_idx, clip_idx } => {
							let t = self.snap_time(self.x_to_time(bounds, position.x));
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(clip) = track.clips.get_mut(clip_idx)
							{
								clip.duration = (t - clip.start).max(0.1);
								ctx.request_redraw();
							}
						}
					}
					return EventResponse::Handled;
				}

				let hov = self.hit_keyframe(bounds, *position);
				if hov != self.hovered_kf {
					self.hovered_kf = hov;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Middle,
				state: ButtonState::Pressed,
				position,
			} => {
				self.drag = Some(DragAction::Pan {
					start_x: position.x,
					start_scroll: self.scroll_x,
				});
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Middle,
				state: ButtonState::Released,
				..
			} => {
				if matches!(self.drag, Some(DragAction::Pan { .. })) {
					self.drag = None;
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Right,
				state: ButtonState::Pressed,
				position,
			} => {
				// Right-click on a keyframe deletes it; otherwise right-drag pans
				if let Some((ti, ki)) = self.hit_keyframe(bounds, *position)
					&& let Some(track) = self.tracks.get_mut(ti)
					&& !track.locked
					&& ki < track.keyframes.len()
				{
					track.keyframes.remove(ki);
					self.selected_keyframe = None;
					if let Some(cb) = &self.on_keyframe_changed {
						cb.call2(ti as i32, -1.0f32);
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				self.drag = Some(DragAction::Pan {
					start_x: position.x,
					start_scroll: self.scroll_x,
				});
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Right,
				state: ButtonState::Released,
				..
			} => {
				if matches!(self.drag, Some(DragAction::Pan { .. })) {
					self.drag = None;
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Pressed,
				position,
			} => {
				// Left header column (select track / mute / lock)
				if position.x < bounds.x() + self.header_width {
					if let Some(ti) = self.track_at_y(bounds, position.y) {
						let (mute_r, lock_r) = self.header_btn_rects(bounds, ti);
						if mute_r.contains_point(*position) {
							self.tracks[ti].muted = !self.tracks[ti].muted;
							if let Some(cb) = &self.on_keyframe_changed {
								cb.call2(ti as i32, -2.0f32);
							}
						} else if lock_r.contains_point(*position) {
							self.tracks[ti].locked = !self.tracks[ti].locked;
							if let Some(cb) = &self.on_keyframe_changed {
								cb.call2(ti as i32, -2.0f32);
							}
						} else {
							self.selected_track = Some(ti);
							if let Some(cb) = &self.on_track_selected {
								cb.call1(ti as i32);
							}
						}
						ctx.request_redraw();
						return EventResponse::Handled;
					}
					return EventResponse::Ignored;
				}

				// Ruler scrubbing
				if position.y <= bounds.y() + RULER_HEIGHT {
					let t = self.snap_time(self.x_to_time(bounds, position.x));
					self.set_playhead(t);
					self.drag = Some(DragAction::ScrubPlayhead);
					if let Some(cb) = &self.on_time_changed {
						cb.call1(t);
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				// Keyframe hit
				if let Some((ti, ki)) = self.hit_keyframe(bounds, *position) {
					self.selected_track = Some(ti);
					self.selected_keyframe = Some((ti, ki));
					if !self.tracks[ti].locked {
						self.drag = Some(DragAction::MoveKeyframe {
							track_idx: ti,
							kf_idx: ki,
						});
					}
					if let Some(cb) = &self.on_track_selected {
						cb.call1(ti as i32);
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				// Clip hit (resize left/right or move)
				if let Some(ti) = self.track_at_y(bounds, position.y) {
					self.selected_track = Some(ti);
					if let Some(cb) = &self.on_track_selected {
						cb.call1(ti as i32);
					}
					if !self.tracks[ti].locked {
						for (ci, clip) in self.tracks[ti].clips.iter().enumerate() {
							let x0 = self.time_to_x(bounds, clip.start);
							let x1 = self.time_to_x(bounds, clip.end());
							if position.x >= x0 - 2.0 && position.x <= x1 + 2.0 {
								if (position.x - x0).abs() <= CLIP_EDGE_WIDTH {
									self.drag = Some(DragAction::ResizeClipLeft {
										track_idx: ti,
										clip_idx: ci,
										orig_end: clip.end(),
									});
								} else if (position.x - x1).abs() <= CLIP_EDGE_WIDTH {
									self.drag = Some(DragAction::ResizeClipRight {
										track_idx: ti,
										clip_idx: ci,
									});
								} else {
									let t_click = self.x_to_time(bounds, position.x);
									self.drag = Some(DragAction::MoveClip {
										track_idx: ti,
										clip_idx: ci,
										grab_offset_sec: t_click - clip.start,
									});
								}
								ctx.request_redraw();
								return EventResponse::Handled;
							}
						}

						// Double-click on empty track lane inserts a keyframe
						let now = Instant::now();
						let is_double = self.last_click.is_some_and(|(prev_t, prev_p)| {
							now.duration_since(prev_t).as_millis() < 350
								&& (*position - prev_p).length() < 6.0
						});
						self.last_click = Some((now, *position));

						if is_double {
							let t = self.snap_time(self.x_to_time(bounds, position.x));
							let val = self.tracks[ti].sample(t);
							self.tracks[ti].keyframes.push(Keyframe::new(t, val));
							self.tracks[ti].sort_keyframes();
							if let Some(ki) = self.tracks[ti]
								.keyframes
								.iter()
								.position(|k| (k.time - t).abs() < 1e-4)
							{
								self.selected_keyframe = Some((ti, ki));
							}
							if let Some(cb) = &self.on_keyframe_changed {
								cb.call2(ti as i32, t);
							}
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}

					// Single click/drag on empty lane scrubs playhead for effortless control
					let t = self.snap_time(self.x_to_time(bounds, position.x));
					self.set_playhead(t);
					self.drag = Some(DragAction::ScrubPlayhead);
					if let Some(cb) = &self.on_time_changed {
						cb.call1(t);
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}

				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if let Some(action) = self.drag.take() {
					if let DragAction::MoveKeyframe { track_idx, kf_idx } = action
						&& let Some(track) = self.tracks.get_mut(track_idx)
					{
						let moved_t = track.keyframes.get(kf_idx).map(|k| k.time);
						track.sort_keyframes();
						if let Some(t) = moved_t
							&& let Some(new_idx) =
								track.keyframes.iter().position(|k| (k.time - t).abs() < 1e-4)
						{
							self.selected_keyframe = Some((track_idx, new_idx));
						}
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerScroll { delta } => {
				if delta.x.abs() > 0.01 {
					self.scroll_x = (self.scroll_x - delta.x * 24.0).max(0.0);
				}
				if delta.y.abs() > 0.01 {
					let lane_left = bounds.x() + self.header_width;
					let rel_x = (self.last_pointer.x - lane_left).clamp(0.0, (bounds.width() - self.header_width).max(1.0));
					let t_anchor = (rel_x + self.scroll_x) / self.pixels_per_sec.max(1.0);
					let factor = if delta.y > 0.0 { 1.12 } else { 1.0 / 1.12 };
					self.pixels_per_sec = (self.pixels_per_sec * factor).clamp(24.0, 500.0);
					self.scroll_x = (t_anchor * self.pixels_per_sec - rel_x).max(0.0);
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
			"duration" => self.duration = value.expect_f32()?.max(0.1),
			"fps" => self.fps = value.expect_f32()?.clamp(1.0, 240.0),
			"playhead" => self.set_playhead(value.expect_f32()?),
			"playing" => self.set_playing(value.expect_bool()?),
			"looping" => self.looping = value.expect_bool()?,
			"snap" => self.snap = value.expect_bool()?,
			"pixels_per_sec" => self.pixels_per_sec = value.expect_f32()?.clamp(24.0, 500.0),
			"header_width" => self.header_width = value.expect_f32()?.clamp(80.0, 320.0),
			"row_height" => self.row_height = value.expect_f32()?.clamp(20.0, 60.0),
			"background" => self.background = Some(value.expect_dcolor()?),
			"corner_radius" => self.corner_radius = Some(value.expect_f32()?.max(0.0)),
			"tracks" => {
				if let Some(tracks) = parse_tracks_from_property(value) {
					self.tracks = tracks;
				} else {
					return Err(PropertyError::InvalidValue {
						expected: "array of track objects",
					});
				}
			}
			_ => return Err(PropertyError::NotFound),
		}
		Ok(())
	}

	fn is_method(&self, name: &str) -> bool {
		matches!(
			name,
			"on_time_changed" | "on_keyframe_changed" | "on_track_selected"
		)
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		match method_name {
			"on_time_changed" => {
				self.on_time_changed = Some(callback);
				true
			}
			"on_keyframe_changed" => {
				self.on_keyframe_changed = Some(callback);
				true
			}
			"on_track_selected" => {
				self.on_track_selected = Some(callback);
				true
			}
			_ => false,
		}
	}
}


#[derive(Default)]
pub struct TimelineViewBuilder {
	inner: TimelineView,
}

impl TimelineViewBuilder {
	pub fn new() -> Self {
		Self {
			inner: TimelineView::default(),
		}
	}

	pub fn duration(mut self, duration: f32) -> Self {
		self.inner.duration = duration.max(0.1);
		self
	}

	pub fn fps(mut self, fps: f32) -> Self {
		self.inner.fps = fps.clamp(1.0, 240.0);
		self
	}

	pub fn tracks(mut self, tracks: Vec<TimelineTrack>) -> Self {
		self.inner.tracks = tracks;
		self
	}

	pub fn snap(mut self, snap: bool) -> Self {
		self.inner.snap = snap;
		self
	}

	pub fn on_time_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_time_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn on_keyframe_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_keyframe_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> TimelineView {
		self.inner
	}
}

impl From<TimelineViewBuilder> for TimelineView {
	fn from(builder: TimelineViewBuilder) -> Self {
		builder.build()
	}
}
