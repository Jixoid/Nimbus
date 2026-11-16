/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_rhi::{AllocationCreate, AllocationLocation, Buffer, BufferUsage};
use nimbus_rhi::format::Format;
use nimbus_rhi::manager::allocator::Allocator;
use nimbus_rhi::render_target::RenderTarget;
use nimbus_rhi::shader::ShaderStage;
use nimbus_rhi_ash::{AshCommandQueue, AshVulkanBackend, Command, Device};

use crate::atlas::GlyphAtlas;
use crate::font::Font;
use crate::rect::{RectInstance, RectPipeline, RectPushConstants};
use crate::text::{GlyphInstance, TextPipeline, TextPushConstants};

const INITIAL_BUFFER_CAPACITY: usize = 1024;


pub struct Canvas2D {
	rect_pipeline: RectPipeline,
	text_pipeline: TextPipeline,
	atlas: GlyphAtlas,
	font: Font,
	rect_instances: Vec<RectInstance>,
	glyph_instances: Vec<GlyphInstance>,
	rect_buffer: Option<Buffer<AshVulkanBackend>>,
	glyph_buffer: Option<Buffer<AshVulkanBackend>>,
}

impl Canvas2D {
	pub fn new(
		device: &Device,
		allocator: &Allocator<AshVulkanBackend>,
		color_format: Format,
	) -> Self {
		let rect_pipeline = RectPipeline::new(device, color_format);
		let text_pipeline = TextPipeline::new(device, color_format);
		let atlas = GlyphAtlas::new(device, allocator);
		let font = Font::default_font();

		Self {
			rect_pipeline,
			text_pipeline,
			atlas,
			font,
			rect_instances: Vec::with_capacity(INITIAL_BUFFER_CAPACITY),
			glyph_instances: Vec::with_capacity(INITIAL_BUFFER_CAPACITY),
			rect_buffer: None,
			glyph_buffer: None,
		}
	}

	pub fn clear_batches(&mut self) {
		self.rect_instances.clear();
		self.glyph_instances.clear();
	}

	pub fn draw_rect(&mut self, rect: [f32; 4], fill: [f32; 4]) {
		self.rect_instances.push(RectInstance {
			rect,
			radii: [0.0; 4],
			fill_color: fill,
			border_color: [0.0; 4],
			shadow_color: [0.0; 4],
			shadow_offset: [0.0; 2],
			border_width: 0.0,
			shadow_blur: 0.0,
		});
	}

	pub fn draw_rounded_rect(&mut self, rect: [f32; 4], radius: f32, fill: [f32; 4]) {
		self.rect_instances.push(RectInstance {
			rect,
			radii: [radius; 4],
			fill_color: fill,
			border_color: [0.0; 4],
			shadow_color: [0.0; 4],
			shadow_offset: [0.0; 2],
			border_width: 0.0,
			shadow_blur: 0.0,
		});
	}

	pub fn draw_rounded_rect_varying(&mut self, rect: [f32; 4], radii: [f32; 4], fill: [f32; 4]) {
		self.rect_instances.push(RectInstance {
			rect,
			radii,
			fill_color: fill,
			border_color: [0.0; 4],
			shadow_color: [0.0; 4],
			shadow_offset: [0.0; 2],
			border_width: 0.0,
			shadow_blur: 0.0,
		});
	}

	#[allow(clippy::too_many_arguments)]
	pub fn draw_card(
		&mut self,
		rect: [f32; 4],
		radius: f32,
		fill: [f32; 4],
		border_width: f32,
		border_color: [f32; 4],
		shadow_blur: f32,
		shadow_offset: [f32; 2],
		shadow_color: [f32; 4],
	) {
		self.rect_instances.push(RectInstance {
			rect,
			radii: [radius; 4],
			fill_color: fill,
			border_color,
			shadow_color,
			shadow_offset,
			border_width,
			shadow_blur,
		});
	}

	pub fn draw_circle(&mut self, center: [f32; 2], radius: f32, fill: [f32; 4]) {
		self.draw_rounded_rect(
			[
				center[0] - radius,
				center[1] - radius,
				radius * 2.0,
				radius * 2.0,
			],
			radius,
			fill,
		);
	}

	pub fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: [f32; 4]) {
		let mut pen_x = x;
		let line_height = self
			.font
			.raw()
			.horizontal_line_metrics(font_size)
			.map(|m| m.new_line_size)
			.unwrap_or(font_size * 1.2);
		let mut pen_y = y + line_height * 0.8; // baseline offset

		for ch in text.chars() {
			if ch == '\n' {
				pen_x = x;
				pen_y += line_height;
				continue;
			}

			if let Some(glyph) = self
				.atlas
				.get_or_cache(ch, font_size as u32, self.font.raw())
			{
				if glyph.size[0] > 0.0 && glyph.size[1] > 0.0 {
					let gx = pen_x + glyph.offset[0];
					let gy = pen_y - glyph.offset[1] - glyph.size[1];

					self.glyph_instances.push(GlyphInstance {
						rect: [gx, gy, glyph.size[0], glyph.size[1]],
						uv_rect: glyph.uv_rect,
						color,
					});
				}
				pen_x += glyph.advance;
			}
		}
	}

	pub fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
		self.font.measure_text(text, font_size)
	}

	pub fn render(
		&mut self,
		cmd: &mut Command,
		_target: &RenderTarget<AshVulkanBackend>,
		width: u32,
		height: u32,
		device: &Device,
		allocator: &Allocator<AshVulkanBackend>,
		queue: &AshCommandQueue,
	) {
		// 1. Flush any new glyphs to the GPU atlas texture
		self.atlas.update_gpu(device, allocator, queue);

		let screen_size = [width as f32, height as f32];
		cmd.set_viewport_and_scissor(width, height);

		// 2. Render all rectangle & card instances
		if !self.rect_instances.is_empty() {
			let byte_size = (self.rect_instances.len() * std::mem::size_of::<RectInstance>()) as u64;

			let needs_realloc = match &self.rect_buffer {
				Some(buf) => buf.size() < byte_size,
				None => true,
			};

			if needs_realloc {
				let alloc_size = byte_size
					.max(INITIAL_BUFFER_CAPACITY as u64 * 96)
					.next_power_of_two();
				self.rect_buffer = Some(allocator.alloc_buffer(
					alloc_size,
					BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
					AllocationCreate::Mapped | AllocationCreate::HostSequentialWrite,
					AllocationLocation::PreferHost,
				));
			}

			if let Some(buf) = &mut self.rect_buffer {
				let slice = unsafe {
					std::slice::from_raw_parts(
						self.rect_instances.as_ptr() as *const u8,
						byte_size as usize,
					)
				};
				buf
					.write_bytes(0, slice)
					.expect("Failed to write rect instances to GPU buffer");

				cmd.bind_pipeline(self.rect_pipeline.pipeline());

				let pc = RectPushConstants {
					instance_addr: buf.address(),
					screen_size,
				};
				cmd.push_constant(ShaderStage::Vertex, 0, &pc);
				cmd.draw(6, self.rect_instances.len() as u32, 0, 0);
			}
		}

		// 3. Render all text glyph instances
		if !self.glyph_instances.is_empty() {
			let byte_size = (self.glyph_instances.len() * std::mem::size_of::<GlyphInstance>()) as u64;

			let needs_realloc = match &self.glyph_buffer {
				Some(buf) => buf.size() < byte_size,
				None => true,
			};

			if needs_realloc {
				let alloc_size = byte_size
					.max(INITIAL_BUFFER_CAPACITY as u64 * 48)
					.next_power_of_two();
				self.glyph_buffer = Some(allocator.alloc_buffer(
					alloc_size,
					BufferUsage::StorageBuffer | BufferUsage::DeviceAddress,
					AllocationCreate::Mapped | AllocationCreate::HostSequentialWrite,
					AllocationLocation::PreferHost,
				));
			}

			if let Some(buf) = &mut self.glyph_buffer {
				let slice = unsafe {
					std::slice::from_raw_parts(
						self.glyph_instances.as_ptr() as *const u8,
						byte_size as usize,
					)
				};
				buf
					.write_bytes(0, slice)
					.expect("Failed to write glyph instances to GPU buffer");

				cmd.bind_pipeline(self.text_pipeline.pipeline());

				let pc = TextPushConstants {
					glyph_addr: buf.address(),
					screen_size,
					texture_id: self.atlas.texture_id(),
					_pad: 0,
				};
				cmd.push_constant(ShaderStage::Vertex | ShaderStage::Pixel, 0, &pc);
				cmd.draw(6, self.glyph_instances.len() as u32, 0, 0);
			}
		}
	}
}
