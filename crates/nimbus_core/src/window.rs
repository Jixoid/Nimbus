/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_theme::Theme;
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use nimbus_except::*;
use nimbus_math::{Vec2, Vec4};
use nimbus_platform::Display;
use nimbus_render2d::Canvas2D;
use nimbus_rhi::manager::allocator::Allocator;
use nimbus_rhi_ash::{AshVulkanBackend, Command, CommandManager, Device, Synchronizer};

use crate::event::{ButtonState, Cursor, InputEvent, MouseButton};
use crate::id::WindowHandle;
use crate::task::{IntoTask, Task, TaskQueue};
use crate::tree::UiTree;


const FRAMES_IN_FLIGHT: u32 = 2;

pub struct Window {
	pub(crate) handle: WindowHandle,
	pub(crate) winit_id: Option<WindowId>,
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub vsync: bool,
	pub tree: UiTree,
	pub(crate) display: Option<Display>,
	pub(crate) canvas: Option<Canvas2D>,
	pub(crate) synchronizer: Option<Synchronizer>,
	pub(crate) cmd_buffers: Vec<Command>,
	pub(crate) last_cursor: Vec2<f32>,
	pub(crate) last_cursor_icon: Cursor,
	pub(crate) should_close: bool,
	pub(crate) scale_override: Option<f32>,
	pub(crate) touch: TouchGestureState,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum TouchGesture {
	Undecided,
	Scrolling(crate::ObjectID),
	DirectDrag,
}

pub(crate) struct TouchGestureState {
	pub finger_id: Option<u64>,
	pub start_pos: Vec2<f32>,
	pub last_pos: Vec2<f32>,
	pub last_time: std::time::Instant,
	pub gesture: TouchGesture,
	pub velocity: Vec2<f32>,
	pub fling_target: Option<crate::ObjectID>,
	pub last_fling_time: std::time::Instant,
}

impl Default for TouchGestureState {
	fn default() -> Self {
		let now = std::time::Instant::now();
		Self {
			finger_id: None,
			start_pos: Vec2::ZERO,
			last_pos: Vec2::ZERO,
			last_time: now,
			gesture: TouchGesture::Undecided,
			velocity: Vec2::ZERO,
			fling_target: None,
			last_fling_time: now,
		}
	}
}

impl Window {
	pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
		Self {
			handle: WindowHandle::default(),
			winit_id: None,
			title: title.into(),
			width,
			height,
			vsync: true,
			tree: UiTree::new(),
			display: None,
			canvas: None,
			synchronizer: None,
			cmd_buffers: Vec::new(),
			last_cursor: Vec2::ZERO,
			last_cursor_icon: Cursor::Default,
			should_close: false,
			scale_override: None,
			touch: TouchGestureState::default(),
		}
	}

	pub fn with_vsync(mut self, vsync: bool) -> Self {
		self.vsync = vsync;
		self
	}

	pub fn handle(&self) -> WindowHandle {
		self.handle
	}

	pub fn winit_id(&self) -> Option<WindowId> {
		self.winit_id
	}


	pub fn tree(&self) -> &UiTree {
		&self.tree
	}

	pub fn tree_mut(&mut self) -> &mut UiTree {
		&mut self.tree
	}

	pub fn task_queue(&self) -> TaskQueue {
		self.tree.task_queue()
	}

	pub fn push_task<M, F: IntoTask<M>>(&self, task: F) {
		self.tree.push_task(task);
	}

	pub fn process_tasks(&mut self) -> bool {
		let mut executed = false;
		for _ in 0..64 {
			let tasks = self.tree.task_queue.drain_all();
			if tasks.is_empty() {
				break;
			}
			for task in tasks {
				match task {
					Task::Plain(f) => {
						f();
						executed = true;
					}
					Task::Tree(f) => {
						f(&mut self.tree);
						self.tree.needs_redraw = true;
						executed = true;
					}
					Task::Window(f) => {
						f(self);
						self.tree.needs_redraw = true;
						executed = true;
					}
					Task::Theme(_) => {}
				}
			}
		}
		executed
	}


	pub fn request_redraw(&self) {
		if let Some(display) = &self.display {
			display.request_redraw();
		}
	}

	pub fn set_title(&mut self, title: &str) {
		self.title = title.to_string();
		if let Some(display) = &self.display {
			display.set_title(title);
		}
	}

	pub fn close(&mut self) {
		self.should_close = true;
	}

	pub fn is_open(&self) -> bool {
		self.display.is_some() && !self.should_close
	}

	pub(crate) fn check_size_changed(&self) -> bool {
		if let Some(display) = &self.display {
			let inner_sz = display.window().inner_size();
			inner_sz.width > 0
				&& inner_sz.height > 0
				&& (inner_sz.width != display.width() || inner_sz.height != display.height())
		} else {
			false
		}
	}

	pub fn width(&self) -> u32 {
		self
			.display
			.as_ref()
			.map(|d| d.width())
			.unwrap_or(self.width)
	}

	pub fn height(&self) -> u32 {
		self
			.display
			.as_ref()
			.map(|d| d.height())
			.unwrap_or(self.height)
	}

	fn scale_factor(&self) -> f32 {
		self
			.scale_override
			.unwrap_or_else(|| {
				self
					.display
					.as_ref()
					.map(|d| d.scale_factor() as f32)
					.unwrap_or(1.0)
			})
			.max(0.5)
	}

	pub fn set_scale_override(&mut self, scale: Option<f32>) {
		self.scale_override = scale.map(|s| s.clamp(0.5, 4.0));
		let effective = self.scale_factor();
		if let Some(canvas) = &mut self.canvas {
			canvas.set_scale_factor(effective);
		}
		if let Some(display) = &self.display {
			self
				.tree
				.layout_all(Self::logical_size(display.width(), display.height(), effective));
			self.tree.needs_layout = false;
			self.tree.needs_redraw = true;
			display.request_redraw();
		}
	}

	fn logical_size(phys_w: u32, phys_h: u32, scale: f32) -> Vec2<f32> {
		let s = scale.max(0.5);
		Vec2::new(phys_w as f32 / s, phys_h as f32 / s)
	}

	pub(crate) fn init_graphics(
		&mut self,
		event_loop: &ActiveEventLoop,
		device: &Device,
		allocator: &Allocator<AshVulkanBackend>,
		cmd_manager: &CommandManager,
	) -> Result<WindowId, String> {
		let display = Display::new(
			event_loop,
			device,
			&self.title,
			self.width,
			self.height,
			self.vsync,
		)?;

		let winit_id = display.window().id();
		let scale = self
			.scale_override
			.unwrap_or(display.scale_factor() as f32)
			.max(0.5);
		let mut canvas = Canvas2D::new(device, allocator, display.swapchain().color_format());
		canvas.set_scale_factor(scale);
		let synchronizer = Synchronizer::new(device, FRAMES_IN_FLIGHT);

		let mut cmd_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT as usize);
		for _ in 0..FRAMES_IN_FLIGHT {
			cmd_buffers.push(
				cmd_manager
					.allocate_graphic_command()
					.map_err(|e| format!("Failed to allocate command buffer: {e}"))?,
			);
		}

		self
			.tree
			.layout_all(Self::logical_size(display.width(), display.height(), scale));

		self.winit_id = Some(winit_id);
		self.display = Some(display);
		self.canvas = Some(canvas);
		self.synchronizer = Some(synchronizer);
		self.cmd_buffers = cmd_buffers;

		nimbus_disp!(
			"Window \"{}\" (id: {:?}) graphics initialized successfully!",
			self.title,
			winit_id
		);

		Ok(winit_id)
	}

	fn sync_cursor(&mut self) {
		let next = self.tree.active_cursor();
		if next != self.last_cursor_icon {
			self.last_cursor_icon = next;
			if let Some(display) = &self.display {
				display.set_cursor_icon(next.to_winit());
			}
		}
	}

	pub(crate) fn handle_event(&mut self, event: &WindowEvent, device: &Device) -> bool {
		let mut redraw_needed = false;

		match event {
			WindowEvent::CloseRequested => {
				self.should_close = true;
			}

			WindowEvent::Resized(size) => {
				if size.width > 0 && size.height > 0 {
					device.wait_idle();
					device
						.raw()
						.compat()
						.render_pass_cache
						.clear_framebuffers(device.raw().device());
					if let Some(display) = &mut self.display {
						display.resize(size.width, size.height);
						self.width = display.width();
						self.height = display.height();
						let scale = self
							.scale_override
							.unwrap_or(display.scale_factor() as f32)
							.max(0.5);
						if let Some(canvas) = &mut self.canvas {
							canvas.set_scale_factor(scale);
						}
						self
							.tree
							.layout_all(Self::logical_size(self.width, self.height, scale));
						self.tree.needs_layout = false;
						self.tree.needs_redraw = true;
						redraw_needed = true;
					}
				}
			}

			WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
				if let Some(display) = &mut self.display {
					display.update_scale_factor(*scale_factor);
					let scale = self.scale_override.unwrap_or(*scale_factor as f32).max(0.5);
					if let Some(canvas) = &mut self.canvas {
						canvas.set_scale_factor(scale);
					}
					self
						.tree
						.layout_all(Self::logical_size(display.width(), display.height(), scale));
					self.tree.needs_layout = false;
					self.tree.needs_redraw = true;
					redraw_needed = true;
				}
			}

			WindowEvent::CursorMoved { position, .. } => {
				let scale = self.scale_factor();
				let pos = Vec2::new(position.x as f32 / scale, position.y as f32 / scale);
				self.last_cursor = pos;
				if self.tree.dispatch_pointer_move(pos) {
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::MouseInput { state, button, .. } => {
				let btn = match button {
					WinitMouseButton::Left => MouseButton::Left,
					WinitMouseButton::Right => MouseButton::Right,
					WinitMouseButton::Middle => MouseButton::Middle,
					WinitMouseButton::Back => MouseButton::Back,
					WinitMouseButton::Forward => MouseButton::Forward,
					WinitMouseButton::Other(_) => return false, // MouseButton::Other(*c),
				};
				let btn_state = match state {
					ElementState::Pressed => ButtonState::Pressed,
					ElementState::Released => ButtonState::Released,
				};

				if self
					.tree
					.dispatch_pointer_button(btn, btn_state, self.last_cursor)
				{
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::MouseWheel { delta, .. } => {
				let scale = self.scale_factor();
				let scroll_delta = match delta {
					MouseScrollDelta::LineDelta(x, y) => Vec2::new(*x * 32.0, *y * 32.0),
					MouseScrollDelta::PixelDelta(p) => {
						Vec2::new(p.x as f32 / scale, p.y as f32 / scale)
					}
				};

				if self.tree.dispatch_pointer_scroll(self.last_cursor, scroll_delta) {
					redraw_needed = true;
				}
			}

			WindowEvent::KeyboardInput { event, .. } => {
				use winit::keyboard::Key;
				let key_name = match &event.logical_key {
					Key::Named(named) => format!("{:?}", named),
					Key::Character(s) => s.as_str().to_string(),
					Key::Unidentified(_) => "Unidentified".to_string(),
					Key::Dead(c) => format!("Dead({:?})", c),
				};

				let text = event.text.as_ref().map(|s| s.as_str().to_string());
				let pressed = event.state == ElementState::Pressed;

				let input_event = InputEvent::Key {
					key_name,
					text,
					pressed,
				};

				if self.tree.dispatch_key(&input_event) {
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
				let input_event = InputEvent::Key {
					key_name: "Ime".to_string(),
					text: Some(text.clone()),
					pressed: true,
				};

				if self.tree.dispatch_key(&input_event) {
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::HoveredFile(path) => {
				if self
					.tree
					.dispatch_os_file_hover(path.clone(), self.last_cursor)
				{
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::DroppedFile(path) => {
				if self
					.tree
					.dispatch_os_file_drop(path.clone(), self.last_cursor)
				{
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::HoveredFileCancelled => {
				if self.tree.dispatch_os_file_cancel() {
					redraw_needed = true;
				}
				self.sync_cursor();
			}

			WindowEvent::Touch(touch) => {
				use winit::event::TouchPhase;
				let scale = self.scale_factor();
				let pos = Vec2::new(
					touch.location.x as f32 / scale,
					touch.location.y as f32 / scale,
				);
				let now = std::time::Instant::now();
				self.last_cursor = pos;
				match touch.phase {
					TouchPhase::Started => {
						self.touch.fling_target = None;
						if self.touch.finger_id.is_none() {
							self.touch.finger_id = Some(touch.id);
							self.touch.start_pos = pos;
							self.touch.last_pos = pos;
							self.touch.last_time = now;
							self.touch.gesture = TouchGesture::Undecided;
							self.touch.velocity = Vec2::ZERO;
						}
						if self.tree.dispatch_pointer_move(pos) {
							redraw_needed = true;
						}
						if self
							.tree
							.dispatch_pointer_button(MouseButton::Left, ButtonState::Pressed, pos)
						{
							redraw_needed = true;
						}
					}
					TouchPhase::Moved => {
						if self.touch.finger_id == Some(touch.id) {
							let delta = pos - self.touch.last_pos;
							let dt = now
								.duration_since(self.touch.last_time)
								.as_secs_f32()
								.max(0.001);
							if dt < 0.15 {
								let inst_vel = Vec2::new(delta.x / dt, delta.y / dt);
								self.touch.velocity = Vec2::new(
									self.touch.velocity.x * 0.6 + inst_vel.x * 0.4,
									self.touch.velocity.y * 0.6 + inst_vel.y * 0.4,
								);
							}
							self.touch.last_pos = pos;
							self.touch.last_time = now;

							match self.touch.gesture {
								TouchGesture::Scrolling(target) => {
									if self.tree.dispatch_touch_scroll_to(target, delta) {
										redraw_needed = true;
									}
								}
								TouchGesture::DirectDrag => {
									if self.tree.dispatch_pointer_move(pos) {
										redraw_needed = true;
									}
								}
								TouchGesture::Undecided => {
									let total = pos - self.touch.start_pos;
									let dist = (total.x * total.x + total.y * total.y).sqrt();
									if dist >= 8.0 {
										if let Some(target) =
											self.tree.dispatch_touch_scroll(self.touch.start_pos, total)
										{
											self.touch.gesture = TouchGesture::Scrolling(target);
											self.tree.cancel_pressed();
											redraw_needed = true;
										} else {
											self.touch.gesture = TouchGesture::DirectDrag;
											if self.tree.dispatch_pointer_move(pos) {
												redraw_needed = true;
											}
										}
									} else if self.tree.dispatch_pointer_move(pos) {
										redraw_needed = true;
									}
								}
							}
						}
					}
					TouchPhase::Ended | TouchPhase::Cancelled => {
						if self.touch.finger_id == Some(touch.id) {
							self.touch.finger_id = None;
							let prev = std::mem::replace(&mut self.touch.gesture, TouchGesture::Undecided);
							if let TouchGesture::Scrolling(target) = prev {
								if self.tree.dispatch_touch_scroll_end(target) {
									redraw_needed = true;
								}
								let speed = (self.touch.velocity.x * self.touch.velocity.x
									+ self.touch.velocity.y * self.touch.velocity.y)
									.sqrt();
								if touch.phase == TouchPhase::Ended && speed > 80.0 {
									self.touch.fling_target = Some(target);
									self.touch.last_fling_time = now;
									redraw_needed = true;
								}
							} else if touch.phase == TouchPhase::Cancelled {
								self.tree.cancel_pressed();
								redraw_needed = true;
							} else if self.tree.dispatch_pointer_button(
								MouseButton::Left,
								ButtonState::Released,
								pos,
							) {
								redraw_needed = true;
							}
						}
					}
				}
			}

			_ => {}
		}

		if self.process_tasks() {
			redraw_needed = true;
		}

		if redraw_needed || self.tree.needs_redraw || self.tree.needs_layout {
			self.request_redraw();
		}

		redraw_needed
	}

	pub(crate) fn render_frame(
		&mut self,
		device: &Device,
		allocator: &Allocator<AshVulkanBackend>,
		cmd_manager: &CommandManager,
		theme: &Theme,
	) {
		self.process_tasks();

		if let Some(target) = self.touch.fling_target {
			let now = std::time::Instant::now();
			let dt = now
				.duration_since(self.touch.last_fling_time)
				.as_secs_f32()
				.clamp(0.001, 0.05);
			self.touch.last_fling_time = now;
			let delta = Vec2::new(self.touch.velocity.x * dt, self.touch.velocity.y * dt);
			let handled = self.tree.dispatch_pointer_scroll_to(target, delta);
			let decay = (-5.5 * dt).exp();
			self.touch.velocity.x *= decay;
			self.touch.velocity.y *= decay;
			let speed = (self.touch.velocity.x * self.touch.velocity.x
				+ self.touch.velocity.y * self.touch.velocity.y)
				.sqrt();
			if !handled || speed < 20.0 {
				self.touch.fling_target = None;
			} else {
				self.tree.needs_redraw = true;
			}
		}

		let (display, canvas, sync) =
			match (&mut self.display, &mut self.canvas, &mut self.synchronizer) {
				(Some(d), Some(c), Some(s)) => (d, c, s),
				_ => return,
			};

		let scale = self
			.scale_override
			.unwrap_or(display.scale_factor() as f32)
			.max(0.5);
		canvas.set_scale_factor(scale);

		let inner_sz = display.window().inner_size();
		if inner_sz.width > 0
			&& inner_sz.height > 0
			&& (inner_sz.width != display.width() || inner_sz.height != display.height())
		{
			device.wait_idle();
			device
				.raw()
				.compat()
				.render_pass_cache
				.clear_framebuffers(device.raw().device());
			display.resize(inner_sz.width, inner_sz.height);
			self.width = display.width();
			self.height = display.height();
			self
				.tree
				.layout_all(Self::logical_size(self.width, self.height, scale));
			self.tree.needs_layout = false;
		}

		sync.begin_frame();
		let frame = sync.current_frame() as usize;
		let wait_semaphore = sync.image_available_semaphore();

		let image_index = match display.try_acquire_next_image_with_semaphore(&wait_semaphore) {
			Ok(idx) => idx,
			Err(_) => {
				sync.end_frame();
				device.wait_idle();
				device
					.raw()
					.compat()
					.render_pass_cache
					.clear_framebuffers(device.raw().device());
				let inner_sz = display.window().inner_size();
				if inner_sz.width > 0 && inner_sz.height > 0 {
					display.resize(inner_sz.width, inner_sz.height);
					self.width = display.width();
					self.height = display.height();
				}
				self.tree.needs_layout = true;
				self.tree.needs_redraw = true;
				display.request_redraw();
				return;
			}
		};
		let raw_target = match display.render_target(image_index) {
			Some(t) => t,
			None => {
				sync.end_frame();
				return;
			}
		};

		let target = raw_target.without_depth();
		let width = display.width();
		let height = display.height();
		if width != self.width || height != self.height {
			self.width = width;
			self.height = height;
			self.tree.needs_layout = true;
		}

		// Layout pass if tree layout was invalidated
		if self.tree.needs_layout {
			self
				.tree
				.layout_all(Self::logical_size(width, height, scale));
			self.tree.needs_layout = false;
		}

		// Build UI frame on canvas
		canvas.clear_batches();
		let (anim_active, anim_layout_needed) = self.tree.draw(canvas, theme);
		if anim_layout_needed {
			self.tree.needs_layout = true;
		}

		// Dynamic render pass
		let cmd = &mut self.cmd_buffers[frame];
		cmd.begin();

		let clear_color = Vec4::new(
			theme.monet.surface.container_lowest.r,
			theme.monet.surface.container_lowest.g,
			theme.monet.surface.container_lowest.b,
			1.0,
		);
		cmd.begin_rendering(&target, clear_color);

		canvas.render(
			cmd,
			&target,
			width,
			height,
			device,
			allocator,
			cmd_manager.raw(),
		);

		cmd.end_rendering(&target);
		cmd.end();

		// Submit and present
		let render_finished = display.render_finished_semaphore(image_index);
		let in_flight_fence = sync.in_flight_fence();

		cmd_manager.submit_with_sync(
			cmd,
			Some(&wait_semaphore),
			Some(&render_finished),
			Some(&in_flight_fence),
		);

		display.present_with_semaphore(image_index, &render_finished);
		sync.end_frame();

		// If any animation is actively running or layout/redraw was requested, queue next frame
		if anim_active || self.tree.needs_redraw || self.tree.needs_layout {
			self.tree.needs_redraw = false;
			self.request_redraw();
		}
	}
}
