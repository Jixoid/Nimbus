/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_rhi::format::Format;
use nimbus_rhi::pipeline::{Pipeline, PipelineMeta, PolygonMode, PrimitiveTopology};
use nimbus_rhi::shader::{Shader, ShaderStage};
use nimbus_rhi::types::MsaaSamples;
use nimbus_rhi_ash::{AshVulkanBackend, Device};


pub static RECT_VERT_SPV: &[u8] = include_bytes!("../shaders/rect.vert.spv");
pub static RECT_FRAG_SPV: &[u8] = include_bytes!("../shaders/rect.frag.spv");

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RectInstance {
	pub rect: [f32; 4],
	pub radii: [f32; 4],
	pub fill_color: [f32; 4],
	pub border_color: [f32; 4],
	pub shadow_color: [f32; 4],
	pub shadow_offset: [f32; 2],
	pub border_width: f32,
	pub shadow_blur: f32,
}

const _: () = {
	assert!(std::mem::size_of::<RectInstance>() == 96);
	assert!(std::mem::align_of::<RectInstance>() == 16);
};

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct RectPushConstants {
	pub instance_addr: u64,
	pub screen_size: [f32; 2],
}

pub struct RectPipeline {
	pipeline: Pipeline<AshVulkanBackend>,
}

impl RectPipeline {
	pub fn new(device: &Device, color_format: Format) -> Self {
		let vert_shader = Shader::new(device, ShaderStage::Vertex, RECT_VERT_SPV, "main");
		let frag_shader = Shader::new(device, ShaderStage::Pixel, RECT_FRAG_SPV, "main");

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex,
			std::mem::size_of::<RectPushConstants>() as u32,
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
