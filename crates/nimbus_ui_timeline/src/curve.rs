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
use nimbus_math::{Rect, Vec2};
use nimbus_render2d::{Canvas2D, CurveShape, Font, FontId, FontManager, RectShape, TextShape};
use nimbus_theme::{DColor, MonetRole};

use crate::model::{KeyframeInterp, TimelineTrack, parse_tracks_from_property};


const GUTTER_WIDTH: f32 = 44.0;
const NODE_HIT_RADIUS: f32 = 13.0;
const HANDLE_HIT_RADIUS: f32 = 12.0;


#[derive(Copy, Clone, Debug, PartialEq)]
enum CurveHover {
	Keyframe(usize, usize),
	InTangent(usize, usize),
	OutTangent(usize, usize),
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CurveDrag {
	Keyframe { track_idx: usize, kf_idx: usize },
	InTangent { track_idx: usize, kf_idx: usize },
	OutTangent { track_idx: usize, kf_idx: usize },
	Pan { start: Vec2<f32>, orig_min_t: f32, orig_max_t: f32, orig_min_v: f32, orig_max_v: f32 },
}


pub struct CurveEditor {
	pub tracks: Vec<TimelineTrack>,
	pub playhead: f32,
	pub time_min: f32,
	pub time_max: f32,
	pub value_min: f32,
	pub value_max: f32,
	pub fps: f32,
	pub snap: bool,
	pub background: Option<DColor>,
	pub corner_radius: Option<f32>,
	pub font: Option<FontId>,
	pub font_size: f32,
	pub selected: Option<(usize, usize)>,
	pub on_curve_changed: Option<MethodCallback>,

	hovered: Option<CurveHover>,
	drag: Option<CurveDrag>,
	last_pointer: Vec2<f32>,
	last_bounds: Cell<Rect<f32>>,
}

impl Default for CurveEditor {
	fn default() -> Self {
		Self {
			tracks: Vec::new(),
			playhead: 0.0,
			time_min: 0.0,
			time_max: 6.0,
			value_min: -1.5,
			value_max: 1.5,
			fps: 30.0,
			snap: false,
			background: None,
			corner_radius: None,
			font: None,
			font_size: 11.0,
			selected: None,
			on_curve_changed: None,
			hovered: None,
			drag: None,
			last_pointer: Vec2::ZERO,
			last_bounds: Cell::new(Rect::ZERO),
		}
	}
}

impl CurveEditor {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn builder() -> CurveEditorBuilder {
		CurveEditorBuilder::new()
	}

	fn plot_rect(&self, bounds: Rect<f32>) -> Rect<f32> {
		Rect::new(
			bounds.x() + GUTTER_WIDTH,
			bounds.y() + 8.0,
			(bounds.width() - GUTTER_WIDTH - 10.0).max(1.0),
			(bounds.height() - 16.0).max(1.0),
		)
	}

	fn to_screen(&self, plot: Rect<f32>, time: f32, value: f32) -> Vec2<f32> {
		let tx = (time - self.time_min) / (self.time_max - self.time_min).max(1e-4);
		let ty = (value - self.value_min) / (self.value_max - self.value_min).max(1e-4);
		Vec2::new(plot.x() + tx * plot.width(), plot.max.y - ty * plot.height())
	}

	fn from_screen(&self, plot: Rect<f32>, pos: Vec2<f32>) -> (f32, f32) {
		let tx = (pos.x - plot.x()) / plot.width().max(1.0);
		let ty = (plot.max.y - pos.y) / plot.height().max(1.0);
		let t = self.time_min + tx * (self.time_max - self.time_min);
		let v = self.value_min + ty * (self.value_max - self.value_min);
		(t, v)
	}
}

impl Widget for CurveEditor {
	fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: BoxConstraints) -> Vec2<f32> {
		constraints.constrain(Vec2::new(
			if constraints.max_width.is_finite() {
				constraints.max_width
			} else {
				640.0
			},
			if constraints.max_height.is_finite() {
				constraints.max_height
			} else {
				220.0
			},
		))
	}

	fn draw(&self, ctx: &DrawCtx, canvas: &mut Canvas2D) {
		self.last_bounds.set(ctx.bounds);
		let bounds = ctx.bounds;
		if bounds.width() <= 8.0 || bounds.height() <= 8.0 {
			return;
		}

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

		let plot = self.plot_rect(bounds);
		let fface = FontManager::default_font();
		let font = match self.font {
			Some(id) => Font::with_id(id, fface, self.font_size),
			None => Font::new(fface, self.font_size),
		};

		canvas.push_clip(bounds);

		// Value grid (horizontal)
		let mut grid_col = monet[MonetRole::OnSurface];
		grid_col.a = 0.08;
		let mut zero_col = monet[MonetRole::OnSurface];
		zero_col.a = 0.22;

		let v_steps = 4usize;
		for i in 0..=v_steps {
			let frac = i as f32 / v_steps as f32;
			let val = self.value_min + (self.value_max - self.value_min) * frac;
			let y = plot.max.y - frac * plot.height();
			canvas.draw(RectShape::new(
				Rect::new(plot.x(), y, plot.width(), 1.0),
				if val.abs() < 1e-3 { zero_col } else { grid_col },
			));
			let lbl = format!("{:.1}", val);
			let (lw, lh) = canvas.measure_text(&lbl, self.font_size);
			canvas.draw(TextShape::new(
				&lbl,
				font,
				self.font_size,
				Vec2::new(plot.x() - lw - 6.0, y - lh * 0.5),
				monet[MonetRole::OnSurfaceVariant],
			));
		}

		canvas.push_clip(plot);

		// Vertical time grid
		let t_span = (self.time_max - self.time_min).max(0.1);
		let t_step = if t_span <= 2.0 { 0.5 } else { 1.0 };
		let first_t = (self.time_min / t_step).ceil() * t_step;
		let mut t = first_t;
		while t <= self.time_max + 1e-3 {
			let p = self.to_screen(plot, t, 0.0);
			canvas.draw(RectShape::new(
				Rect::new(p.x, plot.y(), 1.0, plot.height()),
				grid_col,
			));
			t += t_step;
		}

		// Draw curves using GPU CurvePipeline
		for (ti, track) in self.tracks.iter().enumerate() {
			if track.muted || track.keyframes.is_empty() {
				continue;
			}
			let tcol = track.resolved_color(ti, monet);

			for i in 0..track.keyframes.len().saturating_sub(1) {
				let k0 = &track.keyframes[i];
				let k1 = &track.keyframes[i + 1];
				let p0 = self.to_screen(plot, k0.time, k0.value);
				let p3 = self.to_screen(plot, k1.time, k1.value);

				match k0.interp {
					KeyframeInterp::Step => {
						let corner = Vec2::new(p3.x, p0.y);
						canvas.draw(CurveShape::line(p0, corner, 2.2, tcol));
						canvas.draw(CurveShape::line(corner, p3, 2.2, tcol));
					}
					KeyframeInterp::Linear => {
						canvas.draw(CurveShape::line(p0, p3, 2.2, tcol));
					}
					KeyframeInterp::Bezier => {
						let p1 = self.to_screen(
							plot,
							k0.time + k0.out_tangent.x.max(0.0),
							k0.value + k0.out_tangent.y,
						);
						let p2 = self.to_screen(
							plot,
							k1.time + k1.in_tangent.x.min(0.0),
							k1.value + k1.in_tangent.y,
						);
						canvas.draw(CurveShape::cubic(p0, p1, p2, p3, 2.4, tcol));
					}
				}
			}

			// Keyframe nodes & tangent handles
			for (ki, kf) in track.keyframes.iter().enumerate() {
				let p = self.to_screen(plot, kf.time, kf.value);
				let is_sel = self.selected == Some((ti, ki));
				let is_hov = self.hovered == Some(CurveHover::Keyframe(ti, ki));

				if is_sel && kf.interp == KeyframeInterp::Bezier {
					let h_in = self.to_screen(
						plot,
						kf.time + kf.in_tangent.x.min(0.0),
						kf.value + kf.in_tangent.y,
					);
					let h_out = self.to_screen(
						plot,
						kf.time + kf.out_tangent.x.max(0.0),
						kf.value + kf.out_tangent.y,
					);
					let hcol = monet[MonetRole::OnSurfaceVariant];
					canvas.draw(CurveShape::line(p, h_in, 1.5, hcol));
					canvas.draw(CurveShape::line(p, h_out, 1.5, hcol));
					let in_hov = self.hovered == Some(CurveHover::InTangent(ti, ki));
					let out_hov = self.hovered == Some(CurveHover::OutTangent(ti, ki));
					for (h, hov) in [(h_in, in_hov), (h_out, out_hov)] {
						let hr = if hov { 6.5 } else { 5.0 };
						canvas.draw(
							RectShape::new(
								Rect::new(h.x - hr, h.y - hr, hr * 2.0, hr * 2.0),
								monet[MonetRole::Primary],
							)
							.radius(hr)
							.border(monet[MonetRole::OnPrimary], 1.5),
						);
					}
				}

				let nr = if is_sel {
					6.5
				} else if is_hov {
					6.0
				} else {
					4.8
				};
				canvas.draw(
					RectShape::new(Rect::new(p.x - nr, p.y - nr, nr * 2.0, nr * 2.0), tcol)
						.radius(nr)
						.border(
							if is_sel || is_hov {
								monet[MonetRole::OnPrimary]
							} else {
								monet[MonetRole::SurfaceContainerLowest]
							},
							if is_sel { 2.0 } else { 1.5 },
						),
				);
			}

			// Sampled live value dot on playhead
			let sampled = track.sample(self.playhead);
			let sp = self.to_screen(plot, self.playhead, sampled);
			canvas.draw(
				RectShape::new(Rect::new(sp.x - 4.5, sp.y - 4.5, 9.0, 9.0), monet[MonetRole::Error])
					.radius(4.5),
			);
		}

		// Playhead vertical line
		let ph_x = self.to_screen(plot, self.playhead, 0.0).x;
		canvas.draw(RectShape::new(
			Rect::new(ph_x - 0.75, plot.y(), 1.5, plot.height()),
			monet[MonetRole::Error],
		));

		canvas.pop_clip();
		canvas.pop_clip();
	}

	fn handle_event(&mut self, ctx: &mut EventCtx, event: &InputEvent) -> EventResponse {
		let bounds = self.last_bounds.get();
		let plot = self.plot_rect(bounds);

		match event {
			InputEvent::PointerMoved { position } => {
				self.last_pointer = *position;
				if let Some(drag) = self.drag {
					match drag {
						CurveDrag::Keyframe { track_idx, kf_idx } => {
							let (mut t, v) = self.from_screen(plot, *position);
							t = t.max(0.0);
							if self.snap && self.fps > 0.0 {
								let step = 1.0 / self.fps;
								t = (t / step).round() * step;
							}
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(kf) = track.keyframes.get_mut(kf_idx)
							{
								kf.time = t;
								kf.value = v;
								if let Some(cb) = &self.on_curve_changed {
									cb.call0();
								}
								ctx.request_redraw();
							}
						}
						CurveDrag::InTangent { track_idx, kf_idx } => {
							let (t, v) = self.from_screen(plot, *position);
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(kf) = track.keyframes.get_mut(kf_idx)
							{
								kf.in_tangent = Vec2::new((t - kf.time).min(0.0), v - kf.value);
								if let Some(cb) = &self.on_curve_changed {
									cb.call0();
								}
								ctx.request_redraw();
							}
						}
						CurveDrag::OutTangent { track_idx, kf_idx } => {
							let (t, v) = self.from_screen(plot, *position);
							if let Some(track) = self.tracks.get_mut(track_idx)
								&& !track.locked
								&& let Some(kf) = track.keyframes.get_mut(kf_idx)
							{
								kf.out_tangent = Vec2::new((t - kf.time).max(0.0), v - kf.value);
								if let Some(cb) = &self.on_curve_changed {
									cb.call0();
								}
								ctx.request_redraw();
							}
						}
						CurveDrag::Pan {
							start,
							orig_min_t,
							orig_max_t,
							orig_min_v,
							orig_max_v,
						} => {
							let dx = position.x - start.x;
							let dy = position.y - start.y;
							let t_span = (orig_max_t - orig_min_t).max(0.1);
							let dt = -(dx / plot.width().max(1.0)) * t_span;
							let dv = (dy / plot.height().max(1.0)) * (orig_max_v - orig_min_v);
							self.time_min = (orig_min_t + dt).max(0.0);
							self.time_max = self.time_min + t_span;
							self.value_min = orig_min_v + dv;
							self.value_max = orig_max_v + dv;
							ctx.request_redraw();
						}
					}
					return EventResponse::Handled;
				}

				// Hover feedback for handles and keyframe nodes
				let mut new_hov = None;
				if let Some((ti, ki)) = self.selected
					&& let Some(track) = self.tracks.get(ti)
					&& !track.muted
					&& let Some(kf) = track.keyframes.get(ki)
					&& kf.interp == KeyframeInterp::Bezier
				{
					let h_in = self.to_screen(
						plot,
						kf.time + kf.in_tangent.x.min(0.0),
						kf.value + kf.in_tangent.y,
					);
					let h_out = self.to_screen(
						plot,
						kf.time + kf.out_tangent.x.max(0.0),
						kf.value + kf.out_tangent.y,
					);
					if (*position - h_in).length() <= HANDLE_HIT_RADIUS {
						new_hov = Some(CurveHover::InTangent(ti, ki));
					} else if (*position - h_out).length() <= HANDLE_HIT_RADIUS {
						new_hov = Some(CurveHover::OutTangent(ti, ki));
					}
				}
				if new_hov.is_none() {
					'nodes: for (ti, track) in self.tracks.iter().enumerate() {
						if track.muted {
							continue;
						}
						for (ki, kf) in track.keyframes.iter().enumerate() {
							let p = self.to_screen(plot, kf.time, kf.value);
							if (*position - p).length() <= NODE_HIT_RADIUS {
								new_hov = Some(CurveHover::Keyframe(ti, ki));
								break 'nodes;
							}
						}
					}
				}
				if new_hov != self.hovered {
					self.hovered = new_hov;
					ctx.request_redraw();
				}
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Middle | MouseButton::Right,
				state: ButtonState::Pressed,
				position,
			} => {
				self.drag = Some(CurveDrag::Pan {
					start: *position,
					orig_min_t: self.time_min,
					orig_max_t: self.time_max,
					orig_min_v: self.value_min,
					orig_max_v: self.value_max,
				});
				EventResponse::Handled
			}

			InputEvent::PointerButton {
				button: MouseButton::Middle | MouseButton::Right,
				state: ButtonState::Released,
				..
			} => {
				if matches!(self.drag, Some(CurveDrag::Pan { .. })) {
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
				// 1. Check tangent handles of currently selected keyframe
				if let Some((ti, ki)) = self.selected
					&& let Some(track) = self.tracks.get(ti)
					&& !track.muted
					&& !track.locked
					&& let Some(kf) = track.keyframes.get(ki)
					&& kf.interp == KeyframeInterp::Bezier
				{
					let h_in = self.to_screen(
						plot,
						kf.time + kf.in_tangent.x.min(0.0),
						kf.value + kf.in_tangent.y,
					);
					if (*position - h_in).length() <= HANDLE_HIT_RADIUS {
						self.drag = Some(CurveDrag::InTangent {
							track_idx: ti,
							kf_idx: ki,
						});
						return EventResponse::Handled;
					}
					let h_out = self.to_screen(
						plot,
						kf.time + kf.out_tangent.x.max(0.0),
						kf.value + kf.out_tangent.y,
					);
					if (*position - h_out).length() <= HANDLE_HIT_RADIUS {
						self.drag = Some(CurveDrag::OutTangent {
							track_idx: ti,
							kf_idx: ki,
						});
						return EventResponse::Handled;
					}
				}

				// 2. Check keyframe nodes
				for (ti, track) in self.tracks.iter().enumerate() {
					if track.muted {
						continue;
					}
					for (ki, kf) in track.keyframes.iter().enumerate() {
						let p = self.to_screen(plot, kf.time, kf.value);
						if (*position - p).length() <= NODE_HIT_RADIUS {
							self.selected = Some((ti, ki));
							if !track.locked {
								self.drag = Some(CurveDrag::Keyframe {
									track_idx: ti,
									kf_idx: ki,
								});
							}
							ctx.request_redraw();
							return EventResponse::Handled;
						}
					}
				}

				self.selected = None;
				ctx.request_redraw();
				EventResponse::Ignored
			}

			InputEvent::PointerButton {
				button: MouseButton::Left,
				state: ButtonState::Released,
				..
			} => {
				if let Some(drag) = self.drag.take() {
					if let CurveDrag::Keyframe { track_idx, kf_idx } = drag
						&& let Some(track) = self.tracks.get_mut(track_idx)
					{
						let moved_t = track.keyframes.get(kf_idx).map(|k| k.time);
						track.sort_keyframes();
						if let Some(t) = moved_t
							&& let Some(new_idx) =
								track.keyframes.iter().position(|k| (k.time - t).abs() < 1e-4)
						{
							self.selected = Some((track_idx, new_idx));
						}
					}
					ctx.request_redraw();
					return EventResponse::Handled;
				}
				EventResponse::Ignored
			}

			InputEvent::PointerScroll { delta } => {
				if delta.x.abs() > 0.01 {
					let span = (self.time_max - self.time_min).max(0.1);
					let shift = -delta.x * span * 0.06;
					self.time_min = (self.time_min + shift).max(0.0);
					self.time_max = self.time_min + span;
				}
				if delta.y.abs() > 0.01 {
					let frac = ((self.last_pointer.x - plot.x()) / plot.width().max(1.0)).clamp(0.0, 1.0);
					let old_span = (self.time_max - self.time_min).max(0.1);
					let t_anchor = self.time_min + frac * old_span;
					let factor = if delta.y > 0.0 { 1.0 / 1.12 } else { 1.12 };
					let new_span = (old_span * factor).clamp(0.5, 120.0);
					let new_min = (t_anchor - frac * new_span).max(0.0);
					self.time_min = new_min;
					self.time_max = new_min + new_span;
				}
				ctx.request_redraw();
				EventResponse::Handled
			}

			_ => EventResponse::Ignored,
		}
	}

	fn cursor(&self) -> Cursor {
		match self.drag {
			Some(CurveDrag::Pan { .. }) => Cursor::Grabbing,
			Some(
				CurveDrag::Keyframe { .. }
				| CurveDrag::InTangent { .. }
				| CurveDrag::OutTangent { .. },
			) => Cursor::Move,
			None => Cursor::Default,
		}
	}

	fn set_property(
		&mut self,
		key: &str,
		value: &PropertyValue,
		_ctx: &PropertyCtx,
	) -> Result<(), PropertyError> {
		match key {
			"playhead" => self.playhead = value.expect_f32()?.max(0.0),
			"time_min" => self.time_min = value.expect_f32()?,
			"time_max" => self.time_max = value.expect_f32()?,
			"value_min" => self.value_min = value.expect_f32()?,
			"value_max" => self.value_max = value.expect_f32()?,
			"fps" => self.fps = value.expect_f32()?.clamp(1.0, 240.0),
			"snap" => self.snap = value.expect_bool()?,
			"color" => self.background = Some(value.expect_dcolor()?),
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
		matches!(name, "on_curve_changed")
	}

	fn bind_method(&mut self, method_name: &str, callback: MethodCallback) -> bool {
		if method_name == "on_curve_changed" {
			self.on_curve_changed = Some(callback);
			true
		} else {
			false
		}
	}
}


#[derive(Default)]
pub struct CurveEditorBuilder {
	inner: CurveEditor,
}

impl CurveEditorBuilder {
	pub fn new() -> Self {
		Self {
			inner: CurveEditor::default(),
		}
	}

	pub fn tracks(mut self, tracks: Vec<TimelineTrack>) -> Self {
		self.inner.tracks = tracks;
		self
	}

	pub fn time_range(mut self, min: f32, max: f32) -> Self {
		self.inner.time_min = min;
		self.inner.time_max = max.max(min + 0.1);
		self
	}

	pub fn value_range(mut self, min: f32, max: f32) -> Self {
		self.inner.value_min = min;
		self.inner.value_max = max.max(min + 0.1);
		self
	}

	pub fn on_curve_changed<M, F: IntoMethod<M>>(mut self, callback: F) -> Self {
		self.inner.on_curve_changed = Some(MethodCallback::new(callback));
		self
	}

	pub fn build(self) -> CurveEditor {
		self.inner
	}
}

impl From<CurveEditorBuilder> for CurveEditor {
	fn from(builder: CurveEditorBuilder) -> Self {
		builder.build()
	}
}
