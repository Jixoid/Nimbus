/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use nimbus_math::{Color, Rect, Vec2};
use nimbus_rhi::format::Format;
use nimbus_rhi::pipeline::{Pipeline, PipelineMeta, PolygonMode, PrimitiveTopology};
use nimbus_rhi::shader::{Shader, ShaderStage};
use nimbus_rhi::types::MsaaSamples;
use nimbus_rhi_ash::{AshVulkanBackend, Device};


pub static RECT_VERT_SPV: &[u8] = include_bytes!("../../shaders/rect.vert.spv");
pub static RECT_FRAG_SPV: &[u8] = include_bytes!("../../shaders/rect.frag.spv");

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RectInstance {
	rect: [f32; 4],
	radii: [f32; 4],
	fill_color: [f32; 4],
	border_color: [f32; 4],
	shadow_color: [f32; 4],
	shadow_offset: [f32; 2],
	border_width: f32,
	shadow_blur: f32,
	noise: f32,
	_pad: [f32; 3],
}

const _: () = {
	assert!(std::mem::size_of::<RectInstance>() == 112);
	assert!(std::mem::align_of::<RectInstance>() == 16);
};


pub struct RectShape {
	pub(crate) inst: RectInstance,
}

impl RectShape {
	pub fn new(rect: Rect<f32>, fill: Color<f32>) -> Self {
		Self {
			inst: RectInstance {
				rect: [rect.x(), rect.y(), rect.width(), rect.height()],
				fill_color: fill.to_array(),
				radii: [0.0; 4],
				border_color: [0.0; 4],
				shadow_color: [0.0; 4],
				shadow_offset: [0.0; 2],
				border_width: 0.0,
				shadow_blur: 0.0,
				noise: 0.0,
				_pad: [0.0; 3],
			},
		}
	}


	pub fn border(mut self, color: Color<f32>, width: f32) -> Self {
		self.inst.border_color = color.to_array();
		self.inst.border_width = width;
		self
	}

	pub fn radius(mut self, radii: f32) -> Self {
		self.inst.radii = [radii; 4];
		self
	}

	pub fn radius4(mut self, radii: [f32; 4]) -> Self {
		self.inst.radii = radii;
		self
	}

	pub fn shadow(mut self, color: Color<f32>, offset: Vec2<f32>, blur: f32) -> Self {
		self.inst.shadow_color = color.to_array();
		self.inst.shadow_offset = offset.to_array();
		self.inst.shadow_blur = blur;
		self
	}

	pub fn noise(mut self, intensity: f32) -> Self {
		self.inst.noise = intensity.clamp(0.0, 1.0);
		self
	}
}


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default)]
pub struct RectPushConstants {
	pub(crate) instance_addr: u64,
	pub(crate) screen_size: [f32; 2],
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
