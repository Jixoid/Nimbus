/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashMap;

use crate::FontId;
use nimbus_rhi::format::Format;
use nimbus_rhi::image::{Image2Desc, ImageTiling, ImageUsage};
use nimbus_rhi::manager::allocator::{AllocationCreate, AllocationLocation};
use nimbus_rhi::sampler::{SamplerAddressMode, SamplerFilter};
use nimbus_rhi::types::MsaaSamples;
use nimbus_rhi_ash::{Allocator, AshCommandQueue, Device, Image2, ImageView2, Sampler2};
use nimbus_except::*;


const ATLAS_SIZE: u32 = 1024;

#[derive(Copy, Clone, Debug)]
pub struct CachedGlyph {
	pub uv_rect: [f32; 4], // [min_u, min_v, max_u, max_v]
	pub size: [f32; 2],    // [width, height] in pixels
	pub offset: [f32; 2],  // [xmin, ymin] in pixels
	pub advance: f32,      // advance width in pixels
}

pub struct GlyphAtlas {
	image: Image2,
	#[allow(dead_code)]
	view: ImageView2,
	#[allow(dead_code)]
	sampler: Sampler2,
	texture_id: u32,
	width: u32,
	height: u32,
	cursor_x: u32,
	cursor_y: u32,
	row_height: u32,
	cpu_buffer: Vec<u8>,
	cache: HashMap<(FontId, char, u32), CachedGlyph>,
	dirty: bool,
}

impl GlyphAtlas {
	pub fn new(device: &Device, allocator: &Allocator) -> Self {
		let desc = Image2Desc {
			width: ATLAS_SIZE,
			height: ATLAS_SIZE,
			format: Format::Norm8U,
			usage: ImageUsage::Sampled | ImageUsage::TransferDst,
			tiling: ImageTiling::Optimal,
			create: AllocationCreate::empty(),
			location: AllocationLocation::PreferDevice,
			samples: MsaaSamples::Off,
		};

		let image = allocator.alloc_image2(&desc);
		let view = device
			.create_image_view(&image, Format::Norm8U)
			.expect("Failed to create GlyphAtlas image view");

		let sampler = allocator.alloc_sampler2(
			[SamplerAddressMode::ClampToEdge; 3],
			SamplerFilter::Linear,
			SamplerFilter::Linear,
		);

		let texture_id = device.write_texture_descriptor(&view, &sampler);
		nimbus_disp!(
			"GlyphAtlas created ({}x{}) with texture_id={}",
			ATLAS_SIZE,
			ATLAS_SIZE,
			texture_id
		);

		let cpu_buffer = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize];

		Self {
			image,
			view,
			sampler,
			texture_id,
			width: ATLAS_SIZE,
			height: ATLAS_SIZE,
			cursor_x: 1,
			cursor_y: 1,
			row_height: 0,
			cpu_buffer,
			cache: HashMap::new(),
			dirty: true,
		}
	}

	pub fn texture_id(&self) -> u32 {
		self.texture_id
	}

	pub fn get_or_cache(
		&mut self,
		font_id: FontId,
		ch: char,
		size_px: u32,
		font: &fontdue::Font,
	) -> Option<CachedGlyph> {
		if let Some(glyph) = self.cache.get(&(font_id, ch, size_px)) {
			return Some(*glyph);
		}

		let (metrics, bitmap) = font.rasterize(ch, size_px as f32);
		let w = metrics.width as u32;
		let h = metrics.height as u32;

		if w == 0 || h == 0 {
			let cached = CachedGlyph {
				uv_rect: [0.0; 4],
				size: [0.0, 0.0],
				offset: [metrics.xmin as f32, metrics.ymin as f32],
				advance: metrics.advance_width,
			};
			self.cache.insert((font_id, ch, size_px), cached);
			return Some(cached);
		}

		// Shelf packing
		if self.cursor_x + w + 1 >= self.width {
			self.cursor_x = 1;
			self.cursor_y += self.row_height + 1;
			self.row_height = 0;
		}

		if self.cursor_y + h + 1 >= self.height {
			nimbus_warn!("GlyphAtlas is full! Cannot cache glyph '{}'", ch);
			return None;
		}

		// Copy into CPU buffer
		for row in 0..h {
			let src_idx = (row * w) as usize;
			let dst_idx = ((self.cursor_y + row) * self.width + self.cursor_x) as usize;
			self.cpu_buffer[dst_idx..dst_idx + w as usize]
				.copy_from_slice(&bitmap[src_idx..src_idx + w as usize]);
		}

		let uv_rect = [
			self.cursor_x as f32 / self.width as f32,
			self.cursor_y as f32 / self.height as f32,
			(self.cursor_x + w) as f32 / self.width as f32,
			(self.cursor_y + h) as f32 / self.height as f32,
		];

		let cached = CachedGlyph {
			uv_rect,
			size: [w as f32, h as f32],
			offset: [metrics.xmin as f32, metrics.ymin as f32],
			advance: metrics.advance_width,
		};

		self.cursor_x += w + 1;
		self.row_height = self.row_height.max(h);
		self.dirty = true;

		self.cache.insert((font_id, ch, size_px), cached);
		Some(cached)
	}

	pub fn update_gpu(&mut self, device: &Device, allocator: &Allocator, queue: &AshCommandQueue) {
		if self.dirty {
			let res = device.upload_image_data(
				allocator,
				queue,
				&self.image,
				self.width,
				self.height,
				&self.cpu_buffer,
			);
			if let Err(e) = res {
				nimbus_warn!("Failed to upload GlyphAtlas to GPU: {e:?}");
			} else {
				self.dirty = false;
			}
		}
	}
}
