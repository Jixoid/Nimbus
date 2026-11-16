/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, Window, WindowAttributes};

use nimbus_rhi_ash::{self as rhi, Device, Surface, Swapchain};
use nimbus_except::*;


pub struct Display {
	swapchain: Swapchain,
	surface: Surface,
	window: Arc<Window>,
	width: u32,
	height: u32,
	scale_factor: f64,
}

impl Display {
	pub fn new(
		event_loop: &ActiveEventLoop,
		device: &Device,
		title: &str,
		width: u32,
		height: u32,
		vsync: bool,
	) -> Result<Self, String> {
		let attributes = WindowAttributes::default()
			.with_title(title)
			.with_inner_size(LogicalSize::new(width, height));

		let window = Arc::new(
			event_loop
				.create_window(attributes)
				.map_err(|e| format!("Failed to create window: {e}"))?,
		);

		let scale_factor = window.scale_factor();
		let physical_size = window.inner_size();
		let phys_width = physical_size.width.max(1);
		let phys_height = physical_size.height.max(1);

		let surface = device.create_surface(&*window)?;
		let swapchain = Swapchain::new(device, &surface, phys_width, phys_height, vsync);

		nimbus_disp!(
			"Platform window created: \"{}\" (logical: {}x{}, physical: {}x{}, scale: {:.2}) with swapchain ({} images)",
			title,
			width,
			height,
			phys_width,
			phys_height,
			scale_factor,
			swapchain.image_count()
		);

		Ok(Self {
			swapchain,
			surface,
			window,
			width: phys_width,
			height: phys_height,
			scale_factor,
		})
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		if width == 0 || height == 0 {
			return;
		}

		self.width = width;
		self.height = height;
		self.swapchain.resize(width, height);
	}

	pub fn update_scale_factor(&mut self, scale_factor: f64) {
		self.scale_factor = scale_factor;
	}

	pub fn acquire_next_image(&mut self) -> u32 {
		self.swapchain.acquire_next_image()
	}

	pub fn acquire_next_image_with_semaphore(&mut self, signal_semaphore: &rhi::Semaphore) -> u32 {
		self
			.swapchain
			.acquire_next_image_with_semaphore(Some(signal_semaphore))
	}

	pub fn present(&mut self, image_index: u32) {
		self.swapchain.present(image_index);
	}

	pub fn present_with_semaphore(&mut self, image_index: u32, wait_semaphore: &rhi::Semaphore) {
		self
			.swapchain
			.present_with_semaphore(image_index, Some(wait_semaphore));
	}

	pub fn render_target(&self, index: u32) -> Option<rhi::RenderTarget<'_>> {
		self.swapchain.render_target(index)
	}

	pub fn render_finished_semaphore(&self, index: u32) -> rhi::Semaphore {
		self.swapchain.render_finished_semaphore(index)
	}

	pub fn set_cursor_icon(&self, icon: CursorIcon) {
		self.window.set_cursor(icon);
	}

	pub fn set_title(&self, title: &str) {
		self.window.set_title(title);
	}

	pub fn request_redraw(&self) {
		self.window.request_redraw();
	}

	pub fn window(&self) -> &Window {
		&self.window
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn scale_factor(&self) -> f64 {
		self.scale_factor
	}

	pub fn vsync(&self) -> bool {
		self.swapchain.vsync()
	}

	pub fn swapchain(&self) -> &Swapchain {
		&self.swapchain
	}

	pub fn surface(&self) -> &Surface {
		&self.surface
	}
}
