/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::{Color, Vec2};
use nimbus_rhi::format::Format;
use nimbus_rhi::pipeline::{Pipeline, PipelineMeta, PolygonMode, PrimitiveTopology};
use nimbus_rhi::shader::{Shader, ShaderStage};
use nimbus_rhi::types::MsaaSamples;
use nimbus_rhi_ash::{AshVulkanBackend, Device};

use crate::Font;


pub static TEXT_VERT_SPV: &[u8] = include_bytes!("../../shaders/text.vert.spv");
pub static TEXT_FRAG_SPV: &[u8] = include_bytes!("../../shaders/text.frag.spv");

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct GlyphInstance {
	pub rect: [f32; 4],
	pub uv_rect: [f32; 4],
	pub color: [f32; 4],
}

const _: () = {
	assert!(std::mem::size_of::<GlyphInstance>() == 48);
	assert!(std::mem::align_of::<GlyphInstance>() == 16);
};


pub struct TextShape<'a, 'fface> {
	pub(crate) text: &'a str,
	pub(crate) font: Font<'fface>,
	pub(crate) size: f32,
	pub(crate) pos: Vec2<f32>,
	pub(crate) color: Color<f32>,
}

impl<'a, 'fface> TextShape<'a, 'fface> {
	pub fn new(
		text: &'a str,
		font: Font<'fface>,
		size: f32,
		pos: Vec2<f32>,
		color: Color<f32>,
	) -> Self {
		Self {
			text,
			font,
			size,
			pos,
			color,
		}
	}
}


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct TextPushConstants {
	pub glyph_addr: u64,
	pub screen_size: [f32; 2],
	pub texture_id: u32,
	pub _pad: u32,
}

pub struct TextPipeline {
	pipeline: Pipeline<AshVulkanBackend>,
}

impl TextPipeline {
	pub fn new(device: &Device, color_format: Format) -> Self {
		let vert_shader = Shader::new(device, ShaderStage::Vertex, TEXT_VERT_SPV, "main");
		let frag_shader = Shader::new(device, ShaderStage::Pixel, TEXT_FRAG_SPV, "main");

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex | ShaderStage::Pixel,
			std::mem::size_of::<TextPushConstants>() as u32,
		);

		let pipeline = Pipeline::new_graphics(
			device,
			&meta,
			&[&vert_shader, &frag_shader],
			PolygonMode::Fill,
			PrimitiveTopology::TriangleList,
			color_format,
			Format::Undefined,
			true,  // blend_enable
			false, // depth_write
			MsaaSamples::Off,
		);

		Self { pipeline }
	}

	pub fn pipeline(&self) -> &Pipeline<AshVulkanBackend> {
		&self.pipeline
	}
}
