/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use slotmap::SlotMap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use nimbus_except::*;
use nimbus_rhi::manager::allocator::Allocator;
use nimbus_rhi_ash::{AshVulkanBackend, CommandManager, Device};

use crate::id::WindowHandle;
use crate::theme::Theme;
use crate::window::Window;


pub struct Application {
	pub theme: Theme,
	pub windows: SlotMap<WindowHandle, Window>,
}

impl Application {
	pub fn new() -> Self {
		Self {
			theme: Theme::dark(),
			windows: SlotMap::with_key(),
		}
	}

	pub fn new_with_theme(mut self, theme: Theme) -> Self {
		self.theme = theme;
		self
	}


	pub fn create_window(&mut self, mut window: Window) -> WindowHandle {
		self.windows.insert_with_key(|k| {
			window.handle = k;
			window
		})
	}

	pub fn add_window(&mut self, title: impl Into<String>, width: u32, height: u32) -> WindowHandle {
		self.create_window(Window::new(title, width, height))
	}


	pub fn window(&self, handle: WindowHandle) -> Option<&Window> {
		self.windows.get(handle)
	}

	pub fn window_mut(&mut self, handle: WindowHandle) -> Option<&mut Window> {
		self.windows.get_mut(handle)
	}

	pub fn close_window(&mut self, handle: WindowHandle) {
		if let Some(win) = self.windows.get_mut(handle) {
			win.close();
		}
	}


	pub fn run(self) -> Result<(), String> {
		let event_loop = EventLoop::new().map_err(|e| format!("Failed to create EventLoop: {e}"))?;
		event_loop.set_control_flow(ControlFlow::Poll);

		let mut runner = AppRunner {
			theme: self.theme,
			windows: self.windows,
			winit_map: HashMap::new(),
			device: None,
			allocator: None,
			cmd_manager: None,
		};

		event_loop
			.run_app(&mut runner)
			.map_err(|e| format!("Event loop exited with error: {e}"))?;

		Ok(())
	}
}

impl Default for Application {
	fn default() -> Self {
		Self::new()
	}
}


struct AppRunner {
	theme: Theme,
	windows: SlotMap<WindowHandle, Window>,
	winit_map: HashMap<WindowId, WindowHandle>,
	device: Option<Device>,
	allocator: Option<Allocator<AshVulkanBackend>>,
	cmd_manager: Option<CommandManager>,
}

impl ApplicationHandler for AppRunner {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.device.is_none() {
			nimbus_disp!("Initializing Application shared Vulkan RHI Device...");

			let mut device = Device::new_with_options("Nimbus Application", 0, 1, 0, 0, true);
			let allocator = Allocator::new(&device);
			device.init_descriptor_heap(&allocator);
			let cmd_manager = CommandManager::new(&device);

			self.device = Some(device);
			self.allocator = Some(allocator);
			self.cmd_manager = Some(cmd_manager);
		}

		let device = self.device.as_ref().unwrap();
		let allocator = self.allocator.as_ref().unwrap();
		let cmd_manager = self.cmd_manager.as_ref().unwrap();

		for (handle, window) in self.windows.iter_mut() {
			if window.display.is_none() {
				match window.init_graphics(event_loop, device, allocator, cmd_manager) {
					Ok(winit_id) => {
						self.winit_map.insert(winit_id, handle);
					}
					Err(e) => {
						nimbus_error!("Failed to create window \"{}\": {}", window.title, e);
					}
				}
			}
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let handle = match self.winit_map.get(&window_id).copied() {
			Some(h) => h,
			None => return,
		};

		let device = match &self.device {
			Some(d) => d,
			None => return,
		};

		let mut should_destroy = false;

		if let Some(window) = self.windows.get_mut(handle) {
			match &event {
				WindowEvent::CloseRequested => {
					nimbus_disp!("Close requested for window \"{}\"", window.title);
					window.handle_event(&event, device);
					if window.should_close {
						should_destroy = true;
					}
				}

				WindowEvent::RedrawRequested => {
					if let (Some(allocator), Some(cmd_manager)) = (&self.allocator, &self.cmd_manager) {
						window.render_frame(device, allocator, cmd_manager, &self.theme);
					}
				}

				_ => {
					window.handle_event(&event, device);
				}
			}
		}

		if should_destroy {
			device.wait_idle();
			self.winit_map.remove(&window_id);
			self.windows.remove(handle);

			if self.windows.is_empty() {
				nimbus_disp!("All windows closed. Exiting Nimbus Application.");
				event_loop.exit();
			}
		}
	}
}
