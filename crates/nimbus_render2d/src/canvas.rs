/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/

use nimbus_math::Rect;
use nimbus_rhi::types::Rect2D;
use nimbus_rhi::{AllocationCreate, AllocationLocation, BufferUsage, Format, ShaderStage};
use nimbus_rhi_ash::{AshCommandQueue, Command, Device, Allocator, Buffer, RenderTarget};

use crate::{RectShape, TextShape};
use crate::atlas::GlyphAtlas;
use crate::{RectInstance, RectPipeline, RectPushConstants};
use crate::{GlyphInstance, TextPipeline, TextPushConstants};

const INITIAL_BUFFER_CAPACITY: usize = 1024;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DrawBatch {
	Rects {
		start: u32,
		count: u32,
		clip: Option<Rect<f32>>,
	},
	Text {
		start: u32,
		count: u32,
		clip: Option<Rect<f32>>,
	},
}

impl DrawBatch {
	pub fn clip(&self) -> Option<Rect<f32>> {
		match self {
			DrawBatch::Rects { clip, .. } | DrawBatch::Text { clip, .. } => *clip,
		}
	}
}


pub struct Canvas2D {
	rect_pipeline: RectPipeline,
	text_pipeline: TextPipeline,
	atlas: GlyphAtlas,
	rect_instances: Vec<RectInstance>,
	glyph_instances: Vec<GlyphInstance>,
	batches: Vec<DrawBatch>,
	clip_stack: Vec<Rect<f32>>,
	rect_buffer: Option<Buffer>,
	glyph_buffer: Option<Buffer>,
}

impl Canvas2D {
	pub fn new(device: &Device, allocator: &Allocator, color_format: Format) -> Self {
		let rect_pipeline = RectPipeline::new(device, color_format);
		let text_pipeline = TextPipeline::new(device, color_format);
		let atlas = GlyphAtlas::new(device, allocator);

		Self {
			rect_pipeline,
			text_pipeline,
			atlas,
			rect_instances: Vec::with_capacity(INITIAL_BUFFER_CAPACITY),
			glyph_instances: Vec::with_capacity(INITIAL_BUFFER_CAPACITY),
			batches: Vec::with_capacity(INITIAL_BUFFER_CAPACITY),
			clip_stack: Vec::new(),
			rect_buffer: None,
			glyph_buffer: None,
		}
	}

	pub fn clear_batches(&mut self) {
		self.rect_instances.clear();
		self.glyph_instances.clear();
		self.batches.clear();
		self.clip_stack.clear();
	}

	pub fn push_clip(&mut self, clip: Rect<f32>) {
		let effective = match self.clip_stack.last() {
			Some(parent) => parent.intersection(&clip).unwrap_or(Rect::ZERO),
			None => clip,
		};
		self.clip_stack.push(effective);
	}

	pub fn pop_clip(&mut self) {
		self.clip_stack.pop();
	}

	pub fn current_clip(&self) -> Option<Rect<f32>> {
		self.clip_stack.last().copied()
	}

	// Draw API
	pub fn draw<T>(&mut self, shape: T)
	where
		T: DrawShape,
	{
		DrawShape::draw(self, shape);
	}

	pub fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
		crate::FontManager::default_font().measure_text(text, font_size)
	}

	pub fn batches(&self) -> &[DrawBatch] {
		&self.batches
	}

	#[allow(clippy::too_many_arguments)]
	pub fn render(
		&mut self,
		cmd: &mut Command,
		_target: &RenderTarget,
		width: u32,
		height: u32,
		device: &Device,
		allocator: &Allocator,
		queue: &AshCommandQueue,
	) {
		if self.batches.is_empty() {
			return;
		}

		// Flush any new glyphs to the GPU atlas texture
		self.atlas.update_gpu(device, allocator, queue);

		let screen_size = [width as f32, height as f32];
		cmd.set_viewport_and_scissor(width, height);

		// Prepare rect buffer if needed
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
			}
		}

		// Prepare glyph buffer if needed
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
			}
		}

		let mut current_scissor: Option<Rect2D> = None;
		let mut current_is_rect: Option<bool> = None;

		for batch in &self.batches {
			let desired_scissor = match batch.clip() {
				Some(c) => {
					let x0 = (c.x().max(0.0).min(width as f32) as i32).max(0);
					let y0 = (c.y().max(0.0).min(height as f32) as i32).max(0);
					let x1 = (c.max.x.max(0.0).min(width as f32) as i32).max(x0);
					let y1 = (c.max.y.max(0.0).min(height as f32) as i32).max(y0);
					let w = (x1 - x0) as u32;
					let h = (y1 - y0) as u32;
					Rect2D::new(x0, y0, w, h)
				}
				None => Rect2D::new(0, 0, width, height),
			};

			if current_scissor != Some(desired_scissor) {
				cmd.set_scissor(
					desired_scissor.x,
					desired_scissor.y,
					desired_scissor.width,
					desired_scissor.height,
				);
				current_scissor = Some(desired_scissor);
			}

			match batch {
				DrawBatch::Rects { start, count, .. } => {
					if let Some(buf) = &self.rect_buffer {
						if current_is_rect != Some(true) {
							cmd.bind_pipeline(self.rect_pipeline.pipeline());
							let pc = RectPushConstants {
								instance_addr: buf.address(),
								screen_size,
							};
							cmd.push_constant(ShaderStage::Vertex, 0, &pc);
							current_is_rect = Some(true);
						}
						cmd.draw(6, *count, 0, *start);
					}
				}
				DrawBatch::Text { start, count, .. } => {
					if let Some(buf) = &self.glyph_buffer {
						if current_is_rect != Some(false) {
							cmd.bind_pipeline(self.text_pipeline.pipeline());
							let pc = TextPushConstants {
								glyph_addr: buf.address(),
								screen_size,
								texture_id: self.atlas.texture_id(),
								_pad: 0,
							};
							cmd.push_constant(ShaderStage::Vertex | ShaderStage::Pixel, 0, &pc);
							current_is_rect = Some(false);
						}
						cmd.draw(6, *count, 0, *start);
					}
				}
			}
		}
	}
}


pub trait DrawShape {
	fn draw(canvas: &mut Canvas2D, shape: Self);
}

impl DrawShape for RectShape {
	fn draw(canvas: &mut Canvas2D, shape: Self) {
		let clip = canvas.current_clip();
		let idx = canvas.rect_instances.len() as u32;
		canvas.rect_instances.push(shape.inst);

		if let Some(DrawBatch::Rects {
			count,
			clip: batch_clip,
			..
		}) = canvas.batches.last_mut()
			&& *batch_clip == clip
		{
			*count += 1;
			return;
		}

		canvas.batches.push(DrawBatch::Rects {
			start: idx,
			count: 1,
			clip,
		});
	}
}

impl<'a, 'fface> DrawShape for TextShape<'a, 'fface> {
	fn draw(canvas: &mut Canvas2D, shape: Self) {
		let line_height = shape
			.font
			.face()
			.inner
			.horizontal_line_metrics(shape.size)
			.map(|m| m.new_line_size)
			.unwrap_or(shape.size * 1.2);

		let mut pen_x = shape.pos[0];
		let mut pen_y = shape.pos[1] + line_height * 0.8;
		let clip = canvas.current_clip();
		let start_glyph_idx = canvas.glyph_instances.len() as u32;

		for ch in shape.text.chars() {
			if ch == '\n' {
				pen_x = shape.pos[0];
				pen_y += line_height;
				continue;
			}

			if let Some(glyph) = canvas.atlas.get_or_cache(
				shape.font.id(),
				ch,
				shape.size as u32,
				&shape.font.face().inner,
			) {
				if glyph.size[0] > 0.0 && glyph.size[1] > 0.0 {
					let gx = pen_x + glyph.offset[0];
					let gy = pen_y - glyph.offset[1] - glyph.size[1];

					canvas.glyph_instances.push(GlyphInstance {
						rect: [gx, gy, glyph.size[0], glyph.size[1]],
						uv_rect: glyph.uv_rect,
						color: shape.color.to_array(),
					});
				}
				pen_x += glyph.advance;
			}
		}

		let glyph_count = (canvas.glyph_instances.len() as u32) - start_glyph_idx;
		if glyph_count == 0 {
			return;
		}

		if let Some(DrawBatch::Text {
			count,
			clip: batch_clip,
			..
		}) = canvas.batches.last_mut()
			&& *batch_clip == clip
		{
			*count += glyph_count;
			return;
		}

		canvas.batches.push(DrawBatch::Text {
			start: start_glyph_idx,
			count: glyph_count,
			clip,
		});
	}
}
