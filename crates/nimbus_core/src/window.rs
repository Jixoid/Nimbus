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

use crate::event::{ButtonState, InputEvent, MouseButton};
use crate::id::WindowHandle;
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
	pub(crate) should_close: bool,
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
			should_close: false,
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
		let canvas = Canvas2D::new(device, allocator, display.swapchain().color_format());
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
			.layout_all(Vec2::new(display.width() as f32, display.height() as f32));

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

	pub(crate) fn handle_event(&mut self, event: &WindowEvent, device: &Device) -> bool {
		let mut redraw_needed = false;

		match event {
			WindowEvent::CloseRequested => {
				self.should_close = true;
			}

			WindowEvent::Resized(size) => {
				if size.width > 0 && size.height > 0 {
					device.wait_idle();
					if let Some(display) = &mut self.display {
						display.resize(size.width, size.height);
						self.width = size.width;
						self.height = size.height;
						self
							.tree
							.layout_all(Vec2::new(size.width as f32, size.height as f32));
						redraw_needed = true;
					}
				}
			}

			WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
				if let Some(display) = &mut self.display {
					display.update_scale_factor(*scale_factor);
				}
			}

			WindowEvent::CursorMoved { position, .. } => {
				let pos = Vec2::new(position.x as f32, position.y as f32);
				self.last_cursor = pos;
				if self.tree.dispatch_pointer_move(pos) {
					redraw_needed = true;
				}
			}

			WindowEvent::MouseInput { state, button, .. } => {
				let btn = match button {
					WinitMouseButton::Left => MouseButton::Left,
					WinitMouseButton::Right => MouseButton::Right,
					WinitMouseButton::Middle => MouseButton::Middle,
					WinitMouseButton::Back => MouseButton::Other(1),
					WinitMouseButton::Forward => MouseButton::Other(2),
					WinitMouseButton::Other(c) => MouseButton::Other(*c),
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
			}

			WindowEvent::MouseWheel { delta, .. } => {
				let scroll_delta = match delta {
					MouseScrollDelta::LineDelta(x, y) => Vec2::new(*x * 32.0, *y * 32.0),
					MouseScrollDelta::PixelDelta(p) => Vec2::new(p.x as f32, p.y as f32),
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
			}

			_ => {}
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
		let (display, canvas, sync) =
			match (&mut self.display, &mut self.canvas, &mut self.synchronizer) {
				(Some(d), Some(c), Some(s)) => (d, c, s),
				_ => return,
			};

		sync.begin_frame();
		let frame = sync.current_frame() as usize;
		let wait_semaphore = sync.image_available_semaphore();

		let image_index = display.acquire_next_image_with_semaphore(&wait_semaphore);
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

		// Layout pass if tree layout was invalidated
		if self.tree.needs_layout {
			self.tree.layout_all(Vec2::new(width as f32, height as f32));
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
