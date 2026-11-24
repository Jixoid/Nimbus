/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;
use nimbus_theme::Theme;
use slotmap::SlotMap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use nimbus_except::*;
use nimbus_rhi::manager::allocator::Allocator;
use nimbus_rhi_ash::{AshVulkanBackend, CommandManager, Device};

use crate::id::WindowHandle;
use crate::task::{IntoTask, Task, TaskQueue};
use crate::window::Window;


pub type PreRenderCallback = Box<dyn FnMut(&Device, &CommandManager, &mut Window) + 'static>;

pub struct Application {
	pub theme: Theme,
	pub windows: SlotMap<WindowHandle, Window>,
	pub task_queue: TaskQueue,
	on_pre_render: Option<PreRenderCallback>,
}

impl Application {
	pub fn new(theme: Theme) -> Self {
		Self {
			theme,
			windows: SlotMap::with_key(),
			task_queue: TaskQueue::new(),
			on_pre_render: None,
		}
	}

	pub fn set_on_pre_render<F>(&mut self, callback: F)
	where
		F: FnMut(&Device, &CommandManager, &mut Window) + 'static,
	{
		self.on_pre_render = Some(Box::new(callback));
	}

	pub fn task_queue(&self) -> TaskQueue {
		self.task_queue.clone()
	}

	pub fn push_task<M, F: IntoTask<M>>(&self, task: F) {
		self.task_queue.push(task);
	}

	pub fn window_task_queue(&self, handle: WindowHandle) -> Option<TaskQueue> {
		self.windows.get(handle).map(|w| w.task_queue())
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
			task_queue: self.task_queue,
			on_pre_render: self.on_pre_render,
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


struct AppRunner {
	theme: Theme,
	windows: SlotMap<WindowHandle, Window>,
	task_queue: TaskQueue,
	on_pre_render: Option<PreRenderCallback>,
	winit_map: HashMap<WindowId, WindowHandle>,
	cmd_manager: Option<CommandManager>,
	allocator: Option<Allocator<AshVulkanBackend>>,
	device: Option<Device>,
}

impl Drop for AppRunner {
	fn drop(&mut self) {
		if let Some(device) = &self.device {
			device.wait_idle();
		}
		self.windows.clear();
		self.winit_map.clear();
		self.cmd_manager = None;
		self.allocator = None;
		self.device = None;
	}
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
						if let Some(cb) = &mut self.on_pre_render {
							cb(device, cmd_manager, window);
						}
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

	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		for _ in 0..64 {
			let tasks = self.task_queue.drain_all();
			if tasks.is_empty() {
				break;
			}
			for task in tasks {
				match task {
					Task::Plain(f) => f(),
					Task::Tree(f) => {
						if let Some(window) = self.windows.values_mut().next() {
							f(&mut window.tree);
							window.tree.needs_redraw = true;
						}
					}
					Task::Window(f) => {
						if let Some(window) = self.windows.values_mut().next() {
							f(window);
							window.tree.needs_redraw = true;
						}
					}
					Task::Theme(f) => {
						f(&mut self.theme);
						for window in self.windows.values_mut() {
							window.tree.needs_redraw = true;
						}
					}
				}
			}
		}

		let mut closed_windows = Vec::new();
		for (handle, window) in self.windows.iter_mut() {
			if !window.tree.task_queue.is_empty() {
				window.process_tasks();
			}
			if window.should_close {
				closed_windows.push((handle, window.winit_id));
			} else if window.tree.needs_redraw || window.tree.needs_layout {
				window.request_redraw();
			}
		}

		if !closed_windows.is_empty() {
			if let Some(device) = &self.device {
				device.wait_idle();
			}
			for (handle, winit_id) in closed_windows {
				if let Some(wid) = winit_id {
					self.winit_map.remove(&wid);
				}
				self.windows.remove(handle);
			}
			if self.windows.is_empty() {
				nimbus_disp!("All windows closed. Exiting Nimbus Application.");
				event_loop.exit();
			}
		}
	}
}
